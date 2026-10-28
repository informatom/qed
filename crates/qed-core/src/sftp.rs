//! Publishing a built site to a server over SFTP.
//!
//! The server's identity is checked on every connection: qed remembers the
//! fingerprint of the server's key the first time the user trusts it, and
//! refuses to continue if it ever changes (someone could be pretending to be
//! the server).

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

use russh::client;
use russh::keys::ssh_key::LineEnding;
use russh::keys::ssh_key::private::{Ed25519Keypair, KeypairData};
use russh::keys::{HashAlg, PrivateKey, PrivateKeyWithHashAlg, PublicKeyOrCertificate};
use russh_sftp::client::SftpSession;
use serde::Serialize;

use crate::deploy::{self, MANIFEST_FILE, Manifest};
use crate::target::SftpTarget;

/// The secret for logging in: a password, or a private key in OpenSSH format.
pub enum Credentials {
    Password(String),
    Key(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SftpError {
    #[error("can't reach {host}:{port}: {message}")]
    Connect {
        host: String,
        port: u16,
        message: String,
    },

    /// First connection: the user has to decide whether to trust the server.
    #[error("this server isn't trusted yet; its key fingerprint is {fingerprint}")]
    UnknownHostKey { fingerprint: String },

    #[error(
        "the server's key changed (expected {expected}, got {found}); \
         if the server wasn't reinstalled, someone may be pretending to be it"
    )]
    HostKeyChanged { expected: String, found: String },

    #[error("the server didn't accept the {what}")]
    AuthFailed { what: &'static str },

    #[error("{0}")]
    Failed(String),
}

impl SftpError {
    fn failed(context: &str, error: impl std::fmt::Display) -> Self {
        SftpError::Failed(format!("{context}: {error}"))
    }
}

/// Checks the server's key during the connection handshake.
pub(crate) struct HostKeyCheck {
    expected: Option<String>,
    /// The fingerprint the server presented, for the error message.
    seen: Arc<Mutex<Option<String>>>,
}

impl client::Handler for HostKeyCheck {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        let fingerprint = fingerprint(key);
        let trusted = self.expected.as_deref() == Some(fingerprint.as_str());
        if let Ok(mut seen) = self.seen.lock() {
            *seen = Some(fingerprint);
        }
        Ok(trusted)
    }
}

fn fingerprint(key: &PublicKeyOrCertificate) -> String {
    match key {
        PublicKeyOrCertificate::PublicKey { key, .. } => {
            key.fingerprint(HashAlg::Sha256).to_string()
        }
        PublicKeyOrCertificate::Certificate(cert) => {
            cert.public_key().fingerprint(HashAlg::Sha256).to_string()
        }
    }
}

/// An SSH session, logged in as `user`, with the server's key checked
/// against `host_key`. Fails with [`SftpError::UnknownHostKey`] if there's
/// no trusted fingerprint yet. Used for SFTP and for git remotes.
pub(crate) async fn login(
    host: &str,
    port: u16,
    user: &str,
    host_key: Option<&str>,
    credentials: &Credentials,
) -> Result<client::Handle<HostKeyCheck>, SftpError> {
    let seen = Arc::new(Mutex::new(None));
    let check = HostKeyCheck {
        expected: host_key.map(str::to_string),
        seen: Arc::clone(&seen),
    };
    let config = Arc::new(client::Config::default());
    let connected = client::connect(config, (host, port), check).await;

    let mut session = match connected {
        Ok(session) => session,
        Err(error) => {
            let seen = seen.lock().ok().and_then(|seen| seen.clone());
            return Err(match (seen, host_key) {
                (Some(found), None) => SftpError::UnknownHostKey { fingerprint: found },
                (Some(found), Some(expected)) if found != expected => SftpError::HostKeyChanged {
                    expected: expected.to_string(),
                    found,
                },
                _ => SftpError::Connect {
                    host: host.to_string(),
                    port,
                    message: error.to_string(),
                },
            });
        }
    };

    let accepted = match credentials {
        Credentials::Password(password) => session
            .authenticate_password(user, password)
            .await
            .map_err(|e| SftpError::failed("logging in", e))?
            .success(),
        Credentials::Key(pem) => {
            let key = PrivateKey::from_openssh(pem)
                .map_err(|e| SftpError::failed("reading the key", e))?;
            let hash = session
                .best_supported_rsa_hash()
                .await
                .map_err(|e| SftpError::failed("logging in", e))?
                .flatten();
            session
                .authenticate_publickey(user, PrivateKeyWithHashAlg::new(Arc::new(key), hash))
                .await
                .map_err(|e| SftpError::failed("logging in", e))?
                .success()
        }
    };
    if !accepted {
        let what = match credentials {
            Credentials::Password(_) => "password",
            Credentials::Key(_) => "key (is the public key in the server's authorized_keys?)",
        };
        return Err(SftpError::AuthFailed { what });
    }

    Ok(session)
}

/// An open SFTP connection, with the target's folder as its base.
pub struct Connection {
    session: client::Handle<HostKeyCheck>,
    sftp: SftpSession,
    base: String,
}

/// Connects and logs in. Fails with [`SftpError::UnknownHostKey`] if the
/// target has no trusted fingerprint yet.
pub async fn connect(
    target: &SftpTarget,
    credentials: &Credentials,
) -> Result<Connection, SftpError> {
    let session = login(
        &target.host,
        target.port,
        &target.user,
        target.host_key.as_deref(),
        credentials,
    )
    .await?;

    let channel = session
        .channel_open_session()
        .await
        .map_err(|e| SftpError::failed("opening a channel", e))?;
    channel
        .request_subsystem(true, "sftp")
        .await
        .map_err(|e| SftpError::failed("starting SFTP", e))?;
    let sftp = SftpSession::new(channel.into_stream())
        .await
        .map_err(|e| SftpError::failed("starting SFTP", e))?;

    Ok(Connection {
        session,
        sftp,
        base: target.path.trim_end_matches('/').to_string(),
    })
}

impl Connection {
    fn remote(&self, relative: &str) -> String {
        if self.base.is_empty() {
            relative.to_string()
        } else {
            format!("{}/{relative}", self.base)
        }
    }

    /// Makes sure a folder exists, like `mkdir -p`, touching the server as
    /// little as possible: if the folder is there, that's one check. Only if
    /// it's missing does this walk up to the nearest existing parent and
    /// create the missing folders below it. Folders above the target are
    /// never created or even checked when they aren't needed; on a real
    /// server, the user may not be allowed to look at them.
    ///
    /// `known` remembers folders that exist, to save round trips.
    async fn create_dirs(&self, path: &str, known: &mut HashSet<String>) -> Result<(), SftpError> {
        let path = path.trim_end_matches('/');
        if path.is_empty() || known.contains(path) {
            return Ok(());
        }

        // Walk up until a folder exists; collect the missing ones.
        let mut missing = Vec::new();
        let mut current = path;
        loop {
            if known.contains(current) || self.is_dir(current).await {
                break;
            }
            missing.push(current);
            match current.rsplit_once('/') {
                Some((parent, _)) if !parent.is_empty() => current = parent,
                // Reached "/" or a relative path's first part.
                _ => break,
            }
        }

        // Create them top-down.
        for folder in missing.iter().rev() {
            if let Err(error) = self.sftp.create_dir(*folder).await {
                // Someone else may have created it meanwhile; that's fine.
                if !self.is_dir(folder).await {
                    return Err(SftpError::failed(
                        &format!("creating the folder {folder}"),
                        error,
                    ));
                }
            }
        }

        let mut current = path;
        loop {
            known.insert(current.to_string());
            match current.rsplit_once('/') {
                Some((parent, _)) if !parent.is_empty() => current = parent,
                _ => break,
            }
        }
        Ok(())
    }

    /// Whether a folder exists. Anything that prevents checking counts as
    /// "no"; creating it will then report the real problem.
    async fn is_dir(&self, path: &str) -> bool {
        self.sftp
            .metadata(path)
            .await
            // A server that doesn't report the type gets the benefit of the doubt.
            .is_ok_and(|attributes| attributes.permissions.is_none() || attributes.is_dir())
    }

    /// The deepest folder above `path` that the server shows, if any.
    async fn nearest_existing_parent(&self, path: &str) -> Option<String> {
        let mut current = path.trim_end_matches('/');
        while let Some((parent, _)) = current.rsplit_once('/') {
            let parent = if parent.is_empty() { "/" } else { parent };
            if self.is_dir(parent).await {
                return Some(parent.to_string());
            }
            if parent == "/" {
                return None;
            }
            current = parent;
        }
        None
    }

    async fn close(self) {
        let _ = self.sftp.close().await;
        let _ = self
            .session
            .disconnect(russh::Disconnect::ByApplication, "", "en")
            .await;
    }
}

/// Connects, checks the target folder, and disconnects. Returns a short
/// description of what was found.
pub async fn test(target: &SftpTarget, credentials: &Credentials) -> Result<String, SftpError> {
    let connection = connect(target, credentials).await?;
    let folder = connection.base.clone();
    let exists = connection.is_dir(&folder).await;
    let published = exists
        && connection
            .sftp
            .try_exists(connection.remote(MANIFEST_FILE))
            .await
            .unwrap_or(false);

    let report = match (exists, published) {
        (true, true) => format!("Connected. {folder} holds an earlier publish from qed."),
        (true, false) => {
            format!("Connected. {folder} exists; nothing was published there by q.e.d. yet.")
        }
        (false, _) => {
            // Many servers confine SFTP to one folder, so paths differ from
            // what a shell login shows. Say what the server does show.
            let start = connection.sftp.canonicalize(".").await.ok();
            let nearest = connection.nearest_existing_parent(&folder).await;
            let mut text = format!(
                "Connected, but the folder {folder} isn't there as seen over SFTP; publishing would create it."
            );
            if let Some(nearest) = nearest {
                text.push_str(&format!(" The nearest folder that exists is {nearest}."));
            }
            if let Some(start) = start {
                text.push_str(&format!(
                    " Your SFTP login starts in {start}. If the server limits SFTP to a folder, \
                     paths are counted from there."
                ));
            }
            text
        }
    };
    connection.close().await;
    Ok(report)
}

/// How far a publish has got, for a progress bar.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Progress {
    pub stage: Stage,
    pub done_files: usize,
    pub total_files: usize,
    pub done_bytes: u64,
    pub total_bytes: u64,
    /// The file being uploaded or deleted right now.
    pub current: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Connecting,
    Comparing,
    Uploading,
    Deleting,
    Done,
}

/// What a publish did.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PublishReport {
    pub uploaded: usize,
    pub deleted: usize,
    pub unchanged: usize,
    pub bytes: u64,
}

/// Uploads a built site (`out_dir`) to the target: only new and changed
/// files, then the manifest, then (if enabled) deletes stale files.
pub async fn publish(
    target: &SftpTarget,
    credentials: &Credentials,
    out_dir: &Path,
    mut progress: impl FnMut(Progress),
) -> Result<PublishReport, SftpError> {
    let mut report =
        |stage, done_files, total_files, done_bytes, total_bytes, current: Option<&str>| {
            progress(Progress {
                stage,
                done_files,
                total_files,
                done_bytes,
                total_bytes,
                current: current.map(str::to_string),
            });
        };

    report(Stage::Connecting, 0, 0, 0, 0, None);
    let connection = connect(target, credentials).await?;

    report(Stage::Comparing, 0, 0, 0, 0, None);
    let folder = out_dir.to_path_buf();
    let local = tokio::task::spawn_blocking(move || Manifest::of_folder(&folder))
        .await
        .map_err(|e| SftpError::failed("checking the built site", e))?
        .map_err(|e| SftpError::failed("checking the built site", e))?;
    let remote = match connection.sftp.read(connection.remote(MANIFEST_FILE)).await {
        Ok(bytes) => Manifest::from_json(&String::from_utf8_lossy(&bytes)),
        Err(_) => None,
    };
    let plan = deploy::plan(&local, remote.as_ref(), target.delete_stale);

    let mut known_dirs = HashSet::new();
    connection
        .create_dirs(&connection.base, &mut known_dirs)
        .await?;

    let total = plan.upload.len();
    let mut done_bytes = 0;
    for (index, path) in plan.upload.iter().enumerate() {
        report(
            Stage::Uploading,
            index,
            total,
            done_bytes,
            plan.upload_bytes,
            Some(path),
        );
        let remote_path = connection.remote(path);
        if let Some((parent, _)) = remote_path.rsplit_once('/') {
            connection.create_dirs(parent, &mut known_dirs).await?;
        }
        let mut local_file = tokio::fs::File::open(out_dir.join(path))
            .await
            .map_err(|e| SftpError::failed(path, e))?;
        let mut remote_file = connection
            .sftp
            .create(remote_path.as_str())
            .await
            .map_err(|e| SftpError::failed(&format!("uploading {path}"), e))?;
        tokio::io::copy(&mut local_file, &mut remote_file)
            .await
            .map_err(|e| SftpError::failed(&format!("uploading {path}"), e))?;
        tokio::io::AsyncWriteExt::shutdown(&mut remote_file)
            .await
            .map_err(|e| SftpError::failed(&format!("uploading {path}"), e))?;
        done_bytes += local.files.get(path).map_or(0, |entry| entry.size);
    }

    // The new manifest lists what's on the server now: the site, plus stale
    // files that stay because deleting is off (so a later publish with
    // deleting on can still remove them).
    let mut manifest = local.clone();
    if let Some(remote) = &remote {
        for (path, entry) in &remote.files {
            if !plan.delete.contains(path) && !manifest.files.contains_key(path) {
                manifest.files.insert(path.clone(), entry.clone());
            }
        }
    }
    let mut manifest_file = connection
        .sftp
        .create(connection.remote(MANIFEST_FILE))
        .await
        .map_err(|e| SftpError::failed("saving the manifest", e))?;
    tokio::io::AsyncWriteExt::write_all(&mut manifest_file, manifest.to_json().as_bytes())
        .await
        .map_err(|e| SftpError::failed("saving the manifest", e))?;
    tokio::io::AsyncWriteExt::shutdown(&mut manifest_file)
        .await
        .map_err(|e| SftpError::failed("saving the manifest", e))?;

    let deletes = plan.delete.len();
    for (index, path) in plan.delete.iter().enumerate() {
        report(
            Stage::Deleting,
            index,
            deletes,
            done_bytes,
            plan.upload_bytes,
            Some(path),
        );
        // A file someone already removed by hand is fine.
        let _ = connection.sftp.remove_file(connection.remote(path)).await;
    }
    // Folders the deleted files leave empty go too, deepest first. Removing a
    // folder that still holds anything fails, which is what we want: files
    // that qed didn't put there stay.
    for folder in emptied_folders(&plan.delete) {
        let _ = connection.sftp.remove_dir(connection.remote(&folder)).await;
    }

    connection.close().await;
    report(
        Stage::Done,
        total,
        total,
        done_bytes,
        plan.upload_bytes,
        None,
    );

    Ok(PublishReport {
        uploaded: plan.upload.len(),
        deleted: plan.delete.len(),
        unchanged: plan.unchanged,
        bytes: plan.upload_bytes,
    })
}

/// Every folder that contains a deleted file, including parent folders,
/// deepest first: `a/b/c.html` gives `a/b`, then `a`.
fn emptied_folders(deleted: &[String]) -> Vec<String> {
    let mut folders: Vec<String> = deleted
        .iter()
        .flat_map(|path| {
            let mut parents = Vec::new();
            let mut current = path.as_str();
            while let Some((parent, _)) = current.rsplit_once('/') {
                parents.push(parent.to_string());
                current = parent;
            }
            parents
        })
        .collect();
    folders.sort_by(|a, b| {
        b.matches('/')
            .count()
            .cmp(&a.matches('/').count())
            .then(a.cmp(b))
    });
    folders.dedup();
    folders
}

/// A new key pair for logging in: the private key in OpenSSH format (for the
/// keychain) and the public key line for the server's `authorized_keys`.
pub fn generate_key(comment: &str) -> Result<(String, String), SftpError> {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).map_err(|e| SftpError::failed("generating a key", e))?;
    let keypair = Ed25519Keypair::from_seed(&seed);
    let private = PrivateKey::new(KeypairData::from(keypair), comment)
        .map_err(|e| SftpError::failed("generating a key", e))?;
    let private_pem = private
        .to_openssh(LineEnding::LF)
        .map_err(|e| SftpError::failed("generating a key", e))?;
    let public_line = private
        .public_key()
        .to_openssh()
        .map_err(|e| SftpError::failed("generating a key", e))?;
    Ok((private_pem.to_string(), public_line))
}

/// The public key line for a stored private key, to show it again.
pub fn public_key_of(private_pem: &str) -> Result<String, SftpError> {
    let key = PrivateKey::from_openssh(private_pem)
        .map_err(|e| SftpError::failed("reading the key", e))?;
    key.public_key()
        .to_openssh()
        .map_err(|e| SftpError::failed("reading the key", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_emptied_folders_deepest_first() {
        let deleted = [
            "a/b/c.html".to_string(),
            "a/d.html".into(),
            "top.html".into(),
        ];
        assert_eq!(emptied_folders(&deleted), ["a/b", "a"]);
    }

    #[test]
    fn generates_ed25519_keys_in_openssh_format() {
        let (private, public) = generate_key("qed test").unwrap();
        assert!(
            private.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----"),
            "{private}"
        );
        assert!(public.starts_with("ssh-ed25519 "), "{public}");
        assert!(public.ends_with(" qed test"), "{public}");
        assert_eq!(public_key_of(&private).unwrap(), public);

        // Two keys are never the same.
        let (_, other) = generate_key("qed test").unwrap();
        assert_ne!(public, other);
    }
}

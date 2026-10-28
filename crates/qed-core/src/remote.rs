//! Pushing a site's history to a git remote and pulling it back, so a site
//! can be edited on two computers and backed up.
//!
//! The remotes are git's own (`.git/config`), so `git remote add` works as
//! well as the app. q.e.d. logs in over SSH with its own key (kept in the
//! keychain by the app) and checks the server's key like SFTP publishing
//! does: the trusted fingerprint is kept in the remote's config as
//! `remote.<name>.qedhostkey`.
//!
//! git2 is built without network support. Instead q.e.d. hands libgit2 an
//! SSH connection of its own ([`SshTransport`], on russh like SFTP): libgit2
//! speaks the git protocol, q.e.d. carries the bytes.

use std::cell::Cell;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::transport::{Service, SmartSubtransport, SmartSubtransportStream, Transport};
use git2::{
    AutotagOption, BranchType, ErrorCode, FetchOptions, Oid, PushOptions, RemoteCallbacks,
    Repository,
};
use russh::ChannelMsg;
use russh::client;
use serde::Serialize;

use crate::history::{self, HistoryError};
use crate::sftp::{self, Credentials, HostKeyCheck, SftpError};

/// The git config key, below `remote.<name>.`, with the server's trusted
/// key fingerprint.
const HOST_KEY: &str = "qedhostkey";

#[derive(Debug, thiserror::Error)]
pub enum RemoteError {
    #[error("this site has no version history yet")]
    NotTracked,

    #[error(
        "this site is part of the git repository at {}; push and pull that one with git",
        .0.display()
    )]
    Shared(PathBuf),

    #[error("there is no remote called {0}")]
    NoRemote(String),

    #[error("“{0}” can't be the name of a remote (letters, digits and - only)")]
    BadName(String),

    #[error("there is a remote called {0} already")]
    NameTaken(String),

    #[error(
        "q.e.d. pushes and pulls over SSH only; “{0}” isn't such an address \
         (git@host:owner/site.git or ssh://git@host/owner/site.git)"
    )]
    BadAddress(String),

    #[error("this server isn't trusted yet; its key fingerprint is {fingerprint}")]
    UnknownHostKey { fingerprint: String },

    #[error(
        "the server's key changed (expected {expected}, got {found}); \
         if the server wasn't reinstalled, someone may be pretending to be it"
    )]
    HostKeyChanged { expected: String, found: String },

    #[error(
        "the server didn't accept q.e.d.'s key; add the public key to the repository \
         (as a deploy key with write access) or to your account there"
    )]
    AuthFailed,

    #[error("can't reach the server: {0}")]
    Connect(String),

    #[error("the server said: {0}")]
    Server(String),

    #[error("{0} has versions this computer doesn't have yet; pull first, then push")]
    Rejected(String),

    #[error(
        "both computers changed {}; nothing was changed here. Keep one side \
         (“Go back” in the Versions list) or fix it with git",
        .0.join(", ")
    )]
    Conflict(Vec<String>),

    #[error(
        "this site and {0} have separate histories, so they can't be merged; \
         to edit a site from git on this computer, get it with “Get a site from git”"
    )]
    Unrelated(String),

    #[error("this site has no versions yet, so there's nothing to push")]
    Empty,

    #[error("{} isn't empty; choose a new folder for the site", .0.display())]
    FolderNotEmpty(PathBuf),

    #[error("that repository has no q.e.d. site in it (no site.toml)")]
    NotASite,

    #[error("another push or pull is still running")]
    Busy,

    #[error("can't set up git over SSH: {0}")]
    Setup(String),

    #[error("can't write {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    History(#[from] HistoryError),

    #[error("git: {}", .0.message())]
    Git(#[from] git2::Error),
}

/// A remote of the site's repository, as the Versions tab shows it.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
    /// The keychain account of q.e.d.'s key for this address.
    pub key_account: String,
    /// The server's trusted key fingerprint, once the user trusted it.
    pub host_key: Option<String>,
    /// The current branch pushes to and pulls from this remote.
    pub upstream: bool,
    /// Versions here that the remote doesn't have, as far as this computer
    /// knows (from the last push, pull or check).
    pub ahead: u32,
    /// Versions on the remote that aren't here yet, as of the last check.
    pub behind: u32,
    /// The remote has the current branch at all (known since the last
    /// push, pull or check).
    pub has_branch: bool,
}

/// What a push, pull or check did.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct RemoteReport {
    /// One sentence for the user.
    pub message: String,
    /// The site's files changed (a pull brought something), so the app
    /// should load the site again.
    pub changed: bool,
    /// The remote afterwards.
    pub remote: RemoteInfo,
}

/// How far a push, pull or "get a site" has got, for a progress bar.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Transfer {
    pub stage: TransferStage,
    /// Objects (files and versions, as git counts them) done so far.
    pub done: usize,
    pub total: usize,
    /// Bytes sent or received so far.
    pub bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum TransferStage {
    /// Push: collecting what the remote lacks.
    Packing,
    Sending,
    Receiving,
    /// Pull: unpacking what arrived.
    Resolving,
}

/// Passes progress on, at most every 100 ms (libgit2 reports far more
/// often), and always the end of a stage.
struct Throttle<'a> {
    report: &'a dyn Fn(Transfer),
    last: Cell<Option<Instant>>,
}

impl<'a> Throttle<'a> {
    fn new(report: &'a dyn Fn(Transfer)) -> Self {
        Throttle {
            report,
            last: Cell::new(None),
        }
    }

    fn send(&self, stage: TransferStage, done: usize, total: usize, bytes: usize) {
        let now = Instant::now();
        let due = done == total
            || self
                .last
                .get()
                .is_none_or(|last| now - last >= Duration::from_millis(100));
        if due {
            self.last.set(Some(now));
            (self.report)(Transfer {
                stage,
                done,
                total,
                bytes,
            });
        }
    }
}

/// The keychain account for q.e.d.'s key to a remote address. One key per
/// address, since GitHub and others accept a deploy key for one repository
/// only.
pub fn key_account(url: &str) -> String {
    format!("git:{}", url.trim())
}

/// The site's remotes, with what this computer knows about them.
pub fn remotes(root: &Path) -> Result<Vec<RemoteInfo>, RemoteError> {
    let repo = own_repo(root)?;
    let names = repo.remotes()?;
    names
        .iter()
        .flatten()
        .flatten()
        .map(|name| info(&repo, name))
        .collect()
}

/// Adds a remote; `url` must be an SSH address.
pub fn add_remote(root: &Path, name: &str, url: &str) -> Result<RemoteInfo, RemoteError> {
    let repo = own_repo(root)?;
    let (name, url) = (name.trim(), url.trim());
    if !git2::Remote::is_valid_name(name) || name.contains('/') {
        return Err(RemoteError::BadName(name.to_string()));
    }
    if SshAddress::parse(url).is_none() {
        return Err(RemoteError::BadAddress(url.to_string()));
    }
    if repo.find_remote(name).is_ok() {
        return Err(RemoteError::NameTaken(name.to_string()));
    }
    repo.remote(name, url)?;
    info(&repo, name)
}

/// Removes a remote from the site's repository (nothing on the server
/// changes).
pub fn remove_remote(root: &Path, name: &str) -> Result<(), RemoteError> {
    let repo = own_repo(root)?;
    find_remote(&repo, name)?;
    repo.remote_delete(name)?;
    Ok(())
}

/// Remembers `fingerprint` as the remote server's key.
pub fn trust(root: &Path, name: &str, fingerprint: &str) -> Result<RemoteInfo, RemoteError> {
    let repo = own_repo(root)?;
    find_remote(&repo, name)?;
    repo.config()?
        .set_str(&format!("remote.{name}.{HOST_KEY}"), fingerprint.trim())?;
    info(&repo, name)
}

/// Asks the remote what it has (a fetch: nothing here changes but git's
/// knowledge of the remote), for "there are new versions, pull?".
pub fn check(
    root: &Path,
    name: &str,
    key: &str,
    progress: &dyn Fn(Transfer),
) -> Result<RemoteReport, RemoteError> {
    let repo = own_repo(root)?;
    let host_key = host_key(&repo, name)?;
    let throttle = Throttle::new(progress);
    with_login(key, host_key, || fetch(&repo, name, &throttle))?;
    let remote = info(&repo, name)?;
    let message = match remote.behind {
        0 => format!("Nothing new on {name}."),
        1 => format!("{name} has 1 version this computer doesn't have yet."),
        n => format!("{name} has {n} versions this computer doesn't have yet."),
    };
    Ok(RemoteReport {
        message,
        changed: false,
        remote,
    })
}

/// Pushes the site's versions and publish tags to the remote. Closes the
/// open version first, with `note` (a pushed version can't change any
/// more). Afterwards the branch follows this remote.
pub fn push(
    root: &Path,
    name: &str,
    key: &str,
    note: Option<&str>,
    progress: &dyn Fn(Transfer),
) -> Result<RemoteReport, RemoteError> {
    let repo = own_repo(root)?;
    let host_key = host_key(&repo, name)?;
    close_open_version(root, note, "Saved before pushing")?;
    let throttle = Throttle::new(progress);
    let branch = current_branch(&repo)?.ok_or(RemoteError::Empty)?;
    let before = info(&repo, name)?;

    let mut specs = vec![format!("refs/heads/{branch}:refs/heads/{branch}")];
    for tag in repo.tag_names(None)?.iter().flatten().flatten() {
        specs.push(format!("refs/tags/{tag}:refs/tags/{tag}"));
    }
    let rejected = Mutex::new(Vec::<String>::new());
    let pushed = with_login(key, host_key, || {
        let mut remote = repo.find_remote(name)?;
        let mut callbacks = RemoteCallbacks::new();
        callbacks.pack_progress(|_, done, total| {
            throttle.send(TransferStage::Packing, done, total, 0);
        });
        callbacks.push_transfer_progress(|done, total, bytes| {
            throttle.send(TransferStage::Sending, done, total, bytes);
        });
        callbacks.push_update_reference(|reference, status| {
            if status.is_some()
                && let Ok(mut rejected) = rejected.lock()
            {
                rejected.push(reference.to_string());
            }
            Ok(())
        });
        let mut options = PushOptions::new();
        options.remote_callbacks(callbacks);
        remote.push(&specs, Some(&mut options))
    });
    match pushed {
        Err(RemoteError::Git(error)) if error.code() == ErrorCode::NotFastForward => {
            return Err(RemoteError::Rejected(name.to_string()));
        }
        other => other?,
    }
    if rejected.lock().is_ok_and(|rejected| !rejected.is_empty()) {
        return Err(RemoteError::Rejected(name.to_string()));
    }

    // From now on this branch pushes to and pulls from `name`.
    repo.find_branch(&branch, BranchType::Local)?
        .set_upstream(Some(&format!("{name}/{branch}")))?;
    let remote = info(&repo, name)?;
    let message = match before.ahead {
        0 => format!("{name} had everything already."),
        1 => format!("Pushed 1 version to {name}."),
        n => format!("Pushed {n} versions to {name}."),
    };
    Ok(RemoteReport {
        message,
        changed: false,
        remote,
    })
}

/// Brings the remote's versions here: closes the open version first (with
/// `note`), then fast-forwards if only the remote changed, or merges if
/// both did. If both changed the same file, nothing changes and the
/// error names the files.
pub fn pull(
    root: &Path,
    name: &str,
    key: &str,
    note: Option<&str>,
    progress: &dyn Fn(Transfer),
) -> Result<RemoteReport, RemoteError> {
    let repo = own_repo(root)?;
    let host_key = host_key(&repo, name)?;
    let throttle = Throttle::new(progress);
    with_login(key, host_key, || fetch(&repo, name, &throttle))?;
    drop(repo);
    pull_fetched(root, name, note)
}

/// The part of [`pull`] after fetching: everything happens here.
fn pull_fetched(root: &Path, name: &str, note: Option<&str>) -> Result<RemoteReport, RemoteError> {
    close_open_version(root, note, "Saved before pulling")?;
    let repo = own_repo(root)?;
    let report = |message: String, changed: bool| -> Result<RemoteReport, RemoteError> {
        Ok(RemoteReport {
            message,
            changed,
            remote: info(&repo, name)?,
        })
    };
    let Some(branch) = current_branch(&repo)? else {
        return report(
            "This site has no versions yet; get it with “Get a site from git” instead.".into(),
            false,
        );
    };
    let Ok(theirs) = repo.find_reference(&format!("refs/remotes/{name}/{branch}")) else {
        return report(format!("{name} has nothing to pull yet."), false);
    };
    let theirs = theirs.peel_to_commit()?;
    let ours = repo.head()?.peel_to_commit()?;
    if repo.merge_base(ours.id(), theirs.id()).is_err() {
        return Err(RemoteError::Unrelated(name.to_string()));
    }
    let (ahead, behind) = repo.graph_ahead_behind(ours.id(), theirs.id())?;
    let count = |n: usize| match n {
        1 => "1 version".to_string(),
        n => format!("{n} versions"),
    };
    if behind == 0 {
        return report(format!("Nothing new on {name}."), false);
    }

    let mut checkout = CheckoutBuilder::new();
    checkout.safe();
    if ahead == 0 {
        // Only the remote changed: move up to it.
        repo.checkout_tree(theirs.as_object(), Some(&mut checkout))?;
        repo.find_reference(&format!("refs/heads/{branch}"))?
            .set_target(theirs.id(), &format!("pull from {name}: fast-forward"))?;
        return report(format!("Got {} from {name}.", count(behind)), true);
    }

    // Both changed: merge, unless the same file changed on both sides.
    let mut index = repo.merge_commits(&ours, &theirs, None)?;
    if index.has_conflicts() {
        let mut files: Vec<String> = index
            .conflicts()?
            .flatten()
            .filter_map(|conflict| conflict.our.or(conflict.their).or(conflict.ancestor))
            .map(|entry| String::from_utf8_lossy(&entry.path).to_string())
            .collect();
        files.sort();
        files.dedup();
        return Err(RemoteError::Conflict(files));
    }
    let tree = repo.find_tree(index.write_tree_to(&repo)?)?;
    repo.checkout_tree(tree.as_object(), Some(&mut checkout))?;
    let signature = repo
        .signature()
        .or_else(|_| git2::Signature::now("q.e.d.", "qed@localhost"))?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &format!("Merged the versions from {name}"),
        &tree,
        &[&ours, &theirs],
    )?;
    report(
        format!(
            "Got {} from {name} and merged them with {} made here.",
            count(behind),
            count(ahead)
        ),
        true,
    )
}

/// Gets a site from a remote into `folder` (new, or empty), with its whole
/// history; the remote is called `origin` there. `host_key` is the
/// server's trusted fingerprint, if the user trusted it already.
pub fn clone_site(
    url: &str,
    folder: &Path,
    key: &str,
    host_key: Option<&str>,
    progress: &dyn Fn(Transfer),
) -> Result<RemoteReport, RemoteError> {
    let url = url.trim();
    if SshAddress::parse(url).is_none() {
        return Err(RemoteError::BadAddress(url.to_string()));
    }
    let existed = folder.exists();
    if existed
        && fs::read_dir(folder)
            .map_err(|source| RemoteError::Io {
                path: folder.to_path_buf(),
                source,
            })?
            .next()
            .is_some()
    {
        return Err(RemoteError::FolderNotEmpty(folder.to_path_buf()));
    }
    let throttle = Throttle::new(progress);
    let repo = with_login(key, host_key.map(str::to_string), || {
        RepoBuilder::new()
            .fetch_options(fetch_options(&throttle))
            .clone(url, folder)
    })?;
    if !folder.join("site.toml").is_file() {
        drop(repo);
        // Only clean up what this call made.
        if !existed {
            let _ = fs::remove_dir_all(folder);
        }
        return Err(RemoteError::NotASite);
    }
    if let Some(fingerprint) = host_key {
        repo.config()?
            .set_str(&format!("remote.origin.{HOST_KEY}"), fingerprint)?;
    }
    let remote = info(&repo, "origin")?;
    let versions = match repo.head().ok().and_then(|head| head.target()) {
        Some(head) => count_versions(&repo, head)?,
        None => 0,
    };
    Ok(RemoteReport {
        message: match versions {
            1 => "Got the site with 1 version.".to_string(),
            n => format!("Got the site with {n} versions."),
        },
        changed: true,
        remote,
    })
}

/// The site's own repository: q.e.d. pushes and pulls only sites that are
/// a repository of their own, not a folder in a larger one.
fn own_repo(root: &Path) -> Result<Repository, RemoteError> {
    let repo = match Repository::discover(root) {
        Ok(repo) => repo,
        Err(error) if error.code() == ErrorCode::NotFound => return Err(RemoteError::NotTracked),
        Err(error) => return Err(error.into()),
    };
    let canonical = |path: &Path| {
        fs::canonicalize(path).map_err(|source| RemoteError::Io {
            path: path.to_path_buf(),
            source,
        })
    };
    let workdir = repo
        .workdir()
        .map(canonical)
        .transpose()?
        .ok_or(RemoteError::NotTracked)?;
    if workdir != canonical(root)? {
        return Err(RemoteError::Shared(workdir));
    }
    Ok(repo)
}

fn find_remote<'r>(repo: &'r Repository, name: &str) -> Result<git2::Remote<'r>, RemoteError> {
    repo.find_remote(name)
        .map_err(|_| RemoteError::NoRemote(name.to_string()))
}

fn host_key(repo: &Repository, name: &str) -> Result<Option<String>, RemoteError> {
    find_remote(repo, name)?;
    Ok(repo
        .config()?
        .get_string(&format!("remote.{name}.{HOST_KEY}"))
        .ok()
        .filter(|key| !key.is_empty()))
}

/// The branch HEAD is on; `None` before the first version.
fn current_branch(repo: &Repository) -> Result<Option<String>, RemoteError> {
    match repo.head() {
        Ok(head) if head.is_branch() => Ok(head.shorthand().ok().map(str::to_string)),
        Ok(_) => Ok(None),
        Err(error) if matches!(error.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound) => {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

fn info(repo: &Repository, name: &str) -> Result<RemoteInfo, RemoteError> {
    let remote = find_remote(repo, name)?;
    let url = remote.url().unwrap_or_default().to_string();
    let branch = current_branch(repo)?;
    let upstream = match &branch {
        Some(branch) => repo
            .branch_upstream_remote(&format!("refs/heads/{branch}"))
            .ok()
            .map(|remote| remote.as_str().is_ok_and(|r| r == name))
            .unwrap_or(false),
        None => false,
    };
    let head = repo.head().ok().and_then(|head| head.target());
    let theirs = branch.as_ref().and_then(|branch| {
        repo.find_reference(&format!("refs/remotes/{name}/{branch}"))
            .ok()
            .and_then(|reference| reference.target())
    });
    let (ahead, behind) = match (head, theirs) {
        (Some(ours), Some(theirs)) => repo.graph_ahead_behind(ours, theirs)?,
        (Some(ours), None) => (count_versions(repo, ours)?, 0),
        _ => (0, 0),
    };
    Ok(RemoteInfo {
        name: name.to_string(),
        key_account: key_account(&url),
        url,
        host_key: host_key(repo, name)?,
        upstream,
        ahead: u32::try_from(ahead).unwrap_or(u32::MAX),
        behind: u32::try_from(behind).unwrap_or(u32::MAX),
        has_branch: theirs.is_some(),
    })
}

fn count_versions(repo: &Repository, head: Oid) -> Result<usize, RemoteError> {
    let mut walk = repo.revwalk()?;
    walk.push(head)?;
    Ok(walk.count())
}

/// Closes the open version (or saves changed files as a version), so
/// what's pushed or merged is a closed version.
fn close_open_version(
    root: &Path,
    note: Option<&str>,
    default_note: &str,
) -> Result<(), RemoteError> {
    if history::status(root)?.needs_note() {
        let note = note
            .map(str::trim)
            .filter(|note| !note.is_empty())
            .unwrap_or(default_note);
        history::save_version(root, note)?;
    }
    Ok(())
}

/// Fetches the remote's branches and tags into `refs/remotes/<name>/`.
fn fetch(repo: &Repository, name: &str, throttle: &Throttle) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote(name)?;
    let mut options = fetch_options(throttle);
    remote.fetch(
        &[format!("+refs/heads/*:refs/remotes/{name}/*")],
        Some(&mut options),
        None,
    )
}

fn fetch_options<'a>(throttle: &'a Throttle<'a>) -> FetchOptions<'a> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(|stats| {
        if stats.received_objects() < stats.total_objects() {
            throttle.send(
                TransferStage::Receiving,
                stats.received_objects(),
                stats.total_objects(),
                stats.received_bytes(),
            );
        } else {
            throttle.send(
                TransferStage::Resolving,
                stats.indexed_deltas(),
                stats.total_deltas(),
                stats.received_bytes(),
            );
        }
        true
    });
    let mut options = FetchOptions::new();
    options
        .remote_callbacks(callbacks)
        .download_tags(AutotagOption::All);
    options
}

// ---- SSH for libgit2 ------------------------------------------------------

/// The login for the git operation that is running: libgit2 asks the
/// registered transport for connections without saying for whom, so the
/// key travels this way. [`with_login`] runs one operation at a time.
struct Login {
    key: String,
    host_key: Option<String>,
    /// Why connecting failed, in detail (libgit2 only passes on a message).
    failure: Mutex<Option<SftpError>>,
    /// What the server wrote to stderr (e.g. "repository not found").
    server_said: Mutex<String>,
}

static OPERATION: Mutex<()> = Mutex::new(());
static LOGIN: Mutex<Option<Arc<Login>>> = Mutex::new(None);
static REGISTERED: OnceLock<Result<(), String>> = OnceLock::new();

/// Runs a git network operation with `key` as the login and turns its
/// errors into ones the user can act on.
fn with_login<T>(
    key: &str,
    host_key: Option<String>,
    operation: impl FnOnce() -> Result<T, git2::Error>,
) -> Result<T, RemoteError> {
    let _running = OPERATION.try_lock().map_err(|_| RemoteError::Busy)?;
    register()?;
    let login = Arc::new(Login {
        key: key.to_string(),
        host_key,
        failure: Mutex::new(None),
        server_said: Mutex::new(String::new()),
    });
    set_login(Some(Arc::clone(&login)));
    let result = operation();
    set_login(None);
    result.map_err(|error| explain(&login, error))
}

fn set_login(login: Option<Arc<Login>>) {
    if let Ok(mut current) = LOGIN.lock() {
        *current = login;
    }
}

fn explain(login: &Login, error: git2::Error) -> RemoteError {
    if let Some(failure) = login.failure.lock().ok().and_then(|mut f| f.take()) {
        return match failure {
            SftpError::UnknownHostKey { fingerprint } => {
                RemoteError::UnknownHostKey { fingerprint }
            }
            SftpError::HostKeyChanged { expected, found } => {
                RemoteError::HostKeyChanged { expected, found }
            }
            SftpError::AuthFailed { .. } => RemoteError::AuthFailed,
            other => RemoteError::Connect(other.to_string()),
        };
    }
    let said = login
        .server_said
        .lock()
        .map(|said| said.trim().to_string())
        .unwrap_or_default();
    if !said.is_empty() {
        return RemoteError::Server(said);
    }
    RemoteError::Git(error)
}

fn register() -> Result<(), RemoteError> {
    REGISTERED
        .get_or_init(|| {
            // SAFETY: registering must not race with libgit2 creating
            // transports. It happens once, while holding OPERATION, and
            // only this module makes network connections (the history
            // works on the local repository only).
            unsafe {
                git2::transport::register("ssh", |remote| {
                    Transport::smart(remote, false, SshTransport)
                })
            }
            .map_err(|error| error.message().to_string())
        })
        .clone()
        .map_err(RemoteError::Setup)
}

/// An SSH address: `git@host:owner/site.git` or
/// `ssh://git@host:port/owner/site.git`.
#[derive(Debug, PartialEq)]
struct SshAddress {
    user: String,
    host: String,
    port: u16,
    path: String,
}

impl SshAddress {
    fn parse(url: &str) -> Option<SshAddress> {
        let url = url.trim();
        let split_user = |authority: &str| match authority.rsplit_once('@') {
            Some((user, host)) => (user.to_string(), host.to_string()),
            None => ("git".to_string(), authority.to_string()),
        };
        let address = if let Some(rest) = url.strip_prefix("ssh://") {
            let (authority, path) = rest.split_once('/')?;
            let (user, host_port) = split_user(authority);
            let (host, port) = match host_port.rsplit_once(':') {
                Some((host, port)) => (host.to_string(), port.parse().ok()?),
                None => (host_port, 22),
            };
            // `ssh://host/~/x` means x in the home folder, like git.
            let path = match path.strip_prefix('~') {
                Some(home) => format!("~{home}"),
                None => format!("/{path}"),
            };
            SshAddress {
                user,
                host,
                port,
                path,
            }
        } else {
            if url.contains("://") {
                return None;
            }
            let (authority, path) = url.split_once(':')?;
            if authority.contains('/') {
                return None;
            }
            let (user, host) = split_user(authority);
            SshAddress {
                user,
                host,
                port: 22,
                path: path.to_string(),
            }
        };
        let valid = !address.user.is_empty()
            && !address.host.is_empty()
            && !address.path.trim_matches('/').is_empty();
        valid.then_some(address)
    }

    /// The command that starts git on the server, quoted like git does.
    fn command(&self, program: &str) -> String {
        format!("{program} '{}'", self.path.replace('\'', r"'\''"))
    }
}

/// The transport libgit2 uses for SSH addresses.
struct SshTransport;

impl SmartSubtransport for SshTransport {
    fn action(
        &self,
        url: &str,
        action: Service,
    ) -> Result<Box<dyn SmartSubtransportStream>, git2::Error> {
        let login = LOGIN
            .lock()
            .ok()
            .and_then(|login| login.clone())
            .ok_or_else(|| git2::Error::from_str("q.e.d. has no login for this remote"))?;
        let address = SshAddress::parse(url)
            .ok_or_else(|| git2::Error::from_str(&format!("not an SSH address: {url}")))?;
        let program = match action {
            Service::UploadPackLs | Service::UploadPack => "git-upload-pack",
            Service::ReceivePackLs | Service::ReceivePack => "git-receive-pack",
        };
        // libgit2 calls this from a plain thread and wants blocking reads
        // and writes; this runtime drives russh for this one connection.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| git2::Error::from_str(&error.to_string()))?;
        let credentials = Credentials::Key(login.key.clone());
        let opened = runtime.block_on(async {
            let session = sftp::login(
                &address.host,
                address.port,
                &address.user,
                login.host_key.as_deref(),
                &credentials,
            )
            .await?;
            let channel = session
                .channel_open_session()
                .await
                .map_err(|e| SftpError::Failed(format!("opening a channel: {e}")))?;
            channel
                .exec(true, address.command(program))
                .await
                .map_err(|e| SftpError::Failed(format!("starting git on the server: {e}")))?;
            Ok::<_, SftpError>((session, channel))
        });
        match opened {
            Ok((session, channel)) => Ok(Box::new(SshStream {
                channel,
                pending: Vec::new(),
                offset: 0,
                done: false,
                login,
                _session: session,
                runtime,
            })),
            Err(failure) => {
                let message = failure.to_string();
                if let Ok(mut slot) = login.failure.lock() {
                    *slot = Some(failure);
                }
                Err(git2::Error::from_str(&message))
            }
        }
    }

    fn close(&self) -> Result<(), git2::Error> {
        Ok(())
    }
}

/// One `git-upload-pack` or `git-receive-pack` on the server, as a
/// blocking stream.
struct SshStream {
    channel: russh::Channel<client::Msg>,
    /// Data received but not read yet.
    pending: Vec<u8>,
    offset: usize,
    /// The server closed its side.
    done: bool,
    login: Arc<Login>,
    // Kept for the connection's lifetime; dropped before the runtime.
    _session: client::Handle<HostKeyCheck>,
    runtime: tokio::runtime::Runtime,
}

impl Read for SshStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        while self.offset >= self.pending.len() {
            if self.done {
                return Ok(0);
            }
            match self.runtime.block_on(self.channel.wait()) {
                Some(ChannelMsg::Data { data }) => {
                    self.pending = data.to_vec();
                    self.offset = 0;
                }
                Some(ChannelMsg::ExtendedData { data, .. }) => {
                    if let Ok(mut said) = self.login.server_said.lock() {
                        said.push_str(&String::from_utf8_lossy(&data));
                    }
                }
                Some(ChannelMsg::Eof | ChannelMsg::Close) | None => self.done = true,
                Some(_) => {}
            }
        }
        let n = buf.len().min(self.pending.len() - self.offset);
        buf[..n].copy_from_slice(&self.pending[self.offset..self.offset + n]);
        self.offset += n;
        Ok(n)
    }
}

impl Write for SshStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.runtime
            .block_on(self.channel.data(buf))
            .map_err(io::Error::other)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_ssh_addresses() {
        assert_eq!(
            SshAddress::parse("git@code.informatom.com:Ponysee/waldgruber.at.git"),
            Some(SshAddress {
                user: "git".into(),
                host: "code.informatom.com".into(),
                port: 22,
                path: "Ponysee/waldgruber.at.git".into(),
            })
        );
        assert_eq!(
            SshAddress::parse("ssh://me@example.org:2222/srv/git/site.git"),
            Some(SshAddress {
                user: "me".into(),
                host: "example.org".into(),
                port: 2222,
                path: "/srv/git/site.git".into(),
            })
        );
        assert_eq!(
            SshAddress::parse("ssh://example.org/~/site.git").map(|a| (a.user, a.path)),
            Some(("git".into(), "~/site.git".into()))
        );
        assert_eq!(SshAddress::parse("https://github.com/me/site.git"), None);
        assert_eq!(SshAddress::parse("/home/me/site.git"), None);
        assert_eq!(SshAddress::parse("git@host:"), None);
    }

    #[test]
    fn quotes_the_server_command() {
        let address = SshAddress::parse("git@host:it's/site.git").expect("an address");
        assert_eq!(
            address.command("git-upload-pack"),
            r"git-upload-pack 'it'\''s/site.git'"
        );
    }

    fn site_with_history(dir: &Path) {
        fs::write(dir.join("site.toml"), "title = \"Test\"\n").expect("site.toml");
        fs::write(dir.join("index.md"), "+++\ntitle = \"Home\"\n+++\nHello\n").expect("page");
        history::start(dir).expect("history");
    }

    #[test]
    fn manages_remotes_in_git_config() {
        let dir = tempfile::tempdir().expect("temp dir");
        site_with_history(dir.path());
        let added = add_remote(dir.path(), "backup", "git@example.org:me/site.git").expect("add");
        assert_eq!(added.key_account, "git:git@example.org:me/site.git");
        assert!(!added.has_branch && !added.upstream);
        assert_eq!(added.ahead, 1);
        assert!(matches!(
            add_remote(dir.path(), "backup", "git@example.org:me/other.git"),
            Err(RemoteError::NameTaken(_))
        ));
        assert!(matches!(
            add_remote(dir.path(), "web", "https://example.org/site.git"),
            Err(RemoteError::BadAddress(_))
        ));
        let trusted = trust(dir.path(), "backup", "SHA256:abc").expect("trust");
        assert_eq!(trusted.host_key.as_deref(), Some("SHA256:abc"));
        assert_eq!(remotes(dir.path()).expect("list").len(), 1);
        remove_remote(dir.path(), "backup").expect("remove");
        assert!(remotes(dir.path()).expect("list").is_empty());
    }

    #[test]
    fn refuses_a_site_inside_a_larger_repository() {
        let dir = tempfile::tempdir().expect("temp dir");
        Repository::init(dir.path()).expect("repo");
        let site = dir.path().join("docs");
        fs::create_dir(&site).expect("site folder");
        assert!(matches!(remotes(&site), Err(RemoteError::Shared(_))));
    }

    /// Two clones of one bare repository, connected through git's own
    /// file transport (the SSH part needs a server): pull fast-forwards,
    /// merges different files, and refuses the same file changed twice.
    #[test]
    fn pulls_by_fast_forward_merge_or_not_at_all() {
        let dir = tempfile::tempdir().expect("temp dir");
        let bare = dir.path().join("bare.git");
        Repository::init_bare(&bare).expect("bare");
        let (a, b) = (dir.path().join("a"), dir.path().join("b"));
        fs::create_dir(&a).expect("a");
        site_with_history(&a);
        let url = bare.to_string_lossy().to_string();
        let repo_a = Repository::open(&a).expect("open a");
        repo_a.remote("origin", &url).expect("remote");
        let branch = current_branch(&repo_a).expect("branch").expect("a branch");
        repo_a
            .find_remote("origin")
            .expect("origin")
            .push(&[format!("refs/heads/{branch}:refs/heads/{branch}")], None)
            .expect("push");
        Repository::clone(&url, &b).expect("clone");

        let local_pull = |root: &Path| -> Result<RemoteReport, RemoteError> {
            // Same as `pull`, minus the SSH login: the file transport
            // needs none.
            let repo = own_repo(root)?;
            fetch(&repo, "origin", &Throttle::new(&|_| {}))?;
            drop(repo);
            pull_fetched(root, "origin", None)
        };
        let push_a = || {
            let repo = Repository::open(&a).expect("open a");
            repo.find_remote("origin")
                .expect("origin")
                .push(&[format!("refs/heads/{branch}:refs/heads/{branch}")], None)
                .expect("push");
        };

        // 1. Only A changed: B fast-forwards.
        fs::write(
            a.join("index.md"),
            "+++\ntitle = \"Home\"\n+++\nHello again\n",
        )
        .expect("edit");
        history::save_version(&a, "Edit on A").expect("version");
        push_a();
        let report = local_pull(&b).expect("pull");
        assert!(report.changed, "{}", report.message);
        assert!(
            fs::read_to_string(b.join("index.md"))
                .expect("read")
                .contains("again")
        );

        // 2. Both changed different files: merged.
        fs::write(a.join("about.md"), "+++\ntitle = \"About\"\n+++\nA\n").expect("edit");
        history::save_version(&a, "About on A").expect("version");
        push_a();
        fs::write(b.join("contact.md"), "+++\ntitle = \"Contact\"\n+++\nB\n").expect("edit");
        history::save_version(&b, "Contact on B").expect("version");
        let report = local_pull(&b).expect("merge");
        assert!(report.message.contains("merged"), "{}", report.message);
        assert!(b.join("about.md").is_file() && b.join("contact.md").is_file());

        // 3. Both changed the same file: refused, nothing changed.
        fs::write(a.join("index.md"), "+++\ntitle = \"Home\"\n+++\nFrom A\n").expect("edit");
        history::save_version(&a, "Home on A").expect("version");
        push_a();
        fs::write(b.join("index.md"), "+++\ntitle = \"Home\"\n+++\nFrom B\n").expect("edit");
        history::save_version(&b, "Home on B").expect("version");
        match local_pull(&b) {
            Err(RemoteError::Conflict(files)) => assert_eq!(files, vec!["index.md".to_string()]),
            other => panic!("expected a conflict, got {other:?}"),
        }
        assert!(
            fs::read_to_string(b.join("index.md"))
                .expect("read")
                .contains("From B")
        );
    }
}

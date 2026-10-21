//! Publish targets: where a site gets published, listed in `targets.toml`
//! next to `site.toml`.
//!
//! ```toml
//! [[targets]]
//! kind = "sftp"
//! name = "My webspace"
//! host = "example.com"
//! user = "me"
//! path = "/var/www/html"
//! auth = "password"
//! ```
//!
//! Nothing secret is stored here. Passwords and private keys live in the
//! operating system's keychain. The file is still kept apart from
//! `site.toml`, because it names your servers: a site folder may be shared
//! publicly (like qed's own `docs/`), and `targets.toml` can then be left out
//! with a `.gitignore` entry.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::site::SiteError;

pub const TARGETS_FILE: &str = "targets.toml";

/// What a new `targets.toml` starts with.
const HEADER: &str = "\
# Where qed publishes this site. Passwords and keys are in your system
# keychain, never here. This file names your servers, so keep it out of
# public repositories (for example with a .gitignore entry).
";

/// The shape of `targets.toml`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetsFile {
    #[serde(default)]
    targets: Vec<Target>,
}

/// Reads the site's targets. A site without `targets.toml` has none.
pub fn load(root: &Path) -> Result<Vec<Target>, SiteError> {
    let path = root.join(TARGETS_FILE);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path).map_err(|source| SiteError::Io {
        path: path.clone(),
        source,
    })?;
    let file: TargetsFile = toml::from_str(&text).map_err(|source| SiteError::Config {
        path: path.clone(),
        source,
    })?;
    Ok(file.targets)
}

/// One place to publish to. Tagged by `kind`, so more kinds (GitHub Pages)
/// can be added later.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Target {
    Sftp(SftpTarget),
}

impl Target {
    pub fn name(&self) -> &str {
        match self {
            Target::Sftp(sftp) => &sftp.name,
        }
    }
}

/// A server reached over SFTP, such as a webspace or a virtual machine.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct SftpTarget {
    pub name: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub user: String,
    /// The folder on the server the site goes into, e.g. `/var/www/html`.
    pub path: String,
    pub auth: SftpAuth,
    /// The server's key fingerprint (`SHA256:…`), saved when the user first
    /// trusts the server. A different key later means something is wrong.
    pub host_key: Option<String>,
    /// Delete files from the server that earlier publishes put there and the
    /// site no longer has. Files qed didn't upload are never touched.
    #[serde(default)]
    pub delete_stale: bool,
}

/// How qed logs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum SftpAuth {
    Password,
    /// A key pair qed generated; the public key goes on the server.
    Key,
}

fn default_port() -> u16 {
    22
}

impl SftpTarget {
    /// Identifies the login in the keychain. Doesn't include the site, so
    /// moving the site folder keeps the saved password.
    pub fn keychain_account(&self) -> String {
        format!("sftp:{}@{}:{}", self.user, self.host, self.port)
    }
}

/// Replaces the targets in `targets.toml`, leaving comments in the file as
/// they were. Creates the file if needed.
pub fn save_targets(root: &Path, targets: &[Target]) -> Result<(), SiteError> {
    let path = root.join(TARGETS_FILE);
    let original = if path.is_file() {
        std::fs::read_to_string(&path).map_err(|source| SiteError::Io {
            path: path.clone(),
            source,
        })?
    } else {
        HEADER.to_string()
    };
    let updated = with_targets(&original, targets).map_err(|message| SiteError::Invalid {
        path: path.clone(),
        message,
    })?;
    crate::site::write_atomically(&path, &updated)
}

/// The text of `targets.toml` with its targets replaced.
///
/// The comment block at the top of the file is kept as it is; the targets
/// below it are written fresh, so comments inside a target don't survive a
/// change made in the app.
fn with_targets(original: &str, targets: &[Target]) -> Result<String, String> {
    let preamble: String = original
        .lines()
        .take_while(|line| line.trim().is_empty() || line.trim_start().starts_with('#'))
        .map(|line| format!("{line}\n"))
        .collect();
    let preamble = match preamble.trim() {
        "" => HEADER.to_string(),
        text => format!("{text}\n"),
    };
    if targets.is_empty() {
        return Ok(preamble);
    }

    #[derive(Serialize)]
    struct Wrapper<'a> {
        targets: &'a [Target],
    }
    let mut document =
        toml_edit::ser::to_document(&Wrapper { targets }).map_err(|e| e.to_string())?;
    // Written as `[[targets]]` sections, which read better than one long
    // inline list.
    if let Some(item) = document.remove("targets") {
        let item = item
            .into_array_of_tables()
            .map(toml_edit::Item::ArrayOfTables)
            .unwrap_or_else(|item| item);
        document.insert("targets", item);
    }
    Ok(format!("{preamble}\n{document}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site;

    fn webspace() -> Target {
        Target::Sftp(SftpTarget {
            name: "Webspace".into(),
            host: "example.com".into(),
            port: 22,
            user: "me".into(),
            path: "/var/www/html".into(),
            auth: SftpAuth::Password,
            host_key: None,
            delete_stale: false,
        })
    }

    #[test]
    fn reads_targets_from_targets_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("site.toml"), "title = \"S\"\n").unwrap();
        assert!(
            site::open(dir.path()).unwrap().targets.is_empty(),
            "no file, no targets"
        );

        std::fs::write(
            dir.path().join("targets.toml"),
            "[[targets]]\nkind = \"sftp\"\nname = \"Webspace\"\nhost = \"example.com\"\nuser = \"me\"\npath = \"/var/www/html\"\nauth = \"password\"\n",
        )
        .unwrap();
        assert_eq!(site::open(dir.path()).unwrap().targets, [webspace()]);
    }

    #[test]
    fn creates_targets_toml_with_a_warning_and_keeps_its_comments() {
        let dir = tempfile::tempdir().unwrap();
        save_targets(dir.path(), &[webspace()]).unwrap();
        let text = std::fs::read_to_string(dir.path().join("targets.toml")).unwrap();
        assert!(
            text.starts_with("# Where qed publishes this site."),
            "{text}"
        );
        assert!(
            text.contains("[[targets]]\nkind = \"sftp\"\nname = \"Webspace\""),
            "{text}"
        );
        assert_eq!(load(dir.path()).unwrap(), [webspace()]);

        // A comment someone wrote at the top survives saving.
        let edited = format!("# My servers\n\n{text}");
        std::fs::write(dir.path().join("targets.toml"), edited).unwrap();
        save_targets(dir.path(), &[webspace()]).unwrap();
        let text = std::fs::read_to_string(dir.path().join("targets.toml")).unwrap();
        assert!(
            text.starts_with("# My servers\n\n# Where qed publishes"),
            "{text}"
        );

        // Removing all targets leaves just the comment.
        save_targets(dir.path(), &[]).unwrap();
        assert!(
            !std::fs::read_to_string(dir.path().join("targets.toml"))
                .unwrap()
                .contains("[[targets]]")
        );
        std::fs::remove_file(dir.path().join("targets.toml")).unwrap();
        save_targets(dir.path(), &[webspace()]).unwrap();
        save_targets(dir.path(), &[]).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.path().join("targets.toml")).unwrap(),
            HEADER
        );
    }

    #[test]
    fn site_toml_no_longer_takes_targets() {
        let text = "title = \"S\"\n[[targets]]\nkind = \"sftp\"\n";
        assert!(toml::from_str::<site::SiteConfig>(text).is_err());
    }

    #[test]
    fn rejects_unknown_target_kinds_and_fields() {
        let unknown = "[[targets]]\nkind = \"ftp\"\nname = \"x\"\n";
        assert!(toml::from_str::<TargetsFile>(unknown).is_err());
        let typo = "[[targets]]\nkind = \"sftp\"\nname = \"x\"\nhost = \"h\"\nuser = \"u\"\npath = \"/\"\nauth = \"key\"\nhots = \"typo\"\n";
        assert!(toml::from_str::<TargetsFile>(typo).is_err());
    }
}

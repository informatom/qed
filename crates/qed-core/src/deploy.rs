//! Working out what a publish has to upload and delete.
//!
//! Every publish leaves a list of the files it uploaded on the server, with
//! their sizes and SHA-256 checksums: `.qed-manifest.json`. The next publish
//! compares the freshly built site with that list:
//!
//! - files that are new or have a different checksum get uploaded;
//! - files on the list that the site no longer has may be deleted;
//! - everything else stays as it is.
//!
//! Because only listed files are ever deleted, qed never touches files on the
//! server it didn't put there itself.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The name of the list on the server, in the site's top folder.
pub const MANIFEST_FILE: &str = ".qed-manifest.json";

/// Files in the built site that are never uploaded.
const NOT_PUBLISHED: [&str; 2] = [".qed-build", MANIFEST_FILE];

/// The files of a built site, by path (`about/index.html`).
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Manifest {
    pub files: BTreeMap<String, FileEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FileEntry {
    pub size: u64,
    /// Hex-encoded SHA-256 of the contents.
    pub sha256: String,
}

impl Manifest {
    /// Lists and checksums every file in a built site folder.
    pub fn of_folder(folder: &Path) -> io::Result<Manifest> {
        let mut files = BTreeMap::new();
        add_folder(folder, folder, &mut files)?;
        Ok(Manifest { files })
    }

    pub fn to_json(&self) -> String {
        // A map of strings and numbers always serializes.
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Reads a manifest from the server. Anything unreadable counts as "no
    /// manifest", which makes the next publish upload everything.
    pub fn from_json(text: &str) -> Option<Manifest> {
        serde_json::from_str(text).ok()
    }
}

/// What a publish will do.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Plan {
    /// New or changed files, by path.
    pub upload: Vec<String>,
    /// Files an earlier publish uploaded that the site no longer has.
    pub delete: Vec<String>,
    /// How many files are already up to date.
    pub unchanged: usize,
    /// Bytes to upload.
    pub upload_bytes: u64,
}

/// Compares the built site with what the last publish left on the server.
///
/// Stale files are only listed for deletion if `delete_stale` is on.
pub fn plan(local: &Manifest, remote: Option<&Manifest>, delete_stale: bool) -> Plan {
    let empty = Manifest::default();
    let remote = remote.unwrap_or(&empty);
    let mut plan = Plan::default();

    for (path, entry) in &local.files {
        if remote.files.get(path) == Some(entry) {
            plan.unchanged += 1;
        } else {
            plan.upload.push(path.clone());
            plan.upload_bytes += entry.size;
        }
    }
    if delete_stale {
        plan.delete = remote
            .files
            .keys()
            .filter(|path| !local.files.contains_key(*path))
            .cloned()
            .collect();
    }
    plan
}

fn add_folder(root: &Path, dir: &Path, files: &mut BTreeMap<String, FileEntry>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            add_folder(root, &path, files)?;
            continue;
        }
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        let name = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        if NOT_PUBLISHED.contains(&name.as_str()) {
            continue;
        }
        files.insert(name, checksum(&path)?);
    }
    Ok(())
}

/// Reads a file in chunks, so large files (audio, later) don't have to fit
/// in memory at once.
fn checksum(path: &Path) -> io::Result<FileEntry> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut size = 0u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size += read as u64;
    }
    let sha256 = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok(FileEntry { size, sha256 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(size: u64, sha256: &str) -> FileEntry {
        FileEntry {
            size,
            sha256: sha256.into(),
        }
    }

    fn manifest(files: &[(&str, FileEntry)]) -> Manifest {
        Manifest {
            files: files
                .iter()
                .map(|(p, e)| (p.to_string(), e.clone()))
                .collect(),
        }
    }

    #[test]
    fn checksums_a_built_folder() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("about")).unwrap();
        fs::write(dir.path().join("index.html"), "abc").unwrap();
        fs::write(dir.path().join("about/index.html"), "").unwrap();
        fs::write(dir.path().join(".qed-build"), "").unwrap();

        let manifest = Manifest::of_folder(dir.path()).unwrap();
        let paths: Vec<&str> = manifest.files.keys().map(String::as_str).collect();
        assert_eq!(paths, ["about/index.html", "index.html"]);
        // The well-known SHA-256 of "abc".
        assert_eq!(
            manifest.files["index.html"],
            entry(
                3,
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            )
        );
    }

    #[test]
    fn uploads_everything_the_first_time() {
        let local = manifest(&[("a.html", entry(1, "x")), ("b.css", entry(2, "y"))]);
        let plan = plan(&local, None, true);
        assert_eq!(plan.upload, ["a.html", "b.css"]);
        assert_eq!(plan.upload_bytes, 3);
        assert!(plan.delete.is_empty());
    }

    #[test]
    fn uploads_only_new_and_changed_files() {
        let local = manifest(&[
            ("same.html", entry(1, "x")),
            ("changed.html", entry(1, "new")),
            ("new.html", entry(1, "z")),
        ]);
        let remote = manifest(&[
            ("same.html", entry(1, "x")),
            ("changed.html", entry(1, "old")),
            ("gone.html", entry(1, "g")),
        ]);

        let plan = plan(&local, Some(&remote), false);
        assert_eq!(plan.upload, ["changed.html", "new.html"]);
        assert_eq!(plan.unchanged, 1);
        assert!(plan.delete.is_empty(), "deleting is off");

        let plan = super::plan(&local, Some(&remote), true);
        assert_eq!(plan.delete, ["gone.html"]);
    }

    #[test]
    fn round_trips_through_json() {
        let original = manifest(&[("a.html", entry(1, "x"))]);
        assert_eq!(Manifest::from_json(&original.to_json()), Some(original));
        assert_eq!(Manifest::from_json("not json"), None);
    }
}

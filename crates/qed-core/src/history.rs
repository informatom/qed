//! A site's version history, kept with git.
//!
//! Every save in the app records the site folder in an *open version*: the
//! first save after a version was closed starts one (a commit marked
//! [`DRAFT_MARK`]), and later saves amend it. "Save version" closes it with
//! a note; publishing closes it and tags it. So every saved state can be
//! brought back, and the history still has one version per piece of work,
//! not one per click. Any page can be read back as it was in an earlier
//! version.
//!
//! The site folder can be a repository of its own, or a folder inside a
//! larger one (qed's own `docs/` is inside the qed repository). Either way,
//! qed only ever commits files inside the site folder: other changes in the
//! repository, staged or not, stay out of its commits. The built site
//! (`_site/`) and `targets.toml` (it names servers) are never committed.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use git2::build::TreeUpdateBuilder;
use git2::{
    DiffOptions, ErrorCode, FileMode, IndexAddOption, Oid, Patch, Repository, Signature,
    StatusOptions, TreeWalkMode, TreeWalkResult,
};
use serde::Serialize;

use crate::generator::OUTPUT_DIR;
use crate::page::{self, PageContent};
use crate::slug::slugify;
use crate::target::TARGETS_FILE;

/// The last line of an open version's commit message. Only commits with it
/// are ever amended.
pub const DRAFT_MARK: &str = "qed-draft: true";

/// The note of an open version.
const DRAFT_NOTE: &str = "Work in progress";

/// How many versions a list shows at most.
const MAX_VERSIONS: usize = 200;

#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("this site has no version history yet")]
    NotTracked,

    #[error("git: {}", .0.message())]
    Git(#[from] git2::Error),

    #[error("can't write {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("{0} exists; move it away to bring back the deleted page")]
    Exists(String),

    #[error("publishing needs a note: what's new on the site?")]
    NoteNeeded,

    #[error("{0} isn't in that version")]
    Missing(String),

    #[error("{file} in version {version} isn't a readable page: {message}")]
    BadPage {
        file: String,
        version: String,
        message: String,
    },
}

/// What the Site tab shows about the history.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct HistoryStatus {
    /// Whether the site folder is in a git repository at all.
    pub tracked: bool,
    /// The repository's folder, if it's larger than the site folder.
    pub shared_with: Option<PathBuf>,
    /// Files changed since the last version, relative to the site folder.
    pub changes: Vec<String>,
    /// There's an open version: saves since the last closed one.
    pub open: bool,
}

impl HistoryStatus {
    /// Whether publishing would save something new, and so needs a note:
    /// there's an open version to close, or files changed since the last
    /// version.
    pub fn needs_note(&self) -> bool {
        self.tracked && (self.open || !self.changes.is_empty())
    }
}

/// One saved version.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Version {
    /// The commit id, in full.
    pub id: String,
    /// The first line of the note.
    pub summary: String,
    pub author: String,
    /// Seconds since 1970 (UTC).
    pub time: i64,
    /// Tags on this version, such as the publishes.
    pub tags: Vec<String>,
    /// Still open: later saves go into it until it's closed.
    pub open: bool,
}

/// A page that a version had and the site folder doesn't any more.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DeletedPage {
    /// The file, relative to the site folder: `pages/about.md`.
    pub file: String,
    pub title: String,
    /// The last version that still had it.
    pub version: String,
    /// When the version without it was saved; `None` if that hasn't
    /// happened yet (deleted since the last version).
    pub deleted_at: Option<i64>,
}

/// One line of a comparison between two versions of a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DiffLine {
    pub kind: DiffKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum DiffKind {
    Added,
    Removed,
    /// Unchanged, shown around the changes.
    Same,
    /// Unchanged lines left out between two changes.
    Gap,
}

/// The repository a site folder is in, and where the folder is inside it.
struct SiteRepo {
    repo: Repository,
    /// The site folder relative to the repository, with `/` at the end;
    /// empty if the repository is the site folder.
    prefix: String,
}

impl SiteRepo {
    /// `None` if the folder isn't in a repository.
    fn open(root: &Path) -> Result<Option<SiteRepo>, HistoryError> {
        let repo = match Repository::discover(root) {
            Ok(repo) => repo,
            Err(error) if error.code() == ErrorCode::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let Some(workdir) = repo.workdir() else {
            return Ok(None);
        };
        let canonical = |path: &Path| {
            fs::canonicalize(path).map_err(|source| HistoryError::Io {
                path: path.to_path_buf(),
                source,
            })
        };
        let (workdir, root) = (canonical(workdir)?, canonical(root)?);
        let relative = root.strip_prefix(&workdir).unwrap_or(Path::new(""));
        let mut prefix: String = relative
            .components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        if !prefix.is_empty() {
            prefix.push('/');
        }
        Ok(Some(SiteRepo { repo, prefix }))
    }

    /// A path inside the site folder, as git names it.
    fn git_path(&self, site_path: &str) -> String {
        // Windows paths come with `\`; git always uses `/`.
        let site_path = site_path.replace('\\', "/");
        format!("{}{}", self.prefix, site_path.trim_start_matches('/'))
    }

    /// The pathspec that covers the site folder.
    fn spec(&self) -> String {
        if self.prefix.is_empty() {
            "*".to_string()
        } else {
            self.prefix.trim_end_matches('/').to_string()
        }
    }

    /// Paths qed never commits, relative to the repository.
    fn excluded(&self, git_path: &str) -> bool {
        let Some(inside) = git_path.strip_prefix(self.prefix.as_str()) else {
            return true;
        };
        inside == TARGETS_FILE || inside.starts_with(&format!("{OUTPUT_DIR}/"))
    }
}

/// Whether the site has a history, and what changed since the last version.
pub fn status(root: &Path) -> Result<HistoryStatus, HistoryError> {
    let Some(site) = SiteRepo::open(root)? else {
        return Ok(HistoryStatus {
            tracked: false,
            shared_with: None,
            changes: Vec::new(),
            open: false,
        });
    };
    let mut options = StatusOptions::new();
    options
        .pathspec(site.spec())
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false);
    let mut changes: Vec<String> = site
        .repo
        .statuses(Some(&mut options))?
        .iter()
        .filter_map(|entry| entry.path().ok().map(str::to_string))
        .filter(|path| !site.excluded(path))
        .filter_map(|path| path.strip_prefix(site.prefix.as_str()).map(str::to_string))
        .collect();
    changes.sort();
    changes.dedup();

    let shared_with = (!site.prefix.is_empty())
        .then(|| site.repo.workdir().map(Path::to_path_buf))
        .flatten();
    let open = open_version(&site)?.is_some();
    Ok(HistoryStatus {
        tracked: true,
        shared_with,
        changes,
        open,
    })
}

/// Makes the site folder a git repository and saves a first version.
/// `.gitignore` gets the built site and `targets.toml`, so plain `git`
/// leaves them out too.
pub fn start(root: &Path) -> Result<Option<Version>, HistoryError> {
    if SiteRepo::open(root)?.is_none() {
        Repository::init(root)?;
    }
    let ignore = root.join(".gitignore");
    let existing = fs::read_to_string(&ignore).unwrap_or_default();
    let mut text = existing.clone();
    for line in [format!("{OUTPUT_DIR}/"), TARGETS_FILE.to_string()] {
        if !existing.lines().any(|l| l.trim() == line) {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&line);
            text.push('\n');
        }
    }
    if text != existing {
        fs::write(&ignore, text).map_err(|source| HistoryError::Io {
            path: ignore.clone(),
            source,
        })?;
    }
    save_version(root, "First version")
}

/// Records the site folder after a save in the app: in the open version
/// if there is one, otherwise in a new one. Returns `None` for a site
/// without history, or if nothing changed.
pub fn record_save(root: &Path) -> Result<Option<Version>, HistoryError> {
    let Some(site) = SiteRepo::open(root)? else {
        return Ok(None);
    };
    let tree = site_tree(&site)?;
    let repo = &site.repo;
    let signature = signature(repo)?;
    let id = match open_version(&site)? {
        Some(open) if open.tree_id() == tree.id() => return Ok(None),
        Some(open) => open.amend(
            Some("HEAD"),
            None,
            Some(&signature),
            None,
            None,
            Some(&tree),
        )?,
        None => {
            let head = head_commit(repo);
            if head
                .as_ref()
                .is_some_and(|commit| commit.tree_id() == tree.id())
            {
                return Ok(None);
            }
            let message = format!("{DRAFT_NOTE}\n\n{DRAFT_MARK}\n");
            let parents: Vec<&git2::Commit> = head.iter().collect();
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &message,
                &tree,
                &parents,
            )?
        }
    };
    Ok(Some(version(repo, id, &BTreeMap::new())?))
}

/// Closes the open version with `note`, after recording the site folder
/// in it. Without an open version, saves the folder as a new version.
/// Returns `None` if there was nothing to save.
pub fn save_version(root: &Path, note: &str) -> Result<Option<Version>, HistoryError> {
    let site = SiteRepo::open(root)?.ok_or(HistoryError::NotTracked)?;
    let tree = site_tree(&site)?;
    let repo = &site.repo;
    let signature = signature(repo)?;
    let note = if note.trim().is_empty() {
        "Saved version"
    } else {
        note.trim()
    };
    let id = match open_version(&site)? {
        Some(open) => open.amend(
            Some("HEAD"),
            None,
            Some(&signature),
            None,
            Some(note),
            Some(&tree),
        )?,
        None => {
            let head = head_commit(repo);
            if head
                .as_ref()
                .is_some_and(|commit| commit.tree_id() == tree.id())
            {
                return Ok(None);
            }
            let parents: Vec<&git2::Commit> = head.iter().collect();
            repo.commit(Some("HEAD"), &signature, &signature, note, &tree, &parents)?
        }
    };
    Ok(Some(version(repo, id, &BTreeMap::new())?))
}

/// Gets the history ready for publishing: closes the open version (or
/// saves the changed files as a new version) with `note`, the note on
/// what's new on the site. A note is required then; without anything new
/// the last version is published as it is and `note` isn't used. Does
/// nothing for a site without history.
pub fn close_for_publish(root: &Path, note: Option<&str>) -> Result<Option<Version>, HistoryError> {
    if !status(root)?.needs_note() {
        return Ok(None);
    }
    match note.map(str::trim) {
        Some(note) if !note.is_empty() => save_version(root, note),
        _ => Err(HistoryError::NoteNeeded),
    }
}

/// The open version, if the latest commit is one and may still change:
/// qed made it (it has [`DRAFT_MARK`]) and it hasn't been pushed anywhere.
/// The latest commit may well be someone else's when the site shares a
/// repository; that one is never touched.
fn open_version(site: &SiteRepo) -> Result<Option<git2::Commit<'_>>, HistoryError> {
    let repo = &site.repo;
    let Some(head) = head_commit(repo) else {
        return Ok(None);
    };
    if !is_open(&head) {
        return Ok(None);
    }
    for reference in repo.references_glob("refs/remotes/*")? {
        let Some(pushed) = reference?.target() else {
            continue;
        };
        if pushed == head.id() || repo.graph_descendant_of(pushed, head.id())? {
            return Ok(None);
        }
    }
    Ok(Some(head))
}

fn is_open(commit: &git2::Commit) -> bool {
    commit
        .message()
        .is_ok_and(|message| message.lines().any(|line| line.trim() == DRAFT_MARK))
}

fn head_commit(repo: &Repository) -> Option<git2::Commit<'_>> {
    repo.head().ok().and_then(|head| head.peel_to_commit().ok())
}

fn signature(repo: &Repository) -> Result<Signature<'static>, HistoryError> {
    Ok(repo
        .signature()
        .or_else(|_| Signature::now("qed", "qed@localhost"))?)
}

/// Stages the site folder and builds the tree for its next commit: the
/// latest commit's tree with only the site folder replaced, so anything
/// else staged in the repository stays out.
fn site_tree(site: &SiteRepo) -> Result<git2::Tree<'_>, HistoryError> {
    let repo = &site.repo;

    // Stage the site folder: new and changed files, and deletions.
    let mut index = repo.index()?;
    let mut skip = |path: &Path, _: &[u8]| -> i32 {
        i32::from(site.excluded(&path.to_string_lossy().replace('\\', "/")))
    };
    index.add_all([site.spec()], IndexAddOption::DEFAULT, Some(&mut skip))?;
    index.update_all([site.spec()], Some(&mut skip))?;
    index.write()?;

    let base = match head_commit(repo) {
        Some(commit) => commit.tree()?,
        None => repo.find_tree(repo.treebuilder(None)?.write()?)?,
    };
    let mut update = TreeUpdateBuilder::new();
    let mut staged = BTreeSet::new();
    for entry in index.iter() {
        let path = String::from_utf8_lossy(&entry.path).to_string();
        // The upper bits of `flags` hold the merge stage; 0 means no conflict.
        if !path.starts_with(site.prefix.as_str())
            || site.excluded(&path)
            || entry.flags & 0x3000 != 0
        {
            continue;
        }
        update.upsert(path.as_str(), entry.id, file_mode(entry.mode));
        staged.insert(path);
    }
    base.walk(TreeWalkMode::PreOrder, |dir, entry| {
        let path = format!("{dir}{}", entry.name().unwrap_or_default());
        if entry.kind() != Some(git2::ObjectType::Tree)
            && path.starts_with(site.prefix.as_str())
            && !site.excluded(&path)
            && !staged.contains(&path)
        {
            update.remove(path.as_str());
        }
        TreeWalkResult::Ok
    })?;
    let id = update.create_updated(repo, &base)?;
    Ok(repo.find_tree(id)?)
}

/// Tags the current version as published to `target`. Does nothing for a
/// site without history.
pub fn tag_publish(root: &Path, target: &str) -> Result<Option<String>, HistoryError> {
    let Some(site) = SiteRepo::open(root)? else {
        return Ok(None);
    };
    let Ok(head) = site.repo.head().and_then(|head| head.peel_to_commit()) else {
        return Ok(None);
    };
    let target_slug = match slugify(target) {
        slug if slug.is_empty() => "target".to_string(),
        slug => slug,
    };
    let name = format!(
        "{}publish/{target_slug}/{}",
        site.prefix,
        utc_stamp(now_seconds())
    );
    let signature = site
        .repo
        .signature()
        .or_else(|_| Signature::now("qed", "qed@localhost"))?;
    site.repo.tag(
        &name,
        head.as_object(),
        &signature,
        &format!("Published to {target}"),
        false,
    )?;
    Ok(Some(name))
}

/// The versions in which the site folder changed (or `file`, a path inside
/// the site folder, if given), newest first.
pub fn versions(root: &Path, file: Option<&str>) -> Result<Vec<Version>, HistoryError> {
    let Some(site) = SiteRepo::open(root)? else {
        return Ok(Vec::new());
    };
    let repo = &site.repo;
    let path = match file {
        Some(file) => site.git_path(file),
        None => site.prefix.trim_end_matches('/').to_string(),
    };
    let tags = tags_by_commit(repo)?;

    let mut walk = repo.revwalk()?;
    if walk.push_head().is_err() {
        return Ok(Vec::new()); // No version yet.
    }
    walk.set_sorting(git2::Sort::TIME)?;
    let mut found = Vec::new();
    for id in walk {
        let commit = repo.find_commit(id?)?;
        let now = entry_id(&commit.tree()?, &path);
        let before = match commit.parent(0) {
            Ok(parent) => entry_id(&parent.tree()?, &path),
            Err(_) => None,
        };
        if now != before {
            found.push(version(repo, commit.id(), &tags)?);
            if found.len() == MAX_VERSIONS {
                break;
            }
        }
    }
    Ok(found)
}

/// A page as it was in `version`, ready for the editor.
pub fn page_at(root: &Path, file: &str, version: &str) -> Result<PageContent, HistoryError> {
    let text = file_at(root, file, version)?.unwrap_or_default();
    page::parse(&text).map_err(|error| HistoryError::BadPage {
        file: file.to_string(),
        version: version.chars().take(8).collect(),
        message: error.message,
    })
}

/// What `version` changed in `file`: its lines compared with the version
/// before.
pub fn changes_in(root: &Path, file: &str, version: &str) -> Result<Vec<DiffLine>, HistoryError> {
    let site = SiteRepo::open(root)?.ok_or(HistoryError::NotTracked)?;
    let repo = &site.repo;
    let commit = repo.find_commit(Oid::from_str(version)?)?;
    let path = site.git_path(file);
    let blob_in = |tree: &git2::Tree| entry_id(tree, &path).and_then(|id| repo.find_blob(id).ok());
    let new = blob_in(&commit.tree()?);
    let old = match commit.parent(0) {
        Ok(parent) => blob_in(&parent.tree()?),
        Err(_) => None,
    };

    let mut options = DiffOptions::new();
    options.context_lines(2);
    // A file that didn't exist on one side compares as empty.
    let old_text = old.as_ref().map_or(&[][..], |blob| blob.content());
    let new_text = new.as_ref().map_or(&[][..], |blob| blob.content());
    let patch = Patch::from_buffers(old_text, None, new_text, None, Some(&mut options))?;
    let mut lines = Vec::new();
    for hunk in 0..patch.num_hunks() {
        if hunk > 0 {
            lines.push(DiffLine {
                kind: DiffKind::Gap,
                text: String::new(),
            });
        }
        for index in 0..patch.num_lines_in_hunk(hunk)? {
            let line = patch.line_in_hunk(hunk, index)?;
            let kind = match line.origin() {
                '+' => DiffKind::Added,
                '-' => DiffKind::Removed,
                ' ' => DiffKind::Same,
                _ => continue, // "no newline at end of file" and the like
            };
            let text = String::from_utf8_lossy(line.content())
                .trim_end_matches(['\n', '\r'])
                .to_string();
            lines.push(DiffLine { kind, text });
        }
    }
    Ok(lines)
}

/// Pages that earlier versions had and the site folder doesn't, most
/// recently deleted first.
pub fn deleted_pages(root: &Path) -> Result<Vec<DeletedPage>, HistoryError> {
    let Some(site) = SiteRepo::open(root)? else {
        return Ok(Vec::new());
    };
    let repo = &site.repo;
    let Ok(head) = repo.head().and_then(|head| head.peel_to_commit()) else {
        return Ok(Vec::new());
    };
    let pages = site.git_path("pages/");
    let workdir = repo.workdir().map(Path::to_path_buf).unwrap_or_default();
    let mut found: Vec<DeletedPage> = Vec::new();
    let mut add = |git_path: &str, version: &git2::Commit, deleted_at: Option<i64>| {
        let gone = !workdir.join(git_path).is_file();
        let file = git_path
            .strip_prefix(site.prefix.as_str())
            .unwrap_or(git_path);
        if gone && git_path.ends_with(".md") && !found.iter().any(|page| page.file == file) {
            let title = version
                .tree()
                .ok()
                .and_then(|tree| entry_id(&tree, git_path))
                .and_then(|id| repo.find_blob(id).ok())
                .and_then(|blob| page::parse(&String::from_utf8_lossy(blob.content())).ok())
                .map_or_else(|| file.to_string(), |content| content.front.title);
            found.push(DeletedPage {
                file: file.to_string(),
                title,
                version: version.id().to_string(),
                deleted_at,
            });
        }
    };

    // Deleted since the last version: still in it, gone from the folder.
    let mut in_head = Vec::new();
    head.tree()?.walk(TreeWalkMode::PreOrder, |dir, entry| {
        let path = format!("{dir}{}", entry.name().unwrap_or_default());
        if path.starts_with(&pages) && entry.kind() == Some(git2::ObjectType::Blob) {
            in_head.push(path);
        }
        TreeWalkResult::Ok
    })?;
    for path in &in_head {
        add(path, &head, None);
    }

    // Deleted in a saved version: there in its parent, gone in it.
    let mut walk = repo.revwalk()?;
    walk.push(head.id())?;
    walk.set_sorting(git2::Sort::TIME)?;
    for id in walk.take(MAX_VERSIONS) {
        let commit = repo.find_commit(id?)?;
        let Ok(parent) = commit.parent(0) else {
            continue;
        };
        let mut options = DiffOptions::new();
        options.pathspec(&pages);
        let diff = repo.diff_tree_to_tree(
            Some(&parent.tree()?),
            Some(&commit.tree()?),
            Some(&mut options),
        )?;
        for delta in diff.deltas() {
            if delta.status() == git2::Delta::Deleted
                && let Some(path) = delta.old_file().path()
            {
                let path = path.to_string_lossy().replace('\\', "/");
                add(&path, &parent, Some(commit.time().seconds()));
            }
        }
    }
    Ok(found)
}

/// Writes a deleted page back into the site folder, as it was in `version`.
pub fn restore_page(root: &Path, file: &str, version: &str) -> Result<(), HistoryError> {
    let text =
        file_at(root, file, version)?.ok_or_else(|| HistoryError::Missing(file.to_string()))?;
    let target = root.join(file);
    if target.exists() {
        return Err(HistoryError::Exists(file.to_string()));
    }
    let io = |source| HistoryError::Io {
        path: target.clone(),
        source,
    };
    if let Some(folder) = target.parent() {
        fs::create_dir_all(folder).map_err(io)?;
    }
    fs::write(&target, text).map_err(io)
}

fn file_at(root: &Path, file: &str, version: &str) -> Result<Option<String>, HistoryError> {
    let site = SiteRepo::open(root)?.ok_or(HistoryError::NotTracked)?;
    let commit = site.repo.find_commit(Oid::from_str(version)?)?;
    let Some(id) = entry_id(&commit.tree()?, &site.git_path(file)) else {
        return Ok(None);
    };
    let blob = site.repo.find_blob(id)?;
    Ok(Some(String::from_utf8_lossy(blob.content()).to_string()))
}

/// The id of the file or folder at `path` in a tree; the tree's own id for
/// `""`.
fn entry_id(tree: &git2::Tree, path: &str) -> Option<Oid> {
    if path.is_empty() {
        Some(tree.id())
    } else {
        tree.get_path(Path::new(path)).ok().map(|entry| entry.id())
    }
}

fn version(
    repo: &Repository,
    id: Oid,
    tags: &BTreeMap<Oid, Vec<String>>,
) -> Result<Version, HistoryError> {
    let commit = repo.find_commit(id)?;
    Ok(Version {
        open: is_open(&commit),
        id: id.to_string(),
        summary: commit
            .summary()
            .ok()
            .flatten()
            .unwrap_or_default()
            .to_string(),
        author: commit.author().name().unwrap_or_default().to_string(),
        time: commit.time().seconds(),
        tags: tags.get(&id).cloned().unwrap_or_default(),
    })
}

/// Every tag name, by the commit it points to.
fn tags_by_commit(repo: &Repository) -> Result<BTreeMap<Oid, Vec<String>>, HistoryError> {
    let mut tags: BTreeMap<Oid, Vec<String>> = BTreeMap::new();
    for name in repo
        .tag_names(None)?
        .iter()
        .filter_map(|name| name.ok().flatten())
    {
        if let Ok(commit) = repo
            .revparse_single(&format!("refs/tags/{name}"))
            .and_then(|object| object.peel_to_commit())
        {
            tags.entry(commit.id()).or_default().push(name.to_string());
        }
    }
    Ok(tags)
}

fn file_mode(mode: u32) -> FileMode {
    match mode {
        0o100755 => FileMode::BlobExecutable,
        0o120000 => FileMode::Link,
        0o160000 => FileMode::Commit,
        _ => FileMode::Blob,
    }
}

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |time| i64::try_from(time.as_secs()).unwrap_or(0))
}

/// `2026-10-21-143005` for a time in seconds since 1970, in UTC. Tag names
/// can't contain `:`.
fn utc_stamp(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let rest = seconds.rem_euclid(86_400);
    // Converts days since 1970 into a calendar date; see
    // https://howardhinnant.github.io/date_algorithms.html#civil_from_days
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}-{:02}{:02}{:02}",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site(dir: &Path) {
        fs::create_dir_all(dir.join("pages")).unwrap();
        fs::write(dir.join("site.toml"), "title = \"S\"\n").unwrap();
        fs::write(
            dir.join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\nHello.\n",
        )
        .unwrap();
        fs::write(dir.join(TARGETS_FILE), "# servers\n").unwrap();
        fs::create_dir_all(dir.join("_site")).unwrap();
        fs::write(dir.join("_site/index.html"), "built").unwrap();
    }

    #[test]
    fn formats_tag_times() {
        assert_eq!(utc_stamp(0), "1970-01-01-000000");
        assert_eq!(utc_stamp(1_792_593_005), "2026-10-21-143005");
        assert_eq!(utc_stamp(951_782_400), "2000-02-29-000000");
    }

    #[test]
    fn publishing_needs_a_note_only_for_something_new() {
        let dir = tempfile::tempdir().unwrap();
        site(dir.path());
        // Without a history there's nothing to close.
        assert!(close_for_publish(dir.path(), None).unwrap().is_none());

        start(dir.path()).unwrap();
        assert!(!status(dir.path()).unwrap().needs_note());
        assert!(close_for_publish(dir.path(), None).unwrap().is_none());

        // A save in the app opens a version: now a note is required.
        fs::write(
            dir.path().join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\nHi.\n",
        )
        .unwrap();
        record_save(dir.path()).unwrap();
        let open = status(dir.path()).unwrap();
        assert!(open.open && open.needs_note());
        assert!(matches!(
            close_for_publish(dir.path(), Some("  ")),
            Err(HistoryError::NoteNeeded)
        ));

        let closed = close_for_publish(dir.path(), Some(" New greeting "))
            .unwrap()
            .unwrap();
        assert_eq!(closed.summary, "New greeting");
        assert!(!closed.open);
        assert!(!status(dir.path()).unwrap().needs_note());

        // A hand edit outside the app needs a note too.
        fs::write(dir.path().join("site.toml"), "title = \"T\"\n").unwrap();
        let changed = status(dir.path()).unwrap();
        assert!(!changed.open && changed.needs_note());
        assert!(close_for_publish(dir.path(), None).is_err());
    }

    #[test]
    fn starts_and_saves_versions_of_its_own_folder() {
        let dir = tempfile::tempdir().unwrap();
        site(dir.path());
        assert!(!status(dir.path()).unwrap().tracked);

        let first = start(dir.path()).unwrap().unwrap();
        assert_eq!(first.summary, "First version");
        let after = status(dir.path()).unwrap();
        assert!(after.tracked && after.shared_with.is_none());
        assert!(after.changes.is_empty(), "{:?}", after.changes);

        let repo = Repository::open(dir.path()).unwrap();
        let tree = repo.head().unwrap().peel_to_tree().unwrap();
        assert!(tree.get_path(Path::new("pages/index.md")).is_ok());
        assert!(tree.get_path(Path::new(".gitignore")).is_ok());
        assert!(
            tree.get_path(Path::new(TARGETS_FILE)).is_err(),
            "never commits targets"
        );
        assert!(
            tree.get_path(Path::new("_site")).is_err(),
            "never commits the build"
        );

        assert!(save_version(dir.path(), "Nothing").unwrap().is_none());

        fs::write(
            dir.path().join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\nHello again.\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("pages/about.md"),
            "+++\ntitle = \"About\"\n+++\n",
        )
        .unwrap();
        assert_eq!(
            status(dir.path()).unwrap().changes,
            ["pages/about.md", "pages/index.md"]
        );
        let second = save_version(dir.path(), "  New text  ").unwrap().unwrap();
        assert_eq!(second.summary, "New text");

        let home = versions(dir.path(), Some("pages/index.md")).unwrap();
        assert_eq!(home.len(), 2);
        assert_eq!(
            versions(dir.path(), Some("pages/about.md")).unwrap().len(),
            1
        );
        assert_eq!(versions(dir.path(), None).unwrap().len(), 2);

        let old = page_at(dir.path(), "pages/index.md", &first.id).unwrap();
        assert!(
            matches!(&old.blocks[0], crate::block::Block::Text { markdown } if markdown.contains("Hello."))
        );
        let diff = changes_in(dir.path(), "pages/index.md", &second.id).unwrap();
        assert!(
            diff.contains(&DiffLine {
                kind: DiffKind::Removed,
                text: "Hello.".into()
            }),
            "{diff:?}"
        );
        assert!(
            diff.contains(&DiffLine {
                kind: DiffKind::Added,
                text: "Hello again.".into()
            }),
            "{diff:?}"
        );

        fs::remove_file(dir.path().join("pages/about.md")).unwrap();
        save_version(dir.path(), "Remove about").unwrap().unwrap();
        let tree = repo.head().unwrap().peel_to_tree().unwrap();
        assert!(
            tree.get_path(Path::new("pages/about.md")).is_err(),
            "deletions count"
        );

        let tag = tag_publish(dir.path(), "My Server").unwrap().unwrap();
        assert!(tag.starts_with("publish/my-server/"), "{tag}");
        assert_eq!(versions(dir.path(), None).unwrap()[0].tags, [tag]);
    }

    #[test]
    fn saves_go_into_one_open_version_until_it_is_closed() {
        let dir = tempfile::tempdir().unwrap();
        site(dir.path());
        start(dir.path()).unwrap();
        let repo = Repository::open(dir.path()).unwrap();
        let count = || versions(dir.path(), None).unwrap().len();
        assert_eq!(count(), 1);

        let home = dir.path().join("pages/index.md");
        fs::write(&home, "+++\ntitle = \"Home\"\n+++\n\nOne.\n").unwrap();
        let open = record_save(dir.path()).unwrap().unwrap();
        assert!(open.open);
        assert_eq!(open.summary, "Work in progress");
        fs::write(&home, "+++\ntitle = \"Home\"\n+++\n\nTwo.\n").unwrap();
        record_save(dir.path()).unwrap().unwrap();
        assert!(record_save(dir.path()).unwrap().is_none(), "nothing new");
        assert_eq!(count(), 2, "the second save amended the open version");

        let closed = save_version(dir.path(), "New intro").unwrap().unwrap();
        assert!(!closed.open);
        assert_eq!(count(), 2, "closing doesn't add a version");
        let text = file_at(dir.path(), "pages/index.md", &closed.id)
            .unwrap()
            .unwrap();
        assert!(text.contains("Two."));

        fs::write(&home, "+++\ntitle = \"Home\"\n+++\n\nThree.\n").unwrap();
        record_save(dir.path()).unwrap().unwrap();
        assert_eq!(count(), 3, "a closed version stays as it is");

        // Once pushed, the open version isn't changed any more.
        let head = repo.head().unwrap().target().unwrap();
        repo.reference("refs/remotes/origin/main", head, true, "pushed")
            .unwrap();
        fs::write(&home, "+++\ntitle = \"Home\"\n+++\n\nFour.\n").unwrap();
        record_save(dir.path()).unwrap().unwrap();
        assert_eq!(count(), 4);
        let pushed = file_at(dir.path(), "pages/index.md", &head.to_string())
            .unwrap()
            .unwrap();
        assert!(pushed.contains("Three."), "the pushed version is unchanged");
    }

    #[test]
    fn a_page_made_and_deleted_in_one_open_version_can_come_back() {
        let dir = tempfile::tempdir().unwrap();
        site(dir.path());
        start(dir.path()).unwrap();
        let page = dir.path().join("pages/new.md");
        fs::write(&page, "+++\ntitle = \"New\"\n+++\n").unwrap();
        record_save(dir.path()).unwrap();

        // What the app does when deleting: close the open version, delete,
        // record the deletion in a new open version.
        save_version(dir.path(), "Before deleting /new").unwrap();
        fs::remove_file(&page).unwrap();
        record_save(dir.path()).unwrap();

        let deleted = deleted_pages(dir.path()).unwrap();
        assert_eq!(deleted.len(), 1);
        assert_eq!(deleted[0].title, "New");
    }

    #[test]
    fn never_amends_someone_elses_commit() {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let docs = dir.path().join("docs");
        site(&docs);
        save_version(&docs, "Docs").unwrap();
        // The owner's own commit on top, as when committing code.
        fs::write(dir.path().join("code.rs"), "fn main() {}\n").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("code.rs")).unwrap();
        index.write().unwrap();
        let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        let signature = Signature::now("Owner", "owner@example.com").unwrap();
        let theirs = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Code",
                &tree,
                &[&parent],
            )
            .unwrap();

        fs::write(
            docs.join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\nNew.\n",
        )
        .unwrap();
        record_save(&docs).unwrap().unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(
            head.parent_id(0).unwrap(),
            theirs,
            "a new commit on top of theirs"
        );
        assert_eq!(
            repo.find_commit(theirs).unwrap().summary().unwrap(),
            Some("Code")
        );
    }

    #[test]
    fn finds_and_restores_deleted_pages() {
        let dir = tempfile::tempdir().unwrap();
        site(dir.path());
        fs::write(
            dir.path().join("pages/about.md"),
            "+++\ntitle = \"About us\"\n+++\n",
        )
        .unwrap();
        fs::create_dir(dir.path().join("pages/blog")).unwrap();
        fs::write(
            dir.path().join("pages/blog/old.md"),
            "+++\ntitle = \"Old post\"\n+++\n",
        )
        .unwrap();
        start(dir.path()).unwrap();
        assert!(deleted_pages(dir.path()).unwrap().is_empty());

        fs::remove_file(dir.path().join("pages/blog/old.md")).unwrap();
        save_version(dir.path(), "Remove the old post").unwrap();
        fs::remove_file(dir.path().join("pages/about.md")).unwrap();

        let deleted = deleted_pages(dir.path()).unwrap();
        let summary: Vec<(&str, &str, bool)> = deleted
            .iter()
            .map(|page| {
                (
                    page.file.as_str(),
                    page.title.as_str(),
                    page.deleted_at.is_some(),
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                ("pages/about.md", "About us", false),
                ("pages/blog/old.md", "Old post", true)
            ]
        );

        let old = &deleted[1];
        restore_page(dir.path(), &old.file, &old.version).unwrap();
        assert!(dir.path().join("pages/blog/old.md").is_file());
        assert!(
            restore_page(dir.path(), &old.file, &old.version).is_err(),
            "won't overwrite"
        );
        assert_eq!(deleted_pages(dir.path()).unwrap().len(), 1);
    }

    #[test]
    fn commits_only_the_site_folder_of_a_larger_repository() {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("code.rs"), "fn main() {}\n").unwrap();
        let docs = dir.path().join("docs");
        site(&docs);
        // Something the owner staged on purpose, outside the site.
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("code.rs")).unwrap();
        index.write().unwrap();

        let status = status(&docs).unwrap();
        assert!(status.tracked);
        assert!(status.shared_with.is_some());
        assert!(status.changes.contains(&"pages/index.md".to_string()));
        assert!(!status.changes.iter().any(|c| c.contains("code.rs")));

        save_version(&docs, "Docs").unwrap().unwrap();
        let tree = repo.head().unwrap().peel_to_tree().unwrap();
        assert!(tree.get_path(Path::new("docs/pages/index.md")).is_ok());
        assert!(
            tree.get_path(Path::new("code.rs")).is_err(),
            "not part of the site"
        );
        assert!(tree.get_path(Path::new("docs/targets.toml")).is_err());
        assert!(
            !dir.path().join("docs/.git").exists(),
            "no repository inside"
        );

        let tag = tag_publish(&docs, "Test").unwrap().unwrap();
        assert!(tag.starts_with("docs/publish/test/"), "{tag}");
    }
}

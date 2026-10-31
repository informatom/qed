+++
title = "Versions with git"
description = "Versions with git: commits from Rust, a tree that changes one folder, and restoring into the editor"
menu = "main"
weight = 16
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "git", "svelte", "ffi"]
summary = "q.e.d. now keeps versions of a site with git. How a commit is built from blobs and trees, why q.e.d. builds its own tree instead of committing the index, what it means to call a C library from Rust, and how Svelte's {#key} lets the editor start over from an old version."
redirect_from = ["/blog/2026-10-21-versions"]
+++

Sites now have a version history. The new **Versions** tab at the top
starts it, shows what changed, and saves a version with a short note.
Publishing saves one by itself and marks it with the target's name. Each
page has a **Versions** tab too, between Content and Settings: it lists the
versions in which the page changed, shows what changed, and restores an
earlier one.

Under the hood it's git. A site folder becomes an ordinary git repository,
so `git log` in a terminal shows the same history, and nothing is locked
into q.e.d.

## git2: a C library, called from Rust

[git2](https://docs.rs/git2/0.21.0/git2/) wraps
[libgit2](https://libgit2.org/), the C library that many git tools are built
on. Rust can call C functions directly through its
[foreign function interface](https://doc.rust-lang.org/nomicon/ffi.html)
(FFI). The `libgit2-sys` crate declares the C functions, and `git2` puts a
safe Rust API on top, with lifetimes that stop you from using a commit after
its repository is gone and `Result`s instead of C's error codes.

```toml
git2 = { version = "0.21.0", default-features = false, features = ["vendored-libgit2"] }
```

`vendored-libgit2` compiles libgit2 from the C sources bundled with the
crate, so the app doesn't depend on whatever version a computer has
installed. That matters for the installers (#062). `default-features =
false` leaves out cloning and pushing over HTTPS and SSH. q.e.d. only works
with the local repository, so it doesn't need OpenSSL.

The other option was [gix](https://docs.rs/gix), a git implementation in
pure Rust. It reads history well, but staging files and committing would
have taken more code of our own. An Elixir app would most likely run the
`git` command with `System.cmd/3`. That works too, but it needs git to be
installed and means parsing text output.

## What a commit is made of

`crates/qed-core/src/history.rs` works with git's
objects directly, so it helps to know them:

- A **blob** is a file's content.
- A **tree** is a folder: names pointing to blobs and to other trees.
- A **commit** points to one tree (the whole project at that moment), to its
  parent commit, and carries the author, time and note.
- The **index** (the "staging area") is the list of files the next commit
  will contain.

Normally `git add` updates the index and `git commit` turns the index into
a tree and commits it. q.e.d. does the first half the same way:

```rust
index.add_all([site.spec()], IndexAddOption::DEFAULT, Some(&mut skip))?;
index.update_all([site.spec()], Some(&mut skip))?;
```

`add_all` stages new and changed files under the site folder, and
`update_all` stages deletions. `skip` is a callback that leaves out
`_site/` and `targets.toml`, whatever the `.gitignore` says. `targets.toml`
names your servers and must never end up in the history.

## A site folder inside a bigger repository

q.e.d.'s own documentation lives in `docs/`, inside the q.e.d. repository. That's
the case where committing "the index" would be wrong. If you had staged a
Rust file for your next commit, q.e.d.'s "Save version" would commit it too,
under the note "New blog post".

So q.e.d. builds the commit's tree itself: it starts from the last commit's
tree and replaces only what's under the site folder with the staged files.
[`TreeUpdateBuilder`](https://docs.rs/git2/0.21.0/git2/build/struct.TreeUpdateBuilder.html)
does that without unpacking the whole project:

```rust
let mut update = TreeUpdateBuilder::new();
for entry in index.iter() {
    let path = String::from_utf8_lossy(&entry.path).to_string();
    if !path.starts_with(site.prefix.as_str()) || site.excluded(&path) || entry.flags & 0x3000 != 0 {
        continue;
    }
    update.upsert(path.as_str(), entry.id, file_mode(entry.mode));
    staged.insert(path);
}
// … and `remove` for files under the site folder that are gone …
let tree_id = update.create_updated(repo, &base)?;
if head.as_ref().is_some_and(|commit| commit.tree_id() == tree_id) {
    return Ok(None); // nothing changed
}
```

If the new tree is the same as the last commit's tree, nothing changed and
no empty version is made. Because git names every object by a hash of its
content, comparing two ids is enough. `entry.flags & 0x3000` reads two bits
of the index entry that mark a file in the middle of a merge conflict; those
are left alone.

A test sets up exactly this situation: a repository with a staged `code.rs`
and a site in `docs/`. After "Save version", the commit contains
`docs/pages/index.md` and not `code.rs`, which stays staged for the owner's
own next commit.

Publish tags get the folder as a prefix (`docs/publish/test/2026-10-21-143005`),
so several sites in one repository don't mix up their tags.

## Which versions changed a page?

To list a page's versions, q.e.d. walks the commits from newest to oldest and
compares the id of `pages/about.md` in each commit's tree with the id in its
parent's. If they differ, that commit changed the page:

```rust
let now = entry_id(&commit.tree()?, &path);
let before = match commit.parent(0) {
    Ok(parent) => entry_id(&parent.tree()?, &path),
    Err(_) => None,
};
if now != before { /* this version changed the page */ }
```

The whole site's list works the same way, with the site folder's tree
instead of one file. In a shared repository, that also skips the commits
that only touched other files.

What a version changed comes from
[`Patch::from_buffers`](https://docs.rs/git2/0.21.0/git2/struct.Patch.html),
which compares two texts line by line, like `git diff`. q.e.d. turns the result
into a list of lines marked `added`, `removed`, `same` or `gap`, which the
Versions tab colors.

## A date without a date library

Publish tags carry the time, and tag names can't contain `:`. Rust's
standard library has the current time only as seconds since 1970, with no
calendar. Instead of adding a date crate for one line, `utc_stamp` converts
days to year, month and day with
[Howard Hinnant's well-known algorithm](https://howardhinnant.github.io/date_algorithms.html#civil_from_days):
a dozen lines of integer arithmetic. A test checks it against a leap day,
`2000-02-29`. Elixir has this built in as `DateTime.from_unix!/1`.

## Restoring into the editor: {#key}

"Restore" doesn't write the file. It loads the old version
(`page_version` parses it like any page file) and puts it into the editor
as unsaved changes. You see the old text in Content and the old settings in
Settings, can change them further, and save when it looks right. Until
then the file is untouched, and leaving without saving discards the
restore, as with any other edit.

The two editor tabs copy their starting values once, when they're created.
So restoring means creating them again, with a different start. Svelte's
[`{#key}`](https://svelte.dev/docs/svelte/key) block does that: whenever
its value changes, it throws away what's inside and builds it fresh.

```svelte
{#key restoreCount}
  <BlockEditor {page} {siteRoot} {onsaved} start={restored?.blocks} bind:dirty={contentDirty} />
  <PageSettings {page} {siteRoot} {onsaved} start={restored?.front} bind:dirty={settingsDirty} />
{/key}
```

`restore` sets `restored` and increases `restoreCount`. Both editors still
compare against `page`, the saved version, so they show the dot for unsaved
changes on their own. No new mechanism was needed for that.

## Further reading

- [Git internals: git objects](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects),
  the chapter that explains blobs, trees and commits
- [git2 documentation](https://docs.rs/git2/0.21.0/git2/)
- [FFI in the Rustonomicon](https://doc.rust-lang.org/nomicon/ffi.html)
- Svelte: [`{#key}`](https://svelte.dev/docs/svelte/key)

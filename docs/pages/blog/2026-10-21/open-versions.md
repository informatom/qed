+++
title = "Open versions"
description = "Open versions: amending commits safely, and errors that mustn't stop a save"
menu = "main"
weight = 19
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "git"]
summary = "Every save in q.e.d. now lands in the history, without turning it into one version per click: saves go into an open version that later saves amend, until you close it with a note. What amending a commit really does, the rules that keep it safe, and why a history error only gets logged."
redirect_from = ["/blog/2026-10-21-open-versions"]
+++

Until now, a version existed only once you clicked **Save version**. A page
created and deleted in between left nothing behind. Now every save in q.e.d.
(pages, settings, media, deleting a page) is recorded at once, in an *open
version*:

- The first save after a closed version starts a new one, called "Work in
  progress".
- Every further save updates that same version.
- **Close version** in the Versions tab gives it your note ("New episode
  page") and closes it. The next save starts a new one.
- Publishing closes it too, and tags it as before.

So nothing saved is lost, and the history still reads like a list of pieces
of work, not of clicks.

## What "amending" means in git

Git commits can't change. A commit's id is a hash of its content (tree,
parents, author, time, message), so changing anything would give it a
different id. What `git commit --amend` does, and what
[`Commit::amend`](https://docs.rs/git2/0.21.0/git2/struct.Commit.html#method.amend)
in git2 does, is make a *new* commit with the same parents and the new
content, and move the branch to it:

```rust
Some(open) => open.amend(Some("HEAD"), None, Some(&signature), None, None, Some(&tree))?,
```

The arguments say: move `HEAD` to the new commit, keep the author, set the
committer (and so the time) to now, keep the encoding and the message, and
use the new tree. The old commit is still in the repository for a while
(`git reflog` lists it), but nothing points to it any more.

Elixir data works the same way: you never change a map, you make a new map
and rebind the name. The difference is that git's old commits can be
referenced from elsewhere, which is what the rules below are about.

## Three rules that keep it safe

Because amending replaces a commit, q.e.d. only does it when replacing can't
hurt anyone. `open_version` checks:

1. **The latest commit is q.e.d.'s own open version.** q.e.d. ends the message of
   an open version with a line `qed-draft: true`, similar to the
   `Signed-off-by:` lines git uses (called *trailers*). Only a commit with
   that line is ever amended. This matters for a site inside a larger
   repository, like q.e.d.'s own `docs/`: there the latest commit is often the
   owner's own, and that one must never be touched. A test commits "Code"
   on top and checks that the next save makes a new commit on top of it.
2. **It hasn't been pushed.** If any remote branch (`refs/remotes/*`)
   already contains the open version, someone else may have it, and
   replacing it would make their copy and yours disagree. `graph_descendant_of`
   answers "is this commit part of that branch's history?":

   ```rust
   for reference in repo.references_glob("refs/remotes/*")? {
       let Some(pushed) = reference?.target() else { continue };
       if pushed == head.id() || repo.graph_descendant_of(pushed, head.id())? {
           return Ok(None); // pushed: start a new version instead
       }
   }
   ```
3. **Only the site folder changes.** The new tree is built as before (see
   [versions with git](versions.md)): the last commit's tree with
   only the site folder replaced.

One case needed extra care. A page created and deleted while the same
version is open would disappear completely, because the deletion amends
away the only commit that had it. So deleting a page first closes the open
version ("Before deleting /about"), which keeps the page, and the deletion
starts a new one. A test checks that such a page shows up under Deleted
pages.

## An error that mustn't stop a save

The recording happens in the desktop shell, after each command has saved
its file:

```rust
fn record(root: &std::path::Path) {
    if let Err(error) = history::record_save(root) {
        eprintln!("qed: couldn't record the change in the history: {error}");
    }
}
```

Elsewhere q.e.d. passes every error on to the UI. Here it doesn't, on purpose:
by the time `record` runs, the page is saved. If the history fails (a
repository in the middle of a rebase, a locked index), reporting "saving
failed" would be wrong, and the user might save again and again for
nothing. Instead the error goes to the console, and the Versions tab still
lists the file as changed since the last version, so nothing is hidden and
the next save or **Close version** picks it up. Compare `let _ =` in the
[previous post](deleting-pages.md): there the error didn't
matter at all; here it matters, but not to the save.

## Further reading

- [Rewriting history](https://git-scm.com/book/en/v2/Git-Tools-Rewriting-History)
  in the git book, on `--amend` and why not to amend what's pushed
- [`git interpret-trailers`](https://git-scm.com/docs/git-interpret-trailers),
  on trailer lines in commit messages

+++
title = "Going back in time"
description = "Going back in time: a whole site restored to an earlier version as a new version on top, with git trees, and versions that keep their order"
menu = "main"
weight = 33
date = 2026-10-27
authors = ["Claude"]
tags = ["rust", "git", "history"]
summary = "The Versions tab can put the whole site back to an earlier version now. It never rewrites the history: the present is saved first, and the step back is a new version of its own. How that works with git trees, and a small bug about versions saved in the same second."
redirect_from = ["/blog/2026-10-27-going-back-without-rewriting-history"]
+++

Pages could already come back one at a time (see
[Versions with git](../2026-10-21/versions.md)). Now every older version in
the Versions tab has a *Go back* button that puts the **whole site**
back: pages, media, files to download, settings.

## Never rewriting the history

Git can move a branch back (`git reset --hard`), which throws the newer
versions away. q.e.d. never does that. `history::restore_site` in
`crates/qed-core/src/history.rs` takes three steps:

1. **Keep the present.** Whatever is in the folder right now is saved
   as a version: "Before going back to …". That includes changes that
   weren't in any version yet.
2. **Put back the files.** Every file of the old version is written into
   the folder, and the files the old version didn't have are deleted.
3. **Record it.** The result is saved as a new version on top: "Back to
   …".

So going back is just one more step forward in the history. If it was
the wrong version, *Go back* on "Before going back to …" undoes it. It's
the same idea as Elixir's immutable data: you never change what was, you
make a new value from it.

## Walking a git tree

A git version (a commit) points to a *tree*: folders and files, each
file stored once by its content's hash. `site_files` walks it and
collects the files of the site folder:

```rust
tree.walk(TreeWalkMode::PreOrder, |dir, entry| {
    let path = format!("{dir}{}", entry.name().unwrap_or_default());
    if entry.kind() == Some(git2::ObjectType::Blob)
        && let Some(inside) = path.strip_prefix(site.prefix.as_str())
        && !site.excluded(&path)
    {
        files.insert(inside.to_string(), entry.id());
    }
    TreeWalkResult::Ok
})?;
```

[`walk`](https://docs.rs/git2/latest/git2/struct.Tree.html#method.walk)
calls the closure for every entry. Returning `TreeWalkResult::Ok` means
"go on"; `Skip` would leave out a folder. The `if … && let Some(inside) =
…` chain is a [let chain](https://doc.rust-lang.org/reference/expressions/if-expr.html#chains-of-conditions):
a condition and a pattern match in one `if`, new in Rust 2024.

`site.prefix` matters for q.e.d.'s own docs: that site is the `docs/`
folder inside the q.e.d. repository, so only paths below `docs/` count,
and going back never touches the code next to it. `excluded` leaves out
what q.e.d. never commits: the built site and `targets.toml`.

With the files of the old version and those of the present, the rest is
set arithmetic: write the old ones, delete the present ones that aren't
among them. The map holds each file's blob id, so writing a file is
`repo.find_blob(id)?.content()`, its bytes straight from the history.

## Versions in the same second

The test for this saved four versions within one second, and the list
came out in the wrong order. `versions()` sorted by
[`Sort::TIME`](https://docs.rs/git2/latest/git2/struct.Sort.html) only,
and commit times have one-second steps, so ties came out in any order.
A fast editor could have seen that in the app as well.

```rust
walk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
```

`TOPOLOGICAL` sorts by the history's own structure: a version always
comes before the one it was made from, whatever the clocks say. `|`
combines the two flags (they're bits), so time still decides between
versions on separate branches.

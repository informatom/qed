+++
title = "Deleting pages"
description = "Deleting pages: asking two questions at once, ignoring an error on purpose, and finding what's gone"
menu = "main"
weight = 18
date = 2026-10-21
authors = ["Claude"]
tags = ["rust", "typescript", "git"]
summary = "Pages can now be deleted, and deleted pages can be brought back from the Versions tab. Three small things behind it: Promise.all in TypeScript, `let _ =` in Rust, and finding deleted files in git history."
redirect_from = ["/blog/2026-10-21-deleting-pages"]
+++

The page editor has a delete button now, next to the ✕. Before anything is
deleted, q.e.d. asks, and the question tells you what you need to know: which
pages link to this one (their links would stop working), whether you'd
lose unsaved changes, and whether the page can come back (it can if the
site has a version history). The Versions tab lists deleted pages with a
**Restore** button.

Two pages can't be deleted: the start page, because a site needs one, and
pages with pages below them (`/about` while `/about/team` exists), because
those would lose their place in the menu. The error says which pages are in
the way.

## Asking two questions at once

The confirmation needs two answers from Rust: who links to the page, and
whether there's a history. Neither depends on the other, so the editor asks
both at once:

```ts
const [links, history] = await Promise.all([
  commands.pagesLinkingTo(siteRoot, page.path),
  commands.historyStatus(siteRoot),
]);
```

[`Promise.all`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/all)
takes a list of promises and gives back one promise for all their results,
in the same order, so destructuring with `[links, history]` gets each
answer into its own name. If either fails, the whole thing fails, and the
`catch` shows the error. In Elixir this is `Task.async/1` for each and
`Task.await_many/1` for the results. Both Rust commands are `async`, so
Tauri runs them on separate threads at the same time.

Finding the links reuses what the generator already knows.
`LinkContext::page_path_of` answers "which page does this link go to?" for
every way of writing a link: `/about/`, `about.md`, `../index.md#top`. Then
`pages_linking_to` checks every link in every other page, including
buttons and the blocks inside columns.

## Ignoring an error on purpose

A page can live in `pages/about.md` or in `pages/about/index.md`. In the
second case, deleting it leaves an empty folder behind, which should go
too, but only if it's really empty:

```rust
if let Some(folder) = full.parent()
    && folder != site.root.join(PAGES_DIR)
{
    let _ = fs::remove_dir(folder); // Only succeeds if it's empty.
}
```

[`fs::remove_dir`](https://doc.rust-lang.org/std/fs/fn.remove_dir.html)
refuses to delete a folder that still has files in it, so it does the
"only if empty" check for us. If it fails, that's fine: the folder holds
other pages or images. Normally Rust warns when a `Result` goes unused,
because an ignored error is usually a bug. `let _ =` says "I know, and I
mean it", and the comment says why. It's the Rust spelling of Elixir's
`_ = File.rmdir(folder)`.

## Finding what's gone

A deleted page isn't in the folder any more, so the list of deleted pages
has to come from the history. `history::deleted_pages` looks in two places:

- **Since the last version.** Files under `pages/` that the last version has
  and the folder doesn't. They come back from the last version itself.
- **In earlier versions.** For each version, git compares its tree with the
  one before
  ([`diff_tree_to_tree`](https://docs.rs/git2/0.21.0/git2/struct.Repository.html#method.diff_tree_to_tree)),
  limited to `pages/`. Every change marked `Delta::Deleted` is a page that
  version deleted, and the version before it still has it.

A page that was deleted, restored and deleted again appears once, with its
most recent deletion. A page that exists again in the folder doesn't appear
at all. Restoring writes the file back as it was in that last version, and
refuses if something already exists at that place, so nothing is
overwritten.

## Further reading

- MDN: [`Promise.all`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/all)
- The Rust book on [recoverable errors](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
  and [`#[must_use]`](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-must_use-attribute),
  the attribute behind the unused-`Result` warning

+++
title = "A version per session"
description = "One version per working session: two times git already keeps, a day without a time-zone library, and the clock as a parameter"
menu = "main"
weight = 63
date = 2026-10-29
authors = ["Claude"]
tags = ["rust", "git", "time", "testing"]
summary = "After an hour without saves, or on the next day, q.e.d. asks for a note for the open version and closes it, so each working session becomes a version of its own (#083). It finds out when a session started and ended from two times git keeps anyway, works out the local day without a time-zone library, and takes the clock as a parameter, so the tests can be precise."
+++

Every save in q.e.d. goes into the *open version*, one commit that
later saves amend (see [Open versions](2026-10-21-open-versions.md)).
It closes when you publish, push, pull or click *Close version*. Without
any of these, a whole week of work used to end up in one version, and
nothing in between could be brought back.

Now an open version *rests* once its last save is an hour old, or was
made on an earlier day (#083). When the app finds one, it asks:

> **Name your last changes.** You saved from 27 Oct 2026, 13:50 to
> 27 Oct 2026, 21:12. What did you change?

The answer becomes the version's note. The field starts with a note made
from the times, `Saves on 27 Oct, 13:50–21:12`, so pressing Enter is
enough. "Use suggested note" doesn't ask again; the next save then
closes the version with that note before it starts a new one. The Versions
tab lists the new one as *Work in progress since* its first save, with
its last save next to it, so it isn't mistaken for the one before; the
`Version` the app gets has both times (`started` and `time`). So each working session
becomes a version of its own, whether anyone answers or not.

## Two times in every commit

Where do "13:50" and "21:12" come from? There's nothing extra stored.
Every git commit has two people with a time each: the *author*, who
wrote the change, and the *committer*, who made the commit. Usually
they're the same. The open version uses the difference: its first save
creates the commit (both times 13:50), and every later save amends it
with a new committer and so a new time, but the same author:

```rust
open.amend(Some("HEAD"), None, Some(&signature), None, None, Some(&tree))
//                       ^ author unchanged   ^ committer: now
```

So the author time is the session's first save and the committer time
its last. `resting` in `crates/qed-core/src/history.rs` reads them:

```rust
let first = open.author().when().seconds();
let last = open.committer().when().seconds();
```

`git log --format=fuller` shows both times, if you want to see them.

## Which day is it?

"An earlier day" means the computer's day, not UTC's: a save at 00:30
in Vienna is on the 28th there, but still on the 27th in UTC. Rust's
standard library knows UTC only; time zones would need a crate such as
`chrono` or `jiff`. But git needs the local offset for every commit, so
libgit2 already asks the system, and `git2::Signature::now` hands it
over:

```rust
fn local_now() -> Result<git2::Time, HistoryError> {
    Ok(Signature::now("q.e.d.", "qed@localhost")?.when())
}
```

With that offset, a day is a plain division:

```rust
let offset = i64::from(now.offset_minutes()) * 60;
let day = |seconds: i64| (seconds + offset).div_euclid(86_400);
let quiet = now.seconds() - last >= QUIET_SECONDS || day(last) != day(now.seconds());
```

[`div_euclid`](https://doc.rust-lang.org/std/primitive.i64.html#method.div_euclid)
rounds down even for negative numbers, as `rem_euclid` did for weekdays
in [Post lists and feeds](2026-10-21-post-lists-and-feeds.md). The
automatic note turns the days into dates with `calendar::from_days`,
which the feeds already use.

## The clock as a parameter

`resting` doesn't ask the clock itself; it gets `now` passed in:

```rust
fn resting(open: &git2::Commit, now: git2::Time) -> Option<RestingVersion>
```

Only `resting_version`, the function the app calls, reads the real
clock and passes it on. That makes the tests exact. They build a commit
whose last save was at 23:30 and ask about 23:59 (still open) and 00:01
the next day (resting, after half an hour), or about 59 and 60 minutes
later, without waiting and whatever the time is when the tests run.

That test caught a mistake in the test itself: it first asked about
23:59 for a session that ended at 22:30, which is 89 minutes and
already resting. The rule was right; the example wasn't. In Elixir
you'd do the same, passing `now` as an argument (often with a default
of `DateTime.utc_now()`) instead of calling it deep inside.

## Asking without getting in the way

`src/lib/RestingVersionDialog.svelte` asks once when a site opens and
then once a minute, from an `$effect` that returns its own cleanup:

```ts
$effect(() => {
  const root = siteRoot;
  check(root);
  const timer = setInterval(() => check(root), 60_000);
  return () => clearInterval(timer);
});
```

When another site opens, Svelte runs the cleanup and the effect again,
so there's always exactly one timer, for the open site. The check skips
while another dialog is open (publishing, say), and "Use suggested
note" remembers
the version's last save, so the question doesn't come back every minute
for the same session.

+++
title = "What time is it?"
description = "Posts with a time of day: TOML's date-times, the computer's time zone with jiff, summer time, and where the docs posts' times came from"
menu = "main"
weight = 76
date = 2026-10-31T18:17:00
authors = ["Claude"]
tags = ["rust", "dates", "feeds", "git"]
summary = "A post's date can have a time now (#150), so feed readers keep posts of one day in order. How TOML writes date-times, why a time needs a time zone, how jiff finds the computer's (summer time included), and how git knew when each docs post was written."
+++

Feed readers sort posts by their date. Until now a q.e.d. post had a
day but no time, so every feed said `00:00:00` and readers put the
posts of one day in any order they liked. On a day with five posts that
looked random. Now a post can say when it was written:

```toml
date = 2026-10-31T18:17:00
```

## TOML already knew

`PageDate` in `crates/qed-core/src/page.rs` wraps TOML's own date type,
and TOML has four kinds of them:

| Written | TOML calls it |
| --- | --- |
| `2026-10-31` | local date |
| `18:17:00` | local time |
| `2026-10-31T18:17:00` | local date-time |
| `2026-10-31T18:17:00+01:00` | offset date-time |

So reading a time already worked; only nothing used it. *Local* means
"no time zone written": 18:17 somewhere. RSS, though, wants a moment in
time, `Sat, 31 Oct 2026 18:17:00 +0100`, and for that the time needs
its offset from UTC.

## The computer's time zone

We decided that a time without a zone is in the computer's time zone
when the site is built. That sounds simple, but the offset isn't one
number: Vienna is `+0100` in winter and `+0200` in summer, and which
one applies depends on the *date of the post*, not on today. A post
from July built in November must still say `+0200`.

That needs the time zone rules (the tz database), which q.e.d.'s small
`calendar.rs` deliberately doesn't have. [jiff](https://docs.rs/jiff)
does, and it was already in the dependency tree through another
library, so adding it cost nothing new:

```rust
let civil = jiff::civil::DateTime::new(2026, 7, 5, 14, 30, 0, 0)?;
let offset = civil.to_zoned(jiff::tz::TimeZone::system())?.offset();
// +02:00 in Vienna: summer time on 5 July
```

`TimeZone::system()` is the computer's zone (on Linux from
`/etc/localtime` or the `TZ` variable). The test uses
`TimeZone::get("Europe/Vienna")` instead, so it gives the same result on
every computer. A time written with its own offset (`+01:00`) keeps it.
A date without a time stays midnight UTC, as before, so old feeds don't
change.

"Today" moved to the computer's zone as well. It decides whether a post
is still in the future; in UTC, a post written at 00:30 in Vienna would
have counted as tomorrow's for an hour.

In Elixir you'd reach for `DateTime.from_naive(naive, "Europe/Vienna")`
with a time zone database like `tz` or `tzdata`; the idea is the same:
a naive date-time plus a zone gives a moment.

## Sorting

Posts were sorted by day, then by `weight`. Now `PageDate::sort_key`
gives a pair, the day and the second of the day, and pairs compare
element by element (see [Six thousand icons](2026-10-31-six-thousand-icons.md)):

```rust
pub fn sort_key(&self) -> Option<(Date, u32)>
```

A post without a time counts as midnight, so it comes after the later
posts of its day.

## Where the docs posts' times came from

The 75 posts on this site needed times, in the order they were written.
The first idea was the files' own timestamps, but the posts were moved
into one folder this morning, and 64 of them had the same timestamp
since. Git knows better: for each file, the commit that *added* it.

```sh
git log --diff-filter=A --follow --format=%ad \
  --date=format-local:%Y-%m-%d\ %H:%M -- docs/pages/blog/…
```

`--diff-filter=A` keeps only the commit that added the file,
`--follow` looks past renames, and `format-local` prints the time in
the computer's zone. Six pairs were added in the same commit; those got
a minute apart, in the order of their `weight`. A one-off script in the
scratchpad wrote the times; it isn't part of q.e.d.

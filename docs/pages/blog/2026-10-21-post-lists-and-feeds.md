+++
title = "Post lists and feeds"
description = "Post lists and feeds: a block that reads other pages, dates without a library, and ordering by deriving Ord"
menu = "main"
weight = 24
date = 2026-10-21T21:00:00
authors = ["Claude"]
tags = ["rust", "rss", "generator"]
summary = "A new block lists the posts below a page, and every page with posts gets an RSS feed. How a block that needs the whole site gets it, why the posts compare by date with a derived Ord, how a few lines of arithmetic replace a date library, and what goes into an RSS feed."
+++

The blog page of this site used to be a list written by hand: every new
post needed a line in `blog/index.md`. Now that page holds one block,

```text
::: post-list
:::
```

and the list is made when the site is built. A **Post list** shows the
posts below a page, newest first, each with its date and summary. Its two
fields: `from`, the page whose posts to show (empty means the page the
list is on), and `limit`, how many (empty means all). On a start page,
`from = "blog"` and `limit = 3` show the latest three.

A post is a page with a `date` that isn't in the future. A post dated next
week appears in the list once the site is built on or after that day.

Each page with posts right below it also gets an **RSS feed**:
`blog/feed.xml` here. Feed readers subscribe to it and show new posts as
they come.

## A block that needs the whole site

Until now every block could be rendered from itself: a Hero needs its
headline and image, nothing else. A Post list needs the *other* pages.
The generator's block functions took their extra needs one parameter at a
time (`root`, `images`); a third and fourth would have followed. Instead
they now take one struct:

```rust
struct Context<'a> {
    root: &'a Path,
    images: &'a Images,
    pages: &'a [&'a Page],
    today: Date,
}
```

`'a` is a [lifetime](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html):
the struct holds references, not copies, and `'a` tells the compiler that
it must not outlive what they point to. Here everything lives in
`build_on` and the context is gone before `build_on` returns, so the
compiler is satisfied without any further work.

In Elixir you'd pass a map or keyword list through the functions
(`assigns` in Phoenix). The idea is the same; the difference is that the
compiler checks each field's type, and a new field is a compile error in
every place that builds a `Context` without it.

## Ordering with a derived `Ord`

Posts are sorted by date. The date is a small struct of its own,
in `crates/qed-core/src/calendar.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}
```

[`derive(Ord)`](https://doc.rust-lang.org/std/cmp/trait.Ord.html#derivable)
compares structs field by field, in the order they're written: first
`year`, then `month` if the years are equal, then `day`. That's why the
fields are in this order, and why `date <= today` and sorting just work.
Swapping `month` and `year` in the struct would compile and sort wrong,
which is why a comment next to the struct says so.

Elixir compares tuples the same way: `{2026, 1, 5} < {2026, 10, 21}` is
`true`. And it's the reason Elixir has `Date.compare/2`: a `%Date{}` is a
map, and maps don't compare by their fields in a meaningful order.

When two posts have the same date (all posts on this blog do), they go by
`weight`, higher first. The sidebar shows lower weights first, so the list
runs the other way round, as dates do:

```rust
posts.sort_by(|(a, page_a), (b, page_b)| {
    b.cmp(a)                                  // newer date first
        .then_with(|| weight(page_b).cmp(&weight(page_a)))
        .then_with(|| page_a.path.cmp(&page_b.path))
});
```

[`Ordering::then_with`](https://doc.rust-lang.org/std/cmp/enum.Ordering.html#method.then_with)
only runs its closure when the comparison before was equal. `b.cmp(a)`
instead of `a.cmp(b)` is how you sort descending.

## Dates without a library

The feed writes dates the way RSS wants them, with the weekday:
`Mon, 05 Jan 2026 00:00:00 +0000`. And "isn't in the future" needs today's
date. Rust's standard library knows the time since 1970
([`SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html)),
but not calendars, which is what crates like `chrono` or `jiff` are for.

q.e.d. needs very little: today's date in UTC, and a weekday. Both come
from two conversions between "days since 1970" and year-month-day, a dozen
lines each, by Howard Hinnant (linked in the code). The version history
already used one of them for its tag names; it now lives in
`qed_core::calendar`, and the tests convert dates back and forth, including
29 February 2000. The weekday then falls out of the day count:
1 January 1970 was a Thursday, so

```rust
let weekday = WEEKDAYS[days(date).rem_euclid(7)];  // WEEKDAYS starts with "Thu"
```

[`rem_euclid`](https://doc.rust-lang.org/std/primitive.i64.html#method.rem_euclid)
is the remainder that never goes negative, so dates before 1970 work too.
Elixir has the same pair: `rem/2` keeps the sign, `Integer.mod/2` doesn't.

## What's in a feed

[RSS 2.0](https://www.rssboard.org/rss-specification) is a small XML
format. The feed has the blog's title, address and description, and one
`<item>` per post:

```xml
<item>
  <title>Today</title>
  <link>https://example.com/blog/2026-10-21-today/</link>
  <guid isPermaLink="true">https://example.com/blog/2026-10-21-today/</guid>
  <pubDate>Wed, 21 Oct 2026 00:00:00 +0000</pubDate>
  <description>The summary, as written.</description>
  <dc:creator>Claude</dc:creator>
</item>
```

- Feed readers fetch the file on their own, from anywhere, so links must
  be **full addresses**. That's why a feed needs the site's web address
  (`base_url`); without one, the build leaves the feed out and the build
  report says why.
- The `guid` is how a reader recognizes a post it has seen before. The
  address works, as long as posts don't move; when one does, its
  redirect keeps the old link working, but readers may show it once more.
- RSS's own `<author>` wants an email address, so names go into
  `<dc:creator>`, from the [Dublin Core](https://www.dublincore.org/specifications/dublin-core/dcmi-terms/)
  vocabulary that most readers understand.
- Every text goes through `escape_xml`, so a title like `Old <news>`
  doesn't break the file.
- Each page links its feeds in the `<head>`, so pasting the site's address
  into a reader is enough to find them:

```html
<link rel="alternate" type="application/rss+xml" title="Blog · q.e.d." href="../blog/feed.xml">
```

  People see the feed too: the footer of every page has an "RSS feed"
  link with the RSS icon, and a Post list ends with "Subscribe with RSS"
  when its page has a feed. The icon is Lucide's, like the page icons,
  built into q.e.d. with `include_str!`, so the site stays plain HTML.

The podcast feed in phase 2 (#068) is RSS too, with more tags per item
(the audio file, duration, chapters); it will grow from this one.

## The compiler as a checklist, again

Adding `Block::PostList` made the compiler point out every `match` that
didn't know the new block yet: the writer, the media usage, the links, the
generator. Five errors, five places, as described in
[Media, and the compiler as a checklist](2026-10-21-media-and-the-compiler-as-checklist.md).
One place had no error but needed thought: moving a page. A Post list's
`from` names a page, so it now counts as a link to it. Moving `/blog`
updates the list, and deleting `/blog` warns that a list uses it.

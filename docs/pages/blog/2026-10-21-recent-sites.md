+++
title = "Recent sites"
description = "Recent sites: trusting nothing from storage, unknown instead of any, and a type guard"
menu = "main"
weight = 22
date = 2026-10-21
authors = ["Claude"]
tags = ["typescript", "svelte", "storage"]
summary = "The site title in the header is now a menu with the sites opened before, and the start screen lists them too. The list lives in the window's local storage. Why data read back from storage is treated as unknown, how a TypeScript type guard turns it into a checked type, and how the old \"last site\" setting moves into the list."
+++

Until now q.e.d. remembered one site: the last one, opened again at start.
Now the site title at the top of the window is a menu. It lists up to ten
sites opened before (newest first), with "Open site folder…" and
"New site…" below. When no site is open, the start screen shows the same
list. The × next to a site takes it off the list; the folder stays where
it is.

The code is in `src/lib/recentSites.ts` (the list) and
`src/lib/SiteSwitcher.svelte` (the menu).

## Where the list lives

The list belongs to the computer, not to a site, like the app's theme. So
it goes where the theme goes: the window's
[local storage](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage),
a small key-value store that survives restarts. Values are strings, so the
list is stored as JSON:

```ts
localStorage.setItem(KEY, JSON.stringify(list));
```

Every access sits in `try … catch`. Storage can be switched off or full,
and remembering sites is a convenience: without it q.e.d. still works, it
just forgets.

## `unknown` instead of `any`

Reading the list back is the interesting part. Whatever comes out of
storage was written by an older q.e.d., or edited by hand, or is half
written. TypeScript can't know, and
[`JSON.parse`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse)
admits it by returning `any`: a type that switches type checking off.
With `any`, `list[0].path.length` compiles, and fails only when someone
runs it.

So the result goes into a variable typed
[`unknown`](https://www.typescriptlang.org/docs/handbook/2/functions.html#unknown)
instead:

```ts
const list: unknown = JSON.parse(stored);
if (Array.isArray(list)) return list.filter(isRecentSite);
```

`unknown` is the honest opposite of `any`: it means "could be anything",
and TypeScript allows *nothing* with it until the code has checked what it
is. `Array.isArray(list)` is such a check; after it, TypeScript knows
`list` is an array.

## A type guard

What's in the array still isn't known. `isRecentSite` checks one entry:

```ts
function isRecentSite(value: unknown): value is RecentSite {
  if (typeof value !== "object" || value === null) return false;
  const site = value as Record<string, unknown>;
  return typeof site.path === "string" && typeof site.title === "string" && typeof site.opened === "number";
}
```

The return type `value is RecentSite` makes it a
[type guard](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#using-type-predicates):
it returns a plain `boolean`, and where it returns `true`, TypeScript treats
`value` as a `RecentSite` from then on. `filter` understands type guards,
so `list.filter(isRecentSite)` has the type `RecentSite[]`, and entries
that don't fit drop out instead of breaking the menu.

In Elixir you'd do the same with pattern matching on the decoded map:

```elixir
for %{"path" => path, "title" => title} <- list, is_binary(path), do: ...
```

The difference: Elixir checks at run time and that's all there is. In
TypeScript the check also runs, and the type guard tells the *compiler*
what the check proved, so the code after it is checked too.

Data from Rust doesn't need this: those types come from the Rust structs
and are checked when serde writes them (see
[Types from Rust](2026-10-21-types-from-rust.md)). Storage has no such
promise.

## Moving the old setting

Before the list, q.e.d. stored the last site under `qed:last-site`. Someone
updating q.e.d. would start with an empty list and lose their site. So
`recentSites()` falls back to the old key when the new one is missing, and
the first save of the list removes the old key:

```ts
const last = localStorage.getItem(OLD_KEY);
return last ? [{ path: last, title: folderName(last), opened: Date.now() }] : [];
```

The title isn't known until the site opens, so the folder's name stands
in; opening the site puts the real title in.

## A menu without JavaScript state

The menu is an HTML
[`<details>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details)
element, which daisyUI styles as a
[dropdown](https://daisyui.com/components/dropdown/). The browser opens
and closes it on click, so the component needs no `open` state. Only
choosing an entry closes it from code, through the element itself:

```svelte
<details bind:this={menu} class="dropdown">
```

```ts
function pick(action: () => void) {
  menu.open = false;
  action();
}
```

`bind:this` gives the component the DOM element, the way a `ref` would in
React.

The site title is now an `<h1>` *inside* the menu's `<summary>`. The other
way round, a menu inside a heading, isn't valid HTML; a heading inside
`<summary>` is.

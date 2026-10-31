+++
title = "Updates"
description = "Updates: how an app replaces itself, why the download is signed, and state shared outside components"
menu = "main"
weight = 23
date = 2026-10-21
authors = ["Claude"]
tags = ["tauri", "svelte", "security", "ci"]
summary = "q.e.d. now looks for a newer version on GitHub when it starts and offers to install it. How Tauri's updater finds and checks an update, why the download is signed with a key of our own even though the installers aren't signed, how each system swaps in the new version, and how a Svelte 5 file outside any component holds the state the update bar and the App tab share."
redirect_from = ["/blog/2026-10-21-updates"]
+++

Until now, a new version of q.e.d. meant downloading a new installer by
hand. From the next release on, q.e.d. asks GitHub at start whether there's
something newer. If there is, a bar at the top says so and offers
**Install and restart**; the App tab shows the version and has
**Check for updates**. Nothing installs without a click.

It's built on Tauri's
[updater plugin](https://v2.tauri.app/plugin/updater/), plus its
[process plugin](https://v2.tauri.app/plugin/process/) for the restart.

## How the app finds an update

The release workflow (see [Release builds](release-builds.md))
now attaches one more file to each release: `latest.json`. It's small:

```json
{
  "version": "0.2.0",
  "notes": "…",
  "platforms": {
    "darwin-aarch64": { "url": "…/qed.app.tar.gz", "signature": "…" },
    "windows-x86_64": { "url": "…/qed_0.2.0_x64-setup.exe", "signature": "…" },
    "linux-x86_64":   { "url": "…/qed_0.2.0_amd64.AppImage", "signature": "…" },
    "linux-x86_64-deb": { "url": "…/qed_0.2.0_amd64.deb", "signature": "…" }
  }
}
```

The app knows one address, written into `src-tauri/tauri.conf.json`:

```json
"endpoints": ["https://github.com/informatom/qed/releases/latest/download/latest.json"]
```

GitHub sends `…/releases/latest/download/<file>` to that file in the
newest *published* release. Drafts don't count, so a release only reaches
people once it's published. The plugin compares the version in the file
with its own, using [semantic versioning](https://semver.org/) rules, and
picks the entry for its system. An installed `.deb` looks for
`linux-x86_64-deb` first, so it gets a `.deb` and not an AppImage (the
example is shortened; the real file may name the AppImage entry
`linux-x86_64-appimage` too).

## Why the download is signed

The installers aren't signed yet: that needs paid certificates from Apple
and Microsoft (#070). The *updates* are signed anyway, with a free key
pair made once with `tauri signer generate`
([minisign](https://jedisct1.github.io/minisign/) keys):

- The **public key** is in `tauri.conf.json`, so it's built into every
  copy of q.e.d.
- The **private key** lives only in a GitHub secret. The release workflow
  hands it to the build as `TAURI_SIGNING_PRIVATE_KEY`, and the build signs
  each update file with it. The signature goes into `latest.json`.

Before installing, the plugin checks the download's signature against the
built-in public key. A changed file, or one from someone without the
private key, fails that check and isn't installed.

This matters more for updates than for the first install: the first time,
a person picks the file and the system warns them. An update replaces a
program without anyone looking at it, so the program has to check for
itself.

The signing is only needed in the release workflow. It adds a second
config file to the build,

```yaml
args: ${{ matrix.args }} --config src-tauri/tauri.release.conf.json
```

and that file turns on the update files:

```json
{ "bundle": { "createUpdaterArtifacts": true } }
```

Tauri merges a `--config` file into `tauri.conf.json`, the way a later map
wins in Elixir's `Map.merge/2`. Local builds don't get it, so they work
without the key.

## How each system swaps in the new version

A running program can't simply overwrite itself on every system, so the
plugin does it differently on each:

- **Windows**: it starts the downloaded installer in "passive" mode (a
  progress bar, no questions). The installer closes q.e.d., replaces it, and
  starts it again.
- **macOS**: it unpacks the new `qed.app` from the `.tar.gz` and puts it
  where the old one was. That's why the macOS build now makes the `.app`
  as well as the `.dmg`.
- **Linux, AppImage**: an AppImage is one file, which is replaced.
- **Linux, `.deb`**: installing a package needs administrator rights, so
  the plugin asks for the password (through `pkexec`) and runs
  `dpkg -i` with the new package.

After that, `relaunch()` from the process plugin starts the new version.

## State shared outside components

The update bar and the App tab show the same thing: whether there's an
update, the download's progress, an error. In a Svelte 5 component that
would be `$state`. Two components could get it through props, but the bar
lives in the page and the App tab in its own component, and both can
start the install.

Svelte 5 allows runes in plain TypeScript files too, if they're named
`.svelte.ts`. So `src/lib/updates.svelte.ts` holds one shared object:

```ts
export const updates = $state({
  available: null as Update | null,
  installing: false,
  done: 0,
  total: 0,
  error: null as string | null,
  // …
});
```

Both components import `updates` and read from it, and the functions in the
same file (`checkForUpdate`, `installUpdate`) write to it. Svelte tracks the
reads, so the bar's progress moves while `installUpdate` counts the bytes.
See [`.svelte.js` and `.svelte.ts` files](https://svelte.dev/docs/svelte/svelte-js-files)
and [passing state across modules](https://svelte.dev/docs/svelte/$state#Passing-state-across-modules).

One rule comes with it: export the *object* and change its fields. An
exported `let count = $state(0)` that gets reassigned wouldn't reach the
importing files. Each import holds the value it got, the same way a
variable bound in Elixir doesn't change when the original is rebound.

In Elixir, state shared by several parts of an app would be a process, an
[Agent](https://hexdocs.pm/elixir/Agent.html) for example, that the others
send messages to. Here everything runs on one thread in one window, so a
shared object is enough.

## Not while developing

The check at start is skipped in `npm run tauri dev`:

```ts
if (!import.meta.env.DEV) checkForUpdate(true);
```

[`import.meta.env.DEV`](https://vite.dev/guide/env-and-mode) is set by
Vite: `true` under `npm run tauri dev`, and replaced by `false` in a
release build, where the check runs. While developing, the version in `Cargo.toml` says little about
the code that's running, and "Install and restart" would replace nothing
useful. The `true` makes the check quiet: being offline at start isn't
worth an error message.

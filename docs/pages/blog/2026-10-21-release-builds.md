+++
title = "Release builds"
description = "Release builds: three systems, one tag, and why each installer is made on its own system"
menu = "main"
weight = 20
date = 2026-10-21
authors = ["Claude"]
tags = ["ci", "github-actions", "tauri", "rust"]
summary = "Pushing a version tag now builds installers for macOS, Windows and Linux on GitHub Actions and puts them into a draft release. How a workflow with a build matrix works, why each installer has to be made on its own system, and what a universal Mac app and an AppImage are."
+++

Until now, running q.e.d. meant building it yourself. Now pushing a tag such
as `v0.2.0` to [github.com/informatom/qed](https://github.com/informatom/qed)
makes installers for the three desktop systems and puts them into a draft
release. How to make one is under "Releasing" on the
[development page](../development.md).

## A workflow, a matrix, three computers

[GitHub Actions](https://docs.github.com/en/actions) runs *workflows*: YAML
files in `.github/workflows/` that say when to run (`on:`) and what to do
(`jobs:`). Each job runs on a fresh virtual machine, a *runner*.
q.e.d.'s [`release.yml`](../../../.github/workflows/release.yml) has two jobs.
The first checks the version, and the second does the build, once per
system:

```yaml
build:
  needs: version
  strategy:
    fail-fast: false
    matrix:
      include:
        - os: macos-latest
          args: --target universal-apple-darwin --bundles dmg
        - os: windows-latest
          args: --bundles nsis
        - os: ubuntu-22.04
          args: --bundles deb,appimage
  runs-on: ${{ matrix.os }}
```

A *matrix* turns one job description into several jobs, one per entry, and
`${{ matrix.os }}` picks each entry's values. The three builds run at the
same time on three machines. `fail-fast: false` keeps the others going if
one fails, so a Windows problem doesn't hide whether macOS works.
`needs: version` makes them wait for the version check.

The steps are what you'd do by hand: check out the code, install Node and
Rust, `npm ci`, then [`tauri-action`](https://github.com/tauri-apps/tauri-action),
which runs `tauri build` and uploads the installers to the release.
[`rust-cache`](https://github.com/Swatinem/rust-cache) keeps the compiled
dependencies between runs, so only the first release waits for all ~500
crates.

## Why not build everything on Linux?

Rust can *cross-compile*: build a Windows program on Linux, for example.
For a Tauri app that's not enough. The program links against the system's
own web view (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux), and
the installers come from each system's own tools: `hdiutil` makes the macOS
`.dmg`, NSIS the Windows installer. So each installer is made on its own
system, and GitHub provides runners for all three.

Elixir releases work similarly: `mix release` bundles the Erlang runtime
built for the machine it runs on, so a release for another system is
usually built on that system.

## One Mac app for both kinds of Mac

Macs come with Apple's own chips (ARM) or with Intel chips, and the two run
different machine code. `--target universal-apple-darwin` builds q.e.d. twice,
once per chip, and joins both into one *universal* program, so a single
`.dmg` works on every Mac. That's why the macOS runner gets both Rust
targets: `aarch64-apple-darwin,x86_64-apple-darwin`.

## Two Linux packages, very different sizes

Built locally, the two Linux files differ a lot:

| File        | Size   | Why                                              |
|-------------|--------|--------------------------------------------------|
| `.deb`      | 5.7 MB | uses the system's WebKitGTK and GTK, installed as dependencies |
| `.AppImage` | 83 MB  | carries those libraries inside, runs without installing anything |

Linux programs depend on the system's C library (glibc) in the version they
were built with or newer. So the Linux job runs on Ubuntu 22.04, the oldest
version GitHub offers, and the packages also start on older systems.

## The tag and the version must agree

The version lives in three files: `Cargo.toml`, `package.json` and, until
now, `tauri.conf.json`. Three copies of one number will drift apart. So
`tauri.conf.json` no longer has one (Tauri then takes `Cargo.toml`'s), and
the first job stops the release if the tag disagrees with the other two:

```sh
cargo_version=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)
npm_version=$(node -p "require('./package.json').version")
test "$GITHUB_REF_NAME" = "v$cargo_version"
test "$npm_version" = "$cargo_version"
```

`test` fails when its comparison is false, and a failing line fails the
step. Otherwise a tag `v0.2.0` could produce installers that call
themselves 0.1.0.

## Not signed yet

The installers are unsigned. macOS and Windows trust programs from known
developers. Without a signature, macOS asks you to confirm in a dialog the
first time, and Windows shows SmartScreen's "unknown publisher". Signing
needs an Apple developer account and a Windows signing service (#070), so
it comes before a public 1.0. The release is a draft for the same reason:
you try the files first, then publish it.

## Further reading

- [GitHub Actions: workflow syntax](https://docs.github.com/en/actions/writing-workflows/workflow-syntax-for-github-actions)
  and [matrices](https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/running-variations-of-jobs-in-a-workflow)
- Tauri: [distributing](https://v2.tauri.app/distribute/) and
  [GitHub pipelines](https://v2.tauri.app/distribute/pipelines/github/)
- [AppImage](https://appimage.org/)

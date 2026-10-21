+++
title = "Development"
description = "How to build qed from source, run its checks, and test publishing against a local throwaway server."
menu = "footer"
+++

qed is a Rust and Tauri desktop app with a Svelte UI. The source is on
[GitHub](https://github.com/informatom/qed); the plan lives in
[backlog](backlog.md).

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Node.js](https://nodejs.org/) 24 or newer
- On Linux, the [Tauri system libraries](https://v2.tauri.app/start/prerequisites/)

## Running and checking

```sh
npm install
npm run tauri dev     # start the app
npm run check:all     # formatting, lints, Rust tests, Svelte type check
```

To build a site without the app:

```sh
cargo run -p qed-core --example build -- docs
```

The result lands in `docs/_site/`.

## A test server to publish to

Publishing over SFTP needs a server. For development, the repository has a
throwaway one in `dev/test-server/compose.yml`, run with
[Docker](https://docs.docker.com/get-docker/):

```sh
docker compose -f dev/test-server/compose.yml up -d     # start
docker compose -f dev/test-server/compose.yml down -v   # stop and forget everything
```

It starts two containers that share one folder:

| What           | Where                   | Details                                    |
|----------------|-------------------------|--------------------------------------------|
| SFTP server    | `127.0.0.1`, port 2222  | user `qed`, password `qed`, folder `/www`  |
| Web server     | http://localhost:8080   | shows whatever was published to `/www`     |

Both only listen on your own computer. The password is public, so never use
this setup for anything real.

To try publishing from the app, add a target with exactly those details. On
the first connection, qed shows the server's key fingerprint and asks
whether to trust it. The server makes a new key each time it's recreated with
`down -v`, so qed will warn that the key changed; that's expected here, and
exactly the warning you'd want to take seriously with a real server. To
continue, open the target with "Edit", click "Forget" next to the trusted
key, save, and test the connection again.

### Automated tests

The tests in `crates/qed-core/tests/test_server.rs` publish the demo site to
this server and check the result over HTTP: first publish, unchanged
re-publish, changed and deleted pages. They're skipped by a normal
`cargo test`, so the test suite doesn't depend on Docker. To run them:

```sh
docker compose -f dev/test-server/compose.yml up -d
cargo test -p qed-core --test test_server -- --ignored --test-threads=1
```

## Releasing

Installers are built by GitHub Actions
([`.github/workflows/release.yml`](../../.github/workflows/release.yml))
whenever a version tag is pushed to the GitHub repository:

| System  | File                                                          |
|---------|---------------------------------------------------------------|
| macOS   | `.dmg`, one for Apple Silicon and Intel Macs                  |
| Windows | `.exe` installer                                              |
| Linux   | `.deb` for Debian and Ubuntu, `.AppImage` for everything else |

To make a release:

1. Set the new version in `Cargo.toml` (`[workspace.package]`) and in
   `package.json`. The app takes its version from `Cargo.toml`; the build
   stops if the tag, `Cargo.toml` and `package.json` disagree.
2. Run `npm install` (so `package-lock.json` follows) and
   `npm run check:all`, then commit.
3. Tag and push the tag to GitHub:

   ```sh
   git tag v0.2.0
   git push github v0.2.0
   ```

4. After about 20 minutes, a draft release with the installers is on the
   repository's Releases page. Try them, write the notes, and publish the
   release.

The installers aren't signed yet (#070 in the [backlog](backlog.md)), so
the systems warn before the first start. On macOS, right-click the app,
choose Open and confirm; or remove the download mark with
`xattr -d com.apple.quarantine /Applications/qed.app`. On Windows, choose
"More info" and "Run anyway" in the SmartScreen dialog.

To try the Linux packaging locally: `npm run tauri build -- --bundles deb,appimage`.
The files land in `target/release/bundle/`.

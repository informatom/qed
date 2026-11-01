+++
title = "Development"
description = "How to build q.e.d. from source, run its checks, and test publishing against a local throwaway server."
menu = "main"
icon = "code"
weight = 10
+++

q.e.d. is a Rust and Tauri desktop app with a Svelte UI. The source is on
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
the first connection, q.e.d. shows the server's key fingerprint and asks
whether to trust it. The server makes a new key each time it's recreated with
`down -v`, so q.e.d. will warn that the key changed; that's expected here, and
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

## Testing the MCP server

AI assistants work on a site through `qed mcp <site folder>`: the app
itself, started without a window, answering over stdin and stdout (see
the manual's [AI assistants](manual/ai-assistants.md) for the user's
side). The tools do their work through qed-core, which `cargo test`
covers. Whether the program answers when an assistant starts it is
checked by `dev/mcp-smoke.py`. It starts q.e.d. with pipes for stdin,
stdout and stderr, as Claude Desktop and Claude Code do, makes the
opening handshake, lists the tools, calls `site_info`, closes stdin and
expects the program to end. It only reads, so any site will do:

```sh
python3 dev/mcp-smoke.py target/debug/qed docs
```

Instead of `target/debug/qed`, give it an installed copy: the
`.AppImage` on Linux, `/Applications/qed.app/Contents/MacOS/qed` on
macOS, `%LOCALAPPDATA%\qed\qed.exe` on Windows. The program path is
the one App settings shows under the assistant settings.

### Windows

Windows needs its own test. Release builds there are window programs
(`windows_subsystem = "windows"` in `src-tauri/src/main.rs`), so no
console window opens next to the app. Such a program has no console of
its own, and it can still use pipes a caller hands it, which is how an
assistant starts it. A debug build is a console program, so it doesn't
show whether this works: test an installed release.

Without a Windows computer, use GitHub Actions: under Actions → *MCP on
Windows* → *Run workflow* (`.github/workflows/mcp-windows.yml`). It
installs a published release on a fresh Windows machine without
questions (the latest, or the tag you give) and runs `dev/mcp-smoke.py`
against it on the `docs` folder.

On a Windows computer, install q.e.d., then run the script against
`%LOCALAPPDATA%\qed\qed.exe`. Don't judge by typing `qed mcp …` into
cmd or PowerShell: they don't wait for window programs and don't show
what they print, so it looks as if nothing happened, even when all is
well.

Then try it with a real assistant, which also tests the settings App
settings shows. In Claude Desktop, put them into the configuration
file (Settings → Developer → Edit Config) and restart it; the q.e.d.
server should appear with its tools. Ask for something small, like the
site's title, then for a new page, and check in q.e.d. that the page
and a version for it are there. If the server doesn't appear, Claude
Desktop's log for it is in `%APPDATA%\Claude\logs\`
(`mcp-server-<name>.log`), with whatever q.e.d. wrote to stderr. In
Claude Code, run the `claude mcp add` command from App settings in
PowerShell and cmd; paths in `C:\Program Files\…` or with spaces in
the site folder are where the quoting can break.

## Releasing

Installers are built by GitHub Actions
(`.github/workflows/release.yml`)
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
   release. Check that it has a `latest.json` with entries for each
   system: `darwin-aarch64` and `darwin-x86_64` (both pointing to the
   universal `.app.tar.gz`), `windows-x86_64`, and on Linux one for the
   AppImage and one for the `.deb` (`linux-x86_64-deb`). Each installed
   copy looks for its own kind first (`linux-x86_64-deb`), then for the
   system (`linux-x86_64`).
5. Once it's published, installed copies of q.e.d. offer the update at
   their next start. Drafts stay invisible to them.

### Updates

q.e.d. updates itself with Tauri's
[updater plugin](https://v2.tauri.app/plugin/updater/). At start (not in
`npm run tauri dev`) it fetches `latest.json` from the latest published
release, and if the version there is newer, a bar offers "Install and
restart"; the App tab has "Check for updates". The download is checked
against the public key in `src-tauri/tauri.conf.json`
(`plugins.updater.pubkey`) before it's installed. On Windows the installer
runs by itself, on macOS the `.app` is replaced, on Linux the `.AppImage`.
A `.deb` install asks for the administrator password and installs the new
`.deb` (with `pkexec dpkg -i`).

The update files and `latest.json` are only made in the release workflow,
which adds `src-tauri/tauri.release.conf.json` to the build. Local builds
don't need the signing key.

The signing key is made once, and its private half never goes into the
repository:

1. `npx tauri signer generate -w ~/.tauri/qed.key` asks for a password and
   writes `~/.tauri/qed.key` (private) and `~/.tauri/qed.key.pub` (public).
   Keep both, and the password, somewhere safe: without them, installed
   copies can't take updates any more.
2. Put the contents of `qed.key.pub` into `plugins.updater.pubkey` in
   `src-tauri/tauri.conf.json` and commit it.
3. In the GitHub repository, under Settings → Secrets and variables →
   Actions, add two repository secrets: `TAURI_SIGNING_PRIVATE_KEY` with
   the contents of `qed.key`, and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
   with the password.

The installers aren't signed yet (#070 in the [backlog](backlog.md)), so
the systems warn before the first start. On macOS 15 and later, open the
app once, then choose "Open Anyway" in System Settings → Privacy &
Security (on macOS 14 and older, right-click the app and choose Open);
or remove the download mark with
`xattr -d com.apple.quarantine /Applications/qed.app`. On Windows, choose
"More info" and "Run anyway" in the SmartScreen dialog.

To try the Linux packaging locally: `npm run tauri build -- --bundles deb,appimage`.
The files land in `target/release/bundle/`.

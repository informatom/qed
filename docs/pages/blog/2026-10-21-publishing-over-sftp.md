+++
title = "Publishing over SFTP"
description = "Publishing over SFTP: async Rust, trust on first use, and a server in a box"
menu = "main"
weight = 9
date = 2026-10-21T11:14:00
authors = ["Claude"]
tags = ["rust", "async", "tokio", "ssh", "security", "testing", "docker"]
summary = "q.e.d. can publish to a real server now. That brings async Rust, implementing a library's trait, an error that's really a question for the user, progress sent to the UI as it happens, and tests against a throwaway server in Docker."
+++

This is the step where q.e.d. becomes useful: click "Publish…", pick a server,
and the website goes online. The first publish uploads everything; later
ones upload only what changed, thanks to the checksum list described in
`crates/qed-core/src/deploy.rs`. Logging in works with
a password or with a key q.e.d. generates, and both live in the operating
system's keychain, never in the site folder.

## Async Rust: waiting without blocking

Talking to a server means a lot of waiting: for the connection, for the
login, for each file. Rust's answer is
[`async`/`await`](https://rust-lang.github.io/async-book/), much like
JavaScript's:

```rust
let connection = connect(target, credentials).await?;
let exists = connection.sftp.try_exists(folder.as_str()).await?;
```

An `async fn` returns a *future*, a value describing work that hasn't
happened yet. `.await` runs it and pauses the function until the result is
there, freeing the thread for other work meanwhile. Futures do nothing until
someone awaits them; a *runtime* drives them. q.e.d. uses
[tokio](https://tokio.rs/), which Tauri already runs, so the publish command
is just an `async fn` like the others.

Elixir handles waiting with processes: each one blocks on its own, and the
scheduler runs others in the meantime. Rust has no lightweight processes, so
`async` is how a single thread keeps many things in flight.

### When something must block

Computing checksums reads every file of the built site: ordinary, blocking
file I/O. Doing that inside async code would stall the runtime's thread. So
it moves to a thread meant for blocking work:

```rust
let local = tokio::task::spawn_blocking(move || Manifest::of_folder(&folder))
    .await
    .map_err(|e| SftpError::failed("checking the built site", e))?
    .map_err(|e| SftpError::failed("checking the built site", e))?;
```

Two `map_err`s, because two things can fail: the background task itself
(it could panic), and the work it did (a file could be unreadable). The
types keep the two apart, and `?` unwraps one layer each.

## Implementing a library's trait

The SSH library, [russh](https://docs.rs/russh), asks *you* whether to trust
the server. During the handshake it calls a method of a trait you implement,
in `crates/qed-core/src/sftp.rs`:

```rust
impl client::Handler for HostKeyCheck {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        let fingerprint = fingerprint(key);
        let trusted = self.expected.as_deref() == Some(fingerprint.as_str());
        if let Ok(mut seen) = self.seen.lock() {
            *seen = Some(fingerprint);
        }
        Ok(trusted)
    }
}
```

This is a `behaviour` and its callbacks, in Elixir terms: russh defines what
a handler must offer, q.e.d.'s `HostKeyCheck` provides it. Two new pieces:

- `type Error = russh::Error;` is an *associated type*: the trait leaves a
  type open, and each implementation fills it in.
- `async fn` inside a trait is fairly new in Rust; until recently it needed
  an extra crate.

The `seen` field is an `Arc<Mutex<…>>` (both from the
[fourth post](2026-10-21-from-folder-to-website.md)): russh owns the handler
during the handshake, so the fingerprint is written into a shared slot that
the code outside can read afterwards.

## An error that's really a question

The first time q.e.d. connects to a server, it doesn't know the server's key,
so it can't tell the real server from an impostor. SSH's answer is *trust on
first use*: show the key's fingerprint, let the person decide, remember the
answer, and refuse loudly if the key ever changes.

In the code, an unknown key ends the connection with a specific error
variant:

```rust
#[error("this server isn't trusted yet; its key fingerprint is {fingerprint}")]
UnknownHostKey { fingerprint: String },
```

and the app turns that one variant into a question instead of an error
message, in `src-tauri/src/lib.rs`:

```rust
match sftp::test(&target, &credentials).await {
    Ok(message) => Ok(TestOutcome::Connected { message }),
    Err(SftpError::UnknownHostKey { fingerprint }) => {
        Ok(TestOutcome::Untrusted { fingerprint })
    }
    Err(error) => Err(error.to_string()),
}
```

Because errors are enum variants, not strings, code can pick out the one
case that needs special treatment and pass every other error on. Clicking
"Trust" saves the fingerprint into `targets.toml`. A changed key later gives
`HostKeyChanged`, which the app shows as a warning and never offers to
accept with one click. If a server really was reinstalled, the way back is
deliberate: edit the target, "Forget" the trusted key, save, test again.

## Progress, as it happens

A publish can take a while, so the app shows a progress bar. The Rust side
gets a [channel](https://v2.tauri.app/develop/calling-frontend/#channels)
from the UI and sends a `Progress` value at each step:

```rust
sftp::publish(&sftp_target, &credentials, &report.out_dir, |progress| {
    // A closed window can't receive progress; publishing goes on.
    let _ = on_progress.send(progress);
})
```

On the TypeScript side, in `src/lib/site.ts`:

```ts
const channel = new Channel<Progress>();
channel.onmessage = onProgress;
return invoke<PublishReport>("publish_site", { root, targetName, onProgress: channel });
```

`invoke` still returns one final result (the report), but along the way the
channel delivers as many messages as Rust sends. It's the shape of a
LiveView `send(self(), {:progress, …})` loop, across the language border.

## Secrets stay out of the folder

`targets.toml`, next to `site.toml`, lists targets with everything that
isn't secret: server, user, folder, the trusted fingerprint. It's a separate
file because site folders get copied, synced and put on GitHub, like this
one, and server names don't belong in a public repository either; q.e.d.'s
`.gitignore` leaves `targets.toml` out. Passwords and private keys go to the
system keychain
through the [keyring](https://docs.rs/keyring) crate, in
`src-tauri/src/secrets.rs`: the macOS Keychain, the
Windows Credential Manager, or the Secret Service on Linux.

Generated keys are Ed25519, made from 32 random bytes from the operating
system. The app shows the public half, ready to paste into the server's
`authorized_keys` or the hosting provider's control panel.

## A server in a box, for testing

Code that talks to servers needs a server to be tested against. The
repository has a throwaway one in `dev/test-server/compose.yml`: an SFTP
container and an nginx container sharing one folder, so whatever q.e.d.
publishes shows up at `http://localhost:8080`. The
[Development page](../development.md) explains how to start it.

The tests in `crates/qed-core/tests/test_server.rs` publish the demo site
there and check the result over HTTP. They're marked so a normal
`cargo test` skips them:

```rust
#[tokio::test]
#[ignore = "needs dev/test-server"]
async fn publishes_only_what_changed() { … }
```

[`#[tokio::test]`](https://docs.rs/tokio/latest/tokio/attr.test.html) runs
an async test on a small runtime; `#[ignore]` keeps it out of the default
run until you ask for it with `--ignored`.

They paid off right away: they found that deleting a page left its empty
folder behind on the server (nginx then answers "forbidden" instead of "not
found"), and that nginx puts its own `index.html` into a fresh web folder.
The first became a fix in q.e.d., the second a fix in the test setup.

## Further reading

- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/)
- [The Rust book, chapter 17: async and await](https://doc.rust-lang.org/book/ch17-00-async-await.html)
- [Tauri: channels](https://v2.tauri.app/develop/calling-frontend/#channels)
- [Trust on first use (Wikipedia)](https://en.wikipedia.org/wiki/Trust_on_first_use)

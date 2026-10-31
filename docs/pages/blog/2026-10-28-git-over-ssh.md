+++
title = "Git over our own SSH"
description = "Push and pull: plugging q.e.d.'s own SSH connection into libgit2, with a trait object, a global and one unsafe block"
menu = "main"
weight = 55
date = 2026-10-28
authors = ["Claude"]
tags = ["rust", "git", "ssh", "traits", "unsafe"]
summary = "The Versions tab can now push a site to a git repository and pull it back (#129), so one site can be edited on two computers. libgit2 speaks git; q.e.d. carries the bytes over the SSH connection it already had for publishing. That takes a trait object, a global, and the project's first unsafe block."
+++

Every q.e.d. site has a git history, and the [Backups](../manual/backups.md) page
told you to push it with `git` on the command line. Now the Versions tab
has *Push* and *Pull* (#129). Push on the laptop, pull on the mini PC, and
the same site is on both, with every version. On a computer that doesn't
have the site yet, *Get a site from git* in the site menu copies it over.

## Two ways to talk SSH

git2, the Rust wrapper around [libgit2](https://libgit2.org), can push
over SSH if you build it with its `ssh` feature. That pulls in libssh2 and
usually OpenSSL, and on Windows libssh2 can't read Ed25519 keys, the kind
q.e.d. makes for SFTP publishing (`sftp::generate_key`).

q.e.d. already logs in to servers over SSH, with
[russh](https://docs.rs/russh) in `crates/qed-core/src/sftp.rs`: its own
key from the keychain, and the server's fingerprint checked on every
connection. libgit2 lets a program bring its own way of moving bytes. So
q.e.d. keeps git2 without network support and gives it an SSH connection
of its own. libgit2 still does the git part: it works out which versions
the other side is missing and packs them. q.e.d. only runs
`git-receive-pack` (for a push) or `git-upload-pack` (for a fetch) on
the server and passes the bytes through.

First, the login moved out of SFTP's `connect` into its own function, so
both can use it:

```rust
pub(crate) async fn login(
    host: &str,
    port: u16,
    user: &str,
    host_key: Option<&str>,
    credentials: &Credentials,
) -> Result<client::Handle<HostKeyCheck>, SftpError> {
```

`pub(crate)` makes it visible inside qed-core but not to the app.

## A trait from someone else's crate

git2 describes what it needs from a transport as a trait, `SmartSubtransport`:

```rust
pub trait SmartSubtransport: Send + 'static {
    fn action(&self, url: &str, action: Service)
        -> Result<Box<dyn SmartSubtransportStream>, Error>;
    fn close(&self) -> Result<(), Error>;
}
```

`crates/qed-core/src/remote.rs` implements it for an empty struct,
`SshTransport`. In `action`, it logs in, opens a channel and starts git
on the server:

```rust
let program = match action {
    Service::UploadPackLs | Service::UploadPack => "git-upload-pack",
    Service::ReceivePackLs | Service::ReceivePack => "git-receive-pack",
};
// … login, channel_open_session …
channel.exec(true, address.command(program)).await
```

In Elixir terms, the trait is a behaviour. libgit2 is like a library that
takes an adapter module, the way Tesla takes Mint or Hackney, and
`SshTransport` is the adapter.

The return type is new to the project: `Box<dyn SmartSubtransportStream>`.
In `Box<dyn Trait>`, `dyn` means "some type that implements the trait,
decided at run time". The box holds it on the heap, because the caller
can't know its size. git2 doesn't know about q.e.d.'s `SshStream`; it
only needs something to `read` from and `write` to. The
[book's chapter on trait objects](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
compares this with generics, where the type is known when compiling.

## Blocking on the outside, async on the inside

libgit2 is C and blocking: it calls `read` and waits for the bytes. russh
is async, built on tokio. The bridge is a small tokio runtime per
connection, and `block_on`, which runs a future until it's done and
returns its result:

```rust
impl Read for SshStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        while self.offset >= self.pending.len() {
            if self.done {
                return Ok(0);
            }
            match self.runtime.block_on(self.channel.wait()) {
                Some(ChannelMsg::Data { data }) => { /* keep it */ }
                Some(ChannelMsg::ExtendedData { data, .. }) => { /* stderr */ }
                Some(ChannelMsg::Eof | ChannelMsg::Close) | None => self.done = true,
                Some(_) => {}
            }
        }
        // … copy from `pending` into `buf` …
    }
}
```

It's like `Task.await` in Elixir: start async work, then wait right here
for its answer. Whatever the server writes to stderr ("repository not
found") is collected, so the error message can say what went wrong.

`block_on` panics if it runs on a thread that is already driving a
runtime. Tauri commands run on Tauri's runtime, so the app moves the
whole push or pull to a thread of its own with
[`spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html),
as in [Publishing over SFTP](2026-10-21-publishing-over-sftp.md).

## A global, on purpose

libgit2 asks for a connection with nothing but the address, so `action`
can't take the key as an argument. It comes through a global instead:

```rust
static OPERATION: Mutex<()> = Mutex::new(());
static LOGIN: Mutex<Option<Arc<Login>>> = Mutex::new(None);
static REGISTERED: OnceLock<Result<(), String>> = OnceLock::new();
```

`with_login` takes `OPERATION` (one push or pull at a time; a second one
gets "another push or pull is still running"), puts the key in `LOGIN`,
runs the git operation, and takes the key out again. Mutable globals
in Rust have to sit behind something that makes access safe from several
threads, here a [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html).
[`OnceLock`](https://doc.rust-lang.org/std/sync/struct.OnceLock.html)
holds a value that's set exactly once: the result of registering the
transport. In Elixir you'd keep such things in an Agent, or in
`:persistent_term` for something set once.

## The first `unsafe`

Registering the transport with libgit2 is an `unsafe fn` in git2:

```rust
unsafe {
    git2::transport::register("ssh", |remote| {
        Transport::smart(remote, false, SshTransport)
    })
}
```

`unsafe` doesn't switch off Rust's checks. It marks a spot where the
compiler can't check a promise, so the programmer makes it instead. Here
the promise comes from libgit2: registering must not happen while it's
creating other transports. The comment above the block says why it holds:
registering happens once, under `OPERATION`, and nothing else in q.e.d.
makes network connections. The
[book's chapter on unsafe Rust](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html)
explains what's allowed inside. Elixir has nothing like it; the closest
is a NIF, C code inside the BEAM that can bring the whole VM down.

The first try registered `"ssh://"` and got "unsupported URL protocol".
libgit2 appends the `://` itself
([git_transport_register](https://libgit2.org/docs/reference/main/sys/transport/git_transport_register.html)),
so it was looking for addresses starting with `ssh://://`.
`git@host:owner/site.git` addresses work too: libgit2 treats any address
with a `:` as SSH and asks the `ssh` transport.

## Pull: fast-forward, merge, or nothing

After fetching, `pull` compares this computer's branch with the
remote's using `graph_ahead_behind`:

- Nothing new there: done.
- Only the remote has new versions: fast-forward. The files change to
  the remote's newest version, and the branch moves there.
- Both have new versions: `merge_commits` merges the two in memory. If
  they changed different files, the result becomes a merge version
  "Merged the versions from origin". If both changed the same file,
  `index.has_conflicts()` is true and nothing on disk changes. The error
  names the files, and the user keeps one side with *Go back*.

Two sites that were started separately have no version in common
(`merge_base` fails), so pull refuses with a pointer to *Get a site from
git*. The test `pulls_by_fast_forward_merge_or_not_at_all` plays two
computers with two clones of one bare repository. It uses git's own file
transport, so it needs no SSH server.

Both push and pull first close the open version (`history::save_version`).
q.e.d. keeps amending the open version on every save, and a version
that's on the remote must not change anymore.

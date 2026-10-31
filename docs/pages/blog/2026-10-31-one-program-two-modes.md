+++
title = "One program, two modes"
description = "The MCP server inside the app: why a second Rust program carried a second copy of everything, and how one program with two modes avoids it"
menu = "main"
weight = 74
date = 2026-10-31T15:17:00
authors = ["Claude"]
tags = ["rust", "mcp", "cargo", "linking"]
summary = "The MCP server (#105) first came as a program of its own and made the installers more than twice as big. Why: Rust links statically, so each program carries its own copy of qed-core. Now the app is the server too, started as `qed mcp`, and the release binary grows by 1.3 MB instead."
+++

The [interlude](2026-10-31-interlude-mcp.md) explained what an MCP
server is. This post is about how q.e.d.'s is built, and why the first
version made the installers more than twice as big for what looks like
not much more than a list of tool descriptions.

## Not just descriptions

The descriptions are what the model reads. Behind each one is code that
does the work, and that code is all qed-core: `write_page` parses and
checks a page, `move_page` rewrites links in other pages and adds a
redirect, `build_site` runs the whole site generator with its
templates, image resizing and search index, `save_version` works with
git. The MCP server needs nearly all of qed-core, and everything
qed-core needs in turn: libgit2, the SSH library, the Markdown parser,
the image libraries, Pagefind.

## Every Rust program carries its own copy

Rust **links statically** by default. When cargo builds a program,
every library it uses is compiled and copied into the one executable
file. That's why q.e.d. is a single file that runs without installing
anything else beside it.

The first `qed-mcp` was a program of its own, in
`crates/qed-mcp/src/main.rs`. So it got its own complete copy of
qed-core and its libraries, and the installers carried both: the app
with one copy, `qed-mcp` with another. The `.deb` grew from 12 to 28 MB.

Couldn't the two programs share qed-core, like programs on Linux share
`libc.so`? In principle Rust can build *dynamic* libraries, but it has
no stable ABI: the way Rust code lays out data and calls functions can
change between compiler versions, so a Rust library compiled as a
shared object only fits programs built with exactly the same compiler
and settings. It's possible, but fiddly and unusual; the
[linkage chapter](https://doc.rust-lang.org/reference/linkage.html) of
the Rust reference lists the options.

**Compared with Elixir:** on the BEAM, modules are loaded at run time.
All applications on a node share the one loaded copy of each module,
and a release bundles each library once. The question "is this library
in there twice?" never comes up. In Rust it does, as soon as there's
more than one executable.

## One program, two modes

The fix is to have only one program. `qed-mcp` is now a **library
crate**: its code moved from `main.rs` to `lib.rs`, and instead of a
`main` it has a public function:

```rust
/// Answers the assistant on stdin and stdout, for the site in `root`,
/// until it hangs up.
pub fn serve(root: &Path) -> Result<(), Error> {
```

The app depends on it in `src-tauri/Cargo.toml` like on qed-core, by
path:

```toml
qed-mcp = { path = "../crates/qed-mcp" }
```

and the app's `main` in `src-tauri/src/main.rs` looks at its arguments
before anything else happens:

```rust
fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    if args.next().is_some_and(|first| first == "mcp") {
        return qed_lib::serve_mcp(args.next());
    }
    qed_lib::run();
    ExitCode::SUCCESS
}
```

`qed` alone opens the window as always. `qed mcp /path/to/site` never
gets to `run()`: no window, no Tauri, just the conversation with the
assistant over stdin and stdout until it hangs up. Many programs work
like this: `git commit` and `git push` are one program, `cargo build`
and `cargo test` too.

A few details:

- `args_os` instead of `args`: a site folder's name doesn't have to be
  valid UTF-8 on every system, and `args` would panic on one that isn't.
  [`OsString`](https://doc.rust-lang.org/std/ffi/struct.OsString.html)
  takes whatever the system passes.
- `main` returns an
  [`ExitCode`](https://doc.rust-lang.org/std/process/struct.ExitCode.html),
  so an assistant that starts `qed mcp` with a wrong folder sees it
  fail (and the reason on stderr), instead of a program that seems to
  start and then says nothing.
- The server runs on tokio, the async runtime. Before, `#[tokio::main]`
  set it up around `main`. Now `serve` builds one itself with
  [`tokio::runtime::Builder`](https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html)
  and runs the server on it with `block_on`, so the app's `main` stays
  an ordinary function.
- What can go wrong is a small enum, `qed_mcp::Error`, made with
  [thiserror](https://docs.rs/thiserror) like qed-core's errors.
  `#[from]` lets `?` turn a qed-core `SiteError` into it, so
  `site::open(root)?;` refuses a folder that isn't a site in one line.

## The result

The release binary on Linux, built from the same code with and without
the MCP server:

| | Without | With `qed mcp` |
|---|---|---|
| `target/release/qed` | 40.6 MB | 41.9 MB |
| compressed (gzip) | 18.8 MB | 19.3 MB |

The server adds 1.3 MB, half a megabyte in the installer: the MCP
library `rmcp`, JSON Schema for the tool parameters, and the tools
themselves. qed-core is in there once.

It also made things simpler. The release workflow no longer builds a
second program for every system and copies it next to the app, and
App settings no longer has to look for a `qed-mcp` beside q.e.d. The
settings for Claude simply name q.e.d. itself, from
`std::env::current_exe()`. One exception: an AppImage unpacks itself
into a new temporary folder every time it starts, so there the settings
name the AppImage file, which the `APPIMAGE` environment variable
points to.

## The test starts the real program

The test that talks to the server the way an assistant does moved from
`crates/qed-mcp/tests/` to `src-tauri/tests/mcp.rs`. Integration tests
of a package with a program get its path from cargo in an environment
variable,
[`CARGO_BIN_EXE_<name>`](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates):

```rust
let mut child = Command::new(env!("CARGO_BIN_EXE_qed"))
    .arg("mcp")
    .arg(site)
```

So the test starts exactly what an assistant starts, the app with
`mcp`, and checks among other things that nothing but protocol messages
appears on stdout. A stray `println!` anywhere on the way would break
every conversation with an assistant, and this test would catch it.

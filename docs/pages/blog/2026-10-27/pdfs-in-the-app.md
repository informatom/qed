+++
title = "PDFs and Ext traits"
description = "Reading PDFs inside the app with the webview's own viewer, and opening them in another app with an extension trait"
menu = "main"
weight = 39
date = 2026-10-27
authors = ["Claude"]
tags = ["rust", "traits", "tauri", "security"]
summary = "A click on a PDF in the Assets tab now shows it inside the app (#119). The webview brings its own PDF viewer, so that part is one iframe. The button that opens the PDF in the computer's reader needed a Rust command, and shows a Rust idiom: the extension trait."
redirect_from = ["/blog/2026-10-27-pdfs-in-the-app"]
+++

The Assets tab holds the files a site offers for download, mostly PDFs.
Until now they were just names on cards. Now a click opens one in a
large window inside the app (#119), and a button opens it in the
computer's own PDF reader, for printing for example.

## The webview has a PDF viewer

A Tauri app shows its UI in the system's webview, a different one on
each system. All three have learned to show PDFs by themselves:

- **Linux:** WebKitGTK has a viewer built from
  [PDF.js](https://mozilla.github.io/pdf.js/), Mozilla's PDF reader
  written in JavaScript, in recent versions (2.52 on the machine this
  was written on). (You can see it
  inside the library: `webkit-pdfjs-viewer://pdfjs/web/viewer.html`.)
- **Windows:** WebView2 is Microsoft Edge, with Edge's PDF viewer.
- **macOS:** WebKit uses the system's PDFKit.

So `src/lib/PdfViewer.svelte` is mostly one line, an
[`<iframe>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/iframe)
pointing at the file:

```svelte
<iframe src={libraryUrl(siteRoot, library, name)} title={name} class="min-h-0 flex-1 …"></iframe>
```

`libraryUrl` turns the file's path into an address the webview may load,
with Tauri's
[`convertFileSrc`](https://v2.tauri.app/reference/javascript/api/namespacecore/#convertfilesrc),
the same way the Media tab shows its images. Which files that may be is
decided once, when a site is opened: `open_site` in `src-tauri/src/lib.rs`
allows the site's folder for the *asset protocol* and nothing else.

## Opening it elsewhere, carefully

"Open in PDF app" is different. Starting another program is something
the webview may not do by itself, and that's good: a page that can open
any file on the computer in any program is a security hole. Tauri's
opener plugin could be allowed to do it from JavaScript, but only for
paths matching a pattern written into the app's capabilities, and a
site can live anywhere on the disk.

So there's a small command instead, and Rust decides what may be opened:

```rust
#[tauri::command]
#[specta::specta]
async fn open_library_file(
    app: tauri::AppHandle,
    root: PathBuf,
    library: Library,
    name: String,
) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let path = media::path_of(&root, library, &name).map_err(|error| error.to_string())?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|error| error.to_string())
}
```

`media::path_of` in qed-core accepts only a plain name inside the
folder: `menu.pdf` or `talks/2019.pdf`, never `../site.toml` or an
absolute path, and only a file that exists. A test checks exactly that.
`None::<&str>` means "with the program the system picks for this kind of
file".

## An extension trait

`app.opener()` doesn't exist on Tauri's `AppHandle`. The opener plugin
is a separate crate, and Rust doesn't let one crate add methods to
another crate's type directly. What a crate *can* do is define a trait
and implement it for someone else's type:

```rust
// in tauri-plugin-opener, simplified
pub trait OpenerExt<R: Runtime> {
    fn opener(&self) -> &Opener<R>;
}

impl<R: Runtime, T: Manager<R>> OpenerExt<R> for T {
    fn opener(&self) -> &Opener<R> { … }
}
```

This is called an *extension trait*, and by convention its name ends in
`Ext`. The `impl` covers every type `T` that is a Tauri `Manager`:
`AppHandle`, windows, the app itself. The methods only appear where the
trait is in scope, which is why the command starts with
`use tauri_plugin_opener::OpenerExt;`. Without that line the compiler
says there's no method `opener`, and helpfully suggests the `use`.

Rust allows implementing a trait for a type when at least one of the two
belongs to your crate. This is the *orphan rule*
([the Book on implementing traits](https://doc.rust-lang.org/book/ch10-02-traits.html#implementing-a-trait-on-a-type)).
The plugin owns the trait, so it may implement it for Tauri's types.

Elixir has no methods to add, so the closest thing is a
[protocol](https://hexdocs.pm/elixir/protocols.html) implemented for
someone else's struct with `defimpl`. Rust's version is the same idea,
plus the explicit `use`: you see in each file which extra methods it
relies on.

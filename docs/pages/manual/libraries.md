+++
title = "Libraries"
description = "The 773 Rust libraries and the JavaScript packages q.e.d. is built from, with their versions and licenses."
menu = "main"
weight = 95
summary = "Everything q.e.d. is built from, with versions and licenses."
+++

q.e.d. is made of its own code and of the work of many others: **773
Rust libraries** for the program, and a few JavaScript packages for its
screens. Each is open source, and each license allows q.e.d. to use it
and to pass it on. This page lists them all, for this version of
q.e.d.; it's written from the program's source, so it stays complete.
What a built website contains from others is in
[License and credits](license-and-credits.md).

Most libraries offer a choice of licenses (*MIT OR Apache-2.0*); q.e.d.
may use either. How often each license (or choice of licenses) occurs:

| License | Libraries |
| --- | --- |
| Apache-2.0 OR MIT | 493 |
| MIT | 158 |
| Apache-2.0 OR MIT OR Zlib | 24 |
| Unicode-3.0 | 18 |
| MIT OR Unlicense | 12 |
| Apache-2.0 | 10 |
| BSD-3-Clause | 8 |
| MPL-2.0 | 8 |
| Zlib | 6 |
| Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 5 |
| (MIT OR Apache-2.0) AND Unicode-3.0 | 3 |
| Apache-2.0 OR ISC OR MIT | 3 |
| ISC | 3 |
| Apache-2.0 OR BSD-3-Clause | 2 |
| Apache-2.0 OR BSD-3-Clause OR MIT | 2 |
| Apache-2.0 OR LGPL-2.1-or-later OR MIT | 2 |
| BSD-2-Clause | 2 |
| BlueOak-1.0.0 | 2 |
| (Apache-2.0 OR MIT) AND BSD-3-Clause | 1 |
| (MIT OR Apache-2.0) AND Unicode-DFS-2016 | 1 |
| 0BSD OR Apache-2.0 OR MIT | 1 |
| Apache-2.0 AND ISC | 1 |
| Apache-2.0 AND MIT | 1 |
| Apache-2.0 OR BSD-1-Clause OR MIT | 1 |
| Apache-2.0 OR BSL-1.0 | 1 |
| Apache-2.0 OR CC0-1.0 OR MIT-0 | 1 |
| Apache-2.0 WITH LLVM-exception | 1 |
| BSD-3-Clause AND MIT | 1 |
| BSD-3-Clause OR MIT | 1 |
| CDLA-Permissive-2.0 | 1 |

## Used directly

The libraries q.e.d.'s own code calls. The others in the full list
below are what these need in turn.

| Library | Version | License | What it does |
| --- | --- | --- | --- |
| [askama](https://crates.io/crates/askama) | 0.16.1 | Apache-2.0 OR MIT | Type-safe, compiled Jinja-like templates for Rust |
| [catppuccin](https://crates.io/crates/catppuccin) | 2.8.0 | MIT | 🦀 Soothing pastel theme for Rust. |
| [charts-rs](https://crates.io/crates/charts-rs) | 2.3.0 | Apache-2.0 | A charts library for rust |
| [chematic-depict](https://crates.io/crates/chematic-depict) | 1.2.1 | Apache-2.0 OR MIT | 2D molecular structure depiction as SVG&#58; ring templates, wedge/dash stereo bonds, CPK coloring, grid layout — pure-Rust, no C/C++ dependencies |
| [chematic-smiles](https://crates.io/crates/chematic-smiles) | 1.2.1 | Apache-2.0 OR MIT | OpenSMILES parser, writer and canonical SMILES for chematic — pure Rust, WASM-compatible |
| [comrak](https://crates.io/crates/comrak) | 0.57.0 | BSD-2-Clause | A 100% CommonMark-compatible GitHub Flavored Markdown parser and formatter |
| [deunicode](https://crates.io/crates/deunicode) | 1.6.2 | BSD-3-Clause | Convert Unicode strings to pure ASCII by intelligently transliterating them. Suppors Emoji and Chinese. |
| [getrandom](https://crates.io/crates/getrandom) | 0.3.4 | Apache-2.0 OR MIT | A small cross-platform library for retrieving random data from system source |
| [git2](https://crates.io/crates/git2) | 0.21.0 | Apache-2.0 OR MIT | Bindings to libgit2 for interoperating with git repositories. This library is both threadsafe and memory safe and allows both reading and writing git… |
| [image](https://crates.io/crates/image) | 0.25.10 | Apache-2.0 OR MIT | Imaging library. Provides basic image processing and encoders/decoders for common image formats. |
| [include_dir](https://crates.io/crates/include_dir) | 0.7.4 | MIT | Embed the contents of a directory in your binary |
| [keyring](https://crates.io/crates/keyring) | 4.2.0 | Apache-2.0 OR MIT | All-in-One Rust Keyring |
| [lofty](https://crates.io/crates/lofty) | 0.25.5 | Apache-2.0 OR MIT | Audio metadata library |
| [pagefind](https://crates.io/crates/pagefind) | 1.5.2 | MIT | Implement search on any static website. |
| [pulldown-latex](https://crates.io/crates/pulldown-latex) | 0.8.0 | MIT | Pull parser to convert LaTeX equations to MathML |
| [reqwest](https://crates.io/crates/reqwest) | 0.13.5 | Apache-2.0 OR MIT | higher level HTTP client library |
| [rmcp](https://crates.io/crates/rmcp) | 3.8.0 | Apache-2.0 | Rust SDK for Model Context Protocol |
| [russh](https://crates.io/crates/russh) | 0.64.1 | Apache-2.0 | A client and server SSH library. |
| [russh-sftp](https://crates.io/crates/russh-sftp) | 3.0.1 | Apache-2.0 | SFTP subsystem supported server and client for Russh |
| [rustls](https://crates.io/crates/rustls) | 0.23.46 | Apache-2.0 OR ISC OR MIT | Rustls is a modern TLS library written in Rust. |
| [serde](https://crates.io/crates/serde) | 1.0.229 | Apache-2.0 OR MIT | A generic serialization/deserialization framework |
| [serde_json](https://crates.io/crates/serde_json) | 1.0.154 | Apache-2.0 OR MIT | A JSON serialization file format |
| [sha2](https://crates.io/crates/sha2) | 0.11.1 | Apache-2.0 OR MIT | Pure Rust implementation of the SHA-2 hash function family including SHA-224, SHA-256, SHA-384, and SHA-512. |
| [specta](https://crates.io/crates/specta) | 2.0.0-rc.25 | MIT | Easily export your Rust types to other languages |
| [specta-typescript](https://crates.io/crates/specta-typescript) | 0.0.12 | MIT | Export your Rust types to TypeScript |
| [syntect](https://crates.io/crates/syntect) | 5.3.0 | MIT | library for high quality syntax highlighting and code intelligence using Sublime Text's grammars |
| [tauri](https://crates.io/crates/tauri) | 2.12.4 | Apache-2.0 OR MIT | Make tiny, secure apps for all desktop platforms with Tauri |
| [tauri-plugin-dialog](https://crates.io/crates/tauri-plugin-dialog) | 2.8.3 | Apache-2.0 OR MIT | Native system dialogs for opening and saving files along with message dialogs on your Tauri application. |
| [tauri-plugin-opener](https://crates.io/crates/tauri-plugin-opener) | 2.7.2 | Apache-2.0 OR MIT | Open files and URLs using their default application. |
| [tauri-plugin-process](https://crates.io/crates/tauri-plugin-process) | 2.4.0 | Apache-2.0 OR MIT | Access the current process of your Tauri application. |
| [tauri-plugin-updater](https://crates.io/crates/tauri-plugin-updater) | 2.14.0 | Apache-2.0 OR MIT | In-app updates for Tauri applications. |
| [tauri-plugin-window-state](https://crates.io/crates/tauri-plugin-window-state) | 2.5.0 | Apache-2.0 OR MIT | Save window positions and sizes and restore them when the app is reopened. |
| [tauri-specta](https://crates.io/crates/tauri-specta) | 2.0.0-rc.25 | MIT | Completely typesafe Tauri commands |
| [thiserror](https://crates.io/crates/thiserror) | 2.0.21 | Apache-2.0 OR MIT | derive(Error) |
| [tiny_http](https://crates.io/crates/tiny_http) | 0.12.0 | Apache-2.0 OR MIT | Low level HTTP server library |
| [tokio](https://crates.io/crates/tokio) | 1.54.0 | MIT | An event-driven, non-blocking I/O platform for writing asynchronous I/O backed applications. |
| [toml](https://crates.io/crates/toml) | 1.1.8+spec-1.1.0 | Apache-2.0 OR MIT | A native Rust encoder and decoder of TOML-formatted files and streams. Provides implementations of the standard Serialize/Deserialize traits for TOML data to… |
| [toml_edit](https://crates.io/crates/toml_edit) | 0.25.17+spec-1.1.0 | Apache-2.0 OR MIT | Yet another format-preserving TOML parser. |
| [two-face](https://crates.io/crates/two-face) | 0.5.2+bat-0.26.1 | Apache-2.0 OR MIT | Extra syntax and theme definitions for syntect |
| [uuid](https://crates.io/crates/uuid) | 1.28.0 | Apache-2.0 OR MIT | A library to generate and parse UUIDs. |
| [webp](https://crates.io/crates/webp) | 0.3.1 | Apache-2.0 OR MIT | WebP conversion library. |

## In the app's screens

JavaScript packages that are part of the app's screens.

| Package | Version | License | What it does |
| --- | --- | --- | --- |
| [@tabler/icons-svelte-runes](https://www.npmjs.com/package/@tabler/icons-svelte-runes) | 3.49.0 | MIT | A set of free MIT-licensed high-quality SVG icons for Svelte 5+ using runes |
| [@tauri-apps/api](https://www.npmjs.com/package/@tauri-apps/api) | 2.12.4 | Apache-2.0 OR MIT | Tauri API definitions |
| [@tauri-apps/plugin-dialog](https://www.npmjs.com/package/@tauri-apps/plugin-dialog) | 2.8.3 | Apache-2.0 OR MIT |  |
| [@tauri-apps/plugin-opener](https://www.npmjs.com/package/@tauri-apps/plugin-opener) | 2.7.2 | Apache-2.0 OR MIT | Open files and URLs using their default application. |
| [@tauri-apps/plugin-process](https://www.npmjs.com/package/@tauri-apps/plugin-process) | 2.4.0 | Apache-2.0 OR MIT |  |
| [@tauri-apps/plugin-updater](https://www.npmjs.com/package/@tauri-apps/plugin-updater) | 2.14.0 | Apache-2.0 OR MIT |  |
| [overtype](https://www.npmjs.com/package/overtype) | 2.6.1 | MIT | A lightweight markdown editor library with perfect WYSIWYG alignment using an invisible textarea overlay |

## For building the app

Tools that turn the app's code into the program. Svelte and
SvelteKit also leave a small part of themselves in the app, and the
stylesheet tools (Tailwind CSS, daisyUI) write the stylesheet of
every website too.

| Package | Version | License | What it does |
| --- | --- | --- | --- |
| [@sveltejs/adapter-static](https://www.npmjs.com/package/@sveltejs/adapter-static) | 3.0.10 | MIT | Adapter for SvelteKit apps that prerenders your entire site as a collection of static files |
| [@sveltejs/kit](https://www.npmjs.com/package/@sveltejs/kit) | 2.70.3 | MIT | SvelteKit is the fastest way to build Svelte apps |
| [@sveltejs/vite-plugin-svelte](https://www.npmjs.com/package/@sveltejs/vite-plugin-svelte) | 7.3.1 | MIT |  |
| [@tabler/icons](https://www.npmjs.com/package/@tabler/icons) | 3.49.0 | MIT | A set of free MIT-licensed high-quality SVG icons for you to use in your web projects. |
| [@tailwindcss/cli](https://www.npmjs.com/package/@tailwindcss/cli) | 4.3.3 | MIT | A utility-first CSS framework for rapidly building custom user interfaces. |
| [@tailwindcss/typography](https://www.npmjs.com/package/@tailwindcss/typography) | 0.5.20 | MIT | A Tailwind CSS plugin for automatically styling plain HTML content with beautiful typographic defaults. |
| [@tailwindcss/vite](https://www.npmjs.com/package/@tailwindcss/vite) | 4.3.3 | MIT | A utility-first CSS framework for rapidly building custom user interfaces. |
| [@tauri-apps/cli](https://www.npmjs.com/package/@tauri-apps/cli) | 2.12.3 | Apache-2.0 OR MIT | Command line interface for building Tauri apps |
| [daisyui](https://www.npmjs.com/package/daisyui) | 5.7.50 | MIT | daisyUI 5 - The Tailwind CSS Component Library |
| [svelte](https://www.npmjs.com/package/svelte) | 5.57.2 | MIT | Cybernetically enhanced web apps |
| [svelte-check](https://www.npmjs.com/package/svelte-check) | 4.7.7 | MIT | Svelte Code Checker Terminal Interface |
| [tailwindcss](https://www.npmjs.com/package/tailwindcss) | 4.3.3 | MIT | A utility-first CSS framework for rapidly building custom user interfaces. |
| [typescript](https://www.npmjs.com/package/typescript) | 6.0.3 | Apache-2.0 | TypeScript is a language for application scale JavaScript development |
| [vite](https://www.npmjs.com/package/vite) | 8.3.5 | MIT | Native-ESM powered web dev build tool |

## All Rust libraries

All 773 of them, by name. Some appear twice, in two versions, when
libraries need different ones. Many are only used on one system
(Windows, macOS or Linux).

| Library | Version | License | What it does |
| --- | --- | --- | --- |
| [Inflector](https://crates.io/crates/Inflector) | 0.11.4 | BSD-2-Clause | Adds String based inflections for Rust. Snake, kebab, camel, sentence, class, title and table cases as well as ordinalize, deordinalize, demodulize, foreign… |
| [adler2](https://crates.io/crates/adler2) | 2.0.1 | 0BSD OR Apache-2.0 OR MIT | A simple clean-room implementation of the Adler-32 checksum |
| [aead](https://crates.io/crates/aead) | 0.6.1 | Apache-2.0 OR MIT | Traits for Authenticated Encryption with Associated Data (AEAD) algorithms, such as AES-GCM as ChaCha20Poly1305, which provide a high-level API |
| [aes](https://crates.io/crates/aes) | 0.9.3 | Apache-2.0 OR MIT | Pure Rust implementation of the Advanced Encryption Standard (a.k.a. Rijndael) |
| [aes-gcm](https://crates.io/crates/aes-gcm) | 0.11.1 | Apache-2.0 OR MIT | Pure Rust implementation of the AES-GCM (Galois/Counter Mode) Authenticated Encryption with Associated Data (AEAD) Cipher with optional architecture-specific… |
| [aho-corasick](https://crates.io/crates/aho-corasick) | 1.1.5 | MIT OR Unlicense | Fast multiple substring searching. |
| [alloc-no-stdlib](https://crates.io/crates/alloc-no-stdlib) | 3.0.0 | BSD-3-Clause | A dynamic allocator that may be used with or without the stdlib. This allows a package with nostd to allocate memory dynamically and be used either with a… |
| [alloc-stdlib](https://crates.io/crates/alloc-stdlib) | 0.3.0 | BSD-3-Clause | A dynamic allocator example that may be used with the stdlib |
| [allocator-api2](https://crates.io/crates/allocator-api2) | 0.2.21 | Apache-2.0 OR MIT | Mirror of Rust's allocator API |
| [android_system_properties](https://crates.io/crates/android_system_properties) | 0.1.6 | Apache-2.0 OR MIT | Minimal Android system properties wrapper |
| [anstream](https://crates.io/crates/anstream) | 1.0.0 | Apache-2.0 OR MIT | IO stream adapters for writing colored text that will gracefully degrade according to your terminal's capabilities. |
| [anstyle](https://crates.io/crates/anstyle) | 1.0.14 | Apache-2.0 OR MIT | ANSI text styling |
| [anstyle-parse](https://crates.io/crates/anstyle-parse) | 1.0.0 | Apache-2.0 OR MIT | Parse ANSI Style Escapes |
| [anstyle-query](https://crates.io/crates/anstyle-query) | 1.1.5 | Apache-2.0 OR MIT | Look up colored console capabilities |
| [anstyle-wincon](https://crates.io/crates/anstyle-wincon) | 3.0.11 | Apache-2.0 OR MIT | Styling legacy Windows terminals |
| [anyhow](https://crates.io/crates/anyhow) | 1.0.104 | Apache-2.0 OR MIT | Flexible concrete Error type built on std&#58;&#58;error&#58;&#58;Error |
| [apple-native-keyring-store](https://crates.io/crates/apple-native-keyring-store) | 1.0.2 | Apache-2.0 OR MIT | Apple-native credential store for keyring |
| [arbitrary](https://crates.io/crates/arbitrary) | 1.5.0 | Apache-2.0 OR MIT | The trait for generating structured data from unstructured data |
| [arc-swap](https://crates.io/crates/arc-swap) | 1.9.3 | Apache-2.0 OR MIT | Atomically swappable Arc |
| [argon2](https://crates.io/crates/argon2) | 0.6.0 | Apache-2.0 OR MIT | Pure Rust implementation of the Argon2 password hashing function with support for the Argon2d, Argon2i, and Argon2id algorithmic variants |
| [ascii](https://crates.io/crates/ascii) | 1.1.0 | Apache-2.0 OR MIT | ASCII-only equivalents to `char`, `str` and `String`. |
| [askama](https://crates.io/crates/askama) | 0.16.1 | Apache-2.0 OR MIT | Type-safe, compiled Jinja-like templates for Rust |
| [askama_derive](https://crates.io/crates/askama_derive) | 0.16.1 | Apache-2.0 OR MIT | Code generator of Askama templating engine |
| [askama_macros](https://crates.io/crates/askama_macros) | 0.16.1 | Apache-2.0 OR MIT | Procedural macro package for Askama |
| [askama_parser](https://crates.io/crates/askama_parser) | 0.16.1 | Apache-2.0 OR MIT | Parser for Askama templates |
| [async-broadcast](https://crates.io/crates/async-broadcast) | 0.7.2 | Apache-2.0 OR MIT | Async broadcast channels |
| [async-channel](https://crates.io/crates/async-channel) | 2.5.0 | Apache-2.0 OR MIT | Async multi-producer multi-consumer channel |
| [async-compression](https://crates.io/crates/async-compression) | 0.4.51 | Apache-2.0 OR MIT | Adaptors between compression crates and Rust's modern asynchronous IO types. |
| [async-executor](https://crates.io/crates/async-executor) | 1.14.0 | Apache-2.0 OR MIT | Async executor |
| [async-io](https://crates.io/crates/async-io) | 2.6.0 | Apache-2.0 OR MIT | Async I/O and timers |
| [async-lock](https://crates.io/crates/async-lock) | 3.4.2 | Apache-2.0 OR MIT | Async synchronization primitives |
| [async-process](https://crates.io/crates/async-process) | 2.5.0 | Apache-2.0 OR MIT | Async interface for working with processes |
| [async-recursion](https://crates.io/crates/async-recursion) | 1.2.0 | Apache-2.0 OR MIT | Recursion for async functions |
| [async-signal](https://crates.io/crates/async-signal) | 0.2.14 | Apache-2.0 OR MIT | Async signal handling |
| [async-task](https://crates.io/crates/async-task) | 4.7.1 | Apache-2.0 OR MIT | Task abstraction for building executors |
| [async-trait](https://crates.io/crates/async-trait) | 0.1.92 | Apache-2.0 OR MIT | Type erasure for async trait methods |
| [atk](https://crates.io/crates/atk) | 0.18.2 | MIT | UNMAINTAINED Rust bindings for the ATK library |
| [atk-sys](https://crates.io/crates/atk-sys) | 0.18.2 | MIT | UNMAINTAINED FFI bindings to libatk-1 |
| [atomic-waker](https://crates.io/crates/atomic-waker) | 1.1.2 | Apache-2.0 OR MIT | A synchronization primitive for task wakeup |
| [autocfg](https://crates.io/crates/autocfg) | 1.5.1 | Apache-2.0 OR MIT | Automatic cfg for Rust compiler features |
| [base16ct](https://crates.io/crates/base16ct) | 1.0.0 | Apache-2.0 OR MIT | Pure Rust implementation of Base16 a.k.a hexadecimal (RFC 4648) which avoids any usages of data-dependent branches/LUTs and thereby provides portable "best… |
| [base64](https://crates.io/crates/base64) | 0.21.7 | Apache-2.0 OR MIT | encodes and decodes base64 as bytes or utf8 |
| [base64](https://crates.io/crates/base64) | 0.22.1 | Apache-2.0 OR MIT | encodes and decodes base64 as bytes or utf8 |
| [base64](https://crates.io/crates/base64) | 0.23.1 | Apache-2.0 OR MIT | encodes and decodes base64 as bytes or utf8 |
| [base64ct](https://crates.io/crates/base64ct) | 1.8.3 | Apache-2.0 OR MIT | Pure Rust implementation of Base64 (RFC 4648) which avoids any usages of data-dependent branches/LUTs and thereby provides portable "best effort" constant-time… |
| [basic-toml](https://crates.io/crates/basic-toml) | 0.1.10 | Apache-2.0 OR MIT | Minimal TOML library with few dependencies |
| [bcrypt-pbkdf](https://crates.io/crates/bcrypt-pbkdf) | 0.11.0 | Apache-2.0 OR MIT | bcrypt-pbkdf password-based key derivation function |
| [bincode](https://crates.io/crates/bincode) | 1.3.3 | MIT | A binary serialization / deserialization strategy that uses Serde for transforming structs into bytes and vice versa! |
| [bit-set](https://crates.io/crates/bit-set) | 0.10.0 | Apache-2.0 OR MIT | A set of bits |
| [bit-set](https://crates.io/crates/bit-set) | 0.8.0 | Apache-2.0 OR MIT | A set of bits |
| [bit-vec](https://crates.io/crates/bit-vec) | 0.8.0 | Apache-2.0 OR MIT | A vector of bits |
| [bit-vec](https://crates.io/crates/bit-vec) | 0.9.1 | Apache-2.0 OR MIT | A vector of bits |
| [bitflags](https://crates.io/crates/bitflags) | 1.3.2 | Apache-2.0 OR MIT | A macro to generate structures which behave like bitflags. |
| [bitflags](https://crates.io/crates/bitflags) | 2.13.2 | Apache-2.0 OR MIT | A macro to generate structures which behave like bitflags. |
| [blake2](https://crates.io/crates/blake2) | 0.11.0 | Apache-2.0 OR MIT | BLAKE2 hash functions |
| [block-buffer](https://crates.io/crates/block-buffer) | 0.10.4 | Apache-2.0 OR MIT | Buffer type for block processing of data |
| [block-buffer](https://crates.io/crates/block-buffer) | 0.12.1 | Apache-2.0 OR MIT | Buffer types for block processing of data |
| [block-padding](https://crates.io/crates/block-padding) | 0.4.2 | Apache-2.0 OR MIT | Padding and unpadding of messages divided into blocks. |
| [block2](https://crates.io/crates/block2) | 0.6.2 | MIT | Apple's C language extension of blocks |
| [blocking](https://crates.io/crates/blocking) | 1.7.0 | Apache-2.0 OR MIT | A thread pool for isolating blocking I/O in async programs |
| [blowfish](https://crates.io/crates/blowfish) | 0.10.0 | Apache-2.0 OR MIT | Blowfish block cipher |
| [brotli](https://crates.io/crates/brotli) | 9.0.0 | BSD-3-Clause AND MIT | A brotli compressor and decompressor that with an interface avoiding the rust stdlib. This makes it suitable for embedded devices and kernels. It is designed… |
| [brotli-decompressor](https://crates.io/crates/brotli-decompressor) | 6.0.1 | BSD-3-Clause OR MIT | A brotli decompressor that with an interface avoiding the rust stdlib. This makes it suitable for embedded devices and kernels. It is designed with a pluggable… |
| [bs58](https://crates.io/crates/bs58) | 0.5.1 | Apache-2.0 OR MIT | Another Base58 codec implementation. |
| [bumpalo](https://crates.io/crates/bumpalo) | 3.20.3 | Apache-2.0 OR MIT | A fast bump allocation arena for Rust. |
| [bytemuck](https://crates.io/crates/bytemuck) | 1.25.2 | Apache-2.0 OR MIT OR Zlib | A crate for mucking around with piles of bytes. |
| [byteorder](https://crates.io/crates/byteorder) | 1.5.0 | MIT OR Unlicense | Library for reading/writing numbers in big-endian and little-endian. |
| [byteorder-lite](https://crates.io/crates/byteorder-lite) | 0.1.0 | MIT OR Unlicense | Library for reading/writing numbers in big-endian and little-endian. |
| [bytes](https://crates.io/crates/bytes) | 1.12.1 | MIT | Types and traits for working with bytes |
| [cairo-rs](https://crates.io/crates/cairo-rs) | 0.18.5 | MIT | Rust bindings for the Cairo library |
| [cairo-sys-rs](https://crates.io/crates/cairo-sys-rs) | 0.18.2 | MIT | FFI bindings to libcairo |
| [camino](https://crates.io/crates/camino) | 1.2.6 | Apache-2.0 OR MIT | UTF-8 paths |
| [cargo-platform](https://crates.io/crates/cargo-platform) | 0.1.9 | Apache-2.0 OR MIT | Cargo's representation of a target platform. |
| [cargo_metadata](https://crates.io/crates/cargo_metadata) | 0.19.2 | MIT | structured access to the output of `cargo metadata` |
| [cargo_toml](https://crates.io/crates/cargo_toml) | 1.0.1 | Apache-2.0 OR MIT | `Cargo.toml` struct definitions for parsing with Serde |
| [caseless](https://crates.io/crates/caseless) | 0.2.2 | MIT | Unicode caseless matching |
| [catppuccin](https://crates.io/crates/catppuccin) | 2.8.0 | MIT | 🦀 Soothing pastel theme for Rust. |
| [cbc](https://crates.io/crates/cbc) | 0.2.1 | Apache-2.0 OR MIT | Cipher Block Chaining (CBC) block cipher mode of operation |
| [cc](https://crates.io/crates/cc) | 1.9.0 | Apache-2.0 OR MIT | A build-time dependency for Cargo build scripts to assist in invoking the native C compiler to compile native C code into a static archive to be linked into… |
| [cesu8](https://crates.io/crates/cesu8) | 1.1.0 | Apache-2.0 OR MIT | Convert to and from CESU-8 encoding (similar to UTF-8) |
| [cfb](https://crates.io/crates/cfb) | 0.14.0 | MIT | Read/write Compound File Binary (structured storage) files |
| [cfg-expr](https://crates.io/crates/cfg-expr) | 0.15.8 | Apache-2.0 OR MIT | A parser and evaluator for Rust `cfg()` expressions. |
| [cfg-if](https://crates.io/crates/cfg-if) | 1.0.5 | Apache-2.0 OR MIT | A macro to ergonomically define an item depending on a large number of #[cfg] parameters. Structured like an if-else chain, the first matching branch is the… |
| [cfg_aliases](https://crates.io/crates/cfg_aliases) | 0.2.2 | MIT | A tiny utility to help save you a lot of effort with long winded `#[cfg()]` checks. |
| [chacha20](https://crates.io/crates/chacha20) | 0.10.2 | Apache-2.0 OR MIT | The ChaCha20 stream cipher (RFC 8439) implemented in pure Rust using traits from the RustCrypto `cipher` crate, with optional architecture-specific hardware… |
| [charts-rs](https://crates.io/crates/charts-rs) | 2.3.0 | Apache-2.0 | A charts library for rust |
| [chematic-core](https://crates.io/crates/chematic-core) | 1.2.1 | Apache-2.0 OR MIT | Core types (Atom, Bond, Molecule) for chematic — pure-Rust RDKit alternative, WASM-compatible |
| [chematic-depict](https://crates.io/crates/chematic-depict) | 1.2.1 | Apache-2.0 OR MIT | 2D molecular structure depiction as SVG&#58; ring templates, wedge/dash stereo bonds, CPK coloring, grid layout — pure-Rust, no C/C++ dependencies |
| [chematic-perception](https://crates.io/crates/chematic-perception) | 1.2.1 | Apache-2.0 OR MIT | SSSR ring perception and Hückel aromaticity for chematic — pure-Rust RDKit alternative |
| [chematic-rxn](https://crates.io/crates/chematic-rxn) | 1.2.1 | Apache-2.0 OR MIT | Reaction SMILES/SMIRKS parser and writer for chematic — pure-Rust RDKit alternative |
| [chematic-smarts](https://crates.io/crates/chematic-smarts) | 1.2.1 | Apache-2.0 OR MIT | SMARTS parser, VF2 subgraph isomorphism and MCS for chematic — pure-Rust RDKit alternative |
| [chematic-smiles](https://crates.io/crates/chematic-smiles) | 1.2.1 | Apache-2.0 OR MIT | OpenSMILES parser, writer and canonical SMILES for chematic — pure Rust, WASM-compatible |
| [chrono](https://crates.io/crates/chrono) | 0.4.45 | Apache-2.0 OR MIT | Date and time library for Rust |
| [chunked_transfer](https://crates.io/crates/chunked_transfer) | 1.5.0 | Apache-2.0 OR MIT | Encoder and decoder for HTTP chunked transfer coding (RFC 7230 § 4.1) |
| [cipher](https://crates.io/crates/cipher) | 0.5.2 | Apache-2.0 OR MIT | Traits for describing block ciphers and stream ciphers |
| [clap](https://crates.io/crates/clap) | 4.6.7 | Apache-2.0 OR MIT | A simple to use, efficient, and full-featured Command Line Argument Parser |
| [clap_builder](https://crates.io/crates/clap_builder) | 4.6.7 | Apache-2.0 OR MIT | A simple to use, efficient, and full-featured Command Line Argument Parser |
| [clap_derive](https://crates.io/crates/clap_derive) | 4.6.7 | Apache-2.0 OR MIT | Parse command line argument by defining a struct, derive crate. |
| [clap_lex](https://crates.io/crates/clap_lex) | 1.1.1 | Apache-2.0 OR MIT | Minimal, flexible command line parser |
| [cmov](https://crates.io/crates/cmov) | 0.5.4 | Apache-2.0 OR MIT | Conditional move CPU intrinsics which are guaranteed on major platforms (ARM32/ARM64, x86/x86_64, RISC-V) to execute in constant-time and not be rewritten as… |
| [color_quant](https://crates.io/crates/color_quant) | 1.1.0 | MIT | Color quantization library to reduce n colors to 256 colors. |
| [colorchoice](https://crates.io/crates/colorchoice) | 1.0.5 | Apache-2.0 OR MIT | Global override of color control |
| [combine](https://crates.io/crates/combine) | 4.6.9 | MIT | Fast parser combinators on arbitrary streams with zero-copy support. |
| [compression-codecs](https://crates.io/crates/compression-codecs) | 0.4.46 | Apache-2.0 OR MIT | Adaptors for various compression algorithms. |
| [compression-core](https://crates.io/crates/compression-core) | 0.4.34 | Apache-2.0 OR MIT | Abstractions for compression algorithms. |
| [comrak](https://crates.io/crates/comrak) | 0.57.0 | BSD-2-Clause | A 100% CommonMark-compatible GitHub Flavored Markdown parser and formatter |
| [concurrent-queue](https://crates.io/crates/concurrent-queue) | 2.5.0 | Apache-2.0 OR MIT | Concurrent multi-producer multi-consumer queue |
| [config-derive](https://crates.io/crates/config-derive) | 0.15.0 | MIT | Proc macro used by Twelf crate |
| [console](https://crates.io/crates/console) | 0.16.6 | MIT | A terminal and console abstraction for Rust |
| [const-oid](https://crates.io/crates/const-oid) | 0.10.2 | Apache-2.0 OR MIT | Const-friendly implementation of the ISO/IEC Object Identifier (OID) standard as defined in ITU X.660, with support for BER/DER encoding/decoding as well as… |
| [const_format](https://crates.io/crates/const_format) | 0.2.37 | Zlib | Compile-time string formatting |
| [const_format_proc_macros](https://crates.io/crates/const_format_proc_macros) | 0.2.37 | Zlib | Implementation detail of the `const_format` crate |
| [convert_case](https://crates.io/crates/convert_case) | 0.11.0 | MIT | Convert strings into any case |
| [cookie](https://crates.io/crates/cookie) | 0.18.2 | Apache-2.0 OR MIT | HTTP cookie parsing and cookie jar management. Supports signed and private (encrypted, authenticated) jars. |
| [core-foundation](https://crates.io/crates/core-foundation) | 0.10.1 | Apache-2.0 OR MIT | Bindings to Core Foundation for macOS |
| [core-foundation](https://crates.io/crates/core-foundation) | 0.9.4 | Apache-2.0 OR MIT | Bindings to Core Foundation for macOS |
| [core-foundation-sys](https://crates.io/crates/core-foundation-sys) | 0.8.7 | Apache-2.0 OR MIT | Bindings to Core Foundation for macOS |
| [core-graphics](https://crates.io/crates/core-graphics) | 0.25.0 | Apache-2.0 OR MIT | Bindings to Core Graphics for macOS |
| [core-graphics-types](https://crates.io/crates/core-graphics-types) | 0.2.0 | Apache-2.0 OR MIT | Bindings for some fundamental Core Graphics types |
| [core_detect](https://crates.io/crates/core_detect) | 1.0.0 | Apache-2.0 OR MIT | A `no_std` version of the `std&#58;&#58;is_x86_feature_detected!` macro. |
| [cpubits](https://crates.io/crates/cpubits) | 0.1.1 | Apache-2.0 OR MIT | Compile-time detection heuristics for the optimal word size to use for the target CPU, which in some cases may differ from its address size a.k.a… |
| [cpufeatures](https://crates.io/crates/cpufeatures) | 0.2.17 | Apache-2.0 OR MIT | Lightweight runtime CPU feature detection for aarch64, loongarch64, and x86/x86_64 targets, with no_std support and support for mobile targets including Android… |
| [cpufeatures](https://crates.io/crates/cpufeatures) | 0.3.1 | Apache-2.0 OR MIT | Lightweight runtime CPU feature detection for aarch64, loongarch64, and x86/x86_64 targets, with no_std support and support for mobile targets including Android… |
| [crc32fast](https://crates.io/crates/crc32fast) | 1.5.3 | Apache-2.0 OR MIT | Fast, SIMD-accelerated CRC32 (IEEE) checksum computation |
| [crossbeam-channel](https://crates.io/crates/crossbeam-channel) | 0.5.17 | Apache-2.0 OR MIT | Multi-producer multi-consumer channels for message passing |
| [crossbeam-deque](https://crates.io/crates/crossbeam-deque) | 0.8.8 | Apache-2.0 OR MIT | Concurrent work-stealing deque |
| [crossbeam-epoch](https://crates.io/crates/crossbeam-epoch) | 0.9.21 | Apache-2.0 OR MIT | Epoch-based garbage collection |
| [crossbeam-utils](https://crates.io/crates/crossbeam-utils) | 0.8.23 | Apache-2.0 OR MIT | Utilities for concurrent programming |
| [crypto-bigint](https://crates.io/crates/crypto-bigint) | 0.7.5 | Apache-2.0 OR MIT | Pure Rust implementation of a big integer library which has been designed from the ground-up for use in cryptographic applications. Provides constant-time… |
| [crypto-common](https://crates.io/crates/crypto-common) | 0.1.7 | Apache-2.0 OR MIT | Common cryptographic traits |
| [crypto-common](https://crates.io/crates/crypto-common) | 0.2.2 | Apache-2.0 OR MIT | Common traits used by cryptographic algorithms |
| [crypto-primes](https://crates.io/crates/crypto-primes) | 0.7.2 | Apache-2.0 OR MIT | Random prime number generation and primality checking library |
| [cssparser](https://crates.io/crates/cssparser) | 0.36.0 | MPL-2.0 | Rust implementation of CSS Syntax Level 3 |
| [cssparser](https://crates.io/crates/cssparser) | 0.37.0 | MPL-2.0 | Rust implementation of CSS Syntax Level 3 |
| [cssparser-macros](https://crates.io/crates/cssparser-macros) | 0.6.1 | MPL-2.0 | Procedural macros for cssparser |
| [cssparser-macros](https://crates.io/crates/cssparser-macros) | 0.7.1 | MPL-2.0 | Procedural macros for cssparser |
| [ctor](https://crates.io/crates/ctor) | 1.0.13 | Apache-2.0 OR MIT | Global, no_std-compatible constructors for all platforms that run before main (like C/C++ __attribute__((constructor))) |
| [ctr](https://crates.io/crates/ctr) | 0.10.1 | Apache-2.0 OR MIT | CTR block modes of operation |
| [ctutils](https://crates.io/crates/ctutils) | 0.4.3 | Apache-2.0 OR MIT | Constant-time utility library with selection and equality testing support targeting cryptographic applications. Supports `const fn` where appropriate. Built on… |
| [curve25519-dalek](https://crates.io/crates/curve25519-dalek) | 5.0.0 | BSD-3-Clause | A pure-Rust implementation of group operations on ristretto255 and Curve25519 |
| [curve25519-dalek-derive](https://crates.io/crates/curve25519-dalek-derive) | 0.1.1 | Apache-2.0 OR MIT | curve25519-dalek Derives |
| [darling](https://crates.io/crates/darling) | 0.24.1 | MIT | A proc-macro library for reading attributes into structs when implementing custom derives. |
| [darling_core](https://crates.io/crates/darling_core) | 0.24.1 | MIT | Helper crate for proc-macro library for reading attributes into structs when implementing custom derives. Use https&#58;//crates.io/crates/darling in your code. |
| [darling_macro](https://crates.io/crates/darling_macro) | 0.24.1 | MIT | Internal support for a proc-macro library for reading attributes into structs when implementing custom derives. Use https&#58;//crates.io/crates/darling in your… |
| [dashmap](https://crates.io/crates/dashmap) | 6.2.1 | MIT | Blazing fast concurrent HashMap for Rust. |
| [data-encoding](https://crates.io/crates/data-encoding) | 2.11.1 | MIT | Efficient and customizable data-encoding functions like base64, base32, and hex |
| [dbus](https://crates.io/crates/dbus) | 0.9.12 | Apache-2.0 OR MIT | Bindings to D-Bus, which is a bus commonly used on Linux for inter-process communication. |
| [defmt](https://crates.io/crates/defmt) | 1.2.0 | Apache-2.0 OR MIT | A highly efficient logging framework that targets resource-constrained devices, like microcontrollers |
| [defmt-macros](https://crates.io/crates/defmt-macros) | 1.2.0 | Apache-2.0 OR MIT | defmt macros |
| [defmt-parser](https://crates.io/crates/defmt-parser) | 1.1.0 | Apache-2.0 OR MIT | Parsing library for defmt format strings |
| [delegate](https://crates.io/crates/delegate) | 0.13.5 | Apache-2.0 OR MIT | Method delegation with less boilerplate |
| [der](https://crates.io/crates/der) | 0.8.2 | Apache-2.0 OR MIT | Pure Rust embedded-friendly implementation of the Distinguished Encoding Rules (DER) for Abstract Syntax Notation One (ASN.1) as described in ITU X.690 with… |
| [deranged](https://crates.io/crates/deranged) | 0.5.9 | Apache-2.0 OR MIT | Ranged integers |
| [derive_arbitrary](https://crates.io/crates/derive_arbitrary) | 1.5.0 | Apache-2.0 OR MIT | Derives arbitrary traits |
| [derive_more](https://crates.io/crates/derive_more) | 2.1.1 | MIT | Adds #[derive(x)] macros for more traits |
| [derive_more-impl](https://crates.io/crates/derive_more-impl) | 2.1.1 | MIT | Internal implementation of `derive_more` crate |
| [des](https://crates.io/crates/des) | 0.9.0 | Apache-2.0 OR MIT | DES and Triple DES (3DES, TDES) block ciphers implementation |
| [deunicode](https://crates.io/crates/deunicode) | 1.6.2 | BSD-3-Clause | Convert Unicode strings to pure ASCII by intelligently transliterating them. Suppors Emoji and Chinese. |
| [digest](https://crates.io/crates/digest) | 0.10.7 | Apache-2.0 OR MIT | Traits for cryptographic hash functions and message authentication codes |
| [digest](https://crates.io/crates/digest) | 0.11.3 | Apache-2.0 OR MIT | Traits for cryptographic hash functions and message authentication codes |
| [dirs](https://crates.io/crates/dirs) | 7.0.0 | Apache-2.0 OR MIT | A tiny low-level library that provides platform-specific standard locations of directories for config, cache and other data on Linux, Windows, macOS and Redox… |
| [dirs-sys](https://crates.io/crates/dirs-sys) | 0.5.0 | Apache-2.0 OR MIT | System-level helper functions for the dirs and directories crates. |
| [dispatch2](https://crates.io/crates/dispatch2) | 0.3.1 | Apache-2.0 OR MIT OR Zlib | Bindings and wrappers for Apple's Grand Central Dispatch (GCD) |
| [displaydoc](https://crates.io/crates/displaydoc) | 0.2.7 | Apache-2.0 OR MIT | A derive macro for implementing the display Trait via a doc comment and string interpolation |
| [dlopen2](https://crates.io/crates/dlopen2) | 0.8.2 | MIT | Library for opening and operating on dynamic link libraries (also known as shared objects or shared libraries). |
| [dlopen2_derive](https://crates.io/crates/dlopen2_derive) | 0.4.3 | MIT | Derive macros for the dlopen2 crate. |
| [dom_query](https://crates.io/crates/dom_query) | 0.28.0 | MIT | HTML querying and manipulation with CSS selectors |
| [dpi](https://crates.io/crates/dpi) | 0.1.2 | Apache-2.0 AND MIT | Types for handling UI scaling |
| [dtoa](https://crates.io/crates/dtoa) | 1.0.11 | Apache-2.0 OR MIT | Fast floating point primitive to string conversion |
| [dtoa-short](https://crates.io/crates/dtoa-short) | 0.3.5 | MPL-2.0 | Serialize float number and truncate to certain precision |
| [dunce](https://crates.io/crates/dunce) | 1.0.5 | Apache-2.0 OR CC0-1.0 OR MIT-0 | Normalize Windows paths to the most compatible format, avoiding UNC where possible |
| [dyn-clone](https://crates.io/crates/dyn-clone) | 1.0.20 | Apache-2.0 OR MIT | Clone trait that is dyn-compatible |
| [ecdsa](https://crates.io/crates/ecdsa) | 0.17.0 | Apache-2.0 OR MIT | Pure Rust implementation of the Elliptic Curve Digital Signature Algorithm (ECDSA) as specified in FIPS 186-4 (Digital Signature Standard), implemented… |
| [ed25519](https://crates.io/crates/ed25519) | 3.0.0 | Apache-2.0 OR MIT | Edwards Digital Signature Algorithm (EdDSA) over Curve25519 (as specified in RFC 8032) support library providing signature type definitions and PKCS#8 private… |
| [ed25519-dalek](https://crates.io/crates/ed25519-dalek) | 3.0.0 | BSD-3-Clause | Fast and efficient ed25519 EdDSA key generations, signing, and verification in pure Rust. |
| [either](https://crates.io/crates/either) | 1.19.0 | Apache-2.0 OR MIT | The enum `Either` with variants `Left` and `Right` is a general purpose sum type with two cases. |
| [elliptic-curve](https://crates.io/crates/elliptic-curve) | 0.14.1 | Apache-2.0 OR MIT | General purpose Elliptic Curve Cryptography (ECC) support, including traits and generic types for representing various elliptic curve forms, scalars, points… |
| [embed-resource](https://crates.io/crates/embed-resource) | 3.0.12 | MIT | A Cargo library to handle compilation and inclusion of Windows resources in the most resilient fashion imaginable |
| [embed_plist](https://crates.io/crates/embed_plist) | 1.2.2 | Apache-2.0 OR MIT | Embed property list files like Info.plist directly in your executable binary. |
| [emojis](https://crates.io/crates/emojis) | 0.8.2 | (MIT OR Apache-2.0) AND Unicode-3.0 | ✨ Lookup emoji in *O(1)* time, access metadata and GitHub shortcodes, iterate over all emoji, and more! |
| [encode_unicode](https://crates.io/crates/encode_unicode) | 1.0.0 | Apache-2.0 OR MIT | UTF-8 and UTF-16 character types, iterators and related methods for char, u8 and u16. |
| [encoding_rs](https://crates.io/crates/encoding_rs) | 0.8.42 | (Apache-2.0 OR MIT) AND BSD-3-Clause | A Gecko-oriented implementation of the Encoding Standard |
| [endi](https://crates.io/crates/endi) | 1.1.1 | MIT | A simple endian-handling library |
| [entities](https://crates.io/crates/entities) | 1.0.1 | MIT | Provides the raw data needed to convert to and from HTML entities. |
| [enum_dispatch](https://crates.io/crates/enum_dispatch) | 0.3.13 | Apache-2.0 OR MIT | Near drop-in replacement for dynamic-dispatched method calls with up to 10x the speed |
| [enumflags2](https://crates.io/crates/enumflags2) | 0.7.12 | Apache-2.0 OR MIT | Enum-based bit flags |
| [enumflags2_derive](https://crates.io/crates/enumflags2_derive) | 0.7.12 | Apache-2.0 OR MIT | Do not use directly, use the reexport in the `enumflags2` crate. This allows for better compatibility across versions. |
| [envy](https://crates.io/crates/envy) | 0.4.2 | MIT | deserialize env vars into typesafe structs |
| [equivalent](https://crates.io/crates/equivalent) | 1.0.3 | Apache-2.0 OR MIT | Traits for key comparison in maps. |
| [erased-serde](https://crates.io/crates/erased-serde) | 0.4.10 | Apache-2.0 OR MIT | Type-erased Serialize and Serializer traits |
| [errno](https://crates.io/crates/errno) | 0.3.14 | Apache-2.0 OR MIT | Cross-platform interface to the `errno` variable. |
| [event-listener](https://crates.io/crates/event-listener) | 5.4.2 | Apache-2.0 OR MIT | Notify async tasks or threads |
| [event-listener-strategy](https://crates.io/crates/event-listener-strategy) | 0.5.4 | Apache-2.0 OR MIT | Block or poll on event_listener easily |
| [fancy-regex](https://crates.io/crates/fancy-regex) | 0.16.2 | MIT | An implementation of regexes, supporting a relatively rich set of features, including backreferences and look-around. |
| [fastrand](https://crates.io/crates/fastrand) | 2.5.0 | Apache-2.0 OR MIT | A simple and fast random number generator |
| [fdeflate](https://crates.io/crates/fdeflate) | 0.3.7 | Apache-2.0 OR MIT | Fast specialized deflate implementation |
| [fearless_simd](https://crates.io/crates/fearless_simd) | 1.1.0 | Apache-2.0 OR MIT | Safer and easier SIMD |
| [fearless_simd_macros](https://crates.io/crates/fearless_simd_macros) | 0.1.0 | Apache-2.0 OR MIT | Procedural macros for fearless_simd |
| [ff](https://crates.io/crates/ff) | 0.14.0 | Apache-2.0 OR MIT | Library for building and interfacing with finite fields |
| [fiat-crypto](https://crates.io/crates/fiat-crypto) | 0.3.0 | Apache-2.0 OR BSD-1-Clause OR MIT | Fiat-crypto generated Rust |
| [field-offset](https://crates.io/crates/field-offset) | 0.3.6 | Apache-2.0 OR MIT | Safe pointer-to-member implementation |
| [filetime](https://crates.io/crates/filetime) | 0.2.29 | Apache-2.0 OR MIT | Platform-agnostic accessors of timestamps in File metadata |
| [find-msvc-tools](https://crates.io/crates/find-msvc-tools) | 0.1.15 | Apache-2.0 OR MIT | Find windows-specific tools, read MSVC versions from the registry and from COM interfaces |
| [finl_unicode](https://crates.io/crates/finl_unicode) | 1.5.0 | (MIT OR Apache-2.0) AND Unicode-DFS-2016 | Library for handling Unicode functionality for finl (categories and grapheme segmentation) |
| [flate2](https://crates.io/crates/flate2) | 1.1.10 | Apache-2.0 OR MIT | DEFLATE compression and decompression exposed as Read/BufRead/Write streams. Supports miniz_oxide and multiple zlib implementations. Supports zlib, gzip, and… |
| [fnv](https://crates.io/crates/fnv) | 1.0.7 | Apache-2.0 OR MIT | Fowler–Noll–Vo hash function |
| [foldhash](https://crates.io/crates/foldhash) | 0.2.0 | Zlib | A fast, non-cryptographic, minimally DoS-resistant hashing algorithm. |
| [foreign-types](https://crates.io/crates/foreign-types) | 0.5.0 | Apache-2.0 OR MIT | A framework for Rust wrappers over C APIs |
| [foreign-types-macros](https://crates.io/crates/foreign-types-macros) | 0.2.4 | Apache-2.0 OR MIT | An internal crate used by foreign-types |
| [foreign-types-shared](https://crates.io/crates/foreign-types-shared) | 0.3.1 | Apache-2.0 OR MIT | An internal crate used by foreign-types |
| [form_urlencoded](https://crates.io/crates/form_urlencoded) | 1.2.2 | Apache-2.0 OR MIT | Parser and serializer for the application/x-www-form-urlencoded syntax, as used by HTML forms. |
| [futures](https://crates.io/crates/futures) | 0.3.34 | Apache-2.0 OR MIT | An implementation of futures and streams featuring zero allocations, composability, and iterator-like interfaces. |
| [futures-channel](https://crates.io/crates/futures-channel) | 0.3.34 | Apache-2.0 OR MIT | Channels for asynchronous communication using futures-rs. |
| [futures-core](https://crates.io/crates/futures-core) | 0.3.34 | Apache-2.0 OR MIT | The core traits and types in for the `futures` library. |
| [futures-executor](https://crates.io/crates/futures-executor) | 0.3.34 | Apache-2.0 OR MIT | Executors for asynchronous tasks based on the futures-rs library. |
| [futures-io](https://crates.io/crates/futures-io) | 0.3.34 | Apache-2.0 OR MIT | The `AsyncRead`, `AsyncWrite`, `AsyncSeek`, and `AsyncBufRead` traits for the futures-rs library. |
| [futures-lite](https://crates.io/crates/futures-lite) | 2.6.1 | Apache-2.0 OR MIT | Futures, streams, and async I/O combinators |
| [futures-macro](https://crates.io/crates/futures-macro) | 0.3.34 | Apache-2.0 OR MIT | The futures-rs procedural macro implementations. |
| [futures-sink](https://crates.io/crates/futures-sink) | 0.3.34 | Apache-2.0 OR MIT | The asynchronous `Sink` trait for the futures-rs library. |
| [futures-task](https://crates.io/crates/futures-task) | 0.3.34 | Apache-2.0 OR MIT | Tools for working with tasks. |
| [futures-util](https://crates.io/crates/futures-util) | 0.3.34 | Apache-2.0 OR MIT | Common utilities and extension traits for the futures-rs library. |
| [gdk](https://crates.io/crates/gdk) | 0.18.2 | MIT | UNMAINTAINED Rust bindings for the GDK 3 library (use gdk4 instead) |
| [gdk-pixbuf](https://crates.io/crates/gdk-pixbuf) | 0.18.5 | MIT | Rust bindings for the GdkPixbuf library |
| [gdk-pixbuf-sys](https://crates.io/crates/gdk-pixbuf-sys) | 0.18.0 | MIT | FFI bindings to libgdk_pixbuf-2.0 |
| [gdk-sys](https://crates.io/crates/gdk-sys) | 0.18.2 | MIT | UNMAINTAINED FFI bindings to libgdk-3 (use gdk4-sys instead) |
| [gdkwayland-sys](https://crates.io/crates/gdkwayland-sys) | 0.18.2 | MIT | UNMAINTAINED FFI bindings to libgdk-3-wayland (use gdk4-wayland-sys instead) |
| [gdkx11](https://crates.io/crates/gdkx11) | 0.18.2 | MIT | UNMAINTAINED Rust bindings for the GDK X11 library (use gdk4-x11 instead) |
| [gdkx11-sys](https://crates.io/crates/gdkx11-sys) | 0.18.2 | MIT | UNMAINTAINED FFI binding for libgdkx11 (use gdk4-x11-sys instead) |
| [generic-array](https://crates.io/crates/generic-array) | 0.14.7 | MIT | Generic types implementing functionality of arrays |
| [generic-array](https://crates.io/crates/generic-array) | 1.4.5 | MIT | Generic types implementing functionality of arrays |
| [getrandom](https://crates.io/crates/getrandom) | 0.2.17 | Apache-2.0 OR MIT | A small cross-platform library for retrieving random data from system source |
| [getrandom](https://crates.io/crates/getrandom) | 0.3.4 | Apache-2.0 OR MIT | A small cross-platform library for retrieving random data from system source |
| [getrandom](https://crates.io/crates/getrandom) | 0.4.3 | Apache-2.0 OR MIT | A small cross-platform library for retrieving random data from system source |
| [ghash](https://crates.io/crates/ghash) | 0.6.0 | Apache-2.0 OR MIT | Universal hash over GF(2^128) useful for constructing a Message Authentication Code (MAC), as in the AES-GCM authenticated encryption cipher. |
| [gif](https://crates.io/crates/gif) | 0.14.2 | Apache-2.0 OR MIT | GIF de- and encoder |
| [gio](https://crates.io/crates/gio) | 0.18.4 | MIT | Rust bindings for the Gio library |
| [gio-sys](https://crates.io/crates/gio-sys) | 0.18.1 | MIT | FFI bindings to libgio-2.0 |
| [git2](https://crates.io/crates/git2) | 0.21.0 | Apache-2.0 OR MIT | Bindings to libgit2 for interoperating with git repositories. This library is both threadsafe and memory safe and allows both reading and writing git… |
| [glib](https://crates.io/crates/glib) | 0.18.5 | MIT | Rust bindings for the GLib library |
| [glib-macros](https://crates.io/crates/glib-macros) | 0.18.5 | MIT | Rust bindings for the GLib library, proc macros crate |
| [glib-sys](https://crates.io/crates/glib-sys) | 0.18.1 | MIT | FFI bindings to libglib-2.0 |
| [glob](https://crates.io/crates/glob) | 0.3.4 | Apache-2.0 OR MIT | Support for matching file paths against Unix shell style patterns. |
| [gloo-timers](https://crates.io/crates/gloo-timers) | 0.4.0 | Apache-2.0 OR MIT | Convenience crate for working with JavaScript timers |
| [gobject-sys](https://crates.io/crates/gobject-sys) | 0.18.0 | MIT | FFI bindings to libgobject-2.0 |
| [group](https://crates.io/crates/group) | 0.14.0 | Apache-2.0 OR MIT | Elliptic curve group traits and utilities |
| [gtk](https://crates.io/crates/gtk) | 0.18.2 | MIT | UNMAINTAINED Rust bindings for the GTK+ 3 library (use gtk4 instead) |
| [gtk-sys](https://crates.io/crates/gtk-sys) | 0.18.2 | MIT | UNMAINTAINED FFI bindings to libgtk-3 (use gtk4-sys instead) |
| [gtk3-macros](https://crates.io/crates/gtk3-macros) | 0.18.2 | MIT | UNMAINTAINED Rust bindings for the GTK 3 library (use gtk4-macros instead) |
| [hashbrown](https://crates.io/crates/hashbrown) | 0.12.3 | Apache-2.0 OR MIT | A Rust port of Google's SwissTable hash map |
| [hashbrown](https://crates.io/crates/hashbrown) | 0.14.5 | Apache-2.0 OR MIT | A Rust port of Google's SwissTable hash map |
| [hashbrown](https://crates.io/crates/hashbrown) | 0.16.1 | Apache-2.0 OR MIT | A Rust port of Google's SwissTable hash map |
| [hashbrown](https://crates.io/crates/hashbrown) | 0.17.1 | Apache-2.0 OR MIT | A Rust port of Google's SwissTable hash map |
| [heck](https://crates.io/crates/heck) | 0.4.1 | Apache-2.0 OR MIT | heck is a case conversion library. |
| [heck](https://crates.io/crates/heck) | 0.5.0 | Apache-2.0 OR MIT | heck is a case conversion library. |
| [hermit-abi](https://crates.io/crates/hermit-abi) | 0.5.3 | Apache-2.0 OR MIT | Hermit system calls definitions. |
| [hex](https://crates.io/crates/hex) | 0.4.3 | Apache-2.0 OR MIT | Encoding and decoding data into/from hexadecimal representation. |
| [hex-literal](https://crates.io/crates/hex-literal) | 1.1.0 | Apache-2.0 OR MIT | A macro for converting hexadecimal string literals to a byte array at compile time |
| [hkdf](https://crates.io/crates/hkdf) | 0.13.0 | Apache-2.0 OR MIT | HMAC-based Extract-and-Expand Key Derivation Function (HKDF) |
| [hmac](https://crates.io/crates/hmac) | 0.13.0 | Apache-2.0 OR MIT | Generic implementation of Hash-based Message Authentication Code (HMAC) |
| [html-escape](https://crates.io/crates/html-escape) | 0.2.15 | MIT | This library is for encoding/escaping special characters in HTML and decoding/unescaping HTML entities as well. |
| [html5ever](https://crates.io/crates/html5ever) | 0.39.0 | Apache-2.0 OR MIT | High-performance browser-grade HTML5 parser |
| [http](https://crates.io/crates/http) | 1.5.0 | Apache-2.0 OR MIT | A set of types for representing HTTP requests and responses. |
| [http-body](https://crates.io/crates/http-body) | 1.1.0 | MIT | Trait representing an asynchronous, streaming, HTTP request or response body. |
| [http-body-util](https://crates.io/crates/http-body-util) | 0.1.5 | MIT | Combinators and adapters for HTTP request or response bodies. |
| [http-range](https://crates.io/crates/http-range) | 0.1.6 | MIT | HTTP Range header parser |
| [httparse](https://crates.io/crates/httparse) | 1.10.1 | Apache-2.0 OR MIT | A tiny, safe, speedy, zero-copy HTTP/1.x parser. |
| [httpdate](https://crates.io/crates/httpdate) | 1.0.3 | Apache-2.0 OR MIT | HTTP date parsing and formatting |
| [hybrid-array](https://crates.io/crates/hybrid-array) | 0.4.15 | Apache-2.0 OR MIT | Hybrid typenum-based and const generic array types designed to provide the flexibility of typenum-based expressions while also allowing interoperability and a… |
| [hyper](https://crates.io/crates/hyper) | 1.12.0 | MIT | A protective and efficient HTTP library for all. |
| [hyper-rustls](https://crates.io/crates/hyper-rustls) | 0.27.10 | Apache-2.0 OR ISC OR MIT | Rustls+hyper integration for pure rust HTTPS |
| [hyper-util](https://crates.io/crates/hyper-util) | 0.1.21 | MIT | hyper utilities |
| [iana-time-zone](https://crates.io/crates/iana-time-zone) | 0.1.65 | Apache-2.0 OR MIT | get the IANA time zone for the current system |
| [iana-time-zone-haiku](https://crates.io/crates/iana-time-zone-haiku) | 0.1.2 | Apache-2.0 OR MIT | iana-time-zone support crate for Haiku OS |
| [ico](https://crates.io/crates/ico) | 0.5.0 | MIT | A library for encoding/decoding ICO image files |
| [icu_collections](https://crates.io/crates/icu_collections) | 2.3.0 | Unicode-3.0 | Collection of API for use in ICU libraries. |
| [icu_locale_core](https://crates.io/crates/icu_locale_core) | 2.3.0 | Unicode-3.0 | API for managing Unicode Language and Locale Identifiers |
| [icu_normalizer](https://crates.io/crates/icu_normalizer) | 2.3.0 | Unicode-3.0 | API for normalizing text into Unicode Normalization Forms |
| [icu_normalizer_data](https://crates.io/crates/icu_normalizer_data) | 2.3.0 | Unicode-3.0 | Data for the icu_normalizer crate |
| [icu_properties](https://crates.io/crates/icu_properties) | 2.3.0 | Unicode-3.0 | Definitions for Unicode properties |
| [icu_properties_data](https://crates.io/crates/icu_properties_data) | 2.3.0 | Unicode-3.0 | Data for the icu_properties crate |
| [icu_provider](https://crates.io/crates/icu_provider) | 2.3.1 | Unicode-3.0 | Trait and struct definitions for the ICU data provider |
| [ident_case](https://crates.io/crates/ident_case) | 1.0.1 | Apache-2.0 OR MIT | Utility for applying case rules to Rust identifiers. |
| [idna](https://crates.io/crates/idna) | 1.1.0 | Apache-2.0 OR MIT | IDNA (Internationalizing Domain Names in Applications) and Punycode. |
| [idna_adapter](https://crates.io/crates/idna_adapter) | 1.2.2 | Apache-2.0 OR MIT | Back end adapter for idna |
| [image](https://crates.io/crates/image) | 0.25.10 | Apache-2.0 OR MIT | Imaging library. Provides basic image processing and encoders/decoders for common image formats. |
| [image-webp](https://crates.io/crates/image-webp) | 0.2.4 | Apache-2.0 OR MIT | WebP encoding and decoding in pure Rust |
| [include_dir](https://crates.io/crates/include_dir) | 0.7.4 | MIT | Embed the contents of a directory in your binary |
| [include_dir_macros](https://crates.io/crates/include_dir_macros) | 0.7.4 | MIT | The procedural macro used by include_dir |
| [indexmap](https://crates.io/crates/indexmap) | 1.9.3 | Apache-2.0 OR MIT | A hash table with consistent order and fast iteration. |
| [indexmap](https://crates.io/crates/indexmap) | 2.14.2 | Apache-2.0 OR MIT | A hash table with consistent order and fast iteration. |
| [infer](https://crates.io/crates/infer) | 0.22.0 | MIT | Small crate to infer file type based on magic number signatures |
| [inout](https://crates.io/crates/inout) | 0.2.5 | Apache-2.0 OR MIT | Custom reference types for code generic over in-place and buffer-to-buffer modes of operation. |
| [ipnet](https://crates.io/crates/ipnet) | 2.12.2 | Apache-2.0 OR MIT | Provides types and useful methods for working with IPv4 and IPv6 network addresses, commonly called IP prefixes. The new `IpNet`, `Ipv4Net`, and `Ipv6Net` types… |
| [is-docker](https://crates.io/crates/is-docker) | 0.2.0 | MIT | Checks if the process is running inside a Docker container. |
| [is-wsl](https://crates.io/crates/is-wsl) | 0.4.0 | MIT | Checks if the process is running inside Windows Subsystem for Linux. |
| [is_terminal_polyfill](https://crates.io/crates/is_terminal_polyfill) | 1.70.2 | Apache-2.0 OR MIT | Polyfill for `is_terminal` stdlib feature for use with older MSRVs |
| [itertools](https://crates.io/crates/itertools) | 0.14.0 | Apache-2.0 OR MIT | Extra iterator adaptors, iterator methods, free functions, and macros. |
| [itoa](https://crates.io/crates/itoa) | 1.0.18 | Apache-2.0 OR MIT | Fast integer primitive to string conversion |
| [javascriptcore-rs](https://crates.io/crates/javascriptcore-rs) | 1.1.2 | MIT | Rust bindings for the javacriptcore library |
| [javascriptcore-rs-sys](https://crates.io/crates/javascriptcore-rs-sys) | 1.1.1 | MIT | Sys functions for the Rust bindings of the javacriptcore library |
| [jiff](https://crates.io/crates/jiff) | 0.2.38 | MIT OR Unlicense | A date-time library that encourages you to jump into the pit of success. This library is heavily inspired by the Temporal project. |
| [jiff-core](https://crates.io/crates/jiff-core) | 0.1.1 | MIT OR Unlicense | Low level datetime primitives for the Jiff library. |
| [jiff-static](https://crates.io/crates/jiff-static) | 0.2.38 | MIT OR Unlicense | Create static TimeZone values for Jiff (useful in core-only environments). |
| [jiff-tzdb](https://crates.io/crates/jiff-tzdb) | 0.1.9 | MIT OR Unlicense | The entire Time Zone Database embedded into your binary. |
| [jiff-tzdb-platform](https://crates.io/crates/jiff-tzdb-platform) | 0.1.3 | MIT OR Unlicense | The entire Time Zone Database embedded into your binary for specific platforms. |
| [jni](https://crates.io/crates/jni) | 0.21.1 | Apache-2.0 OR MIT | Rust bindings to the JNI |
| [jni](https://crates.io/crates/jni) | 0.22.4 | Apache-2.0 OR MIT | Rust bindings to the JNI |
| [jni-macros](https://crates.io/crates/jni-macros) | 0.22.4 | Apache-2.0 OR MIT | Procedural macros for the jni crate |
| [jni-sys](https://crates.io/crates/jni-sys) | 0.3.1 | Apache-2.0 OR MIT | Rust definitions corresponding to jni.h |
| [jni-sys](https://crates.io/crates/jni-sys) | 0.4.1 | Apache-2.0 OR MIT | Rust definitions corresponding to jni.h |
| [jni-sys-macros](https://crates.io/crates/jni-sys-macros) | 0.4.1 | Apache-2.0 OR MIT | Macros for jni-sys crate |
| [jobserver](https://crates.io/crates/jobserver) | 0.1.35 | Apache-2.0 OR MIT | An implementation of the GNU Make jobserver for Rust. |
| [js-sys](https://crates.io/crates/js-sys) | 0.3.106 | Apache-2.0 OR MIT | Bindings for all JS global objects and functions in all JS environments like Node.js and browsers, built on `#[wasm_bindgen]` using the `wasm-bindgen` crate. |
| [json-patch](https://crates.io/crates/json-patch) | 4.2.0 | Apache-2.0 OR MIT | RFC 6902, JavaScript Object Notation (JSON) Patch |
| [jsonptr](https://crates.io/crates/jsonptr) | 0.7.1 | Apache-2.0 OR MIT | Data structures and logic for resolving, assigning, and deleting by JSON Pointers (RFC 6901) |
| [keccak](https://crates.io/crates/keccak) | 0.2.2 | Apache-2.0 OR MIT | Pure Rust implementation of the Keccak sponge functions |
| [kem](https://crates.io/crates/kem) | 0.3.0 | Apache-2.0 OR MIT | Traits for Key Encapsulation Mechanisms (KEMs)&#58; public-key cryptosystems designed to enable a sender (a.k.a. encapsulator) to generate and encrypt a short… |
| [keyboard-types](https://crates.io/crates/keyboard-types) | 0.8.3 | Apache-2.0 OR MIT | Contains types to define keyboard related events. |
| [keyring](https://crates.io/crates/keyring) | 4.2.0 | Apache-2.0 OR MIT | All-in-One Rust Keyring |
| [keyring-core](https://crates.io/crates/keyring-core) | 1.0.1 | Apache-2.0 OR MIT | Cross-platform library for managing passwords/secrets |
| [konst](https://crates.io/crates/konst) | 0.2.20 | Zlib | Const equivalents of std functions, compile-time comparison, and parsing |
| [konst_macro_rules](https://crates.io/crates/konst_macro_rules) | 0.2.19 | Zlib | Implementation detail of the konst crate |
| [lazy_static](https://crates.io/crates/lazy_static) | 1.5.1 | Apache-2.0 OR MIT | A macro for declaring lazily evaluated statics in Rust. |
| [lexical-core](https://crates.io/crates/lexical-core) | 1.0.6 | Apache-2.0 OR MIT | Lexical, to- and from-string conversion routines. |
| [lexical-parse-float](https://crates.io/crates/lexical-parse-float) | 1.0.6 | Apache-2.0 OR MIT | Efficient parsing of floats from strings. |
| [lexical-parse-integer](https://crates.io/crates/lexical-parse-integer) | 1.0.6 | Apache-2.0 OR MIT | Efficient parsing of integers from strings. |
| [lexical-util](https://crates.io/crates/lexical-util) | 1.0.7 | Apache-2.0 OR MIT | Shared utilities for lexical creates. |
| [lexical-write-float](https://crates.io/crates/lexical-write-float) | 1.0.6 | Apache-2.0 OR MIT | Efficient formatting of floats to strings. |
| [lexical-write-integer](https://crates.io/crates/lexical-write-integer) | 1.0.6 | Apache-2.0 OR MIT | Efficient formatting of integers to strings. |
| [libappindicator](https://crates.io/crates/libappindicator) | 0.9.0 | Apache-2.0 OR MIT | Rust safe bindings for libappindicator |
| [libappindicator-sys](https://crates.io/crates/libappindicator-sys) | 0.9.0 | Apache-2.0 OR MIT | Rust sys bindings for libappindicator |
| [libc](https://crates.io/crates/libc) | 0.2.191 | Apache-2.0 OR MIT | Raw FFI bindings to platform libraries like libc. |
| [libdbus-sys](https://crates.io/crates/libdbus-sys) | 0.2.7 | Apache-2.0 OR MIT | FFI bindings to libdbus. |
| [libgit2-sys](https://crates.io/crates/libgit2-sys) | 0.18.8+1.9.7 | Apache-2.0 OR MIT | Native bindings to the libgit2 library |
| [libloading](https://crates.io/crates/libloading) | 0.7.4 | ISC | Bindings around the platform's dynamic library loading primitives with greatly improved memory safety. |
| [libredox](https://crates.io/crates/libredox) | 0.1.25 | MIT | Redox stable ABI |
| [libwebp-sys](https://crates.io/crates/libwebp-sys) | 0.9.6 | MIT | Bindings to libwebp (bindgen, static linking) |
| [libz-sys](https://crates.io/crates/libz-sys) | 1.1.30 | Apache-2.0 OR MIT | Low-level bindings to the system libz library (also known as zlib). |
| [linked-hash-map](https://crates.io/crates/linked-hash-map) | 0.5.6 | Apache-2.0 OR MIT | A HashMap wrapper that holds key-value pairs in insertion order |
| [linux-raw-sys](https://crates.io/crates/linux-raw-sys) | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Generated bindings for Linux's userspace API |
| [litemap](https://crates.io/crates/litemap) | 0.8.3 | Unicode-3.0 | A key-value Map implementation based on a flat, sorted Vec. |
| [lock_api](https://crates.io/crates/lock_api) | 0.4.14 | Apache-2.0 OR MIT | Wrappers to create fully-featured Mutex and RwLock types. Compatible with no_std. |
| [lofty](https://crates.io/crates/lofty) | 0.25.5 | Apache-2.0 OR MIT | Audio metadata library |
| [lofty_attr](https://crates.io/crates/lofty_attr) | 0.13.0 | Apache-2.0 OR MIT | Macros for Lofty |
| [log](https://crates.io/crates/log) | 0.4.34 | Apache-2.0 OR MIT | A lightweight logging facade for Rust |
| [lol_html](https://crates.io/crates/lol_html) | 2.9.0 | BSD-3-Clause | Streaming HTML rewriter/parser with CSS selector-based API |
| [markup5ever](https://crates.io/crates/markup5ever) | 0.39.0 | Apache-2.0 OR MIT | Common code for xml5ever and html5ever |
| [md5](https://crates.io/crates/md5) | 0.8.1 | Apache-2.0 OR MIT | The package provides the MD5 hash function. |
| [memchr](https://crates.io/crates/memchr) | 2.8.3 | MIT OR Unlicense | Provides extremely fast (uses SIMD on x86_64, aarch64 and wasm32) routines for 1, 2 or 3 byte search and single substring search. |
| [memchr-n](https://crates.io/crates/memchr-n) | 0.1.10 | Apache-2.0 OR MIT | Fast SIMD-accelerated search and iteration over the bytes belonging to an arbitrary set of byte values |
| [memoffset](https://crates.io/crates/memoffset) | 0.9.1 | MIT | offset_of functionality for Rust structs. |
| [mime](https://crates.io/crates/mime) | 0.3.17 | Apache-2.0 OR MIT | Strongly Typed Mimes |
| [minicbor](https://crates.io/crates/minicbor) | 2.3.1 | BlueOak-1.0.0 | A small CBOR codec suitable for no_std environments. |
| [minicbor-derive](https://crates.io/crates/minicbor-derive) | 0.19.5 | BlueOak-1.0.0 | Derive minicbor `Decode` and `Encode` traits. |
| [minifier](https://crates.io/crates/minifier) | 0.3.6 | MIT | Minifier tool/lib for JS/CSS/JSON files |
| [minimal-lexical](https://crates.io/crates/minimal-lexical) | 0.2.1 | Apache-2.0 OR MIT | Fast float parsing conversion routines. |
| [minisign-verify](https://crates.io/crates/minisign-verify) | 0.3.0 | MIT | Minisign signature verification for files and streams with Ed25519. A small Rust library with no external dependencies. |
| [miniz_oxide](https://crates.io/crates/miniz_oxide) | 0.8.9 | Apache-2.0 OR MIT OR Zlib | DEFLATE compression and decompression library rewritten in Rust based on miniz |
| [miniz_oxide](https://crates.io/crates/miniz_oxide) | 0.9.1 | Apache-2.0 OR MIT OR Zlib | DEFLATE compression and decompression library rewritten in Rust based on miniz |
| [mio](https://crates.io/crates/mio) | 1.2.4 | MIT | Lightweight non-blocking I/O. |
| [ml-kem](https://crates.io/crates/ml-kem) | 0.3.3 | Apache-2.0 OR MIT | Pure Rust implementation of the Module-Lattice-Based Key-Encapsulation Mechanism Standard (formerly known as Kyber) as described in FIPS 203 |
| [module-lattice](https://crates.io/crates/module-lattice) | 0.2.3 | Apache-2.0 OR MIT | Functionality shared between the `ml-kem` and `ml-dsa` crates, including linear algebra with degree-256 polynomials over a prime-order field, vectors of such… |
| [moxcms](https://crates.io/crates/moxcms) | 0.8.1 | Apache-2.0 OR BSD-3-Clause | Simple Color Management in Rust |
| [muda](https://crates.io/crates/muda) | 0.20.0 | Apache-2.0 OR MIT | Menu Utilities for Desktop Applications |
| [multiversion_no_op](https://crates.io/crates/multiversion_no_op) | 1.0.0 | Apache-2.0 OR MIT | Provides a pass-through function attribute called multiversion for optimizing build times |
| [ndk](https://crates.io/crates/ndk) | 0.9.0 | Apache-2.0 OR MIT | Safe Rust bindings to the Android NDK |
| [ndk-context](https://crates.io/crates/ndk-context) | 0.1.1 | Apache-2.0 OR MIT | Handles for accessing Android APIs |
| [ndk-sys](https://crates.io/crates/ndk-sys) | 0.6.0+11769913 | Apache-2.0 OR MIT | FFI bindings for the Android NDK |
| [new_debug_unreachable](https://crates.io/crates/new_debug_unreachable) | 1.0.6 | MIT | panic in debug, intrinsics&#58;&#58;unreachable() in release (fork of debug_unreachable) |
| [nix](https://crates.io/crates/nix) | 0.31.3 | MIT | Rust friendly bindings to *nix APIs |
| [nom](https://crates.io/crates/nom) | 7.1.3 | MIT | A byte-oriented, zero-copy, parser combinators library |
| [num](https://crates.io/crates/num) | 0.4.3 | Apache-2.0 OR MIT | A collection of numeric types and traits for Rust, including bigint, complex, rational, range iterators, generic integers, and more! |
| [num-bigint](https://crates.io/crates/num-bigint) | 0.4.8 | Apache-2.0 OR MIT | Big integer implementation for Rust |
| [num-bigint](https://crates.io/crates/num-bigint) | 0.5.1 | Apache-2.0 OR MIT | Big integer implementation for Rust |
| [num-complex](https://crates.io/crates/num-complex) | 0.4.6 | Apache-2.0 OR MIT | Complex numbers implementation for Rust |
| [num-conv](https://crates.io/crates/num-conv) | 0.2.2 | Apache-2.0 OR MIT | `num_conv` is a crate to convert between integer types without using `as` casts. This provides better certainty when refactoring, makes the exact behavior of… |
| [num-integer](https://crates.io/crates/num-integer) | 0.1.47 | Apache-2.0 OR MIT | Integer traits and functions |
| [num-iter](https://crates.io/crates/num-iter) | 0.1.46 | Apache-2.0 OR MIT | External iterators for generic mathematics |
| [num-rational](https://crates.io/crates/num-rational) | 0.4.2 | Apache-2.0 OR MIT | Rational numbers implementation for Rust |
| [num-traits](https://crates.io/crates/num-traits) | 0.2.19 | Apache-2.0 OR MIT | Numeric traits for generic mathematics |
| [num_enum](https://crates.io/crates/num_enum) | 0.7.6 | Apache-2.0 OR BSD-3-Clause OR MIT | Procedural macros to make inter-operation between primitives and enums easier. |
| [num_enum_derive](https://crates.io/crates/num_enum_derive) | 0.7.6 | Apache-2.0 OR BSD-3-Clause OR MIT | Internal implementation details for &#58;&#58;num_enum (Procedural macros to make inter-operation between primitives and enums easier) |
| [objc2](https://crates.io/crates/objc2) | 0.6.5 | MIT | Objective-C interface and runtime bindings |
| [objc2-app-kit](https://crates.io/crates/objc2-app-kit) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the AppKit framework |
| [objc2-cloud-kit](https://crates.io/crates/objc2-cloud-kit) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the CloudKit framework |
| [objc2-core-data](https://crates.io/crates/objc2-core-data) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the CoreData framework |
| [objc2-core-foundation](https://crates.io/crates/objc2-core-foundation) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the CoreFoundation framework |
| [objc2-core-graphics](https://crates.io/crates/objc2-core-graphics) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the CoreGraphics framework |
| [objc2-core-image](https://crates.io/crates/objc2-core-image) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the CoreImage framework |
| [objc2-core-location](https://crates.io/crates/objc2-core-location) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the CoreLocation framework |
| [objc2-core-text](https://crates.io/crates/objc2-core-text) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the CoreText framework |
| [objc2-encode](https://crates.io/crates/objc2-encode) | 4.1.0 | MIT | Objective-C type-encoding representation and parsing |
| [objc2-exception-helper](https://crates.io/crates/objc2-exception-helper) | 0.1.1 | Apache-2.0 OR MIT OR Zlib | External helper function for catching Objective-C exceptions |
| [objc2-foundation](https://crates.io/crates/objc2-foundation) | 0.3.2 | MIT | Bindings to the Foundation framework |
| [objc2-io-surface](https://crates.io/crates/objc2-io-surface) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the IOSurface framework |
| [objc2-osa-kit](https://crates.io/crates/objc2-osa-kit) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the OSAKit framework |
| [objc2-quartz-core](https://crates.io/crates/objc2-quartz-core) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the QuartzCore/CoreAnimation framework |
| [objc2-ui-kit](https://crates.io/crates/objc2-ui-kit) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the UIKit framework |
| [objc2-user-notifications](https://crates.io/crates/objc2-user-notifications) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the UserNotifications framework |
| [objc2-web-kit](https://crates.io/crates/objc2-web-kit) | 0.3.2 | Apache-2.0 OR MIT OR Zlib | Bindings to the WebKit framework |
| [ogg_pager](https://crates.io/crates/ogg_pager) | 0.8.0 | Apache-2.0 OR MIT | A simple OGG page reader |
| [once_cell](https://crates.io/crates/once_cell) | 1.21.4 | Apache-2.0 OR MIT | Single assignment cells and lazy values. |
| [once_cell_polyfill](https://crates.io/crates/once_cell_polyfill) | 1.70.2 | Apache-2.0 OR MIT | Polyfill for `OnceCell` stdlib feature for use with older MSRVs |
| [open](https://crates.io/crates/open) | 5.4.4 | MIT | Open a path or URL using the program configured on the system |
| [openssl-probe](https://crates.io/crates/openssl-probe) | 0.2.1 | Apache-2.0 OR MIT | A library for helping to find system-wide trust anchor ("root") certificate locations based on paths typically used by `openssl`. |
| [option-ext](https://crates.io/crates/option-ext) | 0.2.0 | MPL-2.0 | Extends `Option` with additional operations |
| [ordered-stream](https://crates.io/crates/ordered-stream) | 0.2.0 | Apache-2.0 OR MIT | Streams that are ordered relative to external events |
| [osakit](https://crates.io/crates/osakit) | 0.3.1 | Apache-2.0 OR MIT | OSAKit macOS Framework adapted for Rust |
| [p256](https://crates.io/crates/p256) | 0.14.0 | Apache-2.0 OR MIT | Pure Rust implementation of the NIST P-256 (a.k.a. secp256r1, prime256v1) elliptic curve as defined in SP 800-186, with support for ECDH, ECDSA… |
| [p384](https://crates.io/crates/p384) | 0.14.0 | Apache-2.0 OR MIT | Pure Rust implementation of the NIST P-384 (a.k.a. secp384r1) elliptic curve as defined in SP 800-186 with support for ECDH, ECDSA signing/verification, and… |
| [p521](https://crates.io/crates/p521) | 0.14.0 | Apache-2.0 OR MIT | Pure Rust implementation of the NIST P-521 (a.k.a. secp521r1) elliptic curve as defined in SP 800-186, with support for ECDH, ECDSA signing/verification, and… |
| [pageant](https://crates.io/crates/pageant) | 0.2.4 | Apache-2.0 | Pageant SSH agent transport client. |
| [pagefind](https://crates.io/crates/pagefind) | 1.5.2 | MIT | Implement search on any static website. |
| [pagefind_stem](https://crates.io/crates/pagefind_stem) | 1.0.0 | MIT | Snowball stemming algorithms repackaged for Rust, with languages behind feature flags. |
| [pango](https://crates.io/crates/pango) | 0.18.3 | MIT | Rust bindings for the Pango library |
| [pango-sys](https://crates.io/crates/pango-sys) | 0.18.0 | MIT | FFI bindings to libpango-1.0 |
| [parking](https://crates.io/crates/parking) | 2.2.1 | Apache-2.0 OR MIT | Thread parking and unparking |
| [parking_lot](https://crates.io/crates/parking_lot) | 0.12.5 | Apache-2.0 OR MIT | More compact and efficient implementations of the standard synchronization primitives. |
| [parking_lot_core](https://crates.io/crates/parking_lot_core) | 0.9.12 | Apache-2.0 OR MIT | An advanced API for creating custom synchronization primitives. |
| [password-hash](https://crates.io/crates/password-hash) | 0.6.1 | Apache-2.0 OR MIT | Traits which describe the functionality of password hashing algorithms, with optional support for a `no_std`/`no_alloc`-friendly implementation of the PHC… |
| [paste](https://crates.io/crates/paste) | 1.0.15 | Apache-2.0 OR MIT | Macros for all your token pasting needs |
| [pastey](https://crates.io/crates/pastey) | 0.2.3 | Apache-2.0 OR MIT | Macros for all your token pasting needs. Successor of paste. |
| [path-slash](https://crates.io/crates/path-slash) | 0.2.1 | MIT | Conversion to/from a file path from/to slash path |
| [pbkdf2](https://crates.io/crates/pbkdf2) | 0.13.0 | Apache-2.0 OR MIT | Generic implementation of PBKDF2 |
| [pem-rfc7468](https://crates.io/crates/pem-rfc7468) | 1.0.0 | Apache-2.0 OR MIT | PEM Encoding (RFC 7468) for PKIX, PKCS, and CMS Structures, implementing a strict subset of the original Privacy-Enhanced Mail encoding intended specifically… |
| [percent-encoding](https://crates.io/crates/percent-encoding) | 2.3.2 | Apache-2.0 OR MIT | Percent encoding and decoding |
| [phc](https://crates.io/crates/phc) | 0.6.1 | Apache-2.0 OR MIT | Pure Rust implementation of the Password Hashing Competition (PHC) string format which is used for storing password hashes |
| [phf](https://crates.io/crates/phf) | 0.13.1 | MIT | Runtime support for perfect hash function data structures |
| [phf](https://crates.io/crates/phf) | 0.14.0 | MIT | Runtime support for perfect hash function data structures |
| [phf_codegen](https://crates.io/crates/phf_codegen) | 0.13.1 | MIT | Codegen library for PHF types |
| [phf_codegen](https://crates.io/crates/phf_codegen) | 0.14.0 | MIT | Codegen library for PHF types |
| [phf_generator](https://crates.io/crates/phf_generator) | 0.13.1 | MIT | PHF generation logic |
| [phf_generator](https://crates.io/crates/phf_generator) | 0.14.0 | MIT | PHF generation logic |
| [phf_macros](https://crates.io/crates/phf_macros) | 0.13.1 | MIT | Macros to generate types in the phf crate |
| [phf_shared](https://crates.io/crates/phf_shared) | 0.13.1 | MIT | Support code shared by PHF libraries |
| [phf_shared](https://crates.io/crates/phf_shared) | 0.14.0 | MIT | Support code shared by PHF libraries |
| [pin-project-lite](https://crates.io/crates/pin-project-lite) | 0.2.17 | Apache-2.0 OR MIT | A lightweight version of pin-project written with declarative macros. |
| [piper](https://crates.io/crates/piper) | 0.2.5 | Apache-2.0 OR MIT | An asynchronous single-consumer single-producer pipe for bytes. |
| [pkcs1](https://crates.io/crates/pkcs1) | 0.8.0-rc.4 | Apache-2.0 OR MIT | Pure Rust implementation of Public-Key Cryptography Standards (PKCS) #1&#58; RSA Cryptography Specifications Version 2.2 (RFC 8017) |
| [pkcs5](https://crates.io/crates/pkcs5) | 0.8.1 | Apache-2.0 OR MIT | Pure Rust implementation of Public-Key Cryptography Standards (PKCS) #5&#58; Password-Based Cryptography Specification Version 2.1 (RFC 8018) |
| [pkcs8](https://crates.io/crates/pkcs8) | 0.11.0 | Apache-2.0 OR MIT | Pure Rust implementation of Public-Key Cryptography Standards (PKCS) #8&#58; Private-Key Information Syntax Specification (RFC 5208), with additional support for… |
| [pkg-config](https://crates.io/crates/pkg-config) | 0.3.34 | Apache-2.0 OR MIT | A library to run the pkg-config system tool at build time in order to be used in Cargo build scripts. |
| [plist](https://crates.io/crates/plist) | 1.10.2 | MIT | A rusty plist parser. Supports Serde serialization. |
| [png](https://crates.io/crates/png) | 0.17.16 | Apache-2.0 OR MIT | PNG decoding and encoding library in pure Rust |
| [png](https://crates.io/crates/png) | 0.18.1 | Apache-2.0 OR MIT | PNG decoding and encoding library in pure Rust |
| [polling](https://crates.io/crates/polling) | 3.11.0 | Apache-2.0 OR MIT | Portable interface to epoll, kqueue, event ports, and IOCP |
| [poly1305](https://crates.io/crates/poly1305) | 0.9.1 | Apache-2.0 OR MIT | The Poly1305 universal hash function and message authentication code |
| [polyval](https://crates.io/crates/polyval) | 0.7.3 | Apache-2.0 OR MIT | POLYVAL is a GHASH-like universal hash over GF(2^128) useful for constructing a Message Authentication Code (MAC) |
| [pori](https://crates.io/crates/pori) | 0.0.0 | MIT | Parser state and error extensions for nom. |
| [portable-atomic](https://crates.io/crates/portable-atomic) | 1.15.0 | Apache-2.0 OR MIT | Portable atomic types including support for 128-bit atomics, atomic float, etc. |
| [portable-atomic-util](https://crates.io/crates/portable-atomic-util) | 0.2.8 | Apache-2.0 OR MIT | Synchronization primitives built with portable-atomic. |
| [potential_utf](https://crates.io/crates/potential_utf) | 0.1.6 | Unicode-3.0 | Unvalidated string and character types |
| [powerfmt](https://crates.io/crates/powerfmt) | 0.2.1 | Apache-2.0 OR MIT | `powerfmt` is a library that provides utilities for formatting values. This crate makes it significantly easier to support filling to a minimum width with… |
| [precomputed-hash](https://crates.io/crates/precomputed-hash) | 0.1.1 | MIT | A library intending to be a base dependency to expose a precomputed hash |
| [prettyplease](https://crates.io/crates/prettyplease) | 0.2.37 | Apache-2.0 OR MIT | A minimal `syn` syntax tree pretty-printer |
| [primefield](https://crates.io/crates/primefield) | 0.14.0 | Apache-2.0 OR MIT | Generic implementation of prime fields built on `crypto-bigint`, along with macros for writing field element newtypes including ones with formally verified… |
| [primeorder](https://crates.io/crates/primeorder) | 0.14.0 | Apache-2.0 OR MIT | Pure Rust implementation of complete addition formulas for prime order elliptic curves (Renes-Costello-Batina 2015). Generic over field elements and curve… |
| [proc-macro-crate](https://crates.io/crates/proc-macro-crate) | 1.3.1 | Apache-2.0 OR MIT | Replacement for crate (macro_rules keyword) in proc-macros |
| [proc-macro-crate](https://crates.io/crates/proc-macro-crate) | 2.0.2 | Apache-2.0 OR MIT | Replacement for crate (macro_rules keyword) in proc-macros |
| [proc-macro-crate](https://crates.io/crates/proc-macro-crate) | 3.5.0 | Apache-2.0 OR MIT | Replacement for crate (macro_rules keyword) in proc-macros |
| [proc-macro-error](https://crates.io/crates/proc-macro-error) | 1.0.4 | Apache-2.0 OR MIT | Almost drop-in replacement to panics in proc-macros |
| [proc-macro-error-attr](https://crates.io/crates/proc-macro-error-attr) | 1.0.4 | Apache-2.0 OR MIT | Attribute macro for proc-macro-error crate |
| [proc-macro2](https://crates.io/crates/proc-macro2) | 1.0.107 | Apache-2.0 OR MIT | A substitute implementation of the compiler's `proc_macro` API to decouple token-based libraries from the procedural macro use case. |
| [pulldown-latex](https://crates.io/crates/pulldown-latex) | 0.8.0 | MIT | Pull parser to convert LaTeX equations to MathML |
| [pxfm](https://crates.io/crates/pxfm) | 0.1.30 | Apache-2.0 OR BSD-3-Clause | Fast and accurate math |
| [quick-error](https://crates.io/crates/quick-error) | 2.0.1 | Apache-2.0 OR MIT | A macro which makes error types pleasant to write. |
| [quick-xml](https://crates.io/crates/quick-xml) | 0.42.0 | MIT | High performance xml reader and writer |
| [quote](https://crates.io/crates/quote) | 1.0.47 | Apache-2.0 OR MIT | Quasi-quoting macro quote!(...) |
| [r-efi](https://crates.io/crates/r-efi) | 5.3.0 | Apache-2.0 OR LGPL-2.1-or-later OR MIT | UEFI Reference Specification Protocol Constants and Definitions |
| [r-efi](https://crates.io/crates/r-efi) | 6.0.0 | Apache-2.0 OR LGPL-2.1-or-later OR MIT | UEFI Reference Specification Protocol Constants and Definitions |
| [rand](https://crates.io/crates/rand) | 0.10.3 | Apache-2.0 OR MIT | Random number generators and other randomness functionality. |
| [rand_core](https://crates.io/crates/rand_core) | 0.10.1 | Apache-2.0 OR MIT | Core random number generation traits and tools for implementation. |
| [raw-window-handle](https://crates.io/crates/raw-window-handle) | 0.6.2 | Apache-2.0 OR MIT OR Zlib | Interoperability library for Rust Windowing applications. |
| [rayon](https://crates.io/crates/rayon) | 1.12.0 | Apache-2.0 OR MIT | Simple work-stealing parallelism for Rust |
| [rayon-core](https://crates.io/crates/rayon-core) | 1.13.0 | Apache-2.0 OR MIT | Core APIs for Rayon |
| [redox_syscall](https://crates.io/crates/redox_syscall) | 0.5.18 | MIT | A Rust library to access raw Redox system calls |
| [redox_users](https://crates.io/crates/redox_users) | 0.5.3 | MIT | A Rust library to access Redox users and groups functionality |
| [ref-cast](https://crates.io/crates/ref-cast) | 1.0.27 | Apache-2.0 OR MIT | Safely cast &T to &U where the struct U contains a single field of type T. |
| [ref-cast-impl](https://crates.io/crates/ref-cast-impl) | 1.0.27 | Apache-2.0 OR MIT | Derive implementation for ref_cast&#58;&#58;RefCast. |
| [regex](https://crates.io/crates/regex) | 1.13.1 | Apache-2.0 OR MIT | An implementation of regular expressions for Rust. This implementation uses finite automata and guarantees linear time matching on all inputs. |
| [regex-automata](https://crates.io/crates/regex-automata) | 0.4.18 | Apache-2.0 OR MIT | Automata construction and matching using regular expressions. |
| [regex-syntax](https://crates.io/crates/regex-syntax) | 0.8.11 | Apache-2.0 OR MIT | A regular expression parser. |
| [reqwest](https://crates.io/crates/reqwest) | 0.13.5 | Apache-2.0 OR MIT | higher level HTTP client library |
| [rfc6979](https://crates.io/crates/rfc6979) | 0.6.0 | Apache-2.0 OR MIT | Pure Rust implementation of RFC6979&#58; Deterministic Usage of the Digital Signature Algorithm (DSA) and Elliptic Curve Digital Signature Algorithm (ECDSA) |
| [rfd](https://crates.io/crates/rfd) | 0.16.0 | MIT | Rusty File Dialog |
| [ring](https://crates.io/crates/ring) | 0.17.14 | Apache-2.0 AND ISC | An experiment. |
| [rmcp](https://crates.io/crates/rmcp) | 3.8.0 | Apache-2.0 | Rust SDK for Model Context Protocol |
| [rmcp-macros](https://crates.io/crates/rmcp-macros) | 3.8.0 | Apache-2.0 | Rust SDK for Model Context Protocol macros library |
| [rsa](https://crates.io/crates/rsa) | 0.10.0-rc.18 | Apache-2.0 OR MIT | Pure Rust RSA implementation |
| [russh](https://crates.io/crates/russh) | 0.64.1 | Apache-2.0 | A client and server SSH library. |
| [russh-cryptovec](https://crates.io/crates/russh-cryptovec) | 0.62.0 | Apache-2.0 | A vector which zeroes its memory on clears and reallocations. |
| [russh-sftp](https://crates.io/crates/russh-sftp) | 3.0.1 | Apache-2.0 | SFTP subsystem supported server and client for Russh |
| [russh-util](https://crates.io/crates/russh-util) | 0.52.0 | Apache-2.0 | Runtime abstraction utilities for russh. |
| [rust-patch](https://crates.io/crates/rust-patch) | 0.1.3 | Apache-2.0 OR MIT | Patch structs with other structs |
| [rust-patch-derive](https://crates.io/crates/rust-patch-derive) | 0.1.3 | Apache-2.0 OR MIT | Derive macro implementation for rust-patch |
| [rustc-hash](https://crates.io/crates/rustc-hash) | 2.1.3 | Apache-2.0 OR MIT | A speedy, non-cryptographic hashing algorithm used by rustc |
| [rustc_version](https://crates.io/crates/rustc_version) | 0.4.1 | Apache-2.0 OR MIT | A library for querying the version of a installed rustc compiler |
| [rustix](https://crates.io/crates/rustix) | 1.1.5 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Safe Rust bindings to POSIX/Unix/Linux/Winsock-like syscalls |
| [rustls](https://crates.io/crates/rustls) | 0.23.46 | Apache-2.0 OR ISC OR MIT | Rustls is a modern TLS library written in Rust. |
| [rustls-native-certs](https://crates.io/crates/rustls-native-certs) | 0.8.5 | Apache-2.0 OR ISC OR MIT | rustls-native-certs allows rustls to use the platform native certificate store |
| [rustls-pki-types](https://crates.io/crates/rustls-pki-types) | 1.15.1 | Apache-2.0 OR MIT | Shared types for the rustls PKI ecosystem |
| [rustls-platform-verifier](https://crates.io/crates/rustls-platform-verifier) | 0.7.1 | Apache-2.0 OR MIT | rustls-platform-verifier supports verifying TLS certificates in rustls with the operating system verifier |
| [rustls-platform-verifier-android](https://crates.io/crates/rustls-platform-verifier-android) | 0.2.0 | Apache-2.0 OR MIT | The internal JVM support component of the rustls-platform-verifier crate. You shouldn't depend on this directly. |
| [rustls-webpki](https://crates.io/crates/rustls-webpki) | 0.103.16 | ISC | Web PKI X.509 Certificate Verification. |
| [rustversion](https://crates.io/crates/rustversion) | 1.0.23 | Apache-2.0 OR MIT | Conditional compilation according to rustc compiler version |
| [ryu](https://crates.io/crates/ryu) | 1.0.23 | Apache-2.0 OR BSL-1.0 | Fast floating point to string conversion |
| [salsa20](https://crates.io/crates/salsa20) | 0.11.0 | Apache-2.0 OR MIT | Pure Rust implementation of the Salsa20 stream cipher |
| [same-file](https://crates.io/crates/same-file) | 1.0.6 | MIT OR Unlicense | A simple crate for determining whether two file paths point to the same file. |
| [schannel](https://crates.io/crates/schannel) | 0.1.30 | MIT | Schannel bindings for rust, allowing SSL/TLS (e.g. https) without openssl |
| [schemars](https://crates.io/crates/schemars) | 0.8.22 | MIT | Generate JSON Schemas from Rust code |
| [schemars](https://crates.io/crates/schemars) | 0.9.0 | MIT | Generate JSON Schemas from Rust code |
| [schemars](https://crates.io/crates/schemars) | 1.2.3 | MIT | Generate JSON Schemas from Rust code |
| [schemars_derive](https://crates.io/crates/schemars_derive) | 0.8.22 | MIT | Macros for #[derive(JsonSchema)], for use with schemars |
| [schemars_derive](https://crates.io/crates/schemars_derive) | 1.2.3 | MIT | Macros for #[derive(JsonSchema)], for use with schemars |
| [scopeguard](https://crates.io/crates/scopeguard) | 1.2.0 | Apache-2.0 OR MIT | A RAII scope guard that will run a given closure when it goes out of scope, even if the code between panics (assuming unwinding panic). Defines the macros… |
| [scrypt](https://crates.io/crates/scrypt) | 0.12.0 | Apache-2.0 OR MIT | Scrypt password-based key derivation function |
| [sec1](https://crates.io/crates/sec1) | 0.8.1 | Apache-2.0 OR MIT | Pure Rust implementation of SEC1&#58; Elliptic Curve Cryptography encoding formats including ASN.1 DER encoded private keys as well as the… |
| [secret-service](https://crates.io/crates/secret-service) | 5.2.0 | Apache-2.0 OR MIT | Library to interface with Secret Service API |
| [security-framework](https://crates.io/crates/security-framework) | 3.7.0 | Apache-2.0 OR MIT | Security.framework bindings for macOS and iOS |
| [security-framework-sys](https://crates.io/crates/security-framework-sys) | 2.17.0 | Apache-2.0 OR MIT | Apple `Security.framework` low-level FFI bindings |
| [selectors](https://crates.io/crates/selectors) | 0.37.0 | MPL-2.0 | CSS Selectors matching for Rust |
| [selectors](https://crates.io/crates/selectors) | 0.38.0 | MPL-2.0 | CSS Selectors matching for Rust |
| [semver](https://crates.io/crates/semver) | 1.0.28 | Apache-2.0 OR MIT | Parser and evaluator for Cargo's flavor of Semantic Versioning |
| [serde](https://crates.io/crates/serde) | 1.0.229 | Apache-2.0 OR MIT | A generic serialization/deserialization framework |
| [serde-untagged](https://crates.io/crates/serde-untagged) | 0.1.9 | Apache-2.0 OR MIT | Serde `Visitor` implementation for deserializing untagged enums |
| [serde_bytes](https://crates.io/crates/serde_bytes) | 0.11.19 | Apache-2.0 OR MIT | Optimized handling of `&[u8]` and `Vec&lt;u8>` for Serde |
| [serde_core](https://crates.io/crates/serde_core) | 1.0.229 | Apache-2.0 OR MIT | Serde traits only, with no support for derive -- use the `serde` crate instead |
| [serde_derive](https://crates.io/crates/serde_derive) | 1.0.229 | Apache-2.0 OR MIT | Macros 1.1 implementation of #[derive(Serialize, Deserialize)] |
| [serde_derive_internals](https://crates.io/crates/serde_derive_internals) | 0.29.1 | Apache-2.0 OR MIT | AST representation used by Serde derive macros. Unstable. |
| [serde_derive_internals](https://crates.io/crates/serde_derive_internals) | 0.30.0 | Apache-2.0 OR MIT | AST representation used by Serde derive macros. Unstable. |
| [serde_json](https://crates.io/crates/serde_json) | 1.0.154 | Apache-2.0 OR MIT | A JSON serialization file format |
| [serde_repr](https://crates.io/crates/serde_repr) | 0.1.21 | Apache-2.0 OR MIT | Derive Serialize and Deserialize that delegates to the underlying repr of a C-like enum. |
| [serde_spanned](https://crates.io/crates/serde_spanned) | 0.6.9 | Apache-2.0 OR MIT | Serde-compatible spanned Value |
| [serde_spanned](https://crates.io/crates/serde_spanned) | 1.1.2 | Apache-2.0 OR MIT | Serde-compatible spanned Value |
| [serde_with](https://crates.io/crates/serde_with) | 3.24.0 | Apache-2.0 OR MIT | Custom de/serialization functions for Rust's serde |
| [serde_with_macros](https://crates.io/crates/serde_with_macros) | 3.24.0 | Apache-2.0 OR MIT | proc-macro library for serde_with |
| [serde_yaml](https://crates.io/crates/serde_yaml) | 0.8.26 | Apache-2.0 OR MIT | YAML support for Serde |
| [serdect](https://crates.io/crates/serdect) | 0.4.3 | Apache-2.0 OR MIT | Constant-time serde serializer/deserializer helpers for data that potentially contains secrets (e.g. cryptographic keys) |
| [serialize-to-javascript](https://crates.io/crates/serialize-to-javascript) | 0.1.2 | Apache-2.0 OR MIT | Serialize a serde&#58;&#58;Serialize item to a JavaScript literal template using serde_json |
| [serialize-to-javascript-impl](https://crates.io/crates/serialize-to-javascript-impl) | 0.1.2 | Apache-2.0 OR MIT | Implementation detail of `serialize-to-javascript` |
| [servo_arc](https://crates.io/crates/servo_arc) | 0.4.3 | Apache-2.0 OR MIT | A fork of std&#58;&#58;sync&#58;&#58;Arc with some extra functionality and without weak references |
| [sha-1](https://crates.io/crates/sha-1) | 0.10.1 | Apache-2.0 OR MIT | SHA-1 hash function. This crate is deprecated! Use the sha1 crate instead. |
| [sha1](https://crates.io/crates/sha1) | 0.11.0 | Apache-2.0 OR MIT | SHA-1 hash function |
| [sha1_smol](https://crates.io/crates/sha1_smol) | 1.0.1 | BSD-3-Clause | Minimal dependency-free implementation of SHA1 for Rust. |
| [sha2](https://crates.io/crates/sha2) | 0.10.9 | Apache-2.0 OR MIT | Pure Rust implementation of the SHA-2 hash function family including SHA-224, SHA-256, SHA-384, and SHA-512. |
| [sha2](https://crates.io/crates/sha2) | 0.11.1 | Apache-2.0 OR MIT | Pure Rust implementation of the SHA-2 hash function family including SHA-224, SHA-256, SHA-384, and SHA-512. |
| [sha3](https://crates.io/crates/sha3) | 0.12.0 | Apache-2.0 OR MIT | Implementation of the SHA-3 family of cryptographic hash algorithms |
| [shake](https://crates.io/crates/shake) | 0.1.0 | Apache-2.0 OR MIT | Implementation of the SHAKE family of extendable-output functions (XOFs) |
| [shlex](https://crates.io/crates/shlex) | 2.0.1 | Apache-2.0 OR MIT | Split a string into shell words, like Python's shlex. |
| [signal-hook-registry](https://crates.io/crates/signal-hook-registry) | 1.4.8 | Apache-2.0 OR MIT | Backend crate for signal-hook |
| [signature](https://crates.io/crates/signature) | 3.0.0 | Apache-2.0 OR MIT | Traits for cryptographic signature algorithms (e.g. ECDSA, Ed25519) |
| [simd-adler32](https://crates.io/crates/simd-adler32) | 0.3.10 | MIT | A SIMD-accelerated Adler-32 hash algorithm implementation. |
| [simd_cesu8](https://crates.io/crates/simd_cesu8) | 1.2.0 | Apache-2.0 OR MIT | An extremely fast, SIMD accelerated, encoding and decoding library for CESU-8 and Modified UTF-8. |
| [simdutf8](https://crates.io/crates/simdutf8) | 0.1.5 | Apache-2.0 OR MIT | SIMD-accelerated UTF-8 validation. |
| [siphasher](https://crates.io/crates/siphasher) | 1.0.4 | Apache-2.0 OR MIT | SipHash-2-4 and SipHash-1-3 keyed hash functions in pure Rust, with 64-bit and 128-bit output and no_std support |
| [slab](https://crates.io/crates/slab) | 0.4.12 | MIT | Pre-allocated storage for a uniform data type |
| [smallvec](https://crates.io/crates/smallvec) | 1.16.3 | Apache-2.0 OR MIT | 'Small vector' optimization&#58; store up to a small number of items on the stack |
| [socket2](https://crates.io/crates/socket2) | 0.6.5 | Apache-2.0 OR MIT | Utilities for handling networking sockets with a maximal amount of configuration possible intended. |
| [softbuffer](https://crates.io/crates/softbuffer) | 0.4.8 | Apache-2.0 OR MIT | Cross-platform software buffer |
| [soup3](https://crates.io/crates/soup3) | 0.5.0 | MIT | Soup crate for Rust |
| [soup3-sys](https://crates.io/crates/soup3-sys) | 0.5.0 | MIT | Sys functions for the Rust bindings of the javacriptcore library |
| [specta](https://crates.io/crates/specta) | 2.0.0-rc.25 | MIT | Easily export your Rust types to other languages |
| [specta-macros](https://crates.io/crates/specta-macros) | 2.0.0-rc.25 | MIT | Macros for specta. Don't use directly! |
| [specta-serde](https://crates.io/crates/specta-serde) | 0.0.12 | MIT | Serde support for Specta |
| [specta-typescript](https://crates.io/crates/specta-typescript) | 0.0.12 | MIT | Export your Rust types to TypeScript |
| [specta-util](https://crates.io/crates/specta-util) | 0.0.12 | MIT | High-level utilities for working with Specta |
| [spki](https://crates.io/crates/spki) | 0.8.1 | Apache-2.0 OR MIT | X.509 Subject Public Key Info (RFC5280) describing public keys as well as their associated AlgorithmIdentifiers (i.e. OIDs) |
| [sponge-cursor](https://crates.io/crates/sponge-cursor) | 0.1.0 | Apache-2.0 OR MIT | Cursor for sponge-based absorption and squeezing |
| [ssh-cipher](https://crates.io/crates/ssh-cipher) | 0.3.0 | Apache-2.0 OR MIT | Pure Rust implementation of SSH symmetric encryption including support for the modern aes128-gcm@openssh.com/aes256-gcm@openssh.com and… |
| [ssh-encoding](https://crates.io/crates/ssh-encoding) | 0.3.0 | Apache-2.0 OR MIT | Pure Rust implementation of SSH data type decoders/encoders as described in RFC4251 |
| [ssh-key](https://crates.io/crates/ssh-key) | 0.7.0-rc.11 | Apache-2.0 OR MIT | Pure Rust implementation of SSH key file format decoders/encoders as described in RFC4251/RFC4253 and OpenSSH key formats, as well as "sshsig" signatures and… |
| [stable_deref_trait](https://crates.io/crates/stable_deref_trait) | 1.2.1 | Apache-2.0 OR MIT | An unsafe marker trait for types like Box and Rc that dereference to a stable address even when moved, and hence can be used with libraries such as owning_ref… |
| [static_vcruntime](https://crates.io/crates/static_vcruntime) | 3.0.0 | Apache-2.0 OR MIT OR Zlib | Statically link the VCRuntime when using the MSVC toolchain. |
| [string_cache](https://crates.io/crates/string_cache) | 0.9.0 | Apache-2.0 OR MIT | A string interning library for Rust, developed as part of the Servo project. |
| [string_cache_codegen](https://crates.io/crates/string_cache_codegen) | 0.6.1 | Apache-2.0 OR MIT | A codegen library for string-cache, developed as part of the Servo project. |
| [strsim](https://crates.io/crates/strsim) | 0.11.1 | MIT | Implementations of string similarity metrics. Includes Hamming, Levenshtein, OSA, Damerau-Levenshtein, Jaro, Jaro-Winkler, and Sørensen-Dice. |
| [subtle](https://crates.io/crates/subtle) | 2.6.1 | BSD-3-Clause | Pure-Rust traits and utilities for constant-time cryptographic implementations. |
| [swift-rs](https://crates.io/crates/swift-rs) | 1.0.9 | Apache-2.0 OR MIT | Call Swift from Rust with ease! |
| [syn](https://crates.io/crates/syn) | 1.0.109 | Apache-2.0 OR MIT | Parser for Rust source code |
| [syn](https://crates.io/crates/syn) | 2.0.119 | Apache-2.0 OR MIT | Parser for Rust source code |
| [syn](https://crates.io/crates/syn) | 3.0.9 | Apache-2.0 OR MIT | Parser for Rust source code |
| [sync_wrapper](https://crates.io/crates/sync_wrapper) | 1.0.2 | Apache-2.0 | A tool for enlisting the compiler's help in proving the absence of concurrency |
| [synstructure](https://crates.io/crates/synstructure) | 0.14.0 | MIT | Helper methods and macros for custom derives |
| [syntect](https://crates.io/crates/syntect) | 5.3.0 | MIT | library for high quality syntax highlighting and code intelligence using Sublime Text's grammars |
| [system-configuration](https://crates.io/crates/system-configuration) | 0.7.0 | Apache-2.0 OR MIT | Bindings to SystemConfiguration framework for macOS |
| [system-configuration-sys](https://crates.io/crates/system-configuration-sys) | 0.6.0 | Apache-2.0 OR MIT | Low level bindings to SystemConfiguration framework for macOS |
| [system-deps](https://crates.io/crates/system-deps) | 6.2.2 | Apache-2.0 OR MIT | Discover and configure system dependencies from declarative dependencies in Cargo.toml |
| [tao](https://crates.io/crates/tao) | 0.37.1 | Apache-2.0 | Cross-platform window manager library. |
| [tao-macros](https://crates.io/crates/tao-macros) | 0.1.4 | Apache-2.0 OR MIT | Proc macros for tao |
| [tar](https://crates.io/crates/tar) | 0.4.46 | Apache-2.0 OR MIT | A Rust implementation of a TAR file reader and writer. This library does not currently handle compression, but it is abstract over all I/O readers and writers… |
| [target-lexicon](https://crates.io/crates/target-lexicon) | 0.12.16 | Apache-2.0 WITH LLVM-exception | Targeting utilities for compilers and related tools |
| [tauri](https://crates.io/crates/tauri) | 2.12.4 | Apache-2.0 OR MIT | Make tiny, secure apps for all desktop platforms with Tauri |
| [tauri-build](https://crates.io/crates/tauri-build) | 2.7.3 | Apache-2.0 OR MIT | build time code to pair with https&#58;//crates.io/crates/tauri |
| [tauri-codegen](https://crates.io/crates/tauri-codegen) | 2.7.3 | Apache-2.0 OR MIT | code generation meant to be consumed inside of `tauri` through `tauri-build` or `tauri-macros` |
| [tauri-macros](https://crates.io/crates/tauri-macros) | 2.7.3 | Apache-2.0 OR MIT | Macros for the tauri crate. |
| [tauri-plugin](https://crates.io/crates/tauri-plugin) | 2.7.2 | Apache-2.0 OR MIT | Build script and runtime Tauri plugin definitions |
| [tauri-plugin-dialog](https://crates.io/crates/tauri-plugin-dialog) | 2.8.3 | Apache-2.0 OR MIT | Native system dialogs for opening and saving files along with message dialogs on your Tauri application. |
| [tauri-plugin-fs](https://crates.io/crates/tauri-plugin-fs) | 2.7.1 | Apache-2.0 OR MIT | Access the file system. |
| [tauri-plugin-opener](https://crates.io/crates/tauri-plugin-opener) | 2.7.2 | Apache-2.0 OR MIT | Open files and URLs using their default application. |
| [tauri-plugin-process](https://crates.io/crates/tauri-plugin-process) | 2.4.0 | Apache-2.0 OR MIT | Access the current process of your Tauri application. |
| [tauri-plugin-updater](https://crates.io/crates/tauri-plugin-updater) | 2.14.0 | Apache-2.0 OR MIT | In-app updates for Tauri applications. |
| [tauri-plugin-window-state](https://crates.io/crates/tauri-plugin-window-state) | 2.5.0 | Apache-2.0 OR MIT | Save window positions and sizes and restore them when the app is reopened. |
| [tauri-runtime](https://crates.io/crates/tauri-runtime) | 2.12.2 | Apache-2.0 OR MIT | Runtime for Tauri applications |
| [tauri-runtime-wry](https://crates.io/crates/tauri-runtime-wry) | 2.12.2 | Apache-2.0 OR MIT | Wry bindings to the Tauri runtime |
| [tauri-specta](https://crates.io/crates/tauri-specta) | 2.0.0-rc.25 | MIT | Completely typesafe Tauri commands |
| [tauri-utils](https://crates.io/crates/tauri-utils) | 2.10.2 | Apache-2.0 OR MIT | Utilities for Tauri |
| [tauri-winres](https://crates.io/crates/tauri-winres) | 0.3.7 | MIT | Create and set windows icons and metadata for executables |
| [tempfile](https://crates.io/crates/tempfile) | 3.27.0 | Apache-2.0 OR MIT | A library for managing temporary files and directories. |
| [tendril](https://crates.io/crates/tendril) | 0.5.1 | Apache-2.0 OR MIT | Compact buffer/string type for zero-copy parsing |
| [thiserror](https://crates.io/crates/thiserror) | 1.0.69 | Apache-2.0 OR MIT | derive(Error) |
| [thiserror](https://crates.io/crates/thiserror) | 2.0.21 | Apache-2.0 OR MIT | derive(Error) |
| [thiserror-impl](https://crates.io/crates/thiserror-impl) | 1.0.69 | Apache-2.0 OR MIT | Implementation detail of the `thiserror` crate |
| [thiserror-impl](https://crates.io/crates/thiserror-impl) | 2.0.21 | Apache-2.0 OR MIT | Implementation detail of the `thiserror` crate |
| [tikv-jemalloc-sys](https://crates.io/crates/tikv-jemalloc-sys) | 0.6.1+5.3.0-1-ge13ca993e8ccb9ba9847cc330696e02839f328f7 | Apache-2.0 OR MIT | Rust FFI bindings to jemalloc |
| [tikv-jemallocator](https://crates.io/crates/tikv-jemallocator) | 0.6.1 | Apache-2.0 OR MIT | A Rust allocator backed by jemalloc |
| [time](https://crates.io/crates/time) | 0.3.55 | Apache-2.0 OR MIT | Date and time library. Fully interoperable with the standard library. Mostly compatible with #![no_std]. |
| [time-core](https://crates.io/crates/time-core) | 0.1.9 | Apache-2.0 OR MIT | This crate is an implementation detail and should not be relied upon directly. |
| [time-macros](https://crates.io/crates/time-macros) | 0.2.32 | Apache-2.0 OR MIT | Procedural macros for the time crate. This crate is an implementation detail and should not be relied upon directly. |
| [tiny_http](https://crates.io/crates/tiny_http) | 0.12.0 | Apache-2.0 OR MIT | Low level HTTP server library |
| [tinystr](https://crates.io/crates/tinystr) | 0.8.4 | Unicode-3.0 | A small ASCII-only bounded length string representation. |
| [tinyvec](https://crates.io/crates/tinyvec) | 1.13.3 | Apache-2.0 OR MIT OR Zlib | `tinyvec` provides 100% safe vec-like data structures. |
| [tokio](https://crates.io/crates/tokio) | 1.54.0 | MIT | An event-driven, non-blocking I/O platform for writing asynchronous I/O backed applications. |
| [tokio-macros](https://crates.io/crates/tokio-macros) | 2.7.2 | MIT | Tokio's proc macros. |
| [tokio-rustls](https://crates.io/crates/tokio-rustls) | 0.26.6 | Apache-2.0 OR MIT | Asynchronous TLS/SSL streams for Tokio using Rustls. |
| [tokio-util](https://crates.io/crates/tokio-util) | 0.7.20 | MIT | Additional utilities for working with Tokio. |
| [toml](https://crates.io/crates/toml) | 0.5.11 | Apache-2.0 OR MIT | A native Rust encoder and decoder of TOML-formatted files and streams. Provides implementations of the standard Serialize/Deserialize traits for TOML data to… |
| [toml](https://crates.io/crates/toml) | 0.8.2 | Apache-2.0 OR MIT | A native Rust encoder and decoder of TOML-formatted files and streams. Provides implementations of the standard Serialize/Deserialize traits for TOML data to… |
| [toml](https://crates.io/crates/toml) | 1.1.8+spec-1.1.0 | Apache-2.0 OR MIT | A native Rust encoder and decoder of TOML-formatted files and streams. Provides implementations of the standard Serialize/Deserialize traits for TOML data to… |
| [toml_datetime](https://crates.io/crates/toml_datetime) | 0.6.3 | Apache-2.0 OR MIT | A TOML-compatible datetime type |
| [toml_datetime](https://crates.io/crates/toml_datetime) | 1.1.2+spec-1.1.0 | Apache-2.0 OR MIT | A TOML-compatible datetime type |
| [toml_edit](https://crates.io/crates/toml_edit) | 0.19.15 | Apache-2.0 OR MIT | Yet another format-preserving TOML parser. |
| [toml_edit](https://crates.io/crates/toml_edit) | 0.20.2 | Apache-2.0 OR MIT | Yet another format-preserving TOML parser. |
| [toml_edit](https://crates.io/crates/toml_edit) | 0.25.17+spec-1.1.0 | Apache-2.0 OR MIT | Yet another format-preserving TOML parser. |
| [toml_parser](https://crates.io/crates/toml_parser) | 1.1.5+spec-1.1.0 | Apache-2.0 OR MIT | Yet another format-preserving TOML parser. |
| [toml_writer](https://crates.io/crates/toml_writer) | 1.1.3+spec-1.1.0 | Apache-2.0 OR MIT | A low-level interface for writing out TOML |
| [tower](https://crates.io/crates/tower) | 0.5.3 | MIT | Tower is a library of modular and reusable components for building robust clients and servers. |
| [tower-http](https://crates.io/crates/tower-http) | 0.6.11 | MIT | Tower middleware and utilities for HTTP clients and servers |
| [tower-layer](https://crates.io/crates/tower-layer) | 0.3.3 | MIT | Decorates a `Service` to allow easy composition between `Service`s. |
| [tower-service](https://crates.io/crates/tower-service) | 0.3.3 | MIT | Trait representing an asynchronous, request / response based, client or server. |
| [tracing](https://crates.io/crates/tracing) | 0.1.44 | MIT | Application-level tracing for Rust. |
| [tracing-attributes](https://crates.io/crates/tracing-attributes) | 0.1.31 | MIT | Procedural macro attributes for automatically instrumenting functions. |
| [tracing-core](https://crates.io/crates/tracing-core) | 0.1.36 | MIT | Core primitives for application-level tracing. |
| [tray-icon](https://crates.io/crates/tray-icon) | 0.25.1 | Apache-2.0 OR MIT | Create tray icons for desktop applications |
| [try-lock](https://crates.io/crates/try-lock) | 0.2.5 | MIT | A lightweight atomic lock. |
| [ttf-parser](https://crates.io/crates/ttf-parser) | 0.25.1 | Apache-2.0 OR MIT | A high-level, safe, zero-allocation font parser for TrueType, OpenType, and AAT. |
| [twelf](https://crates.io/crates/twelf) | 0.15.0 | MIT | Twelf is a configuration solution for Rust including 12-Factor support. It is designed with layers in order to configure different sources and formats to build… |
| [two-face](https://crates.io/crates/two-face) | 0.5.2+bat-0.26.1 | Apache-2.0 OR MIT | Extra syntax and theme definitions for syntect |
| [typed-arena](https://crates.io/crates/typed-arena) | 2.0.2 | MIT | The arena, a fast but limited type of allocator |
| [typed-builder](https://crates.io/crates/typed-builder) | 0.23.2 | Apache-2.0 OR MIT | Compile-time type-checked builder derive |
| [typed-builder-macro](https://crates.io/crates/typed-builder-macro) | 0.23.2 | Apache-2.0 OR MIT | Compile-time type-checked builder derive |
| [typeid](https://crates.io/crates/typeid) | 1.0.3 | Apache-2.0 OR MIT | Const TypeId and non-'static TypeId |
| [typenum](https://crates.io/crates/typenum) | 1.20.1 | Apache-2.0 OR MIT | Typenum is a Rust library for type-level numbers evaluated at compile time. It currently supports bits, unsigned integers, and signed integers. It also provides… |
| [uds_windows](https://crates.io/crates/uds_windows) | 1.2.1 | MIT | Unix Domain Sockets for Windows! |
| [unicode-ident](https://crates.io/crates/unicode-ident) | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | Determine whether characters have the XID_Start or XID_Continue properties according to Unicode Standard Annex #31 |
| [unicode-normalization](https://crates.io/crates/unicode-normalization) | 0.1.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | This crate provides functions for normalization of Unicode strings, including Canonical and Compatible Decomposition and Recomposition, as described in Unicode… |
| [unicode-segmentation](https://crates.io/crates/unicode-segmentation) | 1.13.3 | Apache-2.0 OR MIT | This crate provides Grapheme Cluster, Word and Sentence boundaries according to Unicode Standard Annex #29 rules. |
| [unicode-width](https://crates.io/crates/unicode-width) | 0.2.3 | Apache-2.0 OR MIT | Determine displayed width of `char` and `str` types according to Unicode Standard Annex #11 rules. |
| [unicode-xid](https://crates.io/crates/unicode-xid) | 0.2.6 | Apache-2.0 OR MIT | Determine whether characters have the XID_Start or XID_Continue properties according to Unicode Standard Annex #31. |
| [universal-hash](https://crates.io/crates/universal-hash) | 0.6.1 | Apache-2.0 OR MIT | Traits which describe the functionality of universal hash functions (UHFs) |
| [untrusted](https://crates.io/crates/untrusted) | 0.9.0 | ISC | Safe, fast, zero-panic, zero-crashing, zero-allocation parsing of untrusted inputs in Rust. |
| [url](https://crates.io/crates/url) | 2.5.8 | Apache-2.0 OR MIT | URL library for Rust, based on the WHATWG URL Standard |
| [urlpattern](https://crates.io/crates/urlpattern) | 0.6.0 | MIT | rust-urlpattern is a Rust implementation of the URLPattern standard |
| [utf8_iter](https://crates.io/crates/utf8_iter) | 1.0.4 | Apache-2.0 OR MIT | Iterator by char over potentially-invalid UTF-8 in &[u8] |
| [utf8parse](https://crates.io/crates/utf8parse) | 0.2.2 | Apache-2.0 OR MIT | Table-driven UTF-8 parser |
| [uuid](https://crates.io/crates/uuid) | 1.28.0 | Apache-2.0 OR MIT | A library to generate and parse UUIDs. |
| [vcpkg](https://crates.io/crates/vcpkg) | 0.2.15 | Apache-2.0 OR MIT | A library to find native dependencies in a vcpkg tree at build time in order to be used in Cargo build scripts. |
| [version-compare](https://crates.io/crates/version-compare) | 0.2.1 | MIT | Rust library to easily compare version numbers with no specific format, and test against various comparison operators. |
| [version_check](https://crates.io/crates/version_check) | 0.9.5 | Apache-2.0 OR MIT | Tiny crate to check the version of the installed/running rustc. |
| [vswhom](https://crates.io/crates/vswhom) | 0.1.0 | MIT | FFI to Jon Blow's VS discovery script |
| [vswhom-sys](https://crates.io/crates/vswhom-sys) | 0.1.3 | MIT | Pure FFI to Jon Blow's VS discovery script |
| [walkdir](https://crates.io/crates/walkdir) | 2.5.0 | MIT OR Unlicense | Recursively walk a directory. |
| [want](https://crates.io/crates/want) | 0.3.2 | MIT | Detect when another Future wants a result. |
| [wasi](https://crates.io/crates/wasi) | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Experimental WASI API bindings for Rust |
| [wasip2](https://crates.io/crates/wasip2) | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | WASIp2 API bindings for Rust |
| [wasm-bindgen](https://crates.io/crates/wasm-bindgen) | 0.2.129 | Apache-2.0 OR MIT | Easy support for interacting between JS and Rust. |
| [wasm-bindgen-futures](https://crates.io/crates/wasm-bindgen-futures) | 0.4.79 | Apache-2.0 OR MIT | Bridging the gap between Rust Futures and JavaScript Promises |
| [wasm-bindgen-macro](https://crates.io/crates/wasm-bindgen-macro) | 0.2.129 | Apache-2.0 OR MIT | Definition of the `#[wasm_bindgen]` attribute, an internal dependency |
| [wasm-bindgen-macro-support](https://crates.io/crates/wasm-bindgen-macro-support) | 0.2.129 | Apache-2.0 OR MIT | Implementation APIs for the `#[wasm_bindgen]` attribute |
| [wasm-bindgen-shared](https://crates.io/crates/wasm-bindgen-shared) | 0.2.129 | Apache-2.0 OR MIT | Shared support between wasm-bindgen and wasm-bindgen cli, an internal dependency. |
| [wasm-streams](https://crates.io/crates/wasm-streams) | 0.5.0 | Apache-2.0 OR MIT | Bridging between web streams and Rust streams using WebAssembly |
| [wax](https://crates.io/crates/wax) | 0.7.0 | MIT | Opinionated and portable globs that can be matched against paths and directory trees. |
| [web-sys](https://crates.io/crates/web-sys) | 0.3.106 | Apache-2.0 OR MIT | Bindings for all Web APIs, a procedurally generated crate from WebIDL |
| [web-time](https://crates.io/crates/web-time) | 1.1.0 | Apache-2.0 OR MIT | Drop-in replacement for std&#58;&#58;time for Wasm in browsers |
| [web_atoms](https://crates.io/crates/web_atoms) | 0.2.6 | Apache-2.0 OR MIT | Atoms for xml5ever and html5ever |
| [webkit2gtk](https://crates.io/crates/webkit2gtk) | 2.0.2 | MIT | Rust bindings for webkit-gtk library |
| [webkit2gtk-sys](https://crates.io/crates/webkit2gtk-sys) | 2.0.2 | MIT | Rust binding for webkit-gtk library |
| [webp](https://crates.io/crates/webp) | 0.3.1 | Apache-2.0 OR MIT | WebP conversion library. |
| [webpki-root-certs](https://crates.io/crates/webpki-root-certs) | 1.0.10 | CDLA-Permissive-2.0 | Mozilla trusted certificate authorities in self-signed X.509 format for use with crates other than webpki |
| [webview2-com](https://crates.io/crates/webview2-com) | 0.39.1 | MIT | Rust bindings for the WebView2 COM APIs |
| [webview2-com-macros](https://crates.io/crates/webview2-com-macros) | 0.8.1 | MIT | Macros which generate callback implementations for WebView2 COM APIs |
| [webview2-com-sys](https://crates.io/crates/webview2-com-sys) | 0.39.1 | MIT | Bindings generated with the windows crate for the WebView2 COM APIs |
| [weezl](https://crates.io/crates/weezl) | 0.1.12 | Apache-2.0 OR MIT | Fast LZW compression and decompression. |
| [winapi](https://crates.io/crates/winapi) | 0.3.9 | Apache-2.0 OR MIT | Raw FFI bindings for all of Windows API. |
| [winapi-i686-pc-windows-gnu](https://crates.io/crates/winapi-i686-pc-windows-gnu) | 0.4.0 | Apache-2.0 OR MIT | Import libraries for the i686-pc-windows-gnu target. Please don't use this crate directly, depend on winapi instead. |
| [winapi-util](https://crates.io/crates/winapi-util) | 0.1.11 | MIT OR Unlicense | A dumping ground for high level safe wrappers over windows-sys. |
| [winapi-x86_64-pc-windows-gnu](https://crates.io/crates/winapi-x86_64-pc-windows-gnu) | 0.4.0 | Apache-2.0 OR MIT | Import libraries for the x86_64-pc-windows-gnu target. Please don't use this crate directly, depend on winapi instead. |
| [window-vibrancy](https://crates.io/crates/window-vibrancy) | 0.8.1 | Apache-2.0 OR MIT | Make your windows vibrant. |
| [windows](https://crates.io/crates/windows) | 0.62.2 | Apache-2.0 OR MIT | Rust for Windows |
| [windows-collections](https://crates.io/crates/windows-collections) | 0.3.2 | Apache-2.0 OR MIT | Windows collection types |
| [windows-core](https://crates.io/crates/windows-core) | 0.62.2 | Apache-2.0 OR MIT | Core type support for COM and Windows |
| [windows-future](https://crates.io/crates/windows-future) | 0.3.2 | Apache-2.0 OR MIT | Windows async types |
| [windows-implement](https://crates.io/crates/windows-implement) | 0.60.2 | Apache-2.0 OR MIT | The implement macro for the Windows crates |
| [windows-interface](https://crates.io/crates/windows-interface) | 0.59.3 | Apache-2.0 OR MIT | The interface macro for the Windows crates |
| [windows-link](https://crates.io/crates/windows-link) | 0.2.1 | Apache-2.0 OR MIT | Linking for Windows |
| [windows-native-keyring-store](https://crates.io/crates/windows-native-keyring-store) | 1.1.0 | Apache-2.0 OR MIT | Windows native credential store for keyring |
| [windows-numerics](https://crates.io/crates/windows-numerics) | 0.3.1 | Apache-2.0 OR MIT | Windows numeric types |
| [windows-registry](https://crates.io/crates/windows-registry) | 0.6.1 | Apache-2.0 OR MIT | Windows registry |
| [windows-result](https://crates.io/crates/windows-result) | 0.4.1 | Apache-2.0 OR MIT | Windows error handling |
| [windows-strings](https://crates.io/crates/windows-strings) | 0.5.1 | Apache-2.0 OR MIT | Windows string types |
| [windows-sys](https://crates.io/crates/windows-sys) | 0.45.0 | Apache-2.0 OR MIT | Rust for Windows |
| [windows-sys](https://crates.io/crates/windows-sys) | 0.52.0 | Apache-2.0 OR MIT | Rust for Windows |
| [windows-sys](https://crates.io/crates/windows-sys) | 0.60.2 | Apache-2.0 OR MIT | Rust for Windows |
| [windows-sys](https://crates.io/crates/windows-sys) | 0.61.2 | Apache-2.0 OR MIT | Rust for Windows |
| [windows-targets](https://crates.io/crates/windows-targets) | 0.42.2 | Apache-2.0 OR MIT | Import libs for Windows |
| [windows-targets](https://crates.io/crates/windows-targets) | 0.52.6 | Apache-2.0 OR MIT | Import libs for Windows |
| [windows-targets](https://crates.io/crates/windows-targets) | 0.53.5 | Apache-2.0 OR MIT | Import libs for Windows |
| [windows-threading](https://crates.io/crates/windows-threading) | 0.2.1 | Apache-2.0 OR MIT | Windows threading |
| [windows-version](https://crates.io/crates/windows-version) | 0.1.7 | Apache-2.0 OR MIT | Windows version information |
| [windows_aarch64_gnullvm](https://crates.io/crates/windows_aarch64_gnullvm) | 0.42.2 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_aarch64_gnullvm](https://crates.io/crates/windows_aarch64_gnullvm) | 0.52.6 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_aarch64_gnullvm](https://crates.io/crates/windows_aarch64_gnullvm) | 0.53.1 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_aarch64_msvc](https://crates.io/crates/windows_aarch64_msvc) | 0.42.2 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_aarch64_msvc](https://crates.io/crates/windows_aarch64_msvc) | 0.52.6 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_aarch64_msvc](https://crates.io/crates/windows_aarch64_msvc) | 0.53.1 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_i686_gnu](https://crates.io/crates/windows_i686_gnu) | 0.42.2 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_i686_gnu](https://crates.io/crates/windows_i686_gnu) | 0.52.6 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_i686_gnu](https://crates.io/crates/windows_i686_gnu) | 0.53.1 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_i686_gnullvm](https://crates.io/crates/windows_i686_gnullvm) | 0.52.6 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_i686_gnullvm](https://crates.io/crates/windows_i686_gnullvm) | 0.53.1 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_i686_msvc](https://crates.io/crates/windows_i686_msvc) | 0.42.2 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_i686_msvc](https://crates.io/crates/windows_i686_msvc) | 0.52.6 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_i686_msvc](https://crates.io/crates/windows_i686_msvc) | 0.53.1 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_x86_64_gnu](https://crates.io/crates/windows_x86_64_gnu) | 0.42.2 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_x86_64_gnu](https://crates.io/crates/windows_x86_64_gnu) | 0.52.6 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_x86_64_gnu](https://crates.io/crates/windows_x86_64_gnu) | 0.53.1 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_x86_64_gnullvm](https://crates.io/crates/windows_x86_64_gnullvm) | 0.42.2 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_x86_64_gnullvm](https://crates.io/crates/windows_x86_64_gnullvm) | 0.52.6 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_x86_64_gnullvm](https://crates.io/crates/windows_x86_64_gnullvm) | 0.53.1 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_x86_64_msvc](https://crates.io/crates/windows_x86_64_msvc) | 0.42.2 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_x86_64_msvc](https://crates.io/crates/windows_x86_64_msvc) | 0.52.6 | Apache-2.0 OR MIT | Import lib for Windows |
| [windows_x86_64_msvc](https://crates.io/crates/windows_x86_64_msvc) | 0.53.1 | Apache-2.0 OR MIT | Import lib for Windows |
| [winnow](https://crates.io/crates/winnow) | 0.5.40 | MIT | A byte-oriented, zero-copy, parser combinators library |
| [winnow](https://crates.io/crates/winnow) | 1.0.4 | MIT | A byte-oriented, zero-copy, parser combinators library |
| [winreg](https://crates.io/crates/winreg) | 0.56.0 | MIT | Rust bindings to MS Windows Registry API |
| [wit-bindgen](https://crates.io/crates/wit-bindgen) | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Rust bindings generator and runtime support for WIT and the component model. Used when compiling Rust programs to the component model. |
| [wnaf](https://crates.io/crates/wnaf) | 0.14.1 | Apache-2.0 OR MIT | wNAF (w-ary non-adjacent form) variable-time scalar multiplication implemented generically over elliptic curve groups with full `no_alloc` support, including… |
| [writeable](https://crates.io/crates/writeable) | 0.6.4 | Unicode-3.0 | A more efficient alternative to fmt&#58;&#58;Display |
| [wry](https://crates.io/crates/wry) | 0.57.0 | Apache-2.0 OR MIT | Cross-platform WebView rendering library |
| [x11](https://crates.io/crates/x11) | 2.21.0 | MIT | X11 library bindings for Rust |
| [x11-dl](https://crates.io/crates/x11-dl) | 2.21.0 | MIT | X11 library bindings for Rust |
| [xattr](https://crates.io/crates/xattr) | 1.6.1 | Apache-2.0 OR MIT | unix extended filesystem attributes |
| [yaml-rust](https://crates.io/crates/yaml-rust) | 0.4.5 | Apache-2.0 OR MIT | The missing YAML 1.2 parser for rust |
| [yoke](https://crates.io/crates/yoke) | 0.8.3 | Unicode-3.0 | Abstraction allowing borrowed data to be carried along with the backing data it borrows from |
| [yoke-derive](https://crates.io/crates/yoke-derive) | 0.8.4 | Unicode-3.0 | Custom derive for the yoke crate |
| [zbus](https://crates.io/crates/zbus) | 5.19.0 | MIT | API for D-Bus communication |
| [zbus-secret-service-keyring-store](https://crates.io/crates/zbus-secret-service-keyring-store) | 1.0.1 | Apache-2.0 OR MIT | ZBus Secret Service credential store for keyring |
| [zbus_macros](https://crates.io/crates/zbus_macros) | 5.19.0 | MIT | proc-macros for zbus |
| [zbus_names](https://crates.io/crates/zbus_names) | 4.3.5 | MIT | A collection of D-Bus bus names types (deprecated&#58; merged into zbus) |
| [zcheapstr](https://crates.io/crates/zcheapstr) | 1.1.0 | MIT | An immutable, cheaply cloneable string type with borrowed, static and shared variants |
| [zerofrom](https://crates.io/crates/zerofrom) | 0.1.8 | Unicode-3.0 | ZeroFrom trait for constructing |
| [zerofrom-derive](https://crates.io/crates/zerofrom-derive) | 0.1.8 | Unicode-3.0 | Custom derive for the zerofrom crate |
| [zeroize](https://crates.io/crates/zeroize) | 1.9.1 | Apache-2.0 OR MIT | Securely clear secrets from memory with a simple trait built on stable Rust primitives which guarantee memory is zeroed using an operation will not be… |
| [zerotrie](https://crates.io/crates/zerotrie) | 0.2.5 | Unicode-3.0 | A data structure that efficiently maps strings to integers |
| [zerovec](https://crates.io/crates/zerovec) | 0.11.8 | Unicode-3.0 | Zero-copy vector backed by a byte array |
| [zerovec-derive](https://crates.io/crates/zerovec-derive) | 0.11.6 | Unicode-3.0 | Custom derive for the zerovec crate |
| [zip](https://crates.io/crates/zip) | 4.6.1 | MIT | Library to support the reading and writing of zip files. |
| [zlib-rs](https://crates.io/crates/zlib-rs) | 0.6.8 | Zlib | A memory-safe zlib implementation written in rust |
| [zmij](https://crates.io/crates/zmij) | 1.0.23 | MIT | A double-to-string conversion algorithm based on Schubfach and xjb |
| [zune-core](https://crates.io/crates/zune-core) | 0.5.3 | Apache-2.0 OR MIT OR Zlib | Core utilities for image processing in the zune family of crates |
| [zune-jpeg](https://crates.io/crates/zune-jpeg) | 0.5.15 | Apache-2.0 OR MIT OR Zlib | A fast, correct and safe jpeg decoder |
| [zvariant](https://crates.io/crates/zvariant) | 5.15.1 | MIT | D-Bus & GVariant encoding & decoding (deprecated&#58; merged into zbus) |
| [zvariant_derive](https://crates.io/crates/zvariant_derive) | 5.15.1 | MIT | D-Bus & GVariant encoding & decoding (deprecated&#58; merged into zbus) |
| [zvariant_utils](https://crates.io/crates/zvariant_utils) | 4.2.0 | MIT | Various utilities used internally by the zvariant crate. |

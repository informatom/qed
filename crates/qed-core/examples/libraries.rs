//! Writes the manual's list of libraries, `docs/pages/manual/libraries.md`,
//! from `cargo metadata` and the app's `package.json`:
//!
//! ```sh
//! cargo run -p qed-core --example libraries
//! ```
//!
//! A test in `manual.rs` fails when the page no longer matches `Cargo.lock`
//! or `package.json`; running this again brings it up to date.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

type Error = Box<dyn std::error::Error>;

/// One library: its name, version, license and what it's for.
struct Library {
    /// Cargo's id for it; empty for npm packages.
    id: String,
    name: String,
    version: String,
    license: String,
    description: String,
}

fn main() -> Result<(), Error> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (rust, direct) = rust_libraries(&root)?;
    let (app, tools) = npm_libraries(&root)?;
    let page = page(&rust, &direct, &app, &tools);
    let out = root.join("docs/pages/manual/libraries.md");
    fs::write(&out, page)?;
    println!(
        "{} Rust libraries ({} used directly), {} + {} npm packages → {}",
        rust.len(),
        direct.len(),
        app.len(),
        tools.len(),
        out.display()
    );
    Ok(())
}

/// Every library in `Cargo.lock` but q.e.d.'s own, and the ids of those
/// q.e.d. uses directly, in the versions it resolves to.
fn rust_libraries(root: &Path) -> Result<(Vec<Library>, BTreeSet<String>), Error> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--locked"])
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)?;
    let members: BTreeSet<&str> = metadata["workspace_members"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut direct = BTreeSet::new();
    for node in metadata["resolve"]["nodes"]
        .as_array()
        .into_iter()
        .flatten()
    {
        if !members.contains(node["id"].as_str().unwrap_or_default()) {
            continue;
        }
        for dep in node["deps"].as_array().into_iter().flatten() {
            // Normal dependencies only: not the ones for tests or builds.
            let normal = dep["dep_kinds"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|kind| kind["kind"].is_null());
            let id = dep["pkg"].as_str().unwrap_or_default();
            if normal && !members.contains(id) {
                direct.insert(id.to_string());
            }
        }
    }
    let mut libraries = Vec::new();
    for package in metadata["packages"].as_array().into_iter().flatten() {
        let text = |key: &str| package[key].as_str().unwrap_or_default().to_string();
        if members.contains(package["id"].as_str().unwrap_or_default()) {
            continue;
        }
        libraries.push(Library {
            id: text("id"),
            name: text("name"),
            version: text("version"),
            license: license(&text("license")),
            description: text("description"),
        });
    }
    libraries.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
    Ok((libraries, direct))
}

/// One spelling for a license: a plain choice (`MIT OR Apache-2.0`,
/// `MIT/Apache-2.0`) in alphabetical order, so the same choice is counted
/// once.
fn license(text: &str) -> String {
    let text = text
        .replace('/', " OR ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if text.contains(['(', ')']) || text.contains(" AND ") || text.contains(" WITH ") {
        return text;
    }
    let mut choices: Vec<&str> = text.split(" OR ").collect();
    choices.sort_unstable();
    choices.join(" OR ")
}

/// The packages in `package.json`, as installed in `node_modules`: those
/// that end up in the app, and those only for building it.
fn npm_libraries(root: &Path) -> Result<(Vec<Library>, Vec<Library>), Error> {
    let manifest: Value = serde_json::from_str(&fs::read_to_string(root.join("package.json"))?)?;
    let read = |key: &str| -> Result<Vec<Library>, Error> {
        let names: BTreeMap<&String, &Value> = manifest[key]
            .as_object()
            .map(|deps| deps.iter().collect())
            .unwrap_or_default();
        let mut list = Vec::new();
        for name in names.keys() {
            let installed: Value = serde_json::from_str(&fs::read_to_string(
                root.join("node_modules")
                    .join(name.as_str())
                    .join("package.json"),
            )?)?;
            let text = |key: &str| installed[key].as_str().unwrap_or_default().to_string();
            list.push(Library {
                id: String::new(),
                name: name.to_string(),
                version: text("version"),
                license: license(&text("license")),
                description: text("description"),
            });
        }
        Ok(list)
    };
    Ok((read("dependencies")?, read("devDependencies")?))
}

/// Text for a table cell: one line, nothing the Markdown or q.e.d.'s
/// extras (math, icons) would read as markup.
fn cell(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('$', "\\$")
        .replace('<', "&lt;")
        .replace(':', "&#58;")
}

/// A description cut to about two lines, at a word.
fn short(text: &str) -> String {
    const MAX: usize = 160;
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= MAX {
        return text;
    }
    let mut cut = String::new();
    for word in text.split(' ') {
        if cut.chars().count() + word.chars().count() + 1 > MAX {
            break;
        }
        if !cut.is_empty() {
            cut.push(' ');
        }
        cut.push_str(word);
    }
    format!("{}…", cut.trim_end_matches([',', '.', ';', ':']))
}

fn crate_row(page: &mut String, library: &Library) {
    let _ = writeln!(
        page,
        "| [{name}](https://crates.io/crates/{name}) | {version} | {license} | {description} |",
        name = library.name,
        version = library.version,
        license = cell(&library.license),
        description = cell(&short(&library.description)),
    );
}

fn npm_row(page: &mut String, library: &Library) {
    let _ = writeln!(
        page,
        "| [{name}](https://www.npmjs.com/package/{name}) | {version} | {license} | {description} |",
        name = library.name,
        version = library.version,
        license = cell(&library.license),
        description = cell(&short(&library.description)),
    );
}

fn page(rust: &[Library], direct: &BTreeSet<String>, app: &[Library], tools: &[Library]) -> String {
    let mut licenses: BTreeMap<&str, usize> = BTreeMap::new();
    for library in rust {
        *licenses.entry(library.license.as_str()).or_default() += 1;
    }
    let mut by_count: Vec<(&str, usize)> = licenses.into_iter().collect();
    by_count.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));

    let mut page = String::new();
    let _ = write!(
        page,
        r#"+++
title = "Libraries"
description = "The {count} Rust libraries and the JavaScript packages q.e.d. is built from, with their versions and licenses."
menu = "main"
weight = 95
summary = "Everything q.e.d. is built from, with versions and licenses."
+++

q.e.d. is made of its own code and of the work of many others: **{count}
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
"#,
        count = rust.len(),
    );
    for (license, count) in &by_count {
        let _ = writeln!(page, "| {} | {count} |", cell(license));
    }

    page.push_str(
        "\n## Used directly\n\n\
         The libraries q.e.d.'s own code calls. The others in the full list\n\
         below are what these need in turn.\n\n\
         | Library | Version | License | What it does |\n| --- | --- | --- | --- |\n",
    );
    for library in rust.iter().filter(|l| direct.contains(&l.id)) {
        crate_row(&mut page, library);
    }

    page.push_str(
        "\n## In the app's screens\n\n\
         JavaScript packages that are part of the app's screens.\n\n\
         | Package | Version | License | What it does |\n| --- | --- | --- | --- |\n",
    );
    for library in app {
        npm_row(&mut page, library);
    }
    page.push_str(
        "\n## For building the app\n\n\
         Tools that turn the app's code into the program. Svelte and\n\
         SvelteKit also leave a small part of themselves in the app, and the\n\
         stylesheet tools (Tailwind CSS, daisyUI) write the stylesheet of\n\
         every website too.\n\n\
         | Package | Version | License | What it does |\n| --- | --- | --- | --- |\n",
    );
    for library in tools {
        npm_row(&mut page, library);
    }

    let _ = write!(
        page,
        "\n## All Rust libraries\n\n\
         All {} of them, by name. Some appear twice, in two versions, when\n\
         libraries need different ones. Many are only used on one system\n\
         (Windows, macOS or Linux).\n\n\
         | Library | Version | License | What it does |\n| --- | --- | --- | --- |\n",
        rust.len()
    );
    for library in rust {
        crate_row(&mut page, library);
    }
    page
}

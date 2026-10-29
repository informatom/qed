//! Shared pieces: blocks used on several pages, edited in one place.
//!
//! A piece is a file in `shared/`, written like a page file (a title in the
//! front matter, then blocks), but never built as a page of its own. A page
//! shows it with a Shared block naming the file:
//!
//! ```text
//! ::: shared
//! name = "opening-hours"
//! :::
//! ```
//!
//! When the site is built, each Shared block is replaced by the piece's
//! blocks, so they render with the page's links, like the footer's.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::block::Block;
use crate::page::{self, FrontMatter, PageContent};
use crate::page_writer;
use crate::site::{self, EPISODE_LAYOUT_PATH, FOOTER_PATH, Page, Site, SiteError};
use crate::slug::slugify;

/// The folder that holds the pieces.
pub const SHARED_DIR: &str = "shared";

/// How a piece appears in lists of pages that use a file or link to a page;
/// never a URL path, which can't start with `(`.
pub fn path_of(name: &str) -> String {
    format!("(shared) {name}")
}

/// A piece's name: its file name without `.md`.
pub fn name_of(piece: &Page) -> &str {
    piece
        .file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
}

fn file_of(name: &str) -> PathBuf {
    Path::new(SHARED_DIR).join(format!("{name}.md"))
}

/// Reads every piece in `shared/`, sorted by name.
pub(crate) fn load(root: &Path) -> Result<Vec<Page>, SiteError> {
    let folder = root.join(SHARED_DIR);
    if !folder.is_dir() {
        return Ok(Vec::new());
    }
    let io_error = |source| SiteError::Io {
        path: folder.clone(),
        source,
    };
    let mut names = Vec::new();
    for entry in fs::read_dir(&folder).map_err(io_error)? {
        let path = entry.map_err(io_error)?.path();
        if path.is_file()
            && path.extension().is_some_and(|ext| ext == "md")
            && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
        {
            names.push(stem.to_string());
        }
    }
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let file = file_of(&name);
            let content =
                page::parse(&site::read(&root.join(&file))?).map_err(|error| SiteError::Page {
                    path: file.clone(),
                    error,
                })?;
            Ok(Page {
                path: path_of(&name),
                file,
                content,
            })
        })
        .collect()
}

/// Makes a new piece with these blocks and returns it. Its name comes from
/// the title, with a number added if that name is taken.
pub fn create(root: &Path, title: &str, blocks: Vec<Block>) -> Result<Page, SiteError> {
    let title = title.trim();
    let slug = slugify(title);
    if slug.is_empty() {
        return Err(SiteError::Invalid {
            path: PathBuf::from(SHARED_DIR),
            message: "the name needs at least one letter or digit".to_string(),
        });
    }
    let name = (1..)
        .map(|n| match n {
            1 => slug.clone(),
            n => format!("{slug}-{n}"),
        })
        .find(|name| !root.join(file_of(name)).exists())
        .unwrap_or(slug);
    let file = file_of(&name);
    let content = PageContent {
        front: FrontMatter {
            title: title.to_string(),
            ..FrontMatter::default()
        },
        blocks: without_shared(site::without_empty_text(blocks)),
    };
    site::write_new(&root.join(&file), &page_writer::write(&content, None))?;
    Ok(Page {
        path: path_of(&name),
        file,
        content,
    })
}

/// Saves a piece's blocks and returns it as saved. Shared blocks are left
/// out: a piece can't hold another one.
pub fn save_blocks(root: &Path, name: &str, blocks: Vec<Block>) -> Result<Page, SiteError> {
    let blocks = without_shared(site::without_empty_text(blocks));
    update(root, name, |content| content.blocks = blocks)
}

/// Gives a piece a new title; its name stays, so pages keep finding it.
pub fn save_title(root: &Path, name: &str, title: &str) -> Result<Page, SiteError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(SiteError::Invalid {
            path: file_of(name),
            message: "a shared block needs a name".to_string(),
        });
    }
    update(root, name, |content| {
        content.front.title = title.to_string()
    })
}

fn update(
    root: &Path,
    name: &str,
    change: impl FnOnce(&mut PageContent),
) -> Result<Page, SiteError> {
    let file = existing(root, name)?;
    // The file exists, so the title for a new one isn't needed.
    site::update_part(root, &file, &path_of(name), "", change)
}

/// The piece's file, if `name` is a plain name and the file is there. Names
/// come from the UI, so `..` and the like are refused.
fn existing(root: &Path, name: &str) -> Result<PathBuf, SiteError> {
    let file = file_of(name);
    if !name.is_empty() && slugify(name) == name && root.join(&file).is_file() {
        Ok(file)
    } else {
        Err(SiteError::NotAPage(file))
    }
}

/// The pages (URL paths, sorted), the footer ([`FOOTER_PATH`]) and the
/// episode layout ([`EPISODE_LAYOUT_PATH`]) that show the piece `name`.
pub fn users(site: &Site, name: &str) -> Vec<String> {
    let mut found: Vec<String> = site
        .pages
        .iter()
        .chain(site.footer.as_ref())
        .chain(site.episode_layout.as_ref())
        .filter(|page| names_in(&page.content.blocks).contains(name))
        .map(|page| page.path.clone())
        .collect();
    found.sort();
    found
}

/// Deletes the piece `name`, unless a page or the footer still shows it.
pub fn delete(site: &Site, name: &str) -> Result<(), SiteError> {
    let file = existing(&site.root, name)?;
    let used_by = users(site, name);
    if !used_by.is_empty() {
        let places: Vec<String> = used_by
            .iter()
            .map(|path| match path.as_str() {
                FOOTER_PATH => "the footer".to_string(),
                EPISODE_LAYOUT_PATH => "the episode layout".to_string(),
                path => format!("/{path}"),
            })
            .collect();
        return Err(SiteError::Invalid {
            path: file,
            message: format!("still used on {}", places.join(", ")),
        });
    }
    let full = site.root.join(&file);
    fs::remove_file(&full).map_err(|source| SiteError::Write { path: full, source })
}

/// The names of the pieces shown in a list of blocks, columns included.
fn names_in(blocks: &[Block]) -> BTreeSet<&str> {
    crate::block::all_blocks(blocks)
        .into_iter()
        .filter_map(|block| match block {
            Block::Shared(shared) => shared.name.as_deref(),
            _ => None,
        })
        .collect()
}

/// The blocks without Shared blocks, inside columns and sections too.
fn without_shared(blocks: Vec<Block>) -> Vec<Block> {
    crate::block::retain_deep(blocks, &|block| !matches!(block, Block::Shared(_)))
}

/// The site as the generator sees it: every Shared block on pages, in
/// the footer and in the episode layout replaced by its piece's blocks. A Shared block whose piece
/// is missing shows nothing and adds a note.
pub(crate) fn expanded(site: &Site, notes: &mut Vec<String>) -> Site {
    let mut expanded = site.clone();
    let parts = expanded
        .footer
        .iter_mut()
        .chain(&mut expanded.episode_layout);
    for page in expanded.pages.iter_mut().chain(parts) {
        let mut missing = BTreeSet::new();
        page.content.blocks = expand(&page.content.blocks, &site.shared, &mut missing);
        for name in missing {
            let place = match page.path.as_str() {
                FOOTER_PATH => "The footer".to_string(),
                EPISODE_LAYOUT_PATH => "The episode layout".to_string(),
                path => format!("/{path}"),
            };
            notes.push(if name.is_empty() {
                format!("{place} has a Shared block with no shared block picked.")
            } else {
                format!("{place} uses the shared block \"{name}\", which doesn't exist.")
            });
        }
    }
    expanded
}

fn expand(blocks: &[Block], pieces: &[Page], missing: &mut BTreeSet<String>) -> Vec<Block> {
    let mut out = Vec::new();
    for block in blocks {
        match block {
            Block::Shared(shared) => {
                let name = shared.name.as_deref().map(str::trim).unwrap_or_default();
                match pieces.iter().find(|piece| name_of(piece) == name) {
                    // A piece's own Shared blocks are dropped (it can't
                    // hold any), so pieces never include each other.
                    Some(piece) => out.extend(without_shared(piece.content.blocks.clone())),
                    None => {
                        missing.insert(name.to_string());
                    }
                }
            }
            other => {
                // Shared blocks inside columns and sections too.
                let mut block = other.clone();
                for inner in block.inner_lists_mut() {
                    *inner = expand(inner, pieces, missing);
                }
                out.push(block);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{Column, Columns, Faq, FaqItem, Shared};

    fn site_folder() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("site.toml"), "title = \"Test\"\n").unwrap();
        fs::create_dir(dir.path().join("pages")).unwrap();
        dir
    }

    fn text(markdown: &str) -> Block {
        Block::Text {
            markdown: markdown.to_string(),
        }
    }

    fn shared(name: &str) -> Block {
        Block::Shared(Shared {
            name: Some(name.to_string()),
        })
    }

    #[test]
    fn creates_reads_and_saves_pieces() {
        let dir = site_folder();
        let piece = create(dir.path(), "Öffnungszeiten", vec![text("Mo–Fr 9–17")]).unwrap();
        assert_eq!(piece.file, Path::new("shared/oeffnungszeiten.md"));
        assert_eq!(name_of(&piece), "oeffnungszeiten");
        let second = create(dir.path(), "Öffnungszeiten", Vec::new()).unwrap();
        assert_eq!(
            name_of(&second),
            "oeffnungszeiten-2",
            "a taken name gets a number"
        );

        let site = site::open(dir.path()).unwrap();
        assert_eq!(site.shared.len(), 2);
        assert_eq!(site.shared[0].path, "(shared) oeffnungszeiten");
        assert_eq!(site.shared[0].content.front.title, "Öffnungszeiten");
        assert_eq!(site.shared[0].content.blocks, [text("Mo–Fr 9–17")]);

        let saved = save_blocks(
            dir.path(),
            "oeffnungszeiten",
            vec![text("Mo–Sa 9–18"), shared("oeffnungszeiten-2")],
        )
        .unwrap();
        assert_eq!(
            saved.content.blocks,
            [text("Mo–Sa 9–18")],
            "no pieces in pieces"
        );
        let renamed = save_title(dir.path(), "oeffnungszeiten", "Hours").unwrap();
        assert_eq!(renamed.content.front.title, "Hours");
        assert_eq!(renamed.content.blocks, [text("Mo–Sa 9–18")]);

        assert!(save_blocks(dir.path(), "../site", Vec::new()).is_err());
        assert!(save_blocks(dir.path(), "missing", Vec::new()).is_err());
    }

    #[test]
    fn deletes_only_unused_pieces() {
        let dir = site_folder();
        create(dir.path(), "Hours", vec![text("9–17")]).unwrap();
        fs::write(
            dir.path().join("pages/contact.md"),
            "+++\ntitle = \"Contact\"\n+++\n\n:::: columns\n::: column\n:::\n::: shared\nname = \"hours\"\n:::\n::::\n",
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();
        assert_eq!(users(&site, "hours"), ["contact"]);
        let error = delete(&site, "hours").unwrap_err().to_string();
        assert!(error.contains("still used on /contact"), "{error}");

        fs::remove_file(dir.path().join("pages/contact.md")).unwrap();
        delete(&site::open(dir.path()).unwrap(), "hours").unwrap();
        assert!(!dir.path().join("shared/hours.md").exists());
    }

    #[test]
    fn expands_pieces_on_pages_and_in_columns() {
        let dir = site_folder();
        let faq = Block::Faq(Faq {
            heading: None,
            items: vec![FaqItem {
                question: "When?".to_string(),
                answer: "Now.".to_string(),
            }],
        });
        create(dir.path(), "Hours", vec![text("9–17"), faq.clone()]).unwrap();
        let mut site = site::open(dir.path()).unwrap();
        site.pages.push(Page {
            path: "contact".to_string(),
            file: PathBuf::from("pages/contact.md"),
            content: PageContent {
                front: FrontMatter::default(),
                blocks: vec![
                    text("Before"),
                    shared("hours"),
                    Block::Columns(Columns {
                        items: vec![Column {
                            blocks: vec![shared("hours"), shared("gone")],
                            ..Column::default()
                        }],
                    }),
                ],
            },
        });

        let mut notes = Vec::new();
        let built = expanded(&site, &mut notes);
        let blocks = &built.pages[0].content.blocks;
        assert_eq!(blocks[..3], [text("Before"), text("9–17"), faq.clone()]);
        let Block::Columns(columns) = &blocks[3] else {
            panic!("{blocks:?}")
        };
        assert_eq!(columns.items[0].blocks, [text("9–17"), faq]);
        assert_eq!(
            notes,
            ["/contact uses the shared block \"gone\", which doesn't exist."]
        );
    }
}

//! Renaming and moving pages: a new address for a page and the pages below
//! it, with redirects from the old addresses and links updated everywhere.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::block::{Block, for_each_block_mut, for_each_markdown};
use crate::links::LinkContext;
use crate::markdown;
use crate::page::PageContent;
use crate::site::{self, PAGES_DIR, Page, Site, SiteError};
use crate::slug::slugify;

/// One page that moves.
struct Move {
    old_path: String,
    new_path: String,
    old_file: PathBuf,
    new_file: PathBuf,
}

/// Gives the page in `file` a new address: below the page at `parent`
/// (`""` for the top level), with the last part `slug`. Pages below it move
/// along. Each moved page gets its old address in `redirect_from`, and
/// links to moved pages (and relative links in them) are updated in every
/// page. Returns the page at its new place.
pub fn move_page(site: &Site, file: &Path, parent: &str, slug: &str) -> Result<Page, SiteError> {
    let invalid = |message: String| SiteError::Invalid {
        path: file.to_path_buf(),
        message,
    };
    let page = site
        .pages
        .iter()
        .find(|page| page.file == file)
        .ok_or_else(|| SiteError::NotAPage(file.to_path_buf()))?;
    if page.path.is_empty() {
        return Err(invalid("the start page always stays at /".to_string()));
    }

    let slug = slug.trim();
    let clean = slugify(slug);
    if clean.is_empty() {
        return Err(invalid(
            "the address needs at least one letter or digit".to_string(),
        ));
    }
    if clean != slug {
        return Err(invalid(format!(
            "an address uses lowercase letters, digits and dashes: \"{clean}\""
        )));
    }
    let parent = parent.trim_matches('/');
    if !parent.is_empty() && !site.pages.iter().any(|p| p.path == parent) {
        return Err(invalid(format!(
            "there is no page /{parent} to move it under"
        )));
    }
    if parent == page.path || parent.starts_with(&format!("{}/", page.path)) {
        return Err(invalid("a page can't go below itself".to_string()));
    }
    let new_path = join(parent, slug);
    if new_path == page.path {
        return Ok(page.clone());
    }

    let moves = plan(site, page, &new_path)?;
    let new_paths: BTreeMap<&str, &str> = moves
        .iter()
        .map(|m| (m.old_path.as_str(), m.new_path.as_str()))
        .collect();
    let new_files: BTreeMap<&str, &Path> = site
        .pages
        .iter()
        .map(|p| {
            let file = moves
                .iter()
                .find(|m| m.old_path == p.path)
                .map_or(p.file.as_path(), |m| m.new_file.as_path());
            (p.path.as_str(), file)
        })
        .collect();

    // First the contents, in the old places: redirects and links.
    for other in &site.pages {
        let moving = moves.iter().find(|m| m.old_path == other.path);
        let own_file = moving.map_or(other.file.as_path(), |m| m.new_file.as_path());
        let links = new_links(other, own_file, moving.is_some(), &new_paths, &new_files);
        let taken: Vec<&str> = other
            .content
            .front
            .redirect_from
            .iter()
            .filter(|from| new_paths.values().any(|new| *new == from.trim_matches('/')))
            .map(String::as_str)
            .collect();
        if links.is_empty() && moving.is_none() && taken.is_empty() {
            continue;
        }
        site::update_page(&site.root, &other.file, |content| {
            rewrite_links(content, &links);
            let redirects = &mut content.front.redirect_from;
            // An address a moved page takes over can't redirect anywhere.
            redirects.retain(|from| !new_paths.values().any(|new| *new == from.trim_matches('/')));
            if let Some(moving) = moving
                && !redirects
                    .iter()
                    .any(|from| from.trim_matches('/') == moving.old_path)
            {
                redirects.push(format!("/{}", moving.old_path));
            }
        })?;
    }

    // Entries of a hand-made menu follow their pages.
    let mut config = site.config.clone();
    if move_menu_entries(&mut config.main_menu, &new_paths) {
        site::save_config(&site.root, &config)?;
    }

    // The footer links like any page, but never moves.
    if let Some(footer) = &site.footer {
        let links = new_links(footer, &footer.file, false, &new_paths, &new_files);
        if !links.is_empty() {
            site::update_footer(&site.root, |content| rewrite_links(content, &links))?;
        }
    }
    // So do the episode layout and the shared pieces.
    if let Some(layout) = &site.episode_layout {
        let links = new_links(layout, &layout.file, false, &new_paths, &new_files);
        if !links.is_empty() {
            site::update_episode_layout(&site.root, |content| rewrite_links(content, &links))?;
        }
    }
    for piece in &site.shared {
        let links = new_links(piece, &piece.file, false, &new_paths, &new_files);
        if !links.is_empty() {
            site::update_part(&site.root, &piece.file, &piece.path, "", |content| {
                rewrite_links(content, &links)
            })?;
        }
    }

    // Then the files.
    for moving in &moves {
        let to = site.root.join(&moving.new_file);
        if let Some(folder) = to.parent() {
            fs::create_dir_all(folder).map_err(|source| SiteError::Write {
                path: folder.to_path_buf(),
                source,
            })?;
        }
        fs::rename(site.root.join(&moving.old_file), &to)
            .map_err(|source| SiteError::Write { path: to, source })?;
    }
    for moving in &moves {
        remove_empty_folders(&site.root, &moving.old_file);
    }

    let moved = site::open(&site.root)?;
    moved
        .pages
        .into_iter()
        .find(|p| p.path == new_path)
        .ok_or_else(|| SiteError::NotAPage(moves[0].new_file.clone()))
}

/// Points menu entries at the new addresses of moved pages; `page = …`
/// and links to `/old-address`. Returns whether anything changed.
fn move_menu_entries(items: &mut [site::MenuItem], new_paths: &BTreeMap<&str, &str>) -> bool {
    let mut changed = false;
    for item in items {
        if let Some(page) = &mut item.page
            && let Some(new) = new_paths.get(site::menu_page_path(page).as_str())
        {
            *page = new.to_string();
            changed = true;
        }
        if let Some(link) = &mut item.link
            && let Some(path) = link.strip_prefix('/')
        {
            let (path, rest) = path.split_at(path.find(['#', '?']).unwrap_or(path.len()));
            if let Some(new) = new_paths.get(path.trim_end_matches('/')) {
                *link = format!("/{new}{rest}");
                changed = true;
            }
        }
        changed |= move_menu_entries(&mut item.items, new_paths);
    }
    changed
}

/// The page and every page below it, with their new paths and files.
/// Fails if a new address or file is taken.
fn plan(site: &Site, page: &Page, new_path: &str) -> Result<Vec<Move>, SiteError> {
    let below = format!("{}/", page.path);
    let moves: Vec<Move> = site
        .pages
        .iter()
        .filter(|p| p.path == page.path || p.path.starts_with(&below))
        .map(|p| {
            let path = format!("{new_path}{}", &p.path[page.path.len()..]);
            // `about/index.md` stays in its own folder, `about.md` doesn't.
            let file = if p.file.file_name().is_some_and(|name| name == "index.md") {
                PathBuf::from(PAGES_DIR).join(&path).join("index.md")
            } else {
                PathBuf::from(PAGES_DIR).join(&path).with_extension("md")
            };
            Move {
                old_path: p.path.clone(),
                new_path: path,
                old_file: p.file.clone(),
                new_file: file,
            }
        })
        .collect();

    for moving in &moves {
        let staying = |p: &&Page| !moves.iter().any(|m| m.old_path == p.path);
        if let Some(existing) = site
            .pages
            .iter()
            .filter(staying)
            .find(|p| p.path == moving.new_path)
        {
            return Err(SiteError::DuplicatePath {
                url: moving.new_path.clone(),
                first: existing.file.clone(),
                second: moving.old_file.clone(),
            });
        }
        let leaving = moves.iter().any(|m| m.old_file == moving.new_file);
        if site.root.join(&moving.new_file).exists() && !leaving {
            return Err(SiteError::Invalid {
                path: moving.new_file.clone(),
                message: "this file is in the way; move it aside first".to_string(),
            });
        }
    }
    Ok(moves)
}

/// The links in `page` that change, as written (old to new). `own_file` is
/// where the page will be; `moving` says whether it moves.
fn new_links(
    page: &Page,
    own_file: &Path,
    moving: bool,
    new_paths: &BTreeMap<&str, &str>,
    new_files: &BTreeMap<&str, &Path>,
) -> BTreeMap<String, String> {
    let old = LinkContext::new(&page.path, &page.file);
    let mut changes = BTreeMap::new();
    for link in site::link_targets(&page.content.blocks) {
        let Some(target) = old.page_path_of(&link) else {
            continue;
        };
        // Links that are broken already stay as they are.
        let Some(target_file) = new_files.get(target.as_str()) else {
            continue;
        };
        let new_target = new_paths.get(target.as_str()).copied();
        if !moving && new_target.is_none() {
            continue;
        }
        let split = link.find(['#', '?']).unwrap_or(link.len());
        let (base, rest) = link.split_at(split);
        let new = if base.starts_with('/') {
            let Some(path) = new_target else {
                continue; // `/about` still works from anywhere.
            };
            let slash = if base.len() > 1 && base.ends_with('/') {
                "/"
            } else {
                ""
            };
            match path {
                "" => "/".to_string(),
                path => format!("/{path}{slash}"),
            }
        } else {
            relative(own_file.parent().unwrap_or(Path::new("")), target_file)
        };
        let new = format!("{new}{rest}");
        if new != link {
            changes.insert(link, new);
        }
    }
    changes
}

/// Applies `links` (old to new, as written) to the Markdown and buttons of
/// a page.
fn rewrite_links(content: &mut PageContent, links: &BTreeMap<String, String>) {
    if links.is_empty() {
        return;
    }
    for_each_block_mut(&mut content.blocks, &mut |block| {
        for_each_markdown(block, &mut |text| {
            *text = markdown::replace_link_targets(text, links);
        });
        let from = match block {
            Block::PostList(list) => Some(&mut list.from),
            Block::PageList(list) => Some(&mut list.from),
            _ => None,
        };
        if let Some(from) = from {
            if let Some(new) = from
                .as_deref()
                .and_then(|from| links.get(&site::post_list_link(from)))
            {
                *from = Some(new.trim_start_matches('/').to_string());
            }
            return;
        }
        let button = match block {
            Block::Hero(hero) => &mut hero.button_link,
            Block::CallToAction(cta) => &mut cta.button_link,
            _ => return,
        };
        if let Some(new) = button.as_ref().and_then(|link| links.get(link)) {
            *button = Some(new.clone());
        }
    });
}

/// The relative path from the folder `from` to the file `to`, both
/// relative to the site folder, with `/` between the parts.
fn relative(from: &Path, to: &Path) -> String {
    let parts = |path: &Path| -> Vec<String> {
        path.components()
            .filter_map(|c| match c {
                Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect()
    };
    let (from, to) = (parts(from), parts(to));
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut result: Vec<String> = vec!["..".to_string(); from.len() - common];
    result.extend(to[common..].iter().cloned());
    result.join("/")
}

/// Removes the folders above `file` that are empty now, up to `pages/`.
fn remove_empty_folders(root: &Path, file: &Path) {
    let pages = root.join(PAGES_DIR);
    let mut folder = root.join(file);
    while folder.pop() && folder.starts_with(&pages) && folder != pages {
        if fs::remove_dir(&folder).is_err() {
            break; // Not empty: something else lives there.
        }
    }
}

fn join(parent: &str, slug: &str) -> String {
    if parent.is_empty() {
        slug.to_string()
    } else {
        format!("{parent}/{slug}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, file: &str, text: &str) {
        let path = root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn sample() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(root, "site.toml", "title = \"S\"\n");
        write(
            root,
            "pages/index.md",
            "+++\ntitle = \"Home\"\n+++\n\n\
             Meet [us](/about), the [team](about/team.md#people) and [a\n\
             friend](/elsewhere). Also [contact](contact.md).\n\n\
             ::: hero\nbutton_label = \"Team\"\nbutton_link = \"/about/team/\"\n:::\n\n\
             ::: post-list\nfrom = \"about\"\n:::\n",
        );
        write(
            root,
            "pages/about.md",
            "+++\ntitle = \"About\"\n+++\n\nBack [home](index.md).\n",
        );
        write(
            root,
            "pages/about/team.md",
            "+++\ntitle = \"Team\"\n+++\n\nSee [about](../about.md) and [contact](../contact.md).\n",
        );
        write(
            root,
            "pages/contact.md",
            "+++\ntitle = \"Contact\"\nredirect_from = [\"/company\"]\n+++\n\nHi.\n",
        );
        dir
    }

    fn read(root: &Path, file: &str) -> String {
        fs::read_to_string(root.join(file)).unwrap()
    }

    #[test]
    fn renames_a_page_with_the_pages_below_it() {
        let dir = sample();
        let root = dir.path();
        let site = site::open(root).unwrap();
        let page = move_page(&site, Path::new("pages/about.md"), "", "company").unwrap();
        assert_eq!(page.path, "company");
        assert_eq!(page.file, Path::new("pages/company.md"));
        assert_eq!(page.content.front.redirect_from, ["/about"]);

        assert!(!root.join("pages/about.md").exists());
        assert!(
            !root.join("pages/about").exists(),
            "empty folder left behind"
        );
        let team = read(root, "pages/company/team.md");
        assert!(team.contains("redirect_from = [\"/about/team\"]"), "{team}");
        assert!(team.contains("[about](../company.md)"), "{team}");
        assert!(team.contains("[contact](../contact.md)"), "{team}");

        let home = read(root, "pages/index.md");
        assert!(home.contains("[us](/company)"), "{home}");
        assert!(home.contains("[team](company/team.md#people)"), "{home}");
        assert!(home.contains("friend](/elsewhere)"), "{home}");
        assert!(home.contains("button_link = \"/company/team/\""), "{home}");
        assert!(home.contains("from = \"company\""), "{home}");
        assert!(!home.contains("redirect_from"), "{home}");

        // The address /company was a redirect to Contact; the page took it over.
        let contact = read(root, "pages/contact.md");
        assert!(!contact.contains("redirect_from"), "{contact}");

        crate::generator::build(&site::open(root).unwrap()).unwrap();
    }

    #[test]
    fn moves_a_page_below_another_and_back() {
        let dir = sample();
        let root = dir.path();
        let site = site::open(root).unwrap();
        let page = move_page(&site, Path::new("pages/contact.md"), "about", "contact").unwrap();
        assert_eq!(page.path, "about/contact");
        let team = read(root, "pages/about/team.md");
        assert!(team.contains("[contact](contact.md)"), "{team}");
        let home = read(root, "pages/index.md");
        assert!(home.contains("[contact](about/contact.md)"), "{home}");

        let site = site::open(root).unwrap();
        let back = move_page(&site, Path::new("pages/about/contact.md"), "", "contact").unwrap();
        assert_eq!(
            back.content.front.redirect_from,
            ["/company", "/about/contact"]
        );
        assert!(read(root, "pages/index.md").contains("[contact](contact.md)"));
    }

    #[test]
    fn rewrites_links_in_the_footer() {
        let dir = sample();
        write(
            dir.path(),
            "footer.md",
            "+++\ntitle = \"Footer\"\n+++\n\n[About](/about) and [Home](/)\n",
        );
        let site = site::open(dir.path()).unwrap();
        move_page(&site, Path::new("pages/about.md"), "", "who-we-are").unwrap();
        let footer = read(dir.path(), "footer.md");
        assert!(
            footer.contains("[About](/who-we-are) and [Home](/)"),
            "{footer}"
        );
    }

    #[test]
    fn moves_menu_entries_with_their_pages() {
        let dir = sample();
        let toml = fs::read_to_string(dir.path().join("site.toml")).unwrap();
        fs::write(
            dir.path().join("site.toml"),
            format!("{toml}\n[[main_menu]]\npage = \"about\"\n\n[[main_menu]]\ntitle = \"Team\"\nlink = \"/about/team/#people\"\n"),
        )
        .unwrap();
        let site = site::open(dir.path()).unwrap();
        assert!(site::pages_linking_to(&site, "about").contains(&site::MENU_PATH.to_string()));

        move_page(&site, Path::new("pages/about.md"), "", "who-we-are").unwrap();
        let menu = site::open(dir.path()).unwrap().config.main_menu;
        assert_eq!(menu[0].page.as_deref(), Some("who-we-are"));
        assert_eq!(menu[1].link.as_deref(), Some("/who-we-are/team#people"));
    }

    #[test]
    fn refuses_moves_that_dont_work() {
        let dir = sample();
        let root = dir.path();
        let site = site::open(root).unwrap();
        let attempt = |file: &str, parent: &str, slug: &str| {
            move_page(&site, Path::new(file), parent, slug)
                .unwrap_err()
                .to_string()
        };
        assert!(attempt("pages/index.md", "", "home").contains("start page"));
        assert!(attempt("pages/about.md", "about/team", "x").contains("below itself"));
        assert!(attempt("pages/about.md", "", "contact").contains("/contact"));
        assert!(attempt("pages/about.md", "", "Big News").contains("big-news"));
        assert!(attempt("pages/about.md", "nowhere", "x").contains("/nowhere"));
        // Nothing changed.
        assert_eq!(site::open(root).unwrap().pages.len(), 4);
        assert!(read(root, "pages/index.md").contains("[us](/about)"));
    }

    #[test]
    fn finds_relative_paths() {
        assert_eq!(
            relative(Path::new("pages"), Path::new("pages/a.md")),
            "a.md"
        );
        assert_eq!(
            relative(Path::new("pages/a/b"), Path::new("pages/c.md")),
            "../../c.md"
        );
        assert_eq!(
            relative(Path::new("pages/a"), Path::new("pages/a/b/index.md")),
            "b/index.md"
        );
    }
}

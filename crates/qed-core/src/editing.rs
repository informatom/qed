//! Changes to a site that both shells make, the app and `qed-mcp` (#105),
//! with the same rules: a page's text is checked before it's written, and
//! the open version is closed before a page is deleted or moved, so the
//! history keeps the page as it was and it can come back.
//!
//! Recording the change itself (`history::record_save`) is up to the
//! caller, after any change, like every other save.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Component, Path};

use crate::history::{self, HistoryError};
use crate::moving;
use crate::page;
use crate::shared::SHARED_DIR;
use crate::site::{self, CONFIG_FILE, Page, SiteConfig, SiteError};

#[derive(Debug, thiserror::Error)]
pub enum EditError {
    #[error(transparent)]
    Site(#[from] SiteError),

    #[error(transparent)]
    History(#[from] HistoryError),

    #[error(
        "{file} changed since it was read; read it again and make the change on what it says now"
    )]
    Changed { file: String },
}

/// A page file's text, as written, with a stamp that changes whenever the
/// text does: `write_page_text` takes it back to make sure nobody changed
/// the file in between. Also for the files of blocks that aren't pages
/// (see [`is_part`]); one that doesn't exist yet reads as empty.
pub fn page_text(root: &Path, file: &Path) -> Result<(String, String), EditError> {
    if is_part(file) {
        let full = root.join(file);
        let text = if full.is_file() {
            site::read(&full)?
        } else {
            String::new()
        };
        let stamp = stamp(&text);
        return Ok((text, stamp));
    }
    let page = site::read_page(root, file)?;
    let text = site::read(&root.join(&page.file))?;
    let stamp = stamp(&text);
    Ok((text, stamp))
}

/// Replaces a page file's whole text, if it's a valid page (settings
/// between `+++` lines, blocks as `:::` containers) with a title, and if
/// the file still has the text `stamp` was made from. Returns the page as
/// written.
pub fn write_page_text(
    root: &Path,
    file: &Path,
    text: &str,
    stamp: &str,
) -> Result<Page, EditError> {
    let (now, now_stamp) = page_text(root, file)?;
    if now_stamp != stamp {
        return Err(EditError::Changed {
            file: file.display().to_string(),
        });
    }
    let invalid = |message: String| SiteError::Invalid {
        path: file.to_path_buf(),
        message,
    };
    let content = page::parse(text)
        .map_err(|error| invalid(format!("line {}: {}", error.line, error.message)))?;
    if content.front.title.trim().is_empty() {
        return Err(invalid("a page needs a title".to_string()).into());
    }
    // An episode gets the checks of the app's Settings tab.
    if let Some(episode) = &content.front.episode {
        site::check_episode(root, file, episode)?;
        // Its own cover follows the podcast cover's rules, or the build
        // refuses it later.
        if let Some(image) = episode.image.as_deref().map(str::trim)
            && !image.is_empty()
        {
            let problems = crate::podcast::check_cover(&root.join(site::MEDIA_DIR), image);
            if !problems.is_empty() {
                return Err(invalid(format!(
                    "episode image {image}: {}; without `image`, the podcast's cover shows",
                    problems.join(", ")
                ))
                .into());
            }
        }
    }
    if now != text {
        site::write_atomically(&root.join(file), text)?;
    }
    // An episode written without a guid gets one, as in the app.
    if content
        .front
        .episode
        .as_ref()
        .is_some_and(|episode| episode.guid.trim().is_empty())
    {
        return Ok(site::update_page(root, file, |content| {
            if let Some(episode) = content.front.episode.as_mut() {
                episode.guid = crate::podcast::new_guid();
            }
        })?);
    }
    Ok(site::read_page(root, file)?)
}

/// Whether `file` is one of the files of blocks that aren't pages: the
/// footer (`footer.md`, and `footer.en.md` for a language), the episode
/// layout (`episode.md`) or a shared block (`shared/teaser.md`).
pub fn is_part(file: &Path) -> bool {
    let plain = file.components().all(|c| matches!(c, Component::Normal(_)));
    let markdown = file.extension().is_some_and(|ext| ext == "md");
    let stem = file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let folder = file.parent().unwrap_or(Path::new(""));
    plain
        && markdown
        && !stem.is_empty()
        && if folder == Path::new("") {
            stem == "footer" || stem.starts_with("footer.") || stem == "episode"
        } else {
            folder == Path::new(SHARED_DIR)
        }
}

/// Replaces the text of a file of blocks that isn't a page (see
/// [`is_part`]), like [`write_page_text`]: only if it's valid and still
/// what `stamp` was made from. An empty text removes the file: no footer,
/// the default episode layout, no such shared block.
pub fn write_part_text(root: &Path, file: &Path, text: &str, stamp: &str) -> Result<(), EditError> {
    if !is_part(file) {
        return Err(SiteError::NotAPage(file.to_path_buf()).into());
    }
    let (now, now_stamp) = page_text(root, file)?;
    if now_stamp != stamp {
        return Err(EditError::Changed {
            file: file.display().to_string(),
        });
    }
    let name = file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    if file.starts_with(SHARED_DIR) && crate::slug::slugify(name) != name {
        return Err(SiteError::Invalid {
            path: file.to_path_buf(),
            message: format!(
                "a shared block's name is lowercase letters, digits and dashes: {}",
                crate::slug::slugify(name)
            ),
        }
        .into());
    }
    if !text.trim().is_empty() {
        page::parse(text).map_err(|error| SiteError::Invalid {
            path: file.to_path_buf(),
            message: format!("line {}: {}", error.line, error.message),
        })?;
    }
    replace_checked(root, file, &now, text)
}

/// Notes for `:icon-name:` in a page's text (`raw`, parsed as `content`)
/// that won't become an icon on the website: names that neither q.e.d.
/// nor the site (`media/icons/`) has, and icons written where the page
/// shows plain text (a title, a headline, a button label, a question),
/// since only Markdown text turns them into icons. Without these notes an
/// assistant only learns it from the owner, who sees `:icon-…:` as text.
pub fn icon_notes(root: &Path, content: &page::PageContent, raw: &str) -> Vec<String> {
    if !raw.contains(":icon-") {
        return Vec::new();
    }
    let site_icons = crate::icons::SiteIcons::load(&root.join(site::MEDIA_DIR));
    let mut shown: Vec<&str> = site::markdown_texts(&content.blocks);
    shown.extend(content.front.summary.as_deref());

    let mut notes = Vec::new();
    let mut unknown: Vec<String> = shown
        .iter()
        .flat_map(|text| crate::markdown::unknown_icons(text, &site_icons))
        .collect();
    unknown.sort();
    unknown.dedup();
    for name in unknown {
        notes.push(format!(
            "There is no icon \"{name}\", so `:icon-{name}:` shows as text; find_icons finds the names."
        ));
    }

    // Each name as often as Markdown text has it; what's left over in the
    // whole file is in a field that shows plain text.
    let mut left: Vec<&str> = crate::markdown::icon_names(raw);
    for name in shown
        .iter()
        .flat_map(|text| crate::markdown::icon_names(text))
    {
        if let Some(index) = left.iter().position(|left| *left == name) {
            left.remove(index);
        }
    }
    left.sort_unstable();
    left.dedup();
    for name in left {
        notes.push(format!(
            "`:icon-{name}:` is in a field that shows plain text (a title, headline, button label, question or column title), so it shows as text there. Icons work in Markdown text: Text blocks, the `text` of hero, call-to-action and text-image, image captions, FAQ answers and `summary`. A page's `icon` setting puts an icon beside it in menus."
        ));
    }
    notes
}

/// The text of `site.toml`, the site's settings, with a stamp like
/// [`page_text`]'s.
pub fn settings_text(root: &Path) -> Result<(String, String), EditError> {
    let text = site::read(&root.join(CONFIG_FILE))?;
    let stamp = stamp(&text);
    Ok((text, stamp))
}

/// Replaces `site.toml`, if it's valid and still what `stamp` was made
/// from. The site must open with it, so a menu entry for a page that
/// doesn't exist or a language without its pages is refused too.
pub fn write_settings_text(root: &Path, text: &str, stamp: &str) -> Result<(), EditError> {
    let (now, now_stamp) = settings_text(root)?;
    if now_stamp != stamp {
        return Err(EditError::Changed {
            file: CONFIG_FILE.to_string(),
        });
    }
    let config = toml::from_str::<SiteConfig>(text).map_err(|source| SiteError::Config {
        path: root.join(CONFIG_FILE),
        source,
    })?;
    // The team must be contributors, as the Podcast tab makes sure.
    if let Some(person) = config
        .podcast
        .iter()
        .flat_map(|p| &p.people)
        .find(|person| {
            !root
                .join(crate::contributors::CONTRIBUTORS_DIR)
                .join(format!("{}.md", person.contributor))
                .is_file()
        })
    {
        return Err(SiteError::Invalid {
            path: Path::new(CONFIG_FILE).to_path_buf(),
            message: format!(
                "podcast: there is no contributor “{}”; save them first",
                person.contributor
            ),
        }
        .into());
    }
    replace_checked(root, Path::new(CONFIG_FILE), &now, text)?;
    // A podcast gets what the Podcast tab gives it: its lasting guid once
    // the site has a web address (saving the settings makes it), and
    // `audio/` kept out of the history unless it's meant to be in it.
    if let Some(podcast) = &config.podcast {
        let has_address = config
            .base_url
            .as_deref()
            .is_some_and(|url| !url.trim().is_empty());
        if has_address && podcast.guid.as_deref().is_none_or(|g| g.trim().is_empty()) {
            site::save_config(root, &config)?;
        } else {
            crate::podcast::sync_audio_ignore(root, Some(podcast))?;
        }
    }
    Ok(())
}

/// Writes `text` over `before` in `file` (an empty text removes it), then
/// opens the site: if it doesn't open any more, the file gets `before`
/// back and the reason is returned.
fn replace_checked(root: &Path, file: &Path, before: &str, text: &str) -> Result<(), EditError> {
    if before == text {
        return Ok(());
    }
    let full = root.join(file);
    let write = |text: &str| -> Result<(), SiteError> {
        if text.trim().is_empty() {
            if full.exists() {
                std::fs::remove_file(&full).map_err(|source| SiteError::Write {
                    path: full.clone(),
                    source,
                })?;
            }
            return Ok(());
        }
        if let Some(folder) = full.parent() {
            std::fs::create_dir_all(folder).map_err(|source| SiteError::Write {
                path: folder.to_path_buf(),
                source,
            })?;
        }
        site::write_atomically(&full, text)
    };
    write(text)?;
    if let Err(error) = site::open(root) {
        write(before)?;
        return Err(error.into());
    }
    Ok(())
}

/// Deletes a page, after closing the open version, so it keeps the page.
/// Otherwise the deletion would go into the same open version, and a page
/// made and deleted in it would leave no trace to bring back.
pub fn delete_page(root: &Path, file: &Path) -> Result<(), EditError> {
    let site = site::open(root)?;
    close_before(&site, root, file, "Before deleting")?;
    site::delete_page(&site, file)?;
    Ok(())
}

/// Gives a page a new address (see `moving::move_page`), after closing the
/// open version, so the site from before the move stays in the history.
pub fn move_page(root: &Path, file: &Path, parent: &str, slug: &str) -> Result<Page, EditError> {
    let site = site::open(root)?;
    close_before(&site, root, file, "Before moving")?;
    Ok(moving::move_page(&site, file, parent, slug)?)
}

fn close_before(site: &site::Site, root: &Path, file: &Path, what: &str) -> Result<(), EditError> {
    if !history::status(root)?.tracked {
        return Ok(());
    }
    if let Some(page) = site.pages.iter().find(|page| page.file == file) {
        history::save_version(root, &format!("{what} /{}", page.path))?;
    }
    Ok(())
}

/// A short fingerprint of a text. Not for security, only to notice a change.
fn stamp(text: &str) -> String {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn site() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("site.toml"), "title = \"S\"\n").unwrap();
        fs::create_dir_all(dir.path().join("pages")).unwrap();
        fs::write(
            dir.path().join("pages/index.md"),
            "+++\ntitle = \"Home\"\n+++\n\nHi.\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn writes_valid_text_only_over_what_was_read() {
        let dir = site();
        let (root, file) = (dir.path(), Path::new("pages/index.md"));
        let (text, stamp) = page_text(root, file).unwrap();
        assert!(text.contains("Hi."));

        let error = write_page_text(root, file, "no settings", &stamp).unwrap_err();
        assert!(error.to_string().contains("line 1"), "{error}");
        let error = write_page_text(root, file, "+++\ntitle = \"\"\n+++\n", &stamp).unwrap_err();
        assert!(error.to_string().contains("needs a title"), "{error}");

        let new = "+++\ntitle = \"Home\"\n+++\n\n::: hero\nheadline = \"Welcome\"\n:::\n";
        let page = write_page_text(root, file, new, &stamp).unwrap();
        assert_eq!(page.content.blocks.len(), 1);

        // The stamp from before that write no longer fits.
        let error = write_page_text(root, file, new, &stamp).unwrap_err();
        assert!(matches!(error, EditError::Changed { .. }), "{error}");
    }

    #[test]
    fn notes_icons_that_stay_text() {
        let dir = site();
        let raw = "+++\ntitle = \"Home\"\n+++\n\nCall :icon-phone: or :icon-telephon:.\n\n\
                   ::: hero\nheadline = \":icon-heart: Welcome\"\ntext = \"With :icon-heart:\"\n:::\n";
        let content = page::parse(raw).unwrap();
        let notes = icon_notes(dir.path(), &content, raw);
        assert_eq!(notes.len(), 2, "{notes:?}");
        assert!(notes[0].contains("no icon \"telephon\""), "{notes:?}");
        assert!(
            notes[1].contains("`:icon-heart:` is in a field"),
            "{notes:?}"
        );

        let fine = "+++\ntitle = \"Home\"\n+++\n\nCall :icon-phone:.\n";
        assert!(icon_notes(dir.path(), &page::parse(fine).unwrap(), fine).is_empty());
    }

    #[test]
    fn closes_the_open_version_before_deleting() {
        let dir = site();
        let root = dir.path();
        fs::write(root.join("pages/old.md"), "+++\ntitle = \"Old\"\n+++\n").unwrap();
        history::start(root).unwrap();
        fs::write(root.join("pages/old.md"), "+++\ntitle = \"Old one\"\n+++\n").unwrap();
        history::record_save(root).unwrap();

        delete_page(root, Path::new("pages/old.md")).unwrap();
        history::record_save(root).unwrap();
        let versions = history::versions(root, None).unwrap();
        assert_eq!(versions[1].summary, "Before deleting /old");
        assert!(!root.join("pages/old.md").exists());
    }

    #[test]
    fn writes_the_footer_and_shared_blocks() {
        let dir = site();
        let root = dir.path();
        assert!(is_part(Path::new("footer.md")) && is_part(Path::new("footer.en.md")));
        assert!(is_part(Path::new("shared/teaser.md")) && is_part(Path::new("episode.md")));
        assert!(!is_part(Path::new("pages/index.md")) && !is_part(Path::new("../footer.md")));
        assert!(!is_part(Path::new("site.toml")) && !is_part(Path::new("shared/a/b.md")));

        // A footer that isn't there yet reads as empty, and writing makes it.
        let footer = Path::new("footer.md");
        let (text, stamp) = page_text(root, footer).unwrap();
        assert_eq!(text, "");
        let new = "+++\ntitle = \"Footer\"\n+++\n\n© 2026\n";
        write_part_text(root, footer, new, &stamp).unwrap();
        assert!(site::open(root).unwrap().footer.is_some());
        // Empty text removes it again.
        let (_, stamp) = page_text(root, footer).unwrap();
        write_part_text(root, footer, "", &stamp).unwrap();
        assert!(!root.join("footer.md").exists());

        let shared = Path::new("shared/teaser.md");
        let (_, stamp) = page_text(root, shared).unwrap();
        write_part_text(root, shared, "+++\ntitle = \"Teaser\"\n+++\n\nHi\n", &stamp).unwrap();
        assert_eq!(site::open(root).unwrap().shared.len(), 1);
        let error =
            write_part_text(root, Path::new("shared/My Teaser.md"), "x", &stamp).unwrap_err();
        assert!(error.to_string().contains("my-teaser"), "{error}");
        let error = write_part_text(root, Path::new("pages/index.md"), "x", &stamp).unwrap_err();
        assert!(
            matches!(error, EditError::Site(SiteError::NotAPage(_))),
            "{error}"
        );
    }

    #[test]
    fn writes_settings_only_when_the_site_still_opens() {
        let dir = site();
        let root = dir.path();
        fs::write(
            root.join("site.toml"),
            "title = \"S\"\n[languages.en]\ntitle = \"S\"\n",
        )
        .unwrap();
        fs::write(
            root.join("pages/index.en.md"),
            "+++\ntitle = \"Home\"\n+++\n",
        )
        .unwrap();
        let (text, stamp) = settings_text(root).unwrap();

        let error = write_settings_text(root, "title = ", &stamp).unwrap_err();
        assert!(
            matches!(error, EditError::Site(SiteError::Config { .. })),
            "{error}"
        );
        // Valid TOML, but the site doesn't open without the language its
        // translation is in: the old text stays.
        let error = write_settings_text(root, "title = \"S\"\n", &stamp).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("isn't one of the site's languages"),
            "{error}"
        );
        assert_eq!(settings_text(root).unwrap().0, text);

        let new = text.replacen("\n", "\nbase_url = \"https://example.com/\"\n", 1);
        write_settings_text(root, &new, &stamp).unwrap();
        assert_eq!(
            site::open(root).unwrap().config.base_url.as_deref(),
            Some("https://example.com/")
        );
        let error = write_settings_text(root, &new, &stamp).unwrap_err();
        assert!(matches!(error, EditError::Changed { .. }), "{error}");
    }

    #[test]
    fn podcast_settings_as_text_get_what_the_podcast_tab_gives() {
        let dir = site();
        let root = dir.path();
        let (text, stamp) = settings_text(root).unwrap();

        // The team must be contributors.
        let team = format!(
            "{text}base_url = \"https://pod.example/\"\n\n[podcast]\nauthor = \"S\"\n\n\
             [[podcast.people]]\ncontributor = \"stefan-haslinger\"\n"
        );
        let error = write_settings_text(root, &team, &stamp).unwrap_err();
        assert!(error.to_string().contains("no contributor"), "{error}");

        let podcast =
            format!("{text}base_url = \"https://pod.example/\"\n\n[podcast]\nauthor = \"S\"\n");
        write_settings_text(root, &podcast, &stamp).unwrap();
        let config = site::open(root).unwrap().config;
        let saved = config.podcast.unwrap();
        assert_eq!(saved.author.as_deref(), Some("S"));
        assert_eq!(
            saved.guid.as_deref(),
            Some(crate::podcast::podcast_guid("https://pod.example/episodes.mp3.rss").as_str())
        );
        assert!(root.join("audio/.gitignore").is_file());
    }

    #[test]
    fn checks_episodes_written_as_text() {
        let dir = site();
        let root = dir.path();
        fs::write(root.join("site.toml"), "title = \"S\"\n\n[podcast]\n").unwrap();
        let first = crate::podcast::create_episode_at(
            root,
            "Episode 0",
            Some(0),
            crate::page::PageDate::parse("2016-03-22"),
            Some("episode0"),
            crate::podcast::NewEpisode::default(),
        )
        .unwrap();
        assert_eq!(first.path, "episodes/episode0");
        let second = crate::podcast::create_episode(root, "Episode 1", None).unwrap();
        let old_guid = "https://pod.example/2016/03/22/episode0.html";

        let (text, stamp) = page_text(root, &first.file).unwrap();
        let guid_line = text.lines().find(|l| l.starts_with("guid")).unwrap();
        let with = |replacement: &str| text.replace(guid_line, replacement);

        // Chapters out of order and unknown contributors are refused.
        let chapters = with(&format!(
            "guid = \"{old_guid}\"\n\n[[episode.chapters]]\nstart = \"00:10:00\"\ntitle = \"B\"\n\n\
             [[episode.chapters]]\nstart = \"00:01:00\"\ntitle = \"A\""
        ));
        let error = write_page_text(root, &first.file, &chapters, &stamp).unwrap_err();
        assert!(error.to_string().contains("starts before"), "{error}");
        let same = chapters.replace("00:10:00", "00:01:00");
        let error = write_page_text(root, &first.file, &same, &stamp).unwrap_err();
        assert!(error.to_string().contains("same time"), "{error}");
        // An own cover must follow the cover's rules.
        fs::create_dir_all(root.join("media")).unwrap();
        image::RgbImage::new(360, 360)
            .save(root.join("media/small.png"))
            .unwrap();
        let small = with(&format!("guid = \"{old_guid}\"\nimage = \"small.png\""));
        let error = write_page_text(root, &first.file, &small, &stamp).unwrap_err();
        assert!(error.to_string().contains("at least 1400"), "{error}");
        let people = with(&format!(
            "guid = \"{old_guid}\"\n\n[[episode.people]]\ncontributor = \"nobody\""
        ));
        let error = write_page_text(root, &first.file, &people, &stamp).unwrap_err();
        assert!(error.to_string().contains("no contributor"), "{error}");

        // The old feed's guid is taken, but not twice.
        let page = write_page_text(
            root,
            &first.file,
            &with(&format!("guid = \"{old_guid}\"")),
            &stamp,
        )
        .unwrap();
        assert_eq!(page.content.front.episode.unwrap().guid, old_guid);
        let (text, stamp) = page_text(root, &second.file).unwrap();
        let guid_line = text.lines().find(|l| l.starts_with("guid")).unwrap();
        let error = write_page_text(
            root,
            &second.file,
            &text.replace(guid_line, &format!("guid = \"{old_guid}\"")),
            &stamp,
        )
        .unwrap_err();
        assert!(error.to_string().contains("has the guid"), "{error}");

        // Without a guid, it gets a new one.
        let page =
            write_page_text(root, &second.file, &text.replace(guid_line, ""), &stamp).unwrap();
        assert!(!page.content.front.episode.unwrap().guid.is_empty());
    }
}

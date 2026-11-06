//! The whole site as a PDF (#154), made on request in the app.
//!
//! The pages become [Typst](https://typst.app) markup, and Typst, compiled
//! into q.e.d., sets it as a book: a title page, a table of contents of the
//! top two levels of pages, the pages in the order of the site's menu,
//! then the posts and episodes, each group in a chapter of its own after
//! the page they're listed on. Plain black text on white: the site's
//! theme, fonts and logo stay on the website. A site in several languages
//! gets one PDF per language. The PDFs land in `assets/`, like any file
//! to download, so a page can link to them.
//!
//! Blocks keep what they say: columns stay side by side, a call to action
//! gets a frame, FAQ answers are open, and what only works on a screen (the
//! player, the subscribe button, videos) becomes a link. Links between pages
//! jump inside the PDF; links to other sites show their address in a
//! footnote, so it can be read on paper.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use comrak::nodes::{AstNode, ListType, NodeValue, TableAlignment};
use comrak::{Arena, parse_document};
use serde::Serialize;
use typst::diag::{FileError, FileResult, SourceDiagnostic};
use typst::foundations::{Bytes, Datetime, Smart};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

use crate::block::{self, Block, ListLayout};
use crate::calendar::{self, Date};
use crate::icons::{SITE_ICONS_DIR, SiteIcons};
use crate::links::LinkContext;
use crate::podcast;
use crate::site::{ASSETS_DIR, MEDIA_DIR, Page, Site};

/// The page the generator shows for unknown addresses; not part of the book.
const NOT_FOUND_PATH: &str = "404";

/// Photos are made smaller for the PDF: this many pixels on the long
/// side are sharp in print at the width of the page.
const IMAGE_MAX_PIXELS: u32 = 1600;

/// A table longer than this many rows is set as several (see
/// [`Text::block`]).
const TABLE_ROWS: usize = 30;

/// One PDF that was made.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PdfFile {
    /// The file name in `assets/`.
    pub name: String,
    /// The language it's in, `None` for the main one.
    pub language: Option<String>,
    pub pages: u32,
    /// Its size in bytes.
    pub size: u32,
}

/// What [`export`] did: the PDFs it wrote, and what didn't make it into
/// them.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PdfReport {
    pub files: Vec<PdfFile>,
    pub notes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("can't write {}: {source}", path.display())]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Typst refused the markup made from the pages. That's a mistake in
    /// q.e.d., not in the site, so the message is for developers.
    #[error("Typst couldn't set the PDF: {0}")]
    Typst(String),
}

/// Writes the site as one PDF per language into `assets/`, replacing those
/// of an earlier run.
pub fn export(site: &Site) -> Result<PdfReport, PdfError> {
    export_on(site, calendar::today())
}

fn export_on(site: &Site, today: Date) -> Result<PdfReport, PdfError> {
    let mut notes = Vec::new();
    // Shared blocks become their pieces' blocks, as in the generator.
    let expanded = crate::shared::expanded(site, &mut notes);
    let site = &expanded;
    let folder = site.root.join(ASSETS_DIR);
    std::fs::create_dir_all(&folder).map_err(|source| PdfError::Write {
        path: folder.clone(),
        source,
    })?;
    let mut files = Vec::new();
    for language in languages(site) {
        let book = Book::new(site, language, today);
        if book.pages.is_empty() {
            continue;
        }
        let (source, sources, book_notes) = book.typst();
        for note in book_notes {
            if !notes.contains(&note) {
                notes.push(note);
            }
        }
        let (pdf, pages) = compile(source, sources, today)?;
        let name = file_name(site, language);
        let path = folder.join(&name);
        std::fs::write(&path, &pdf).map_err(|source| PdfError::Write { path, source })?;
        files.push(PdfFile {
            name,
            language: language.map(str::to_string),
            pages,
            size: u32::try_from(pdf.len()).unwrap_or(u32::MAX),
        });
    }
    Ok(PdfReport { files, notes })
}

/// The Typst markup of the book in `language` (`None`: the main one),
/// as [`export`] sets it; for looking into how a page turned out.
pub fn markup(site: &Site, language: Option<&str>) -> String {
    let mut notes = Vec::new();
    let expanded = crate::shared::expanded(site, &mut notes);
    Book::new(&expanded, language, calendar::today()).typst().0
}

/// The site's languages: the main one (`None`) first, then the further
/// ones that have pages.
fn languages(site: &Site) -> Vec<Option<&str>> {
    let mut languages = vec![None];
    for code in site.config.languages.keys() {
        if site
            .pages
            .iter()
            .any(|page| page.language.as_deref() == Some(code.as_str()))
        {
            languages.push(Some(code.as_str()));
        }
    }
    languages
}

/// `my-podcast.pdf`, after the site's title; `my-podcast-en.pdf` in a
/// further language.
pub fn file_name(site: &Site, language: Option<&str>) -> String {
    let mut stem = crate::slug::slugify(site.config.title_in(None));
    if stem.is_empty() {
        stem = "site".to_string();
    }
    match language {
        None => format!("{stem}.pdf"),
        Some(code) => format!("{stem}-{code}.pdf"),
    }
}

/// The words the PDF needs beyond those of the pages.
struct Words {
    contents: &'static str,
    as_of: &'static str,
    posts: &'static str,
    listen: &'static str,
    latest_episode: &'static str,
    feeds: &'static str,
}

impl Words {
    fn for_language(language: &str) -> Words {
        if language.trim().to_lowercase().starts_with("de") {
            Words {
                contents: "Inhalt",
                as_of: "Stand:",
                posts: "Beiträge",
                listen: "Anhören:",
                latest_episode: "Neueste Episode:",
                feeds: "Podcast abonnieren:",
            }
        } else {
            Words {
                contents: "Contents",
                as_of: "As of",
                posts: "Posts",
                listen: "Listen:",
                latest_episode: "Latest episode:",
                feeds: "Subscribe to the podcast:",
            }
        }
    }
}

/// One language's pages, on their way to Typst markup.
struct Book<'a> {
    site: &'a Site,
    language: Option<&'a str>,
    /// The language code the text is in (`de`, `en-US`).
    code: &'a str,
    today: Date,
    /// The pages in the book: published, in this language, no posts from
    /// the future.
    pages: Vec<&'a Page>,
    /// Every published page's address, in all languages, for links.
    all_paths: Arc<BTreeSet<String>>,
    /// The label of each page in the book, by address: `p-3`.
    labels: HashMap<String, String>,
    /// The files Typst reads besides the markup: pictures, by their path
    /// in the book (`/images/3.jpg`).
    files: BTreeMap<String, Bytes>,
    /// Pictures already in `files`: by their file in the site, the path in
    /// the book and their width in pixels.
    images: HashMap<PathBuf, Option<(String, u32)>>,
    /// Formulas already checked, with their Typst math or `None`.
    formulas: HashMap<(String, bool), Option<String>>,
    site_icons: SiteIcons,
    notes: Vec<String>,
    words: Words,
    podcast_words: podcast::Words,
}

/// One part of the book, in order.
enum Entry<'a> {
    /// A page, with the level of its heading.
    Page(&'a Page, usize),
    /// A chapter heading without a page: the posts right below the start
    /// page.
    Chapter(&'static str),
}

/// Where a block is: the page it's on and how deep its headings start.
struct Spot<'a> {
    links: &'a LinkContext,
    /// The level of the page's own heading; headings in its text go below.
    level: usize,
    /// In a column or beside a picture: pictures fill their room.
    narrow: bool,
}

impl<'a> Book<'a> {
    fn new(site: &'a Site, language: Option<&'a str>, today: Date) -> Self {
        let published: Vec<&Page> = site
            .pages
            .iter()
            .filter(|page| !page.content.front.draft && page.path != NOT_FOUND_PATH)
            .collect();
        let all_paths = Arc::new(published.iter().map(|page| page.path.clone()).collect());
        let pages: Vec<&Page> = published
            .into_iter()
            .filter(|page| page.language.as_deref() == language)
            .filter(|page| page.content.front.pdf != Some(false))
            .filter(|page| {
                page.content
                    .front
                    .date
                    .as_ref()
                    .and_then(|date| date.sort_key())
                    .is_none_or(|(day, _)| day <= today)
            })
            .collect();
        let code = language.unwrap_or(&site.config.language);
        Book {
            site,
            language,
            code,
            today,
            pages,
            all_paths,
            labels: HashMap::new(),
            files: BTreeMap::new(),
            images: HashMap::new(),
            formulas: HashMap::new(),
            site_icons: SiteIcons::load(&site.root.join(MEDIA_DIR)),
            notes: Vec::new(),
            words: Words::for_language(code),
            podcast_words: podcast::Words::for_language(&site.config.language),
        }
    }

    /// The book as Typst markup, the files it reads, and notes on what was
    /// left out.
    fn typst(mut self) -> (String, BTreeMap<String, Bytes>, Vec<String>) {
        let order = self.order();
        for (index, entry) in order.iter().enumerate() {
            if let Entry::Page(page, _) = entry {
                self.labels.insert(page.path.clone(), format!("p-{index}"));
            }
        }
        let mut out = self.preamble();
        out.push_str(&self.title_page());
        for entry in &order {
            match entry {
                Entry::Page(page, level) => {
                    let page = self.page(page, *level);
                    out.push_str(&page);
                }
                Entry::Chapter(title) => {
                    out.push_str(&format!("#heading(level: 1)[{}]\n\n", escape(title)));
                }
            }
        }
        (out, self.files, self.notes)
    }

    /// The pages in the order of the book, each with its heading level:
    /// the start page, the tree of pages by weight and title, then a
    /// chapter for each page that has posts below it, with its own pages,
    /// then its posts, oldest first.
    fn order(&self) -> Vec<Entry<'a>> {
        let main = |page: &Page| page.main_path().to_string();
        let (posts, others): (Vec<&Page>, Vec<&Page>) = self
            .pages
            .iter()
            .partition(|page| page.content.front.date.is_some());
        let exists: BTreeSet<String> = others.iter().map(|page| main(page)).collect();
        // The nearest page above, by address; `""` for the start page.
        let above = |path: &str| -> String {
            let mut up = path;
            while let Some((parent, _)) = up.rsplit_once('/') {
                if exists.contains(parent) {
                    return parent.to_string();
                }
                up = parent;
            }
            String::new()
        };
        // The page a post is listed on: above any folders named by a date.
        let mut posts_of: BTreeMap<String, Vec<&Page>> = BTreeMap::new();
        for post in posts {
            let path = main(post);
            let mut parent = crate::generator::feed_parent(&path).to_string();
            while !parent.is_empty() && !exists.contains(&parent) {
                parent = parent.rsplit_once('/').map_or("", |(up, _)| up).to_string();
            }
            posts_of.entry(parent).or_default().push(post);
        }
        // Oldest first: a book is read forward, unlike the website's
        // lists, which start with the newest.
        for list in posts_of.values_mut() {
            list.sort_by(|a, b| {
                let key = |page: &Page| page.content.front.date.as_ref().and_then(|d| d.sort_key());
                key(a)
                    .cmp(&key(b))
                    .then_with(|| a.content.front.weight.cmp(&b.content.front.weight))
                    .then_with(|| podcast::same_day_order(a).cmp(&podcast::same_day_order(b)))
                    .then_with(|| a.path.cmp(&b.path))
            });
        }
        let mut children: BTreeMap<String, Vec<&Page>> = BTreeMap::new();
        for page in &others {
            let path = main(page);
            if !path.is_empty() {
                children.entry(above(&path)).or_default().push(page);
            }
        }
        for list in children.values_mut() {
            list.sort_by(|a, b| {
                a.content
                    .front
                    .weight
                    .cmp(&b.content.front.weight)
                    .then_with(|| {
                        crate::slug::slugify(&a.content.front.title)
                            .cmp(&crate::slug::slugify(&b.content.front.title))
                    })
                    .then_with(|| a.path.cmp(&b.path))
            });
        }

        let mut order = Vec::new();
        let mut chapters: Vec<&Page> = Vec::new();
        if let Some(start) = others.iter().find(|page| main(page).is_empty()) {
            order.push((*start, 1));
        }
        fn tree<'p>(
            parent: &str,
            level: usize,
            children: &BTreeMap<String, Vec<&'p Page>>,
            posts_of: &BTreeMap<String, Vec<&'p Page>>,
            order: &mut Vec<(&'p Page, usize)>,
            chapters: &mut Vec<&'p Page>,
        ) {
            for page in children.get(parent).into_iter().flatten() {
                let path = page.main_path();
                // A page with posts below it opens a chapter of its own
                // after the pages.
                if posts_of.contains_key(path) {
                    chapters.push(page);
                    continue;
                }
                order.push((page, level));
                tree(path, level + 1, children, posts_of, order, chapters);
            }
        }
        tree("", 1, &children, &posts_of, &mut order, &mut chapters);
        let mut chapter_order = Vec::new();
        let mut i = 0;
        // Chapters found inside a chapter's own pages come after it.
        while i < chapters.len() {
            let page = chapters[i];
            chapter_order.push((page, 1));
            let mut nested = Vec::new();
            tree(
                page.main_path(),
                2,
                &children,
                &posts_of,
                &mut chapter_order,
                &mut nested,
            );
            chapters.extend(nested);
            for post in posts_of.get(page.main_path()).into_iter().flatten() {
                chapter_order.push((post, 2));
            }
            i += 1;
        }
        order.extend(chapter_order);
        let mut entries: Vec<Entry> = order
            .into_iter()
            .map(|(page, level)| Entry::Page(page, level))
            .collect();
        // Posts right below the start page: a chapter without a page.
        if let Some(posts) = posts_of.get("") {
            entries.push(Entry::Chapter(self.words.posts));
            entries.extend(posts.iter().map(|post| Entry::Page(post, 2)));
        }
        entries
    }

    /// The settings for the whole book: A4, a serif font, numbered pages,
    /// each top-level page on a new page; and the helpers formulas need
    /// (see [`Book::formula`]).
    fn preamble(&self) -> String {
        let config = &self.site.config;
        let title = config.title_in(self.language);
        let mut out = String::new();
        out.push_str(&format!(
            "#set document(title: {}{})\n",
            string(title),
            config
                .description_in(self.language)
                .map(|text| format!(", description: {}", string(text)))
                .unwrap_or_default()
        ));
        out.push_str(&text_language(self.code));
        out.push_str(PREAMBLE);
        out.push_str(MATH_HELPERS);
        out
    }

    /// The title page and the table of contents.
    fn title_page(&self) -> String {
        let config = &self.site.config;
        let mut out = String::from("#page(numbering: none)[\n#v(25%)\n");
        out.push_str(&format!(
            "#text(size: 28pt, weight: \"bold\")[{}]\n\n",
            escape(config.title_in(self.language))
        ));
        if let Some(description) = config.description_in(self.language) {
            out.push_str(&format!("#text(size: 14pt)[{}]\n\n", escape(description)));
        }
        out.push_str("#v(1fr)\n");
        if let Some(base) = &config.base_url {
            out.push_str(&format!("#link({})\n\n", string(&self.web(""))));
            let _ = base;
        }
        out.push_str(&format!(
            "{} {}\n]\n\n",
            escape(self.words.as_of),
            escape(&calendar::long_date(self.today, self.code))
        ));
        out.push_str(&format!(
            "#outline(title: [{}], depth: 2)\n\n",
            escape(self.words.contents)
        ));
        out
    }

    /// The full address of a page or file on the website, if the site has
    /// one: `https://example.com/about/`.
    fn web(&self, path: &str) -> String {
        let base = self.site.config.base_url.as_deref().unwrap_or_default();
        let base = base.trim_end_matches('/');
        let path = path.trim_matches('/');
        if path.is_empty() {
            format!("{base}/")
        } else if crate::links::looks_like_file(path) {
            format!("{base}/{path}")
        } else {
            format!("{base}/{path}/")
        }
    }

    /// A page: its heading, the line with date and authors, its blocks.
    fn page(&mut self, page: &'a Page, level: usize) -> String {
        let front = &page.content.front;
        let label = self.labels.get(&page.path).cloned().unwrap_or_default();
        let mut out = format!(
            "#heading(level: {level})[{}] <{label}>\n\n",
            escape(&front.title)
        );
        let mut byline: Vec<String> = Vec::new();
        byline.extend(
            front
                .episode
                .as_ref()
                .map(|episode| podcast::label(episode, &self.podcast_words))
                .filter(|label| !label.is_empty()),
        );
        if let Some(date) = &front.date {
            byline.push(date.date().map_or_else(
                || date.to_string(),
                |day| calendar::long_date(day, self.code),
            ));
        }
        if !front.authors.is_empty() {
            byline.push(front.authors.join(", "));
        }
        if !byline.is_empty() {
            out.push_str(&format!(
                "#text(size: 9pt, style: \"italic\")[{}]\n\n",
                escape(&byline.join(" · "))
            ));
        }
        let links = LinkContext::new(&page.path, &page.file)
            .with_own_site(self.site.config.base_url.as_deref())
            .in_language(page.language.as_deref(), self.all_paths.clone());
        let spot = Spot {
            links: &links,
            level,
            narrow: false,
        };
        match &front.episode {
            None => out.push_str(&self.blocks(&page.content.blocks, &spot)),
            Some(episode) => {
                let layout = self
                    .site
                    .episode_layout
                    .as_ref()
                    .map_or(&block::DEFAULT_EPISODE_LAYOUT[..], |layout| {
                        &layout.content.blocks
                    });
                for block in layout {
                    let part = match block {
                        Block::EpisodeShownotes => self.blocks(&page.content.blocks, &spot),
                        block if block.is_episode_part() => {
                            self.episode_part(block, page, episode, &spot)
                        }
                        block => self.block(block, &spot),
                    };
                    out.push_str(&part);
                    out.push_str("\n\n");
                }
            }
        }
        out.push_str("\n\n");
        out
    }

    fn blocks(&mut self, blocks: &[Block], spot: &Spot) -> String {
        let mut out = String::new();
        for block in blocks {
            let text = self.block(block, spot);
            if !text.trim().is_empty() {
                out.push_str(&text);
                out.push_str("\n\n");
            }
        }
        out
    }

    /// One block, as it reads on paper.
    fn block(&mut self, block: &Block, spot: &Spot) -> String {
        match block {
            Block::Text { markdown } => self.markdown(markdown, spot),
            Block::Hero(hero) => {
                let mut out = String::new();
                if let Some(image) = hero.image.as_deref() {
                    out.push_str(&self.block_image(image, None, spot, true));
                }
                if let Some(headline) = hero.headline.as_deref().filter(|h| !h.trim().is_empty()) {
                    out.push_str(&format!(
                        "#text(size: 18pt, weight: \"bold\")[{}]\n\n",
                        escape(headline)
                    ));
                }
                if let Some(text) = &hero.text {
                    out.push_str(&self.markdown(text, spot));
                    out.push_str("\n\n");
                }
                out.push_str(&self.button(&hero.button_label, &hero.button_link, spot));
                out
            }
            Block::Image(image) => match image.image.as_deref() {
                Some(file) => {
                    let mut out = self.block_image(file, image.alt.as_deref(), spot, false);
                    if let Some(caption) = image.caption.as_deref().filter(|c| !c.trim().is_empty())
                    {
                        let caption = self.inline_markdown(caption, spot);
                        out = format!(
                            "#figure({}, caption: [{caption}], supplement: none, numbering: none)\n\n",
                            out.trim().trim_start_matches('#')
                        );
                    }
                    out
                }
                None => String::new(),
            },
            Block::TextImage(text_image) => {
                let narrow = Spot {
                    links: spot.links,
                    level: spot.level,
                    narrow: true,
                };
                let text = text_image
                    .text
                    .as_deref()
                    .map(|text| self.markdown(text, &narrow))
                    .unwrap_or_default();
                let image = text_image
                    .image
                    .as_deref()
                    .map(|image| self.block_image(image, text_image.alt.as_deref(), &narrow, false))
                    .unwrap_or_default();
                let (left, right) = match text_image.image_side {
                    block::Side::Left => (image, text),
                    block::Side::Right => (text, image),
                };
                format!(
                    "#grid(columns: (1fr, 1fr), column-gutter: 1.5em, [\n{left}\n], [\n{right}\n])\n"
                )
            }
            Block::Gallery(gallery) => {
                let narrow = Spot {
                    links: spot.links,
                    level: spot.level,
                    narrow: true,
                };
                let cells: Vec<String> = gallery
                    .images
                    .iter()
                    .map(|image| {
                        format!(
                            "[{}]",
                            self.block_image(&image.image, image.alt.as_deref(), &narrow, false)
                        )
                    })
                    .collect();
                if cells.is_empty() {
                    return String::new();
                }
                format!(
                    "#grid(columns: (1fr, 1fr, 1fr), gutter: 0.8em, {})\n",
                    cells.join(", ")
                )
            }
            Block::Download(download) => {
                let Some(file) = download.file.as_deref().filter(|f| !f.trim().is_empty()) else {
                    return String::new();
                };
                let label = download
                    .label
                    .as_deref()
                    .filter(|l| !l.trim().is_empty())
                    .unwrap_or(file);
                let in_assets = self.site.root.join(ASSETS_DIR).join(file).is_file()
                    || !self.site.root.join(MEDIA_DIR).join(file).is_file();
                let folder = if in_assets { ASSETS_DIR } else { MEDIA_DIR };
                let text = format!("↓ {}", escape(label));
                self.web_link(&format!("{folder}/{file}"), &text, true)
            }
            Block::Faq(faq) => {
                let mut out = String::new();
                if let Some(heading) = faq.heading.as_deref().filter(|h| !h.trim().is_empty()) {
                    out.push_str(&self.inner_heading(heading, 1, spot));
                }
                for item in &faq.items {
                    out.push_str(&format!("*{}*\n\n", escape(&item.question)));
                    out.push_str(&self.markdown(&item.answer, spot));
                    out.push_str("\n\n");
                }
                out
            }
            Block::Columns(columns) => {
                let narrow = Spot {
                    links: spot.links,
                    level: spot.level,
                    narrow: true,
                };
                let widths: Vec<String> = columns
                    .items
                    .iter()
                    .map(|column| format!("{}fr", column.width.unwrap_or(1).max(1)))
                    .collect();
                let cells: Vec<String> = columns
                    .items
                    .iter()
                    .map(|column| {
                        let mut inner = String::new();
                        if let Some(title) = column.title.as_deref().filter(|t| !t.trim().is_empty()) {
                            inner.push_str(&format!("*{}*\n\n", escape(title)));
                        }
                        inner.push_str(&self.blocks(&column.blocks, &narrow));
                        if column.boxed {
                            format!(
                                "[#block(stroke: 0.5pt, inset: 8pt, radius: 3pt, width: 100%)[\n{inner}\n]]"
                            )
                        } else {
                            format!("[\n{inner}\n]")
                        }
                    })
                    .collect();
                if cells.is_empty() {
                    return String::new();
                }
                format!(
                    "#grid(columns: ({},), column-gutter: 1.5em, {})\n",
                    widths.join(", "),
                    cells.join(", ")
                )
            }
            Block::Section(section) => {
                let mut out = String::new();
                if let Some(title) = section.title.as_deref().filter(|t| !t.trim().is_empty()) {
                    out.push_str(&self.inner_heading(title, 1, spot));
                }
                out.push_str(&self.blocks(&section.blocks, spot));
                out
            }
            Block::CallToAction(cta) => {
                let mut inner = String::new();
                if let Some(headline) = cta.headline.as_deref().filter(|h| !h.trim().is_empty()) {
                    inner.push_str(&format!(
                        "#text(size: 13pt, weight: \"bold\")[{}]\n\n",
                        escape(headline)
                    ));
                }
                if let Some(text) = &cta.text {
                    inner.push_str(&self.markdown(text, spot));
                    inner.push_str("\n\n");
                }
                inner.push_str(&self.button(&cta.button_label, &cta.button_link, spot));
                format!(
                    "#block(stroke: 0.75pt, inset: 12pt, radius: 4pt, width: 100%)[\n{inner}\n]\n"
                )
            }
            Block::PostList(list) => {
                let from = list_source(&list.from, spot.links);
                let mut posts: Vec<&Page> = self
                    .pages
                    .iter()
                    .copied()
                    .filter(|page| page.content.front.date.is_some())
                    .filter(|page| {
                        crate::generator::feed_parent(&page.path) == from
                            || (!from.is_empty() && page.path.starts_with(&format!("{from}/")))
                            || from.is_empty()
                    })
                    .filter(|page| match &list.contributor {
                        None => true,
                        Some(id) => page.content.front.episode.as_ref().is_some_and(|episode| {
                            episode
                                .people
                                .iter()
                                .any(|person| &person.contributor == id)
                        }),
                    })
                    .collect();
                posts.sort_by(|a, b| {
                    let key =
                        |page: &Page| page.content.front.date.as_ref().and_then(|d| d.sort_key());
                    key(b).cmp(&key(a)).then_with(|| a.path.cmp(&b.path))
                });
                if let Some(limit) = list.limit {
                    posts.truncate(limit as usize);
                }
                let items: Vec<String> = posts
                    .iter()
                    .map(|post| {
                        let date = post
                            .content
                            .front
                            .date
                            .as_ref()
                            .and_then(|date| date.date())
                            .map(|day| {
                                format!("{} — ", escape(&calendar::long_date(day, self.code)))
                            })
                            .unwrap_or_default();
                        format!("[{date}{}]", self.page_link(post))
                    })
                    .collect();
                list_of(&items, list.layout)
            }
            Block::PageList(list) => {
                let from = list_source(&list.from, spot.links).to_string();
                let pages: Vec<&Page> = crate::generator::children(&self.pages, &from);
                let items: Vec<String> = pages
                    .iter()
                    .map(|page| {
                        let summary = page
                            .content
                            .front
                            .summary
                            .as_deref()
                            .map(|summary| format!(": {}", self.inline_markdown(summary, spot)))
                            .unwrap_or_default();
                        format!("[{}{summary}]", self.page_link(page))
                    })
                    .collect();
                list_of(&items, list.layout)
            }
            Block::Video(video) => {
                let mut out = String::new();
                if let Some(image) = video.image.as_deref() {
                    out.push_str(&self.block_image(image, video.title.as_deref(), spot, false));
                }
                let title = video
                    .title
                    .as_deref()
                    .filter(|t| !t.trim().is_empty())
                    .unwrap_or("Video");
                let text = format!("▶ {}", escape(title));
                match (&video.file, &video.url) {
                    (Some(file), _) if !file.trim().is_empty() => out.push_str(&self.web_link(
                        &format!("{ASSETS_DIR}/{}", file.trim()),
                        &text,
                        true,
                    )),
                    (_, Some(url)) if !url.trim().is_empty() => {
                        out.push_str(&external_link(url.trim(), &text, true))
                    }
                    _ => out.push_str(&text),
                }
                out.push_str("\n\n");
                out
            }
            Block::Shared(_) => String::new(),
            Block::LatestEpisode => {
                let latest = self
                    .pages
                    .iter()
                    .copied()
                    .filter(|page| page.content.front.episode.is_some())
                    .max_by_key(|page| page.content.front.date.as_ref().and_then(|d| d.sort_key()));
                match latest {
                    Some(page) => format!(
                        "{} {}\n\n",
                        escape(self.words.latest_episode),
                        self.page_link(page)
                    ),
                    None => String::new(),
                }
            }
            Block::Subscribe(_) => {
                let (Some(podcast), Some(_)) =
                    (&self.site.config.podcast, &self.site.config.base_url)
                else {
                    return String::new();
                };
                let feeds: Vec<String> = podcast
                    .formats()
                    .into_iter()
                    .map(|format| {
                        let url = self.web(&podcast::feed_file(format));
                        format!("[#link({})]", string(&url))
                    })
                    .collect();
                format!(
                    "{}\n#list({})\n",
                    escape(self.words.feeds),
                    feeds.join(", ")
                )
            }
            Block::Contributors => {
                let names: Vec<String> = self
                    .site
                    .contributors
                    .iter()
                    .map(|person| escape(&person.name))
                    .collect();
                names.join(", ")
            }
            // The parts of an episode show on episode pages only.
            _ => String::new(),
        }
    }

    /// A part of an episode page (see the episode layout): the player as a
    /// link, the people, the chapters, the transcript.
    fn episode_part(
        &mut self,
        block: &Block,
        page: &Page,
        episode: &podcast::Episode,
        spot: &Spot,
    ) -> String {
        let language = &self.site.config.language;
        match block {
            Block::EpisodePlayer => {
                let duration = episode
                    .duration
                    .as_deref()
                    .map(|d| {
                        let readable = podcast::readable_start(d);
                        format!(" ({})", readable.strip_prefix("00:").unwrap_or(&readable))
                    })
                    .unwrap_or_default();
                if self.site.config.base_url.is_some() {
                    let url = self.web(&page.path);
                    format!(
                        "{} #link({})[{}]{}\n\n",
                        escape(self.words.listen),
                        string(&url),
                        escape(&url),
                        escape(&duration)
                    )
                } else if let Some(audio) = episode.audio.first() {
                    let file = audio.file.trim();
                    format!(
                        "{} {}{}\n\n",
                        escape(self.words.listen),
                        escape(file.rsplit('/').next().unwrap_or(file)),
                        escape(&duration)
                    )
                } else {
                    String::new()
                }
            }
            Block::EpisodePeople => {
                let mut order: Vec<&str> = Vec::new();
                for person in &episode.people {
                    if !order.contains(&person.contributor.as_str()) {
                        order.push(&person.contributor);
                    }
                }
                let people: Vec<String> = order
                    .into_iter()
                    .filter_map(|id| {
                        let person = self.site.contributors.iter().find(|c| c.id == id)?;
                        let roles: Vec<&str> = episode
                            .people
                            .iter()
                            .filter(|p| p.contributor == id)
                            .map(|p| p.role.label(language))
                            .collect();
                        Some(format!(
                            "{} ({})",
                            escape(&person.name),
                            escape(&roles.join(", "))
                        ))
                    })
                    .collect();
                if people.is_empty() {
                    return String::new();
                }
                format!(
                    "{}{}\n\n",
                    self.inner_heading(self.podcast_words.contributors, 1, spot),
                    people.join(", ")
                )
            }
            Block::EpisodeChapters => {
                if episode.chapters.is_empty() {
                    return String::new();
                }
                let rows: Vec<String> = episode
                    .chapters
                    .iter()
                    .map(|chapter| {
                        let start = podcast::readable_start(&chapter.start);
                        let title = match chapter.link.as_deref().map(str::trim) {
                            Some(link) if !link.is_empty() => {
                                self.link(link, &escape(&chapter.title), spot.links)
                            }
                            _ => escape(&chapter.title),
                        };
                        format!("[{}], [{title}]", escape(&start))
                    })
                    .collect();
                format!(
                    "{}#table(columns: (auto, 1fr), stroke: none, inset: (x: 0pt, y: 3pt), column-gutter: 1em, {})\n\n",
                    self.inner_heading(self.podcast_words.chapters, 1, spot),
                    rows.join(", ")
                )
            }
            Block::EpisodeTranscript => {
                let paragraphs =
                    crate::generator::paragraphs(&crate::generator::transcript(self.site, episode));
                if paragraphs.is_empty() {
                    return String::new();
                }
                let mut out = self.inner_heading(self.podcast_words.transcript, 1, spot);
                out.push_str("#[\n#set text(size: 9.5pt)\n");
                for paragraph in paragraphs {
                    let voice = paragraph
                        .voice
                        .as_deref()
                        .map(|voice| format!("*{}:* ", escape(voice)))
                        .unwrap_or_default();
                    out.push_str(&format!(
                        "#text(size: 8pt)[{}] {voice}{}\n\n",
                        escape(&paragraph.start),
                        escape(&paragraph.text)
                    ));
                }
                out.push_str("]\n");
                out
            }
            _ => String::new(),
        }
    }

    /// A heading inside a page: below the page's own, not in the table of
    /// contents. `depth` is its level inside the page (1 for `#`).
    fn inner_heading(&self, text: &str, depth: usize, spot: &Spot) -> String {
        heading_markup(&escape(text), depth, spot.level)
    }

    /// A button: its label, linked to where it leads.
    fn button(&mut self, label: &Option<String>, link: &Option<String>, spot: &Spot) -> String {
        let label = label.as_deref().map(str::trim).filter(|l| !l.is_empty());
        let link = link.as_deref().map(str::trim).filter(|l| !l.is_empty());
        match (label, link) {
            (Some(label), Some(link)) => {
                format!(
                    "{}\n\n",
                    self.link(link, &format!("→ {}", escape(label)), spot.links)
                )
            }
            (Some(label), None) => format!("→ {}\n\n", escape(label)),
            _ => String::new(),
        }
    }

    /// A link to a page in the book, with its title.
    fn page_link(&self, page: &Page) -> String {
        let title = escape(&page.content.front.title);
        match self.labels.get(&page.path) {
            Some(label) => format!("#link(<{label}>)[{title}]"),
            None => title,
        }
    }

    /// `text` linked to a file of the website (`assets/menu.pdf`) when the
    /// site has a web address; plain text otherwise.
    fn web_link(&self, path: &str, text: &str, footnote: bool) -> String {
        if self.site.config.base_url.is_some() {
            let url = self.web(path);
            if !footnote {
                return format!("#link({})[{text}]", string(&url));
            }
            format!("#link({})[{text}]#footnote[{}]", string(&url), escape(&url))
        } else {
            text.to_string()
        }
    }

    /// `text` (markup) linked to where `url`, as written in a page, leads:
    /// a page in the book, a page or file of the website, another site.
    fn link(&self, url: &str, text: &str, links: &LinkContext) -> String {
        self.link_with(url, text, links, true)
    }

    /// [`Book::link`], with or without a footnote for addresses outside
    /// the book. Tables go without: hundreds of footnotes take Typst long,
    /// and in a table they're hard to read.
    fn link_with(&self, url: &str, text: &str, links: &LinkContext, footnote: bool) -> String {
        // Full links to the site itself are links inside it.
        let url = match self.site.config.base_url.as_deref() {
            Some(base) if url.starts_with(base.trim_end_matches('/')) => {
                let rest = &url[base.trim_end_matches('/').len()..];
                if rest.is_empty() || rest.starts_with(['/', '#', '?']) {
                    format!("/{}", rest.trim_start_matches('/'))
                } else {
                    url.to_string()
                }
            }
            _ => url.to_string(),
        };
        if let Some(path) = links.page_path_of(&url) {
            let path = path.trim_matches('/').to_string();
            if let Some(label) = self.labels.get(&path) {
                return format!("#link(<{label}>)[{text}]");
            }
            return self.web_link(&path, text, footnote);
        }
        if url.starts_with('#') || url.is_empty() {
            return text.to_string();
        }
        if let Some(path) = url.strip_prefix('/') {
            let path = path.split(['#', '?']).next().unwrap_or_default();
            return self.web_link(path, text, footnote);
        }
        if url.contains(':') || url.starts_with("//") {
            return external_link(&url, text, footnote);
        }
        text.to_string()
    }
}

/// Settings for every page of the book.
const PREAMBLE: &str = r#"#set page(paper: "a4", margin: (x: 2.3cm, top: 2.5cm, bottom: 2.8cm), numbering: "1")
#set text(font: "Libertinus Serif", size: 11pt)
#set par(justify: true, leading: 0.62em, spacing: 1.1em)
#show raw: set text(font: "DejaVu Sans Mono", size: 0.82em)
#show raw.where(block: true): block.with(width: 100%, inset: 8pt, stroke: 0.5pt + luma(140), radius: 3pt)
#show math.equation: set text(font: "New Computer Modern Math")
#set heading(numbering: none)
#show heading: set block(above: 1.6em, below: 0.8em)
#show heading.where(outlined: true, level: 1): it => { pagebreak(weak: true); set text(size: 20pt); it }
#show heading.where(outlined: true, level: 2): set text(size: 15pt)
#show heading.where(outlined: true, level: 2): set block(above: 2.4em)
#show heading.where(outlined: true): it => { if it.level > 2 { set text(size: 13pt); it } else { it } }
#show outline.entry.where(level: 1): set text(weight: "bold")
#show outline.entry.where(level: 1): set block(above: 1.1em)
#set table(stroke: 0.5pt)
#set quote(block: true)
#show quote: set pad(x: 1.5em)
#show figure.caption: set text(size: 9.5pt)
#show link: it => if type(it.dest) == str { underline(stroke: 0.4pt, offset: 2pt, it) } else { it }
"#;

/// What LaTeX formulas need once mitex has turned them into Typst math:
/// the names it writes that Typst doesn't have itself.
const MATH_HELPERS: &str = r#"#let mitexsqrt(..args) = { let a = args.pos(); if a.len() > 1 { math.root(a.at(0), a.at(1)) } else { math.sqrt(a.at(0)) } }
#let mitexdisplay(it) = math.display(it)
#let mitexinline(it) = math.inline(it)
#let mitexbold(it) = math.bold(it)
#let mitexitalic(it) = math.italic(it)
#let mitexupright(it) = math.upright(it)
#let mitexsans(it) = math.sans(it)
#let mitexmono(it) = math.mono(it)
#let mitexfrak(it) = math.frak(it)
#let mitexcal(it) = math.cal(it)
#let mitexoverbrace(..args) = math.overbrace(..args)
#let mitexunderbrace(..args) = math.underbrace(..args)
#let mitexoverbracket(..args) = math.overbracket(..args)
#let mitexunderbracket(..args) = math.underbracket(..args)
#let mitexcolor(..args) = args.pos().last()
#let textmath(it) = it
#let pmatrix(..args) = math.mat(delim: "(", ..args)
#let bmatrix(..args) = math.mat(delim: "[", ..args)
#let Bmatrix(..args) = math.mat(delim: "{", ..args)
#let vmatrix(..args) = math.mat(delim: "|", ..args)
#let Vmatrix(..args) = math.mat(delim: "‖", ..args)
#let matrix(..args) = math.mat(delim: none, ..args)
#let smallmatrix(..args) = math.mat(delim: none, ..args)
#let xrightarrow(..args) = math.attach(math.stretch(sym.arrow.r, size: 100% + 2em), t: args.pos().last())
#let xleftarrow(..args) = math.attach(math.stretch(sym.arrow.l, size: 100% + 2em), t: args.pos().last())
#let overset(a, b) = math.attach(math.limits(b), t: a)
#let underset(a, b) = math.attach(math.limits(b), b: a)
"#;

/// `#set text(lang: "de", region: "AT")` for `de-AT`: hyphenation and
/// quotes in the page's language.
fn text_language(code: &str) -> String {
    let code = code.trim().replace('_', "-");
    let mut parts = code.split('-');
    let lang = parts.next().unwrap_or_default().to_ascii_lowercase();
    if !(2..=3).contains(&lang.len()) || !lang.bytes().all(|b| b.is_ascii_lowercase()) {
        return String::new();
    }
    let region = parts
        .next()
        .filter(|r| r.len() == 2 && r.bytes().all(|b| b.is_ascii_alphabetic()))
        .map(|r| format!(", region: \"{}\"", r.to_ascii_uppercase()))
        .unwrap_or_default();
    format!("#set text(lang: \"{lang}\"{region})\n")
}

/// A heading inside a page, `depth` levels below the page's heading at
/// `level`; the size follows the depth, not the page's level.
fn heading_markup(markup: &str, depth: usize, level: usize) -> String {
    let size = match depth {
        1 | 2 => "13pt",
        3 => "11.5pt",
        _ => "11pt",
    };
    format!(
        "#heading(level: {}, outlined: false)[#text(size: {size})[{markup}]]\n\n",
        level + depth
    )
}

/// The page a Post list or Page list lists from: its own, or another.
fn list_source<'s>(from: &'s Option<String>, links: &'s LinkContext) -> &'s str {
    match from.as_deref().map(|f| f.trim().trim_matches('/')) {
        Some(from) if !from.is_empty() => from,
        _ => links.page_path(),
    }
}

/// Entries of a Post list or Page list; cards and tiles are a list too.
fn list_of(items: &[String], _layout: Option<ListLayout>) -> String {
    if items.is_empty() {
        return String::new();
    }
    format!("#list({})\n", items.join(", "))
}

/// A link to another site, with its address in a footnote unless the
/// text says it already or there's no `footnote` (in a table).
fn external_link(url: &str, text: &str, footnote: bool) -> String {
    let shown = url.strip_prefix("mailto:").unwrap_or(url);
    if !footnote || text == escape(shown) || text == escape(url) {
        format!("#link({})[{text}]", string(url))
    } else {
        format!("#link({})[{text}]#footnote[{}]", string(url), escape(shown))
    }
}

/// Text for Typst markup: every character that means something there is
/// escaped with a backslash, and line breaks become spaces. `(` too: right
/// after `#strong[…]` it would continue the call.
pub(crate) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for c in text.chars() {
        match c {
            '\\' | '#' | '*' | '_' | '`' | '$' | '<' | '>' | '@' | '=' | '-' | '+' | '/' | '['
            | ']' | '~' | '\'' | '"' | '.' | ':' | '(' => {
                out.push('\\');
                out.push(c);
            }
            '\n' | '\r' => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}

/// A Typst string literal.
fn string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

impl<'a> Book<'a> {
    /// Markdown as Typst markup, paragraphs and all.
    fn markdown(&mut self, markdown: &str, spot: &Spot) -> String {
        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &crate::markdown::options());
        let mut footnotes = HashMap::new();
        for node in root.descendants() {
            if let NodeValue::FootnoteDefinition(definition) = &node.data.borrow().value {
                footnotes.insert(definition.name.clone(), node);
            }
        }
        let mut text = Text {
            book: self,
            spot,
            footnotes,
            in_table: false,
        };
        text.children(root)
    }

    /// Markdown that stands in a line, such as a caption: its paragraphs
    /// run together.
    fn inline_markdown(&mut self, markdown: &str, spot: &Spot) -> String {
        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &crate::markdown::options());
        let mut text = Text {
            book: self,
            spot,
            footnotes: HashMap::new(),
            in_table: false,
        };
        root.children()
            .map(|node| text.inlines(node))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// A picture from `media/` (`team.jpg` or `/media/team.jpg`) on a line
    /// of its own; `full` makes it as wide as the text.
    fn block_image(&mut self, image: &str, alt: Option<&str>, spot: &Spot, full: bool) -> String {
        match self.image_expr(image, alt, spot, full) {
            Some(expr) => format!("#align(center, {expr})\n\n"),
            None => String::new(),
        }
    }

    /// `image("/images/3.jpg", width: 100%)` for a picture field.
    fn image_expr(
        &mut self,
        image: &str,
        alt: Option<&str>,
        spot: &Spot,
        full: bool,
    ) -> Option<String> {
        let name = crate::generator::media_name(image.trim())?;
        let file = self.site.root.join(MEDIA_DIR).join(name);
        self.image_at(&file, alt, spot, full)
    }

    fn image_at(
        &mut self,
        file: &Path,
        alt: Option<&str>,
        spot: &Spot,
        full: bool,
    ) -> Option<String> {
        let (path, width) = self.image(file)?;
        let alt = alt
            .filter(|alt| !alt.trim().is_empty())
            .map(|alt| format!(", alt: {}", string(alt)))
            .unwrap_or_default();
        Some(format!(
            "image({}, width: {}{alt})",
            string(&path),
            print_width(width, full || spot.narrow)
        ))
    }

    /// A picture in the book: the path Typst reads it at and its width in
    /// pixels, if known. Photos larger than [`IMAGE_MAX_PIXELS`] get a
    /// smaller copy, so the PDF stays small.
    fn image(&mut self, file: &Path) -> Option<(String, Option<u32>)> {
        if let Some(found) = self.images.get(file) {
            return found
                .clone()
                .map(|(path, width)| (path, Some(width).filter(|w| *w > 0)));
        }
        let loaded = self.load_image(file);
        if loaded.is_none() {
            let shown = file.strip_prefix(&self.site.root).unwrap_or(file);
            self.notes.push(format!(
                "PDF: {} isn't a picture the PDF can show, so it's left out.",
                shown.display()
            ));
        }
        self.images.insert(file.to_path_buf(), loaded.clone());
        loaded.map(|(path, width)| (path, Some(width).filter(|w| *w > 0)))
    }

    fn load_image(&mut self, file: &Path) -> Option<(String, u32)> {
        let bytes = std::fs::read(file).ok()?;
        let extension = file
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        let index = self.files.len();
        if extension == "svg" {
            let width = svg_width(&String::from_utf8_lossy(&bytes)).unwrap_or(0);
            let path = format!("/images/{index}.svg");
            self.files.insert(path.clone(), Bytes::new(bytes));
            return Some((path, width));
        }
        if !matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp") {
            return None;
        }
        let picture = image::load_from_memory(&bytes).ok()?;
        let (width, height) = (picture.width(), picture.height());
        let large = width.max(height) > IMAGE_MAX_PIXELS;
        if !large && matches!(extension.as_str(), "jpg" | "jpeg" | "png") {
            let path = format!("/images/{index}.{extension}");
            self.files.insert(path.clone(), Bytes::new(bytes));
            return Some((path, width));
        }
        let picture = if large {
            picture.resize(
                IMAGE_MAX_PIXELS,
                IMAGE_MAX_PIXELS,
                image::imageops::FilterType::CatmullRom,
            )
        } else {
            picture
        };
        let mut out = Cursor::new(Vec::new());
        let path = if picture.color().has_alpha() {
            picture.write_to(&mut out, image::ImageFormat::Png).ok()?;
            format!("/images/{index}.png")
        } else {
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 85);
            picture.to_rgb8().write_with_encoder(encoder).ok()?;
            format!("/images/{index}.jpg")
        };
        self.files
            .insert(path.clone(), Bytes::new(out.into_inner()));
        Some((path, width))
    }

    /// A drawing made from text (a molecule, a chart, a stratigraphic
    /// column): the SVG in the generator's HTML, in black. If there's none
    /// (the text has a mistake), the text as code.
    fn drawing(&mut self, html: &str, source: &str) -> String {
        let (Some(start), Some(end)) = (html.find("<svg"), html.rfind("</svg>")) else {
            return format!("#raw(block: true, {})\n\n", string(source.trim_end()));
        };
        // `data-pagefind-ignore` (for the site search) is HTML, not SVG.
        let mut svg = html[start..end + "</svg>".len()]
            .replace("currentColor", "black")
            .replace(" data-pagefind-ignore", "");
        // The boxes behind a molecule's letters get their color from the
        // site's stylesheet; on paper they're white.
        if html.contains("class=\"molecule\"")
            && let Some(close) = svg.find('>')
        {
            svg.insert_str(close + 1, "<style>rect { fill: white; }</style>");
        }
        let width = svg_width(&svg);
        let path = format!("/images/{}.svg", self.files.len());
        self.files
            .insert(path.clone(), Bytes::new(svg.into_bytes()));
        format!(
            "#align(center, image({}, width: {}))\n\n",
            string(&path),
            print_width(width, false)
        )
    }

    /// A formula: LaTeX turned into Typst math by mitex. One Typst can't
    /// set shows as its LaTeX in code type, so it can't stop the book.
    fn formula(&mut self, latex: &str, display: bool) -> String {
        let key = (latex.to_string(), display);
        if !self.formulas.contains_key(&key) {
            let math = mitex::convert_math(&crate::chem::expand_ce(latex.trim()), None)
                .ok()
                .filter(|math| sets_math(math));
            if math.is_none() {
                self.notes.push(format!(
                    "PDF: the formula {} is shown as its LaTeX.",
                    latex.trim()
                ));
            }
            self.formulas.insert(key.clone(), math);
        }
        match &self.formulas[&key] {
            Some(math) if display => format!("$ {math} $"),
            Some(math) => format!("${math}$"),
            None => format!("#raw({})", string(latex.trim())),
        }
    }

    /// `:icon-phone:` as a picture the size of the text: the site's own
    /// from `media/icons/`, or q.e.d.'s.
    fn icon(&mut self, name: &str) -> Option<String> {
        if let Some(file) = self.site_icons.file(name) {
            let file = self
                .site
                .root
                .join(MEDIA_DIR)
                .join(SITE_ICONS_DIR)
                .join(file);
            let (path, _) = self.image(&file)?;
            return Some(format!(
                "#box(baseline: 15%, image({}, height: 1em))",
                string(&path)
            ));
        }
        let svg = crate::icons::svg(name)?;
        let path = format!("/icons/{name}.svg");
        if !self.files.contains_key(&path) {
            let svg = svg.replace("currentColor", "black");
            self.files
                .insert(path.clone(), Bytes::new(svg.into_bytes()));
        }
        Some(format!(
            "#box(baseline: 15%, image({}, height: 1em))",
            string(&path)
        ))
    }
}

/// How wide a picture `pixels` wide is printed: at 96 pixels an inch
/// like on a screen, but never wider than the text; as wide as its room
/// when `fill`.
fn print_width(pixels: Option<u32>, fill: bool) -> String {
    match pixels {
        Some(pixels) if !fill && f64::from(pixels) * 0.75 < 440.0 => {
            format!("{}pt", f64::from(pixels) * 0.75)
        }
        None if !fill => "auto".to_string(),
        _ => "100%".to_string(),
    }
}

/// The `width` an SVG gives itself, in pixels.
fn svg_width(svg: &str) -> Option<u32> {
    let tag = &svg[svg.find("<svg")?..];
    let tag = &tag[..tag.find('>')?];
    let at = tag.find(" width=\"")? + " width=\"".len();
    let value = &tag[at..];
    let value = &value[..value.find('"')?];
    let number: String = value
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let rest = &value[number.len()..];
    if !(rest.is_empty() || rest == "px") {
        return None;
    }
    number.parse::<f64>().ok().map(|n| n.round() as u32)
}

/// Turns one piece of Markdown into Typst markup, node by node.
struct Text<'b, 'a, 'n> {
    book: &'b mut Book<'a>,
    spot: &'b Spot<'b>,
    /// The footnotes' texts, by name; they go where they're referred to.
    footnotes: HashMap<String, &'n AstNode<'n>>,
    /// Inside a table, whose links get no footnotes.
    in_table: bool,
}

impl<'n> Text<'_, '_, 'n> {
    /// The blocks inside a node, a blank line apart.
    fn children(&mut self, node: &'n AstNode<'n>) -> String {
        let mut out = String::new();
        for child in node.children() {
            let block = self.block(child);
            if !block.trim().is_empty() {
                out.push_str(block.trim_end());
                out.push_str("\n\n");
            }
        }
        out
    }

    fn block(&mut self, node: &'n AstNode<'n>) -> String {
        let value = node.data.borrow().value.clone();
        match value {
            NodeValue::Paragraph => {
                // A picture alone in its paragraph stands on its own line.
                let mut children = node.children().filter(|child| !is_blank(child));
                if let (Some(only), None) = (children.next(), children.next())
                    && let NodeValue::Image(image) = &only.data.borrow().value
                    && let Some(file) = self.picture_file(&image.url)
                {
                    let alt = plain_text(only);
                    if let Some(expr) = self.book.image_at(&file, Some(&alt), self.spot, false) {
                        return format!("#align(center, {expr})");
                    }
                }
                self.inlines(node)
            }
            NodeValue::Heading(heading) => {
                let text = self.inlines(node);
                heading_markup(&text, usize::from(heading.level), self.spot.level)
            }
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) => {
                format!("#quote[\n{}\n]", self.children(node))
            }
            NodeValue::List(list) => {
                let items: Vec<String> = node
                    .children()
                    .map(|item| {
                        let check = match &item.data.borrow().value {
                            NodeValue::TaskItem(task) => match task.symbol {
                                Some(_) => "☒ ",
                                None => "☐ ",
                            },
                            _ => "",
                        };
                        format!("[{check}{}]", self.children(item).trim_end())
                    })
                    .collect();
                match list.list_type {
                    ListType::Bullet => {
                        format!("#list(tight: {}, {})", list.tight, items.join(", "))
                    }
                    ListType::Ordered => format!(
                        "#enum(tight: {}, start: {}, {})",
                        list.tight,
                        list.start,
                        items.join(", ")
                    ),
                }
            }
            NodeValue::CodeBlock(code) => {
                let info = code.info.split_whitespace().next().unwrap_or_default();
                match info {
                    "math" => self.book.formula(&code.literal, true),
                    "smiles" => self
                        .book
                        .drawing(&crate::chem::molecule(&code.literal), &code.literal),
                    "chart" => self
                        .book
                        .drawing(&crate::chart::chart(&code.literal), &code.literal),
                    "strata" => self
                        .book
                        .drawing(&crate::strata::column(&code.literal), &code.literal),
                    // Without its language: black on white, and Typst's
                    // highlighting would take most of the time for a
                    // blog with much code.
                    _ => format!(
                        "#raw(block: true, {})",
                        string(code.literal.trim_end_matches('\n'))
                    ),
                }
            }
            // Raw HTML shows as text on the website, too.
            NodeValue::HtmlBlock(html) => escape(html.literal.trim()),
            NodeValue::ThematicBreak => "#line(length: 100%, stroke: 0.5pt)".to_string(),
            NodeValue::Table(table) => {
                let columns = table.num_columns.max(1);
                let align: Vec<&str> = table
                    .alignments
                    .iter()
                    .map(|alignment| match alignment {
                        TableAlignment::Center => "center",
                        TableAlignment::Right => "right",
                        _ => "left",
                    })
                    .collect();
                let mut cells: Vec<String> = Vec::new();
                let mut header: Vec<String> = Vec::new();
                self.in_table = true;
                for row in node.children() {
                    let is_header = matches!(row.data.borrow().value, NodeValue::TableRow(true));
                    let mut row_cells: Vec<String> = row
                        .children()
                        .map(|cell| {
                            let text = self.inlines(cell);
                            if is_header {
                                format!("[#strong[{text}]]")
                            } else {
                                format!("[{text}]")
                            }
                        })
                        .collect();
                    row_cells.resize(columns, "[]".to_string());
                    row_cells.truncate(columns);
                    if is_header {
                        header.extend(row_cells);
                    } else {
                        cells.extend(row_cells);
                    }
                }
                self.in_table = false;
                let header = if header.is_empty() {
                    String::new()
                } else {
                    format!("table.header({}), ", header.join(", "))
                };
                // Typst takes very long for one table across dozens of
                // pages, so a long one becomes several, each with the
                // header row.
                let rows: Vec<String> = cells
                    .chunks(columns * TABLE_ROWS)
                    .map(|chunk| {
                        format!(
                            "#table(columns: {columns}, align: ({},), {header}{})",
                            align.join(", "),
                            chunk.join(", ")
                        )
                    })
                    .collect();
                if rows.is_empty() {
                    format!(
                        "#table(columns: {columns}, align: ({},), {header})",
                        align.join(", ")
                    )
                } else {
                    rows.join("\n")
                }
            }
            // Footnotes go where they're referred to.
            NodeValue::FootnoteDefinition(_) | NodeValue::FrontMatter(_) => String::new(),
            _ => self.children(node),
        }
    }

    /// The text inside a node: words, emphasis, links, code, formulas.
    fn inlines(&mut self, node: &'n AstNode<'n>) -> String {
        let mut out = String::new();
        // The few HTML tags text may use, still open: what closes them.
        let mut open: Vec<(&str, &str)> = Vec::new();
        for child in node.children() {
            let value = child.data.borrow().value.clone();
            match value {
                NodeValue::Text(text) => out.push_str(&self.text(&text)),
                NodeValue::SoftBreak => out.push(' '),
                NodeValue::LineBreak => out.push_str("#linebreak() "),
                NodeValue::Code(code) => out.push_str(&format!("#raw({})", string(&code.literal))),
                NodeValue::Emph => out.push_str(&format!("#emph[{}]", self.inlines(child))),
                NodeValue::Strong => out.push_str(&format!("#strong[{}]", self.inlines(child))),
                NodeValue::Strikethrough => {
                    out.push_str(&format!("#strike[{}]", self.inlines(child)))
                }
                NodeValue::Link(link) => {
                    let mut text = self.inlines(child);
                    if text.trim().is_empty() {
                        text = escape(&link.url);
                    }
                    let footnote = !self.in_table;
                    out.push_str(
                        &self
                            .book
                            .link_with(&link.url, &text, self.spot.links, footnote),
                    );
                }
                NodeValue::Image(image) => {
                    let file = self.picture_file(&image.url);
                    let picture = file.and_then(|file| self.book.image(&file));
                    match picture {
                        Some((path, _)) => {
                            out.push_str(&format!("#box(image({}, height: 1.4em))", string(&path)))
                        }
                        None => out.push_str(&escape(&plain_text(child))),
                    }
                }
                NodeValue::Math(math) => {
                    out.push_str(&self.book.formula(&math.literal, math.display_math))
                }
                NodeValue::HtmlInline(html) => {
                    let tag = html.trim().to_ascii_lowercase().replace(' ', "");
                    match tag.as_str() {
                        "<br>" | "<br/>" => out.push_str("#linebreak() "),
                        "<sup>" => {
                            out.push_str("#super[");
                            open.push(("sup", "]"));
                        }
                        "<sub>" => {
                            out.push_str("#sub[");
                            open.push(("sub", "]"));
                        }
                        "<kbd>" => {
                            out.push_str(
                                "#box(stroke: 0.5pt, inset: (x: 2pt), outset: (y: 2pt), radius: 2pt)[#text(font: \"DejaVu Sans Mono\", size: 0.8em)[",
                            );
                            open.push(("kbd", "]]"));
                        }
                        "</sup>" | "</sub>" | "</kbd>" => {
                            if open
                                .last()
                                .is_some_and(|(name, _)| tag[2..].starts_with(name))
                            {
                                let (_, close) = open.pop().unwrap_or_default();
                                out.push_str(close);
                            }
                        }
                        _ => out.push_str(&escape(&html)),
                    }
                }
                NodeValue::FootnoteReference(reference) => {
                    if let Some(definition) = self.footnotes.get(&reference.name).copied() {
                        let text = self.children(definition);
                        out.push_str(&format!("#footnote[{}]", text.trim()));
                    }
                }
                NodeValue::Raw(raw) => out.push_str(&escape(&raw)),
                _ => out.push_str(&self.inlines(child)),
            }
        }
        while let Some((_, close)) = open.pop() {
            out.push_str(close);
        }
        out
    }

    /// Words, with `:icon-name:` as icons.
    fn text(&mut self, text: &str) -> String {
        if !text.contains(":icon-") {
            return escape(text);
        }
        let mut out = String::new();
        let mut rest = text;
        while let Some(start) = rest.find(":icon-") {
            out.push_str(&escape(&rest[..start]));
            let after = &rest[start + ":icon-".len()..];
            let len = after
                .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
                .unwrap_or(after.len());
            let icon = after[len..]
                .starts_with(':')
                .then(|| self.book.icon(&after[..len]))
                .flatten();
            match icon {
                Some(icon) => {
                    out.push_str(&icon);
                    rest = &after[len + 1..];
                }
                None => {
                    out.push_str(&escape(":icon-"));
                    rest = after;
                }
            }
        }
        out.push_str(&escape(rest));
        out
    }

    /// The file of a picture in text: `/media/…` in `media/`, other site
    /// addresses in `static/`. Pictures on other sites stay out.
    fn picture_file(&self, url: &str) -> Option<PathBuf> {
        let url = url.split(['#', '?']).next().unwrap_or_default();
        let path = url.strip_prefix('/')?;
        if path.starts_with('/') {
            return None;
        }
        let root = &self.book.site.root;
        let file = if path.starts_with("media/") {
            root.join(path)
        } else {
            root.join(crate::site::STATIC_DIR).join(path)
        };
        file.is_file().then_some(file)
    }
}

/// A soft line break or blank text, which don't count next to a picture.
fn is_blank(node: &AstNode) -> bool {
    match &node.data.borrow().value {
        NodeValue::SoftBreak | NodeValue::LineBreak => true,
        NodeValue::Text(text) => text.trim().is_empty(),
        _ => false,
    }
}

/// The plain text inside a node, such as an image's description.
fn plain_text<'n>(node: &'n AstNode<'n>) -> String {
    let mut out = String::new();
    for descendant in node.descendants() {
        if let NodeValue::Text(text) = &descendant.data.borrow().value {
            out.push_str(text);
        }
    }
    out
}

/// The fonts of the book, compiled into q.e.d.: Libertinus Serif for the
/// text, DejaVu Sans Mono for code, New Computer Modern Math for formulas.
/// Where they come from: `assets/typst-fonts/NOTICE`.
static FONT_FILES: [&[u8]; 7] = [
    include_bytes!("../assets/typst-fonts/LibertinusSerif-Regular.otf"),
    include_bytes!("../assets/typst-fonts/LibertinusSerif-Italic.otf"),
    include_bytes!("../assets/typst-fonts/LibertinusSerif-Bold.otf"),
    include_bytes!("../assets/typst-fonts/LibertinusSerif-BoldItalic.otf"),
    include_bytes!("../assets/typst-fonts/DejaVuSansMono.ttf"),
    include_bytes!("../assets/typst-fonts/DejaVuSansMono-Bold.ttf"),
    include_bytes!("../assets/typst-fonts/NewCMMath-Regular.otf"),
];

static FONTS: LazyLock<(LazyHash<FontBook>, Vec<Font>)> = LazyLock::new(|| {
    let fonts: Vec<Font> = FONT_FILES
        .iter()
        .flat_map(|data| Font::iter(Bytes::new(*data)))
        .collect();
    (LazyHash::new(FontBook::from_fonts(&fonts)), fonts)
});

static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(|| LazyHash::new(Library::default()));

/// What Typst sees: the book's markup as `/main.typ`, its pictures, the
/// fonts. Nothing else of the computer.
struct PdfWorld {
    main: Source,
    files: BTreeMap<String, Bytes>,
    today: Date,
}

impl PdfWorld {
    fn new(markup: String, files: BTreeMap<String, Bytes>, today: Date) -> Result<Self, PdfError> {
        let path = VirtualPath::new("/main.typ").map_err(|e| PdfError::Typst(e.to_string()))?;
        let id = RootedPath::new(VirtualRoot::Project, path).intern();
        Ok(PdfWorld {
            main: Source::new(id, markup),
            files,
            today,
        })
    }
}

impl World for PdfWorld {
    fn library(&self) -> &LazyHash<Library> {
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &FONTS.0
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::NotSource)
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = id.get().vpath().get_with_slash();
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| FileError::NotFound(PathBuf::from(path)))
    }

    fn font(&self, index: usize) -> Option<Font> {
        FONTS.1.get(index).cloned()
    }

    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        Datetime::from_ymd(
            i32::try_from(self.today.year).ok()?,
            u8::try_from(self.today.month).ok()?,
            u8::try_from(self.today.day).ok()?,
        )
    }
}

/// Sets `markup` with Typst and writes it as a PDF; also says how many
/// pages it has.
fn compile(
    markup: String,
    files: BTreeMap<String, Bytes>,
    today: Date,
) -> Result<(Vec<u8>, u32), PdfError> {
    let world = PdfWorld::new(markup, files, today)?;
    let document = typst::compile::<typst_layout::PagedDocument>(&world)
        .output
        .map_err(|errors| PdfError::Typst(describe(&world, &errors)))?;
    let options = typst_pdf::PdfOptions {
        creator: Smart::Custom(Some("q.e.d.".to_string())),
        ..typst_pdf::PdfOptions::default()
    };
    let pdf = typst_pdf::pdf(&document, &options)
        .map_err(|errors| PdfError::Typst(describe(&world, &errors)))?;
    let pages = u32::try_from(document.pages().len()).unwrap_or(u32::MAX);
    Ok((pdf, pages))
}

/// Whether Typst can set a formula (mitex's output), with the helpers.
fn sets_math(math: &str) -> bool {
    let markup = format!("{MATH_HELPERS}$ {math} $\n");
    let today = Date {
        year: 2026,
        month: 1,
        day: 1,
    };
    PdfWorld::new(markup, BTreeMap::new(), today).is_ok_and(|world| {
        typst::compile::<typst_layout::PagedDocument>(&world)
            .output
            .is_ok()
    })
}

/// Typst's first errors, each with the markup it's about.
fn describe(world: &PdfWorld, errors: &[SourceDiagnostic]) -> String {
    use typst::WorldExt;
    errors
        .iter()
        .take(3)
        .map(|error| {
            let at = world
                .range(error.span)
                .and_then(|range| world.main.text().get(range))
                .map(|text| {
                    let text: String = text.chars().take(120).collect();
                    format!(" (at “{text}”)")
                })
                .unwrap_or_default();
            format!("{}{at}", error.message)
        })
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::site;

    const TODAY: Date = Date {
        year: 2026,
        month: 10,
        day: 21,
    };

    fn write(dir: &Path, file: &str, text: &str) {
        let path = dir.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// A start page, two pages with one below the first, one left out of
    /// the PDF, a blog with two posts, one from the future and a draft.
    fn site_in(dir: &Path) -> Site {
        write(
            dir,
            "site.toml",
            "title = \"Pod & Co\"\nlanguage = \"de\"\nbase_url = \"https://example.com/\"\n\n[languages.en]\ntitle = \"Pod and Co\"\n",
        );
        write(
            dir,
            "pages/index.md",
            "+++\ntitle = \"Start\"\n+++\n\nHello *world*, see [us](about.md) and [Typst](https://typst.app).\n",
        );
        write(
            dir,
            "pages/index.en.md",
            "+++\ntitle = \"Home\"\n+++\n\nHello.\n",
        );
        write(
            dir,
            "pages/about.md",
            "+++\ntitle = \"About\"\nweight = 1\n+++\n\n## Team\n\nWe (two) are #1 at $5 <sup>2</sup> and `code`.\n",
        );
        write(
            dir,
            "pages/about/history.md",
            "+++\ntitle = \"History\"\n+++\n\n- one\n- two\n",
        );
        write(
            dir,
            "pages/contact.md",
            "+++\ntitle = \"Contact\"\nweight = 2\n+++\n\n::: call-to-action\nheadline = \"Write\"\nbutton_label = \"About\"\nbutton_link = \"/about\"\n:::\n",
        );
        write(
            dir,
            "pages/blog.md",
            "+++\ntitle = \"Blog\"\n+++\n\n::: post-list\n:::\n",
        );
        write(
            dir,
            "pages/blog/old.md",
            "+++\ntitle = \"Old\"\ndate = 2026-01-05\nauthors = [\"Ann\"]\n+++\n\nOld news.\n",
        );
        write(
            dir,
            "pages/blog/new.md",
            "+++\ntitle = \"New\"\ndate = 2026-10-20\n+++\n\nFresh: $x^2$\n",
        );
        write(
            dir,
            "pages/blog/later.md",
            "+++\ntitle = \"Later\"\ndate = 2027-01-01\n+++\n",
        );
        write(
            dir,
            "pages/blog/draft.md",
            "+++\ntitle = \"Draft\"\ndate = 2026-02-02\ndraft = true\n+++\n",
        );
        site::open(dir).unwrap()
    }

    fn markup_on(site: &Site, language: Option<&str>) -> String {
        Book::new(site, language, TODAY).typst().0
    }

    #[test]
    fn puts_pages_first_and_posts_in_a_chapter_after_them() {
        let dir = tempfile::tempdir().unwrap();
        let site = site_in(dir.path());
        let markup = markup_on(&site, None);
        let position = |text: &str| {
            markup
                .find(text)
                .unwrap_or_else(|| panic!("{text} is missing:\n{markup}"))
        };
        let order = [
            "#heading(level: 1)[Start]",
            "#heading(level: 1)[About]",
            "#heading(level: 2)[History]",
            "#heading(level: 1)[Contact]",
            "#heading(level: 1)[Blog]",
            "#heading(level: 2)[Old]",
            "#heading(level: 2)[New]",
        ];
        for pair in order.windows(2) {
            assert!(position(pair[0]) < position(pair[1]), "{pair:?}");
        }
        assert!(markup.contains("#outline(title: [Inhalt], depth: 2)"));
        assert!(!markup.contains("Later"), "no posts from the future");
        assert!(!markup.contains("Draft"));
        assert!(!markup.contains("[Home]"), "only the main language");
        assert!(!markup.contains("Long list"), "pdf = false leaves it out");
    }

    #[test]
    fn links_pages_inside_the_pdf_and_other_sites_with_their_address() {
        let dir = tempfile::tempdir().unwrap();
        let site = site_in(dir.path());
        let markup = markup_on(&site, None);
        let about = "#heading(level: 1)[About] <p-1>";
        assert!(markup.contains(about), "{markup}");
        assert!(markup.contains("#link(<p-1>)[us]"), "{markup}");
        assert!(markup.contains("#link(<p-1>)[→ About]"), "a button");
        assert!(
            markup.contains(
                "#link(\"https://typst.app\")[Typst]#footnote[https\\:\\/\\/typst\\.app]"
            )
        );
        // A Post list links the posts, newest first.
        let list = markup.find("#list([").unwrap();
        assert!(markup[list..].find("[New]").unwrap() < markup[list..].find("[Old]").unwrap());
    }

    #[test]
    fn escapes_what_means_something_in_typst() {
        assert_eq!(
            escape("#1 at $5 (two) *a* <b>"),
            "\\#1 at \\$5 \\(two) \\*a\\* \\<b\\>"
        );
        let dir = tempfile::tempdir().unwrap();
        let site = site_in(dir.path());
        let markup = markup_on(&site, None);
        assert!(
            markup.contains("We \\(two) are \\#1 at \\$5 #super[2] and #raw(\"code\")"),
            "{markup}"
        );
    }

    #[test]
    fn sets_formulas_and_shows_those_it_cannot_as_latex() {
        let dir = tempfile::tempdir().unwrap();
        let site = site_in(dir.path());
        let mut book = Book::new(&site, None, TODAY);
        assert_eq!(book.formula(r"\frac{a}{b}", false), "$frac(a ,b )$");
        assert!(book.formula(r"\frac{a}{b}", true).starts_with("$ "));
        assert_eq!(
            book.formula(r"\nonsense{", false),
            "#raw(\"\\\\nonsense{\")"
        );
        assert_eq!(book.notes.len(), 1);
    }

    #[test]
    fn writes_one_pdf_per_language_into_assets() {
        let dir = tempfile::tempdir().unwrap();
        let site = site_in(dir.path());
        let report = export_on(&site, TODAY).unwrap();
        let names: Vec<&str> = report.files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["pod-co.pdf", "pod-co-en.pdf"]);
        for file in &report.files {
            let pdf = fs::read(dir.path().join("assets").join(&file.name)).unwrap();
            assert!(pdf.starts_with(b"%PDF"));
            assert!(file.pages >= 3, "title, contents, pages: {file:?}");
        }
    }

    #[test]
    fn makes_the_docs_site_into_a_pdf() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
        let site = site::open(&docs).unwrap();
        let mut notes = Vec::new();
        let expanded = crate::shared::expanded(&site, &mut notes);
        let (markup, files, _) = Book::new(&expanded, None, TODAY).typst();
        let (pdf, pages) = compile(markup, files, TODAY).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pages > 50, "{pages} pages");
    }
}

//! Podcasting: the podcast's settings (`[podcast]` in `site.toml`) and the
//! episode settings (`[episode]` in an episode page's front matter).
//!
//! A site has at most one podcast. Its episodes are dated pages below
//! `/episodes`, so they get the block editor, links, moving and Post lists
//! like any other post; their shownotes are their blocks:
//!
//! ```text
//! +++
//! title = "Getting started"
//! date = 2026-10-28
//!
//! [episode]
//! season = 1
//! number = 3
//! guid = "8f2c…"
//!
//! [[episode.chapters]]
//! start = "00:00:00"
//! title = "Intro"
//! +++
//! ```

use std::path::{Path, PathBuf};

use image::{ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};

use crate::block::{Block, PostList};
use crate::page::{FrontMatter, Menu, PageContent, PageDate};
use crate::site::{self, Page, Site, SiteError};

/// The page the episodes live below, and the address of the archive.
pub const EPISODES_PATH: &str = "episodes";

/// The podcast feed, at the address jekyll-octopod gave it, so podcasts
/// that move over keep their subscribers without a redirect.
pub const FEED_FILE: &str = "episodes.mp3.rss";

/// The settings of the site's podcast. Having them turns on episodes and
/// the feed.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Podcast {
    /// The name in podcast apps; the site's title if empty.
    pub title: Option<String>,
    /// What the podcast is about; the site's description if empty.
    pub description: Option<String>,
    /// Who makes it, shown below the name in podcast apps.
    pub author: Option<String>,
    /// Who podcast directories write to, e.g. to confirm the podcast is
    /// yours. Not shown to listeners by most apps.
    pub owner_name: Option<String>,
    pub owner_email: Option<String>,
    /// The cover art: a square image in `media/`, see [`check_cover`].
    pub cover: Option<String>,
    /// Apple's categories ([`CATEGORIES`]): `"Technology"`, or a
    /// subcategory as `"Society & Culture > Documentary"`. The first one
    /// counts most.
    #[serde(default)]
    pub categories: Vec<String>,
    /// Whether the podcast has explicit content; episodes can say so
    /// themselves.
    #[serde(default)]
    pub explicit: bool,
    /// Episodic (newest first, the usual) or serial (in order, like a
    /// story told in parts).
    #[serde(default, rename = "type")]
    pub kind: PodcastType,
    /// `podcast:guid`: the podcast's lasting identity, which stays when the
    /// feed moves. Made once from the feed address when the site has a web
    /// address, then never changed.
    pub guid: Option<String>,
    /// A copyright or license line, e.g. `"CC BY 4.0"`.
    pub copyright: Option<String>,
    /// Ways to support the podcast (`podcast:funding`).
    #[serde(default)]
    pub funding: Vec<Funding>,
    /// Where the audio files are kept.
    #[serde(default)]
    pub audio_files: AudioFiles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum PodcastType {
    #[default]
    Episodic,
    Serial,
}

/// A link to support the podcast, such as a Patreon or Liberapay page.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Funding {
    pub url: String,
    /// The link text, e.g. "Support us on Liberapay".
    pub title: String,
}

/// Where a podcast's audio files are kept. Audio files are large (50–150 MB
/// an episode), and a site's history keeps every file it ever had.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum AudioFiles {
    /// In the site's `audio/` folder, left out of its history: versions
    /// stay small, but backups of the history don't hold the audio.
    #[default]
    Local,
    /// In `audio/`, and in the history like every other file.
    InHistory,
    /// Somewhere else (a podcast host, a storage service); episodes give
    /// a web address for their audio.
    External,
}

/// The settings of one episode, `[episode]` in its page's front matter.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Episode {
    pub season: Option<u32>,
    pub number: Option<u32>,
    /// A full episode, a trailer or a bonus.
    #[serde(default, rename = "type")]
    pub kind: EpisodeType,
    /// The episode's own cover in `media/`; the podcast's if empty. The
    /// same rules apply as for the podcast's cover.
    pub image: Option<String>,
    /// Explicit content; the podcast's setting if empty.
    pub explicit: Option<bool>,
    /// The episode's lasting identity in the feed. Podcast apps use it to
    /// tell episodes apart, so it never changes, not even when the page
    /// moves. Made when the episode is made.
    #[serde(default)]
    pub guid: String,
    #[serde(default)]
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum EpisodeType {
    #[default]
    Full,
    Trailer,
    Bonus,
}

impl EpisodeType {
    pub fn as_str(self) -> &'static str {
        match self {
            EpisodeType::Full => "full",
            EpisodeType::Trailer => "trailer",
            EpisodeType::Bonus => "bonus",
        }
    }
}

/// A chapter mark: where it starts in the audio and what it's called.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    /// `"01:02:03"`, `"02:03"`, with milliseconds if needed
    /// (`"00:00:01.500"`), as Podlove Simple Chapters write it.
    pub start: String,
    pub title: String,
    /// A web page about the chapter's topic.
    pub link: Option<String>,
}

/// Reads a chapter start into milliseconds: `"1:02:03.5"` is 3 723 500.
/// `None` for anything else, such as `"1:75"` or `"abc"`.
pub fn start_millis(start: &str) -> Option<u64> {
    let (clock, fraction) = match start.trim().split_once('.') {
        Some((clock, fraction)) => (clock, Some(fraction)),
        None => (start.trim(), None),
    };
    let parts: Vec<&str> = clock.split(':').collect();
    if parts.len() > 3 || parts.iter().any(|part| part.is_empty() || part.len() > 2) {
        return None;
    }
    let mut seconds: u64 = 0;
    for (index, part) in parts.iter().enumerate() {
        let value: u64 = part.parse().ok()?;
        // Hours may be anything; minutes and seconds stay below 60.
        if index > 0 && value >= 60 {
            return None;
        }
        seconds = seconds * 60 + value;
    }
    let millis = match fraction {
        None => 0,
        Some(digits) if (1..=3).contains(&digits.len()) => {
            let value: u64 = digits.parse().ok()?;
            value * 10u64.pow(3 - digits.len() as u32)
        }
        Some(_) => return None,
    };
    Some(seconds * 1000 + millis)
}

/// A chapter start for readers: `"00:01:45.000"` shows as `"00:01:45"`;
/// milliseconds stay only when they aren't zero.
pub fn readable_start(start: &str) -> String {
    let start = start.trim();
    match start.split_once('.') {
        Some((clock, fraction)) if fraction.chars().all(|c| c == '0') => clock.to_string(),
        _ => start.to_string(),
    }
}

impl Episode {
    /// What's wrong with the episode's settings, if anything.
    pub fn problem(&self) -> Option<String> {
        let mut last = None;
        for chapter in &self.chapters {
            let title = chapter.title.trim();
            if title.is_empty() {
                return Some(format!("the chapter at {} needs a title", chapter.start));
            }
            let Some(start) = start_millis(&chapter.start) else {
                return Some(format!(
                    "chapter “{title}”: “{}” isn't a time like 01:02:03 or 02:03",
                    chapter.start
                ));
            };
            if last.is_some_and(|last| start <= last) {
                return Some(format!(
                    "chapter “{title}” starts before the one above it; chapters go in order"
                ));
            }
            last = Some(start);
        }
        None
    }
}

/// A new episode guid: random, so it's unique without asking anyone.
pub fn new_guid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// The `podcast:guid` for a feed address, as the Podcasting 2.0 namespace
/// asks: a UUID (version 5) of the address without its scheme and
/// trailing slashes, so everyone who computes it gets the same.
pub fn podcast_guid(feed_url: &str) -> String {
    /// The namespace the Podcasting 2.0 project picked for `podcast:guid`.
    const NAMESPACE: uuid::Uuid = uuid::uuid!("ead4c236-bf58-58c6-a2c6-a6b28d128cb6");
    let address = feed_url
        .split_once("://")
        .map_or(feed_url, |(_, rest)| rest)
        .trim_end_matches('/');
    uuid::Uuid::new_v5(&NAMESPACE, address.as_bytes()).to_string()
}

/// Apple Podcasts' categories with their subcategories. Other directories
/// use the same list.
pub const CATEGORIES: &[(&str, &[&str])] = &[
    (
        "Arts",
        &[
            "Books",
            "Design",
            "Fashion & Beauty",
            "Food",
            "Performing Arts",
            "Visual Arts",
        ],
    ),
    (
        "Business",
        &[
            "Careers",
            "Entrepreneurship",
            "Investing",
            "Management",
            "Marketing",
            "Non-Profit",
        ],
    ),
    ("Comedy", &["Comedy Interviews", "Improv", "Stand-Up"]),
    (
        "Education",
        &["Courses", "How To", "Language Learning", "Self-Improvement"],
    ),
    ("Fiction", &["Comedy Fiction", "Drama", "Science Fiction"]),
    ("Government", &[]),
    (
        "Health & Fitness",
        &[
            "Alternative Health",
            "Fitness",
            "Medicine",
            "Mental Health",
            "Nutrition",
            "Sexuality",
        ],
    ),
    ("History", &[]),
    (
        "Kids & Family",
        &[
            "Education for Kids",
            "Parenting",
            "Pets & Animals",
            "Stories for Kids",
        ],
    ),
    (
        "Leisure",
        &[
            "Animation & Manga",
            "Automotive",
            "Aviation",
            "Crafts",
            "Games",
            "Hobbies",
            "Home & Garden",
            "Video Games",
        ],
    ),
    (
        "Music",
        &["Music Commentary", "Music History", "Music Interviews"],
    ),
    (
        "News",
        &[
            "Business News",
            "Daily News",
            "Entertainment News",
            "News Commentary",
            "Politics",
            "Sports News",
            "Tech News",
        ],
    ),
    (
        "Religion & Spirituality",
        &[
            "Buddhism",
            "Christianity",
            "Hinduism",
            "Islam",
            "Judaism",
            "Religion",
            "Spirituality",
        ],
    ),
    (
        "Science",
        &[
            "Astronomy",
            "Chemistry",
            "Earth Sciences",
            "Life Sciences",
            "Mathematics",
            "Natural Sciences",
            "Nature",
            "Physics",
            "Social Sciences",
        ],
    ),
    (
        "Society & Culture",
        &[
            "Documentary",
            "Personal Journals",
            "Philosophy",
            "Places & Travel",
            "Relationships",
        ],
    ),
    (
        "Sports",
        &[
            "Baseball",
            "Basketball",
            "Cricket",
            "Fantasy Sports",
            "Football",
            "Golf",
            "Hockey",
            "Rugby",
            "Running",
            "Soccer",
            "Swimming",
            "Tennis",
            "Volleyball",
            "Wilderness",
            "Wrestling",
        ],
    ),
    ("Technology", &[]),
    ("True Crime", &[]),
    (
        "TV & Film",
        &[
            "After Shows",
            "Film History",
            "Film Interviews",
            "Film Reviews",
            "TV Reviews",
        ],
    ),
];

/// How a category and a subcategory are written together.
pub const CATEGORY_SEPARATOR: &str = " > ";

/// Every category as it's written in `site.toml`, subcategories after
/// their category: `"Arts"`, `"Arts > Books"`, …
pub fn category_names() -> Vec<String> {
    CATEGORIES
        .iter()
        .flat_map(|(category, subcategories)| {
            std::iter::once(category.to_string()).chain(
                subcategories
                    .iter()
                    .map(move |sub| format!("{category}{CATEGORY_SEPARATOR}{sub}")),
            )
        })
        .collect()
}

/// Splits `"Arts > Books"` into the category and the subcategory, if the
/// name is on Apple's list.
pub fn category(name: &str) -> Option<(&'static str, Option<&'static str>)> {
    let (main, sub) = match name.split_once(CATEGORY_SEPARATOR.trim()) {
        Some((main, sub)) => (main.trim(), Some(sub.trim())),
        None => (name.trim(), None),
    };
    let (category, subcategories) = CATEGORIES.iter().find(|(c, _)| *c == main)?;
    match sub {
        None => Some((category, None)),
        Some(sub) => subcategories
            .iter()
            .find(|s| **s == sub)
            .map(|s| (*category, Some(*s))),
    }
}

impl Podcast {
    /// What's wrong with the settings, if anything.
    pub fn problem(&self) -> Option<String> {
        if let Some(name) = self.categories.iter().find(|name| category(name).is_none()) {
            return Some(format!("“{name}” isn't one of Apple's podcast categories"));
        }
        if let Some(email) = self.owner_email.as_deref()
            && !email.contains('@')
        {
            return Some(format!("“{email}” isn't an email address"));
        }
        if let Some(funding) = self
            .funding
            .iter()
            .find(|f| f.url.trim().is_empty() || f.title.trim().is_empty())
        {
            return Some(format!(
                "a support link needs an address and a text (“{}”)",
                funding.title.trim()
            ));
        }
        None
    }
}

/// Apple's rules for cover art: square, 1400 to 3000 pixels, JPEG or PNG.
pub const COVER_MIN: u32 = 1400;
pub const COVER_MAX: u32 = 3000;

/// What's wrong with `name` in `media/` as podcast or episode cover art,
/// by Apple's rules; empty if nothing is.
pub fn check_cover(media_dir: &Path, name: &str) -> Vec<String> {
    let path = media_dir.join(name);
    let reader = match ImageReader::open(&path).and_then(|r| r.with_guessed_format()) {
        Ok(reader) => reader,
        Err(_) => return vec![format!("{name} can't be read")],
    };
    let format = reader.format();
    let mut problems = Vec::new();
    if !matches!(format, Some(ImageFormat::Jpeg | ImageFormat::Png)) {
        problems.push("it must be a JPEG or PNG".to_string());
    }
    let Ok((width, height)) = reader.into_dimensions() else {
        problems.push(format!("{name} isn't an image q.e.d. can read"));
        return problems;
    };
    if width != height {
        problems.push(format!("it must be square, not {width} × {height} pixels"));
    }
    let side = width.max(height);
    if width.min(height) < COVER_MIN {
        problems.push(format!(
            "it must be at least {COVER_MIN} × {COVER_MIN} pixels, not {width} × {height}"
        ));
    } else if side > COVER_MAX {
        problems.push(format!(
            "it must be at most {COVER_MAX} × {COVER_MAX} pixels, not {width} × {height}"
        ));
    }
    problems
}

/// The words episode pages use, in the site's language.
pub struct Words {
    pub episodes: &'static str,
    pub season: &'static str,
    pub episode: &'static str,
    pub trailer: &'static str,
    pub bonus: &'static str,
    pub chapters: &'static str,
}

impl Words {
    /// For `de`, `de-AT`, …; English for languages q.e.d. has no words for.
    pub fn for_language(language: &str) -> Words {
        let main = language.trim().to_lowercase();
        let (episodes, season, episode, trailer, bonus, chapters) =
            match main.split(['-', '_']).next().unwrap_or_default() {
                "de" => ("Folgen", "Staffel", "Folge", "Trailer", "Bonus", "Kapitel"),
                "fr" => (
                    "Épisodes",
                    "Saison",
                    "Épisode",
                    "Bande-annonce",
                    "Bonus",
                    "Chapitres",
                ),
                "es" => (
                    "Episodios",
                    "Temporada",
                    "Episodio",
                    "Tráiler",
                    "Extra",
                    "Capítulos",
                ),
                "it" => (
                    "Episodi", "Stagione", "Episodio", "Trailer", "Extra", "Capitoli",
                ),
                "nl" => (
                    "Afleveringen",
                    "Seizoen",
                    "Aflevering",
                    "Trailer",
                    "Bonus",
                    "Hoofdstukken",
                ),
                _ => (
                    "Episodes", "Season", "Episode", "Trailer", "Bonus", "Chapters",
                ),
            };
        Words {
            episodes,
            season,
            episode,
            trailer,
            bonus,
            chapters,
        }
    }
}

/// How an episode introduces itself: "Season 2 · Episode 5 · Trailer",
/// leaving out what it doesn't have; empty if it has none of them.
pub fn label(episode: &Episode, words: &Words) -> String {
    let mut parts = Vec::new();
    parts.extend(episode.season.map(|n| format!("{} {n}", words.season)));
    parts.extend(episode.number.map(|n| format!("{} {n}", words.episode)));
    match episode.kind {
        EpisodeType::Full => {}
        EpisodeType::Trailer => parts.push(words.trailer.to_string()),
        EpisodeType::Bonus => parts.push(words.bonus.to_string()),
    }
    parts.join(" · ")
}

/// The site's episodes: pages below `/episodes` with `[episode]` settings.
pub fn episodes(site: &Site) -> impl Iterator<Item = &Page> {
    let prefix = format!("{EPISODES_PATH}/");
    site.pages
        .iter()
        .filter(move |page| page.path.starts_with(&prefix) && page.content.front.episode.is_some())
}

/// Makes a new episode dated today, below `/episodes`, and returns its
/// page. It gets `number`, or else the next one (see [`next_number`]),
/// the season of the latest episode and a guid of its own. The first episode also makes the `/episodes` page
/// itself, the archive, which lists them.
pub fn create_episode(root: &Path, title: &str, number: Option<u32>) -> Result<Page, SiteError> {
    let site = site::open(root)?;
    if site.config.podcast.is_none() {
        return Err(SiteError::Invalid {
            path: PathBuf::from(site::CONFIG_FILE),
            message: "the site has no podcast; turn it on in the Site settings first".to_string(),
        });
    }
    if !site.pages.iter().any(|page| page.path == EPISODES_PATH) {
        create_archive(root, &site.config.language)?;
    }

    let mut known: Vec<&Episode> = episodes(&site)
        .filter_map(|page| page.content.front.episode.as_ref())
        .collect();
    let number = number.unwrap_or_else(|| next_number(&site));
    // The season of the episode with the highest number goes on.
    known.sort_by_key(|e| e.number);
    let season = known.last().and_then(|e| e.season);

    let page = site::create_page(
        root,
        EPISODES_PATH,
        title,
        PageDate::on(crate::calendar::today()),
    )?;
    site::update_page(root, &page.file, |content| {
        content.front.episode = Some(Episode {
            season,
            number: Some(number),
            guid: new_guid(),
            ..Episode::default()
        });
    })
}

/// The number after the highest episode number; 1 for the first episode.
/// Podcasts that start counting at 0 give their first episode 0 by hand,
/// and the count goes on from there.
pub fn next_number(site: &Site) -> u32 {
    episodes(site)
        .filter_map(|page| page.content.front.episode.as_ref()?.number)
        .max()
        .map_or(1, |n| n + 1)
}

/// The `/episodes` page: in the main menu, with a list of the episodes.
fn create_archive(root: &Path, language: &str) -> Result<(), SiteError> {
    let content = PageContent {
        front: FrontMatter {
            title: Words::for_language(language).episodes.to_string(),
            menu: Menu::Main,
            ..FrontMatter::default()
        },
        blocks: vec![Block::PostList(PostList::default())],
    };
    let file = root
        .join(site::PAGES_DIR)
        .join(format!("{EPISODES_PATH}.md"));
    site::write_new(&file, &crate::page_writer::write(&content, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_chapter_starts() {
        assert_eq!(start_millis("00:00:00"), Some(0));
        assert_eq!(start_millis("1:02:03.5"), Some(3_723_500));
        assert_eq!(start_millis("02:03"), Some(123_000));
        assert_eq!(start_millis("00:00:01.250"), Some(1250));
        assert_eq!(start_millis("1:75"), None);
        assert_eq!(start_millis("abc"), None);
        assert_eq!(start_millis("1:2:3:4"), None);
        assert_eq!(start_millis("00:01.1234"), None);
        assert_eq!(readable_start("00:01:45.000"), "00:01:45");
        assert_eq!(readable_start(" 00:01:45.500"), "00:01:45.500");
        assert_eq!(readable_start("02:03"), "02:03");
    }

    #[test]
    fn wants_chapters_in_order_with_titles() {
        let chapter = |start: &str, title: &str| Chapter {
            start: start.to_string(),
            title: title.to_string(),
            link: None,
        };
        let mut episode = Episode {
            chapters: vec![chapter("00:00", "Intro"), chapter("05:00", "Main")],
            ..Episode::default()
        };
        assert_eq!(episode.problem(), None);
        episode.chapters.push(chapter("04:00", "Late"));
        assert!(episode.problem().unwrap().contains("before"));
        episode.chapters[2] = chapter("7 min", "Odd");
        assert!(episode.problem().unwrap().contains("isn't a time"));
        episode.chapters[2] = chapter("07:00", " ");
        assert!(episode.problem().unwrap().contains("needs a title"));
    }

    #[test]
    fn knows_apples_categories() {
        assert_eq!(category("Technology"), Some(("Technology", None)));
        assert_eq!(
            category("Society & Culture > Documentary"),
            Some(("Society & Culture", Some("Documentary")))
        );
        assert_eq!(category("Technology > Gadgets"), None);
        assert_eq!(category("Cooking"), None);
        let names = category_names();
        assert!(names.contains(&"Arts > Books".to_string()));
        assert_eq!(names.first().map(String::as_str), Some("Arts"));
    }

    #[test]
    fn makes_the_podcast_guid_from_the_feed_address() {
        // The example from the Podcasting 2.0 documentation.
        assert_eq!(
            podcast_guid("https://mp3s.nashownotes.com/pc20rss.xml"),
            "917393e3-1b1e-5cef-ace4-edaa54e1f810"
        );
        assert_eq!(
            podcast_guid("http://mp3s.nashownotes.com/pc20rss.xml/"),
            podcast_guid("https://mp3s.nashownotes.com/pc20rss.xml")
        );
    }

    #[test]
    fn checks_cover_art_by_apples_rules() {
        let dir = tempfile::tempdir().unwrap();
        let save = |name: &str, width: u32, height: u32| {
            image::RgbImage::new(width, height)
                .save(dir.path().join(name))
                .unwrap();
        };
        save("good.png", 1400, 1400);
        save("small.png", 600, 600);
        save("wide.jpg", 3000, 1500);
        save("huge.png", 3200, 3200);
        save("cover.gif", 1500, 1500);

        assert!(check_cover(dir.path(), "good.png").is_empty());
        assert!(check_cover(dir.path(), "small.png")[0].contains("at least 1400"));
        let wide = check_cover(dir.path(), "wide.jpg");
        assert_eq!(wide.len(), 1, "{wide:?}");
        assert!(wide[0].contains("square"));
        assert!(check_cover(dir.path(), "huge.png")[0].contains("at most 3000"));
        assert!(check_cover(dir.path(), "cover.gif")[0].contains("JPEG or PNG"));
        assert!(check_cover(dir.path(), "missing.png")[0].contains("can't be read"));
    }

    fn podcast_site() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        site::create_site(dir.path(), "Pod", "de-AT").unwrap();
        let mut config = site::open(dir.path()).unwrap().config;
        config.base_url = Some("https://pod.example/".to_string());
        config.podcast = Some(Podcast {
            categories: vec!["Technology".to_string()],
            ..Podcast::default()
        });
        site::save_config(dir.path(), &config).unwrap();
        dir
    }

    #[test]
    fn makes_episodes_with_the_next_number_and_an_archive() {
        let dir = podcast_site();
        let first = create_episode(dir.path(), "Hallo Welt", None).unwrap();
        assert!(first.path.starts_with("episodes/"), "{}", first.path);
        let episode = first.content.front.episode.clone().unwrap();
        assert_eq!(episode.number, Some(1));
        assert_eq!(episode.guid.len(), 36);
        assert!(first.content.front.date.is_some());

        let site = site::open(dir.path()).unwrap();
        let archive = site.pages.iter().find(|p| p.path == EPISODES_PATH).unwrap();
        assert_eq!(archive.content.front.title, "Folgen");
        assert_eq!(archive.content.front.menu, Menu::Main);
        assert!(matches!(archive.content.blocks[..], [Block::PostList(_)]));

        // The season goes on, the number counts up.
        let mut front = first.content.front.clone();
        if let Some(episode) = front.episode.as_mut() {
            episode.season = Some(2);
        }
        site::save_front_matter(dir.path(), &first.file, front).unwrap();
        let second = create_episode(dir.path(), "Zweite", None).unwrap();
        let episode = second.content.front.episode.unwrap();
        assert_eq!((episode.season, episode.number), (Some(2), Some(2)));
        assert_ne!(episode.guid, first.content.front.episode.unwrap().guid);
    }

    #[test]
    fn counts_on_from_episode_zero() {
        let dir = podcast_site();
        let zero = create_episode(dir.path(), "Nullnummer", Some(0)).unwrap();
        assert_eq!(zero.content.front.episode.unwrap().number, Some(0));
        let text = std::fs::read_to_string(dir.path().join(&zero.file)).unwrap();
        assert!(text.contains("number = 0"), "{text}");
        let next = create_episode(dir.path(), "Eins", None).unwrap();
        assert_eq!(next.content.front.episode.unwrap().number, Some(1));
    }

    #[test]
    fn needs_the_podcast_switched_on_for_episodes() {
        let dir = tempfile::tempdir().unwrap();
        site::create_site(dir.path(), "Pages only", "en").unwrap();
        let error = create_episode(dir.path(), "One", None).unwrap_err();
        assert!(error.to_string().contains("no podcast"), "{error}");
    }

    #[test]
    fn writes_episode_settings_as_sections_and_keeps_comments() {
        let dir = podcast_site();
        let page = create_episode(dir.path(), "Kapitel", None).unwrap();
        let file = dir.path().join(&page.file);

        // A comment someone wrote into the section by hand.
        let text = std::fs::read_to_string(&file).unwrap();
        let text = text.replace("[episode]\n", "[episode]\n# Aufnahme im Studio\n");
        std::fs::write(&file, text).unwrap();

        let mut front = page.content.front.clone();
        if let Some(episode) = front.episode.as_mut() {
            episode.kind = EpisodeType::Bonus;
            episode.chapters = vec![
                Chapter {
                    start: "00:00:00".to_string(),
                    title: "Intro".to_string(),
                    link: None,
                },
                Chapter {
                    start: "00:01:30.500".to_string(),
                    title: "Thema".to_string(),
                    link: Some("https://example.com".to_string()),
                },
            ];
        }
        let saved = site::save_front_matter(dir.path(), &page.file, front).unwrap();
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("[episode]\n# Aufnahme im Studio\n"), "{text}");
        assert!(text.contains("type = \"bonus\""), "{text}");
        assert!(text.contains("[[episode.chapters]]"), "{text}");
        assert_eq!(
            saved.content.front.episode.as_ref().unwrap().chapters.len(),
            2
        );

        // Chapters out of order are refused.
        let mut front = saved.content.front.clone();
        if let Some(episode) = front.episode.as_mut() {
            episode.chapters.reverse();
        }
        assert!(site::save_front_matter(dir.path(), &page.file, front).is_err());
    }

    #[test]
    fn writes_the_podcast_section_with_a_lasting_guid() {
        let dir = podcast_site();
        let text = std::fs::read_to_string(dir.path().join(site::CONFIG_FILE)).unwrap();
        assert!(text.contains("[podcast]"), "{text}");
        assert!(!text.contains("explicit"), "defaults stay out: {text}");
        let guid = podcast_guid("https://pod.example/episodes.mp3.rss");
        assert!(text.contains(&guid), "{text}");

        // A new web address doesn't change it.
        let mut config = site::open(dir.path()).unwrap().config;
        config.base_url = Some("https://new.example/".to_string());
        if let Some(podcast) = config.podcast.as_mut() {
            podcast.funding = vec![Funding {
                url: "https://liberapay.com/pod".to_string(),
                title: "Unterstützen".to_string(),
            }];
        }
        site::save_config(dir.path(), &config).unwrap();
        let config = site::open(dir.path()).unwrap().config;
        let podcast = config.podcast.unwrap();
        assert_eq!(podcast.guid.as_deref(), Some(guid.as_str()));
        let text = std::fs::read_to_string(dir.path().join(site::CONFIG_FILE)).unwrap();
        assert!(text.contains("[[podcast.funding]]"), "{text}");

        // Unknown categories are refused.
        let mut config = site::open(dir.path()).unwrap().config;
        if let Some(podcast) = config.podcast.as_mut() {
            podcast.categories = vec!["Cooking".to_string()];
        }
        assert!(site::save_config(dir.path(), &config).is_err());
    }

    #[test]
    fn shows_season_number_and_chapters_on_episode_pages() {
        let dir = podcast_site();
        let page = create_episode(dir.path(), "Kapitel", None).unwrap();
        let mut front = page.content.front.clone();
        if let Some(episode) = front.episode.as_mut() {
            episode.season = Some(2);
            episode.kind = EpisodeType::Trailer;
            episode.chapters = vec![Chapter {
                start: "00:01:30".to_string(),
                title: "Thema".to_string(),
                link: Some("/".to_string()),
            }];
        }
        site::save_front_matter(dir.path(), &page.file, front).unwrap();

        let site = site::open(dir.path()).unwrap();
        let out = dir.path().join("_site");
        let report = crate::generator::build_into(&site, &out).unwrap();
        let html = std::fs::read_to_string(out.join(&page.path).join("index.html")).unwrap();
        assert!(html.contains("Staffel 2 · Folge 1 · Trailer · "), "{html}");
        assert!(html.contains("<h2>Kapitel</h2>"), "{html}");
        assert!(html.contains(r#"<a href="../../">Thema</a>"#), "{html}");
        assert!(
            report.notes.iter().any(|note| note.contains("no cover")),
            "{:?}",
            report.notes
        );
        assert!(out.join("episodes/index.html").is_file());
    }
}

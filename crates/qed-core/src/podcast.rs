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

/// The feed of one audio format, at the address jekyll-octopod gave it
/// (`episodes.mp3.rss`), so podcasts that move over keep their
/// subscribers without a redirect.
pub fn feed_file(format: AudioFormat) -> String {
    format!("{EPISODES_PATH}.{}.rss", format.extension())
}

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
    /// The audio formats every episode comes in, each with a feed of its
    /// own; the first is the main one, whose feed goes to Apple and the
    /// directories. MP3 if empty.
    #[serde(default = "default_formats")]
    pub formats: Vec<AudioFormat>,
    /// Where the audio files are kept.
    #[serde(default)]
    pub audio_files: AudioFiles,
    /// For audio kept elsewhere: the address the files are under, such as
    /// `https://cdn.example.com/podcast/`. New episodes' audio addresses
    /// start with it.
    pub audio_base_url: Option<String>,
    /// The podcast's team: contributors and their roles, someone in
    /// several roles once per role. The feeds name them for the whole
    /// podcast, and new episodes start with them.
    #[serde(default)]
    pub people: Vec<crate::contributors::EpisodePerson>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum PodcastType {
    #[default]
    Episodic,
    Serial,
}

fn default_formats() -> Vec<AudioFormat> {
    vec![AudioFormat::Mp3]
}

impl Podcast {
    /// The formats, the main one first; MP3 if none are set.
    pub fn formats(&self) -> Vec<AudioFormat> {
        if self.formats.is_empty() {
            default_formats()
        } else {
            self.formats.clone()
        }
    }

    /// The main format: its feed is the one for Apple and the directories.
    pub fn main_format(&self) -> AudioFormat {
        self.formats.first().copied().unwrap_or(AudioFormat::Mp3)
    }
}

/// The audio formats a podcast can offer. MP3 and M4A play everywhere;
/// Ogg (Vorbis), Opus and FLAC in many open apps and in browsers, but not
/// in Apple Podcasts or Spotify.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Mp3,
    M4a,
    Ogg,
    Opus,
    Flac,
}

impl AudioFormat {
    pub const ALL: [AudioFormat; 5] = [
        AudioFormat::Mp3,
        AudioFormat::M4a,
        AudioFormat::Ogg,
        AudioFormat::Opus,
        AudioFormat::Flac,
    ];

    /// The file extension, also used in the feed's name.
    pub fn extension(self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::M4a => "m4a",
            AudioFormat::Ogg => "ogg",
            AudioFormat::Opus => "opus",
            AudioFormat::Flac => "flac",
        }
    }

    /// How people call it: "MP3", "Ogg".
    pub fn label(self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "MP3",
            AudioFormat::M4a => "M4A",
            AudioFormat::Ogg => "Ogg",
            AudioFormat::Opus => "Opus",
            AudioFormat::Flac => "FLAC",
        }
    }

    /// The MIME type for feeds and players.
    pub fn mime_type(self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "audio/mpeg",
            // Apple's name for it; podcast apps all know it.
            AudioFormat::M4a => "audio/x-m4a",
            AudioFormat::Ogg | AudioFormat::Opus => "audio/ogg",
            AudioFormat::Flac => "audio/flac",
        }
    }

    /// Whether every podcast app plays it, Apple Podcasts and Spotify
    /// included.
    pub fn plays_everywhere(self) -> bool {
        matches!(self, AudioFormat::Mp3 | AudioFormat::M4a)
    }

    /// The format a file name or address ends in, if it's one of these.
    pub fn of(name: &str) -> Option<AudioFormat> {
        let path = name.split(['?', '#']).next().unwrap_or_default();
        let extension = path.rsplit_once('.')?.1.to_lowercase();
        AudioFormat::ALL.into_iter().find(|format| {
            format.extension() == extension || (extension == "oga" && *format == AudioFormat::Ogg)
        })
    }
}

/// One audio file of an episode, `[[episode.audio]]`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct EpisodeAudio {
    pub format: AudioFormat,
    /// A file in `audio/` (`episode1.mp3`), or a web address for audio
    /// kept elsewhere.
    pub file: String,
    /// Size in bytes, as the feed needs it. Read from the file when it's
    /// chosen; for audio elsewhere, given by hand.
    pub length: Option<u64>,
}

impl EpisodeAudio {
    /// The file's name in `audio/`, if it's one (and not a web address).
    pub fn local(&self) -> Option<&str> {
        Some(self.file.trim()).filter(|file| !file.is_empty() && !file.contains("://"))
    }
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
    /// The audio, one file per format of the podcast.
    #[serde(default)]
    pub audio: Vec<EpisodeAudio>,
    /// How long the episode is (`01:24:22`), the same in every format.
    /// Read from the first audio file chosen; for audio elsewhere, given
    /// by hand.
    pub duration: Option<String>,
    #[serde(default)]
    pub chapters: Vec<Chapter>,
    /// Who's in it and what they do; see `contributors.rs`.
    #[serde(default)]
    pub people: Vec<crate::contributors::EpisodePerson>,
    /// What's said, with times: a WebVTT file in `transcripts/`. Apps
    /// show it (`podcast:transcript`), and so will the web player.
    pub transcript: Option<String>,
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
        if let Some(duration) = self.duration.as_deref()
            && start_millis(duration).is_none()
        {
            return Some(format!(
                "the length “{duration}” isn't a time like 01:24:22 or 29:55"
            ));
        }
        if let Some(problem) = crate::contributors::duplicate(&self.people) {
            return Some(problem);
        }
        if let Some(file) = self.transcript.as_deref().map(str::trim)
            && (file.contains(['/', '\\']) || file.starts_with('.') || !file.ends_with(".vtt"))
        {
            return Some(format!(
                "the transcript “{file}” must be a .vtt file in {TRANSCRIPTS_DIR}/"
            ));
        }
        for (index, audio) in self.audio.iter().enumerate() {
            let label = audio.format.label();
            if self.audio[..index].iter().any(|a| a.format == audio.format) {
                return Some(format!("the episode has two {label} files; one is enough"));
            }
            if audio.file.trim().is_empty() {
                return Some(format!("the {label} audio needs a file or a web address"));
            }
            if let Some(file) = audio.local() {
                if file.contains('/') || file.contains('\\') || file.starts_with('.') {
                    return Some(format!(
                        "the audio “{file}” must be a file in {AUDIO_DIR}/ or a web address"
                    ));
                }
                if AudioFormat::of(file) != Some(audio.format) {
                    return Some(format!("{file} isn't a {label} file"));
                }
            }
        }
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

impl Episode {
    /// The episode's audio in `format`, if it has one.
    pub fn audio_in(&self, format: AudioFormat) -> Option<&EpisodeAudio> {
        self.audio.iter().find(|audio| audio.format == format)
    }

    /// The names of its audio files in `audio/`.
    pub fn local_audio(&self) -> impl Iterator<Item = &str> {
        self.audio.iter().filter_map(EpisodeAudio::local)
    }

    /// The podcast's formats this episode has no audio for yet.
    pub fn missing_formats(&self, podcast: &Podcast) -> Vec<AudioFormat> {
        podcast
            .formats()
            .into_iter()
            .filter(|format| self.audio_in(*format).is_none())
            .collect()
    }
}

/// The published episodes that lack audio in one of the podcast's
/// formats, as "/episodes/x: Ogg" lines; publishing waits until there
/// are none, so every feed has every episode. Drafts don't count.
pub fn incomplete_episodes(site: &Site) -> Vec<String> {
    let Some(podcast) = &site.config.podcast else {
        return Vec::new();
    };
    episodes(site)
        .filter(|page| !page.content.front.draft)
        .filter_map(|page| {
            let missing = page
                .content
                .front
                .episode
                .as_ref()?
                .missing_formats(podcast);
            (!missing.is_empty()).then(|| {
                let labels: Vec<&str> = missing.iter().map(|f| f.label()).collect();
                format!("/{}: no {} audio", page.path, labels.join(", "))
            })
        })
        .collect()
}

/// Where the site keeps its audio files.
pub const AUDIO_DIR: &str = "audio";

/// Where the site keeps its transcripts, and where the built site serves
/// them: `/transcripts/episode1.vtt`.
pub const TRANSCRIPTS_DIR: &str = "transcripts";

/// One line of a transcript, in the Podlove Web Player's format.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cue {
    pub start: String,
    pub start_ms: u64,
    pub end: String,
    pub end_ms: u64,
    /// The contributor's id, when the voice is one of them; the player
    /// shows their photo.
    pub speaker: String,
    /// The speaker's name as the file has it (`<v Hannah>`).
    pub voice: String,
    pub text: String,
}

/// The cues of a WebVTT file: each block with a `start --> end` line
/// becomes one, a `<v Name>` voice tag its speaker. Not all of WebVTT
/// (no styling, regions or nested tags), just what transcripts use.
/// `speaker` turns a name into a contributor's id, if they're one.
pub fn transcript_cues(text: &str, speaker: impl Fn(&str) -> Option<String>) -> Vec<Cue> {
    let text = text.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    text.split("\n\n")
        .filter_map(|block| {
            let mut lines = block.lines().skip_while(|line| !line.contains("-->"));
            let timing = lines.next()?;
            let (start, end) = timing.split_once("-->")?;
            // Settings after the end time (`align:start`) aren't times.
            let start = start.trim();
            let end = end.split_whitespace().next()?;
            let (start_ms, end_ms) = (start_millis(start)?, start_millis(end)?);
            let words = lines.collect::<Vec<_>>().join(" ");
            let (voice, words) = match words
                .trim()
                .strip_prefix("<v")
                .and_then(|rest| rest.split_once('>'))
            {
                Some((name, rest)) => (name.trim().to_string(), rest.replace("</v>", "")),
                None => (String::new(), words.trim().to_string()),
            };
            Some(Cue {
                start: clock_millis(start_ms),
                start_ms,
                end: clock_millis(end_ms),
                end_ms,
                speaker: speaker(&voice).unwrap_or_default(),
                voice,
                text: words.trim().to_string(),
            })
        })
        .collect()
}

/// Milliseconds as `01:02:03.500`.
pub fn clock_millis(millis: u64) -> String {
    format!("{}.{:03}", clock(millis / 1000), millis % 1000)
}

/// Whether `text` is a WebVTT file: it starts with `WEBVTT` (after a
/// byte order mark, if any).
pub fn is_webvtt(text: &str) -> bool {
    let text = text.trim_start_matches('\u{feff}');
    text.strip_prefix("WEBVTT")
        .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '\t', '\n', '\r']))
}

/// Where the built site serves them: `/episodes/episode1.mp3`, as
/// jekyll-octopod did, so the addresses in old feeds keep working.
pub const AUDIO_URL_DIR: &str = EPISODES_PATH;

/// Refuses to publish while an episode lacks a format: every feed must
/// have every episode, or listeners of one format would miss some.
pub fn check_publish(site: &Site) -> Result<(), SiteError> {
    let incomplete = incomplete_episodes(site);
    if incomplete.is_empty() {
        return Ok(());
    }
    Err(SiteError::Invalid {
        path: PathBuf::from(site::PAGES_DIR).join(EPISODES_PATH),
        message: format!(
            "not published, because every feed needs every episode: {}. Add the audio, or make these episodes drafts.",
            incomplete.join("; ")
        ),
    })
}

/// What the feed needs to know about an audio file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AudioInfo {
    /// Size in bytes.
    pub length: u64,
    pub format: AudioFormat,
    /// `01:24:22`.
    pub duration: String,
}

/// Reads size, type and length of an audio file. The length comes from the
/// file's header (or, for some MP3s, is worked out from the bitrate), so
/// this is quick even for long episodes.
pub fn audio_info(path: &Path) -> Result<AudioInfo, String> {
    use lofty::file::{AudioFile, FileType, TaggedFileExt};

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let length = std::fs::metadata(path)
        .map_err(|e| format!("can't read {name}: {e}"))?
        .len();
    let unreadable =
        |e: &dyn std::fmt::Display| format!("{name} isn't an audio file q.e.d. can read ({e})");
    // By its content, not its name, so a misnamed file is caught below.
    let file = lofty::probe::Probe::open(path)
        .map_err(|e| unreadable(&e))?
        .guess_file_type()
        .map_err(|e| unreadable(&e))?
        .read()
        .map_err(|e| unreadable(&e))?;
    let format = match file.file_type() {
        FileType::Mpeg => AudioFormat::Mp3,
        FileType::Mp4 => AudioFormat::M4a,
        FileType::Vorbis => AudioFormat::Ogg,
        FileType::Opus => AudioFormat::Opus,
        FileType::Flac => AudioFormat::Flac,
        _ => {
            return Err(format!(
                "{name}: podcasts use MP3, M4A, Ogg, Opus or FLAC; this is none of them"
            ));
        }
    };
    if AudioFormat::of(&name) != Some(format) {
        return Err(format!(
            "{name} is {} audio; rename it to end in .{}",
            format.label(),
            format.extension()
        ));
    }
    Ok(AudioInfo {
        length,
        format,
        duration: clock(file.properties().duration().as_secs()),
    })
}

/// Seconds as `01:02:03`.
pub fn clock(seconds: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

/// The first line of the `.gitignore` q.e.d. puts into `audio/`, to tell
/// it from one written by hand.
const AUDIO_IGNORE_MARK: &str = "# Audio files stay out of the site's history";

/// Keeps `audio/` out of the site's history, or puts it back in, as the
/// podcast's `audio_files` setting says: a `.gitignore` in the folder that
/// ignores everything, itself included. A `.gitignore` written by hand is
/// left alone.
pub fn sync_audio_ignore(root: &Path, podcast: Option<&Podcast>) -> Result<(), SiteError> {
    let dir = root.join(AUDIO_DIR);
    let ignore = dir.join(".gitignore");
    let ours = std::fs::read_to_string(&ignore)
        .map(|text| text.starts_with(AUDIO_IGNORE_MARK))
        .unwrap_or(false);
    let write_error = |path: &Path| {
        let path = path.to_path_buf();
        move |source| SiteError::Write { path, source }
    };
    match podcast.map(|p| p.audio_files) {
        Some(AudioFiles::Local) if !ignore.exists() => {
            std::fs::create_dir_all(&dir).map_err(write_error(&dir))?;
            std::fs::write(
                &ignore,
                format!("{AUDIO_IGNORE_MARK} (Podcast tab → Audio files).\n*\n"),
            )
            .map_err(write_error(&ignore))
        }
        Some(AudioFiles::InHistory) if ours => {
            std::fs::remove_file(&ignore).map_err(write_error(&ignore))
        }
        _ => Ok(()),
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
        if let Some(problem) = crate::contributors::duplicate(&self.people) {
            return Some(format!("team: {problem}"));
        }
        if let Some(base) = self.audio_base_url.as_deref()
            && !(base.starts_with("https://") || base.starts_with("http://"))
        {
            return Some(format!(
                "the address of the audio files must start with https://, not “{base}”"
            ));
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
    pub download: &'static str,
    /// "Mitwirkende": the people section and the contributors' page.
    pub contributors: &'static str,
    /// The heading of the transcript on an episode page.
    pub transcript: &'static str,
    /// The share row (#053): its label, the email link, the copy
    /// button before and after copying, the device's share menu, and the
    /// question for the reader's Mastodon server.
    pub share: &'static str,
    pub email: &'static str,
    pub copy_link: &'static str,
    pub copied: &'static str,
    pub more: &'static str,
    pub mastodon_server: &'static str,
}

impl Words {
    /// For `de`, `de-AT`, …; English for languages q.e.d. has no words for.
    pub fn for_language(language: &str) -> Words {
        let main = language.trim().to_lowercase();
        match main.split(['-', '_']).next().unwrap_or_default() {
            "de" => Words {
                episodes: "Folgen",
                season: "Staffel",
                episode: "Folge",
                trailer: "Trailer",
                bonus: "Bonus",
                chapters: "Kapitel",
                download: "Herunterladen",
                contributors: "Mitwirkende",
                transcript: "Transkript",
                share: "Teilen",
                email: "E-Mail",
                copy_link: "Link kopieren",
                copied: "Kopiert",
                more: "Mehr …",
                mastodon_server: "Dein Mastodon-Server, z. B. mastodon.social",
            },
            "fr" => Words {
                episodes: "Épisodes",
                season: "Saison",
                episode: "Épisode",
                trailer: "Bande-annonce",
                bonus: "Bonus",
                chapters: "Chapitres",
                download: "Télécharger",
                contributors: "Contributeurs",
                transcript: "Transcription",
                share: "Partager",
                email: "E-mail",
                copy_link: "Copier le lien",
                copied: "Copié",
                more: "Plus …",
                mastodon_server: "Votre serveur Mastodon, p. ex. mastodon.social",
            },
            "es" => Words {
                episodes: "Episodios",
                season: "Temporada",
                episode: "Episodio",
                trailer: "Tráiler",
                bonus: "Extra",
                chapters: "Capítulos",
                download: "Descargar",
                contributors: "Participantes",
                transcript: "Transcripción",
                share: "Compartir",
                email: "Correo",
                copy_link: "Copiar enlace",
                copied: "Copiado",
                more: "Más …",
                mastodon_server: "Tu servidor de Mastodon, p. ej. mastodon.social",
            },
            "it" => Words {
                episodes: "Episodi",
                season: "Stagione",
                episode: "Episodio",
                trailer: "Trailer",
                bonus: "Extra",
                chapters: "Capitoli",
                download: "Scarica",
                contributors: "Partecipanti",
                transcript: "Trascrizione",
                share: "Condividi",
                email: "E-mail",
                copy_link: "Copia link",
                copied: "Copiato",
                more: "Altro …",
                mastodon_server: "Il tuo server Mastodon, ad es. mastodon.social",
            },
            "nl" => Words {
                episodes: "Afleveringen",
                season: "Seizoen",
                episode: "Aflevering",
                trailer: "Trailer",
                bonus: "Bonus",
                chapters: "Hoofdstukken",
                download: "Downloaden",
                contributors: "Medewerkers",
                transcript: "Transcript",
                share: "Delen",
                email: "E-mail",
                copy_link: "Link kopiëren",
                copied: "Gekopieerd",
                more: "Meer …",
                mastodon_server: "Je Mastodon-server, bijv. mastodon.social",
            },
            _ => Words {
                episodes: "Episodes",
                season: "Season",
                episode: "Episode",
                trailer: "Trailer",
                bonus: "Bonus",
                chapters: "Chapters",
                download: "Download",
                contributors: "Contributors",
                transcript: "Transcript",
                share: "Share",
                email: "Email",
                copy_link: "Copy link",
                copied: "Copied",
                more: "More …",
                mastodon_server: "Your Mastodon server, e.g. mastodon.social",
            },
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
            message: "the site has no podcast; turn it on in the Podcast tab first".to_string(),
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

    // The podcast's team, to change as the episode needs.
    let people = site
        .config
        .podcast
        .as_ref()
        .map(|podcast| podcast.people.clone())
        .unwrap_or_default();
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
            people,
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

/// Makes a site a podcast from the start: the podcast switched on, and the
/// latest episode and the Subscribe button on the start page, after its
/// Hero. For new sites; later, the Podcast tab switches it on.
pub fn start_podcast(root: &Path) -> Result<(), SiteError> {
    let site = site::open(root)?;
    let mut config = site.config.clone();
    if config.podcast.is_none() {
        config.podcast = Some(Podcast {
            formats: default_formats(),
            ..Podcast::default()
        });
        site::save_config(root, &config)?;
    }
    if let Some(home) = site.pages.iter().find(|page| page.path.is_empty()) {
        let has = |wanted: &Block| home.content.blocks.iter().any(|b| b == wanted);
        let subscribe = Block::Subscribe(crate::block::Subscribe {
            align: Some(crate::block::Align::Center),
        });
        if !has(&Block::LatestEpisode) && !has(&subscribe) {
            let mut blocks = home.content.blocks.clone();
            let after_hero = usize::from(matches!(blocks.first(), Some(Block::Hero(_))));
            blocks.splice(after_hero..after_hero, [Block::LatestEpisode, subscribe]);
            site::save_blocks(root, &home.file, blocks)?;
        }
    }
    Ok(())
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

        // The address of audio elsewhere must be a web address.
        let mut config = site::open(dir.path()).unwrap().config;
        if let Some(podcast) = config.podcast.as_mut() {
            podcast.audio_base_url = Some("cdn.example.com/pod".to_string());
        }
        assert!(site::save_config(dir.path(), &config).is_err());

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

        // Share links, with the page's full address.
        let url = format!("https://pod.example/{}/", page.path);
        assert!(
            html.contains(r#"<nav id="share" aria-label="Teilen""#),
            "{html}"
        );
        assert!(
            html.contains(&format!(
                "https://bsky.app/intent/compose?text=Kapitel%20{}",
                url.replace(':', "%3A").replace('/', "%2F")
            )),
            "{html}"
        );
        assert!(
            html.contains(r#"href="mailto:?subject=Kapitel&#38;body=https%3A%2F%2Fpod.example%2F"#),
            "{html}"
        );
        assert!(html.contains(r#"hidden data-share="copy""#), "{html}");
        assert!(
            html.contains(r#"<script src="../../share.js" defer></script>"#),
            "{html}"
        );
        assert!(out.join("share.js").is_file());
    }

    #[test]
    fn builds_episode_pages_from_the_episode_layout() {
        use crate::block::{Block, DEFAULT_EPISODE_LAYOUT};
        let dir = podcast_site();
        let page = create_episode(dir.path(), "Gespräch", None).unwrap();
        let text = |markdown: &str| Block::Text {
            markdown: markdown.to_string(),
        };
        site::save_blocks(dir.path(), &page.file, vec![text("Unsere Shownotes.")]).unwrap();
        let transcripts = dir.path().join(TRANSCRIPTS_DIR);
        std::fs::create_dir_all(&transcripts).unwrap();
        std::fs::write(
            transcripts.join("one.vtt"),
            "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v Hannah>Hallo\n\n\
             00:00:02.000 --> 00:00:03.000\n<v Hannah>und willkommen.\n\n\
             00:00:03.000 --> 00:00:04.000\n<v Max>Servus!\n",
        )
        .unwrap();
        let mut front = page.content.front.clone();
        if let Some(episode) = front.episode.as_mut() {
            episode.transcript = Some("one.vtt".to_string());
            episode.chapters = vec![Chapter {
                start: "00:00:01".to_string(),
                title: "Anfang".to_string(),
                link: None,
            }];
        }
        site::save_front_matter(dir.path(), &page.file, front).unwrap();

        // The default layout needs no file.
        assert!(
            site::save_episode_layout(dir.path(), DEFAULT_EPISODE_LAYOUT.to_vec())
                .unwrap()
                .is_none()
        );
        assert!(!dir.path().join(site::EPISODE_LAYOUT_FILE).exists());

        let layout = vec![
            text("Vor jeder Folge."),
            Block::EpisodeShownotes,
            Block::EpisodeTranscript,
            Block::EpisodeChapters,
        ];
        site::save_episode_layout(dir.path(), layout.clone()).unwrap();
        let written = std::fs::read_to_string(dir.path().join(site::EPISODE_LAYOUT_FILE)).unwrap();
        assert!(written.contains("::: episode-transcript\n:::"), "{written}");
        let site = site::open(dir.path()).unwrap();
        assert_eq!(site.episode_layout.as_ref().unwrap().content.blocks, layout);

        let out = dir.path().join("_site");
        crate::generator::build_into(&site, &out).unwrap();
        let html = std::fs::read_to_string(out.join(&page.path).join("index.html")).unwrap();
        let at = |needle: &str| {
            html.find(needle)
                .unwrap_or_else(|| panic!("no {needle} in {html}"))
        };
        assert!(at("Vor jeder Folge.") < at("Unsere Shownotes."));
        assert!(at("Unsere Shownotes.") < at(r#"<details id="transcript""#));
        assert!(at(r#"<details id="transcript""#) < at("<h2>Kapitel</h2>"));
        // One voice's cues in a row make a paragraph.
        assert!(
            html.contains("Hannah:</span>Hallo und willkommen.</p>"),
            "{html}"
        );
        assert!(html.contains("Max:</span>Servus!</p>"), "{html}");
        // Parts left out of the layout are left out of the page.
        assert!(!html.contains(r#"id="share""#), "{html}");
        // Pages that aren't episodes don't get the layout.
        let home = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(!home.contains("Vor jeder Folge."), "{home}");
    }

    /// About one second of silent MP3: 39 frames (MPEG-1 Layer III,
    /// 128 kbit/s, 44.1 kHz) of 417 bytes, each a 4-byte header and zeros.
    fn mp3() -> Vec<u8> {
        let mut frame = vec![0xFF, 0xFB, 0x90, 0x64];
        frame.resize(417, 0);
        frame.repeat(39)
    }

    #[test]
    fn reads_size_format_and_length_of_audio() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("one.mp3");
        std::fs::write(&file, mp3()).unwrap();
        let info = audio_info(&file).unwrap();
        assert_eq!(info.length, 39 * 417);
        assert_eq!(info.format, AudioFormat::Mp3);
        assert_eq!(info.duration, "00:00:01");

        // The name must say what it is.
        std::fs::write(dir.path().join("one.ogg"), mp3()).unwrap();
        let error = audio_info(&dir.path().join("one.ogg")).unwrap_err();
        assert!(error.contains("rename it to end in .mp3"), "{error}");
        std::fs::write(dir.path().join("fake.mp3"), b"not audio").unwrap();
        assert!(audio_info(&dir.path().join("fake.mp3")).is_err());
        assert_eq!(clock(5062), "01:24:22");
        assert_eq!(
            AudioFormat::of("https://cdn.example/e1.OGG?x=1"),
            Some(AudioFormat::Ogg)
        );
        assert_eq!(feed_file(AudioFormat::Flac), "episodes.flac.rss");
    }

    #[test]
    fn keeps_local_audio_out_of_the_history() {
        let dir = podcast_site();
        let ignore = dir.path().join(AUDIO_DIR).join(".gitignore");
        // Local is the default: saving the settings wrote it.
        assert!(std::fs::read_to_string(&ignore).unwrap().contains("*"));

        let mut config = site::open(dir.path()).unwrap().config;
        if let Some(podcast) = config.podcast.as_mut() {
            podcast.audio_files = AudioFiles::InHistory;
        }
        site::save_config(dir.path(), &config).unwrap();
        assert!(!ignore.exists());

        // One written by hand stays.
        std::fs::write(&ignore, "*.wav\n").unwrap();
        if let Some(podcast) = config.podcast.as_mut() {
            podcast.audio_files = AudioFiles::Local;
        }
        site::save_config(dir.path(), &config).unwrap();
        assert_eq!(std::fs::read_to_string(&ignore).unwrap(), "*.wav\n");
    }

    fn audio(format: AudioFormat, file: &str) -> EpisodeAudio {
        EpisodeAudio {
            format,
            file: file.to_string(),
            length: None,
        }
    }

    #[test]
    fn puts_audio_on_episode_pages_at_octopods_addresses() {
        let dir = podcast_site();
        let source = dir.path().join("one.mp3");
        std::fs::write(&source, mp3()).unwrap();
        let name = crate::media::import(dir.path(), crate::media::Library::Audio, &source).unwrap();
        assert_eq!(name, "one.mp3");

        let page = create_episode(dir.path(), "Mit Ton", None).unwrap();
        let silent = create_episode(dir.path(), "Ohne Ton", None).unwrap();
        let mut front = page.content.front.clone();
        if let Some(episode) = front.episode.as_mut() {
            episode.audio = vec![
                audio(AudioFormat::Ogg, "https://cdn.example/one.ogg"),
                audio(AudioFormat::Mp3, &name),
            ];
            episode.duration = Some("00:00:01".to_string());
        }
        site::save_front_matter(dir.path(), &page.file, front).unwrap();
        let text = std::fs::read_to_string(dir.path().join(&page.file)).unwrap();
        assert!(
            text.contains("[[episode.audio]]\nformat = \"ogg\""),
            "{text}"
        );

        let site = site::open(dir.path()).unwrap();
        // The episode uses the file, so it can't be deleted.
        assert!(crate::media::delete(&site, crate::media::Library::Audio, &name).is_err());

        let out = dir.path().join("_site");
        let report = crate::generator::build_into(&site, &out).unwrap();
        assert!(out.join("episodes/one.mp3").is_file());
        assert!(!out.join("episodes/.gitignore").exists());
        let html = std::fs::read_to_string(out.join(&page.path).join("index.html")).unwrap();
        // The podcast's main format (MP3) first, whatever the file's order.
        let mp3_at = html.find(r#"<source src="../../episodes/one.mp3" type="audio/mpeg">"#);
        let ogg_at = html.find(r#"<source src="https://cdn.example/one.ogg" type="audio/ogg">"#);
        assert!(
            mp3_at.is_some() && ogg_at.is_some() && mp3_at < ogg_at,
            "{html}"
        );
        assert!(html.contains("MP3, 16 KB</a>"), "{html}");
        assert!(html.contains("<span>00:01</span>"), "{html}");
        assert!(
            report
                .notes
                .iter()
                .any(|note| note.contains(&silent.path) && note.contains("no audio")),
            "{:?}",
            report.notes
        );

        // Wrong kind of file for audio/.
        let text = dir.path().join("notes.txt");
        std::fs::write(&text, "hi").unwrap();
        assert!(crate::media::import(dir.path(), crate::media::Library::Audio, &text).is_err());
    }

    #[test]
    fn publishes_only_when_every_episode_has_every_format() {
        let dir = podcast_site();
        let mut config = site::open(dir.path()).unwrap().config;
        if let Some(podcast) = config.podcast.as_mut() {
            podcast.formats = vec![AudioFormat::Mp3, AudioFormat::Ogg];
        }
        site::save_config(dir.path(), &config).unwrap();
        let toml = std::fs::read_to_string(dir.path().join(site::CONFIG_FILE)).unwrap();
        assert!(toml.contains(r#"formats = ["mp3", "ogg"]"#), "{toml}");

        let page = create_episode(dir.path(), "Halb", None).unwrap();
        let mut front = page.content.front.clone();
        if let Some(episode) = front.episode.as_mut() {
            episode.audio = vec![audio(AudioFormat::Mp3, "https://cdn.example/halb.mp3")];
        }
        site::save_front_matter(dir.path(), &page.file, front.clone()).unwrap();

        let error = check_publish(&site::open(dir.path()).unwrap()).unwrap_err();
        assert!(error.to_string().contains("no Ogg audio"), "{error}");
        let report = crate::generator::build(&site::open(dir.path()).unwrap()).unwrap();
        assert!(
            report.notes.iter().any(|n| n.contains("no Ogg audio yet")),
            "{:?}",
            report.notes
        );

        // A draft doesn't hold publishing up.
        front.draft = true;
        site::save_front_matter(dir.path(), &page.file, front.clone()).unwrap();
        assert!(check_publish(&site::open(dir.path()).unwrap()).is_ok());

        // With both formats it's complete.
        front.draft = false;
        if let Some(episode) = front.episode.as_mut() {
            episode
                .audio
                .push(audio(AudioFormat::Ogg, "https://cdn.example/halb.ogg"));
        }
        site::save_front_matter(dir.path(), &page.file, front.clone()).unwrap();
        assert!(check_publish(&site::open(dir.path()).unwrap()).is_ok());

        // Two files of one format, or a file of the wrong one, are refused.
        if let Some(episode) = front.episode.as_mut() {
            episode
                .audio
                .push(audio(AudioFormat::Ogg, "https://cdn.example/b.ogg"));
        }
        assert!(site::save_front_matter(dir.path(), &page.file, front.clone()).is_err());
        if let Some(episode) = front.episode.as_mut() {
            episode.audio = vec![audio(AudioFormat::Ogg, "halb.mp3")];
        }
        assert!(site::save_front_matter(dir.path(), &page.file, front).is_err());
    }

    #[test]
    fn writes_a_feed_per_format_with_what_apps_read() {
        let dir = podcast_site();
        let mut config = site::open(dir.path()).unwrap().config;
        if let Some(podcast) = config.podcast.as_mut() {
            podcast.formats = vec![AudioFormat::Mp3, AudioFormat::Ogg];
            podcast.categories = vec!["Society & Culture > Documentary".to_string()];
            podcast.owner_email = Some("pod@example.com".to_string());
            podcast.cover = Some("cover.jpg".to_string());
            podcast.funding = vec![Funding {
                url: "https://liberapay.com/pod".to_string(),
                title: "Unterstützen".to_string(),
            }];
        }
        site::save_config(dir.path(), &config).unwrap();
        std::fs::write(dir.path().join("one.mp3"), mp3()).unwrap();
        crate::media::import(
            dir.path(),
            crate::media::Library::Audio,
            &dir.path().join("one.mp3"),
        )
        .unwrap();

        let zero = create_episode(dir.path(), "Null & Nichts", Some(0)).unwrap();
        let mut front = zero.content.front.clone();
        front.summary = Some("Wie alles <begann>".to_string());
        if let Some(episode) = front.episode.as_mut() {
            episode.season = Some(1);
            episode.duration = Some("01:02:03".to_string());
            episode.audio = vec![
                audio(AudioFormat::Mp3, "one.mp3"),
                audio(AudioFormat::Ogg, "https://cdn.example/one.ogg"),
            ];
            episode.chapters = vec![Chapter {
                start: "00:00:00".to_string(),
                title: "Intro".to_string(),
                link: None,
            }];
        }
        site::save_front_matter(dir.path(), &zero.file, front).unwrap();
        site::save_blocks(
            dir.path(),
            &zero.file,
            vec![crate::block::Block::Text {
                markdown: "Siehe [Impressum](/impressum) und ![Bild](/media/a.png)".to_string(),
            }],
        )
        .unwrap();

        let site = site::open(dir.path()).unwrap();
        let out = dir.path().join("_site");
        crate::generator::build_into(&site, &out).unwrap();
        let mp3 = std::fs::read_to_string(out.join("episodes.mp3.rss")).unwrap();
        let ogg = std::fs::read_to_string(out.join("episodes.ogg.rss")).unwrap();
        for expected in [
            "<title>Pod (MP3)</title>",
            r#"<atom:link href="https://pod.example/episodes.mp3.rss" rel="self""#,
            r#"<itunes:category text="Society &amp; Culture"><itunes:category text="Documentary"/></itunes:category>"#,
            "<itunes:email>pod@example.com</itunes:email>",
            r#"<itunes:image href="https://pod.example/media/cover.jpg"/>"#,
            "<itunes:explicit>false</itunes:explicit>",
            "<podcast:guid>",
            r#"<podcast:funding url="https://liberapay.com/pod">Unterstützen</podcast:funding>"#,
            "<title>Null &amp; Nichts</title>",
            "<description>Wie alles &lt;begann&gt;</description>",
            r#"<enclosure url="https://pod.example/episodes/one.mp3" length="16263" type="audio/mpeg"/>"#,
            "<itunes:duration>3723</itunes:duration>",
            "<podcast:episode>0</podcast:episode>",
            "<itunes:season>1</itunes:season>",
            r#"<psc:chapter start="00:00:00" title="Intro"/>"#,
            r#"href="https://pod.example/impressum/""#,
            r#"src="https://pod.example/media/a.png""#,
        ] {
            assert!(mp3.contains(expected), "{expected} missing in\n{mp3}");
        }
        // Episode 0: Apple only takes numbers above 0.
        assert!(!mp3.contains("<itunes:episode>"), "{mp3}");
        assert!(
            ogg.contains(
                r#"<enclosure url="https://cdn.example/one.ogg" length="0" type="audio/ogg"/>"#
            ),
            "{ogg}"
        );
        assert!(ogg.contains("<title>Pod (Ogg)</title>"), "{ogg}");

        // No plain blog feed for the episodes any more, and every page
        // links the podcast feeds.
        assert!(!out.join("episodes/feed.xml").exists());
        let page = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(page.contains(r#"href="episodes.mp3.rss""#), "{page}");
        assert!(page.contains("Podcast feed (Ogg)"), "{page}");
    }

    #[test]
    fn shows_the_latest_episode_and_a_subscribe_button() {
        let dir = podcast_site();
        let mut config = site::open(dir.path()).unwrap().config;
        if let Some(podcast) = config.podcast.as_mut() {
            podcast.formats = vec![AudioFormat::Mp3, AudioFormat::M4a];
            podcast.cover = Some("cover.jpg".to_string());
        }
        site::save_config(dir.path(), &config).unwrap();
        let home = site::open(dir.path())
            .unwrap()
            .pages
            .into_iter()
            .find(|p| p.path.is_empty())
            .unwrap();
        site::save_blocks(
            dir.path(),
            &home.file,
            vec![
                Block::LatestEpisode,
                Block::Subscribe(crate::block::Subscribe {
                    align: Some(crate::block::Align::Center),
                }),
            ],
        )
        .unwrap();
        let text = std::fs::read_to_string(dir.path().join(&home.file)).unwrap();
        assert!(
            text.contains("::: latest-episode\n:::\n\n::: subscribe\nalign = \"center\"\n:::"),
            "{text}"
        );

        for title in ["Erste", "Zweite"] {
            let page = create_episode(dir.path(), title, None).unwrap();
            let mut front = page.content.front.clone();
            front.summary = Some(format!("Über {title}"));
            if let Some(episode) = front.episode.as_mut() {
                episode.audio = vec![audio(
                    AudioFormat::Mp3,
                    &format!("https://cdn.example/{title}.mp3"),
                )];
            }
            site::save_front_matter(dir.path(), &page.file, front).unwrap();
        }

        let site = site::open(dir.path()).unwrap();
        let out = dir.path().join("_site");
        crate::generator::build_into(&site, &out).unwrap();
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        // The newest: both are from today, so the later path wins the tie.
        assert!(
            html.contains("Über Erste") || html.contains("Über Zweite"),
            "{html}"
        );
        assert!(html.contains("Folge"), "{html}");
        assert!(
            html.contains(r#"<source src="https://cdn.example/"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<script src="subscribe-button/podcast-data.js"></script>"#),
            "{html}"
        );
        assert!(
            html.contains(
                r##"src="subscribe-button/javascripts/app.js" data-language="de" data-colors="#"##
            ) && html.contains(r#"data-format="cover""#)
                && html.contains(r#"<div class="subscribe text-center">"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<noscript>"#) && html.contains(r#"href="episodes.m4a.rss""#),
            "{html}"
        );

        assert!(out.join("subscribe-button/javascripts/app.js").is_file());
        assert!(out.join("subscribe-button/button.html").is_file());
        assert!(!out.join("subscribe-button/README.md").exists());
        let data = std::fs::read_to_string(out.join("subscribe-button/podcast-data.js")).unwrap();
        assert!(data.starts_with("window.qedPodcastData = {"), "{data}");
        assert!(
            data.contains(r#""url":"https://pod.example/episodes.mp3.rss""#),
            "{data}"
        );
        assert!(data.contains(r#""format":"aac""#), "{data}");
        assert!(
            data.contains(r#"window.qedPodcastData.cover = new URL("../media/cover.jpg", document.currentScript.src).href;"#),
            "{data}"
        );
    }

    #[test]
    fn starts_a_new_site_as_a_podcast() {
        let dir = tempfile::tempdir().unwrap();
        site::create_site(dir.path(), "Neu", "de").unwrap();
        start_podcast(dir.path()).unwrap();
        let site = site::open(dir.path()).unwrap();
        assert_eq!(
            site.config.podcast.as_ref().map(Podcast::main_format),
            Some(AudioFormat::Mp3)
        );
        let home = site.pages.iter().find(|p| p.path.is_empty()).unwrap();
        assert!(
            matches!(
                home.content.blocks[..],
                [
                    Block::Hero(_),
                    Block::LatestEpisode,
                    Block::Subscribe(_),
                    Block::Text { .. }
                ]
            ),
            "{:?}",
            home.content.blocks
        );
        // Once is enough.
        start_podcast(dir.path()).unwrap();
        let again = site::open(dir.path()).unwrap();
        let home_again = again.pages.iter().find(|p| p.path.is_empty()).unwrap();
        assert_eq!(home_again.content.blocks.len(), 4);
    }

    #[test]
    fn publishes_webvtt_transcripts_for_apps() {
        use crate::media::{self, Library};
        let dir = podcast_site();
        let vtt = dir.path().join("eins.vtt");
        std::fs::write(
            &vtt,
            "WEBVTT\n\n00:00.000 --> 00:02.000\n<v Stefan>Hallo!\n",
        )
        .unwrap();
        assert_eq!(
            media::import(dir.path(), Library::Transcripts, &vtt).unwrap(),
            "eins.vtt"
        );
        let fake = dir.path().join("fake.vtt");
        std::fs::write(&fake, "1\n00:00:00,000 --> 00:00:02,000\nHallo\n").unwrap();
        assert!(media::import(dir.path(), Library::Transcripts, &fake).is_err());

        let page = create_episode(dir.path(), "Eins", None).unwrap();
        let mut front = page.content.front.clone();
        if let Some(episode) = front.episode.as_mut() {
            episode.transcript = Some("eins.vtt".to_string());
            episode.audio = vec![audio(AudioFormat::Mp3, "https://cdn.example/eins.mp3")];
        }
        site::save_front_matter(dir.path(), &page.file, front.clone()).unwrap();
        let site = site::open(dir.path()).unwrap();
        assert!(media::delete(&site, Library::Transcripts, "eins.vtt").is_err());

        let out = dir.path().join("_site");
        crate::generator::build_into(&site, &out).unwrap();
        assert!(out.join("transcripts/eins.vtt").is_file());
        let feed = std::fs::read_to_string(out.join("episodes.mp3.rss")).unwrap();
        assert!(
            feed.contains(r#"<podcast:transcript url="https://pod.example/transcripts/eins.vtt" type="text/vtt" language="de-AT" rel="captions"/>"#),
            "{feed}"
        );

        // Only .vtt files in transcripts/.
        if let Some(episode) = front.episode.as_mut() {
            episode.transcript = Some("eins.srt".to_string());
        }
        assert!(site::save_front_matter(dir.path(), &page.file, front).is_err());
        assert!(is_webvtt("\u{feff}WEBVTT - Folge 1\n"));
        assert!(!is_webvtt("WEBVTTX"));
    }

    #[test]
    fn reads_the_cues_of_a_webvtt_transcript() {
        let vtt = "WEBVTT - Folge 1\n\nNOTE\nnur ein Test\n\n1\n00:00:01.500 --> 00:00:04.000 align:start\n<v Hannah>Hallo,\nStefan!</v>\n\n00:05.000 --> 00:06.000\nOhne Stimme\n";
        let cues = transcript_cues(vtt, |name| (name == "Hannah").then(|| "hannah".to_string()));
        assert_eq!(cues.len(), 2, "{cues:?}");
        assert_eq!(cues[0].start, "00:00:01.500");
        assert_eq!(cues[0].end_ms, 4000);
        assert_eq!(cues[0].voice, "Hannah");
        assert_eq!(cues[0].speaker, "hannah");
        assert_eq!(cues[0].text, "Hallo, Stefan!");
        assert_eq!(cues[1].start, "00:00:05.000");
        assert_eq!(cues[1].voice, "");
    }

    #[test]
    fn hands_the_web_player_everything_about_an_episode() {
        use crate::media::{self, Library};
        let dir = podcast_site();
        let vtt = dir.path().join("eins.vtt");
        std::fs::write(
            &vtt,
            "WEBVTT\n\n00:00.000 --> 00:02.000\n<v Stefan Haslinger>Hallo!\n",
        )
        .unwrap();
        media::import(dir.path(), Library::Transcripts, &vtt).unwrap();
        let stefan = crate::contributors::save(
            dir.path(),
            &crate::contributors::Contributor {
                id: String::new(),
                name: "Stefan Haslinger".to_string(),
                photo: Some("stefan.jpg".to_string()),
                links: Vec::new(),
                bio: String::new(),
            },
        )
        .unwrap();
        std::fs::write(dir.path().join("one.mp3"), mp3()).unwrap();
        media::import(dir.path(), Library::Audio, &dir.path().join("one.mp3")).unwrap();

        let page = create_episode(dir.path(), "Eins", None).unwrap();
        let mut front = page.content.front.clone();
        if let Some(episode) = front.episode.as_mut() {
            episode.audio = vec![audio(AudioFormat::Mp3, "one.mp3")];
            episode.duration = Some("00:00:01".to_string());
            episode.transcript = Some("eins.vtt".to_string());
            episode.chapters = vec![Chapter {
                start: "00:00".to_string(),
                title: "Anfang".to_string(),
                link: None,
            }];
            episode.people = vec![crate::contributors::EpisodePerson {
                contributor: stefan.id.clone(),
                role: crate::contributors::Role::Host,
            }];
        }
        site::save_front_matter(dir.path(), &page.file, front).unwrap();

        let site = site::open(dir.path()).unwrap();
        let out = dir.path().join("_site");
        crate::generator::build_into(&site, &out).unwrap();

        let json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(out.join(&page.path).join("player.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(json["version"], 5);
        assert_eq!(json["title"], "Eins");
        assert_eq!(json["duration"], "00:00:01.000");
        assert_eq!(json["audio"][0]["url"], "../../episodes/one.mp3");
        assert_eq!(json["audio"][0]["mimeType"], "audio/mpeg");
        assert_eq!(json["audio"][0]["size"], 39 * 417);
        assert_eq!(json["chapters"][0]["start"], "00:00:00.000");
        assert_eq!(json["contributors"][0]["avatar"], "../../media/stefan.jpg");
        assert_eq!(json["transcripts"][0]["speaker"], "stefan-haslinger");
        assert_eq!(json["transcripts"][0]["text"], "Hallo!");

        assert!(out.join("podlove-player/embed.js").is_file());
        assert!(
            out.join("podlove-player/5.13.0/player/bootstrap.js")
                .is_file()
        );
        assert!(!out.join("podlove-player/README.md").exists());
        assert!(out.join("player.js").is_file());
        let config: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(out.join("podlove-player/config.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(config["version"], 5);
        assert!(
            config["theme"]["tokens"]["brand"]
                .as_str()
                .unwrap()
                .starts_with('#')
        );

        let html = std::fs::read_to_string(out.join(&page.path).join("index.html")).unwrap();
        assert!(
            html.contains(r#"<div data-podlove-player data-episode="player.json"></div>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<script src="../../podlove-player/embed.js" defer></script>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"class="w-full" data-player-fallback>"#),
            "{html}"
        );
        assert!(
            html.contains(
                r#"<section id="chapters" class="prose max-w-none" data-player-fallback>"#
            ),
            "{html}"
        );
    }
}

//! The podcast feeds: one RSS feed per audio format, at the addresses
//! jekyll-octopod gave them (`episodes.mp3.rss`, `episodes.ogg.rss`), with
//! the tags podcast apps read:
//!
//! - RSS 2.0 for the basics (title, link, enclosure, guid, date);
//! - Apple's `itunes:` tags, which nearly every app and directory uses;
//! - the Podcasting 2.0 `podcast:` namespace (guid, funding, numbers);
//! - Podlove Simple Chapters (`psc:`) for chapter marks.
//!
//! Each feed holds every episode. Feeds split into pages (RFC 5005, as
//! jekyll-octopod could make them) aren't used: Apple Podcasts and most
//! apps read only the first page, so older episodes would go missing.

use crate::calendar::{self, Date};

use crate::podcast::{self, AudioFormat, Episode, EpisodeType, Podcast};
use crate::site::{MEDIA_DIR, Page, SiteConfig};

/// One feed file to write: its name in the built site and its text.
pub struct FeedFile {
    pub file: String,
    pub xml: String,
}

/// What the feeds need from the build besides the site's settings.
pub struct FeedInput<'a> {
    pub config: &'a SiteConfig,
    pub podcast: &'a Podcast,
    /// `https://example.com`, without a slash at the end.
    pub base: &'a str,
    /// The published episodes, newest first, none from the future.
    pub episodes: &'a [&'a Page],
    /// An episode's shownotes as HTML with full links, for
    /// `content:encoded`.
    pub shownotes: &'a dyn Fn(&Page) -> String,
    /// The size of a file in `audio/`, which the file itself knows best.
    pub local_length: &'a dyn Fn(&str) -> Option<u64>,
}

/// Every feed of the podcast: one per format.
pub fn podcast_feeds(input: &FeedInput) -> Vec<FeedFile> {
    input
        .podcast
        .formats()
        .into_iter()
        .map(|format| {
            // Every feed has every episode; Publish makes sure of that, and
            // a preview leaves out what isn't there yet.
            let items: Vec<String> = input
                .episodes
                .iter()
                .filter_map(|page| item(input, page, format))
                .collect();
            FeedFile {
                file: podcast::feed_file(format),
                xml: channel(input, format, &items),
            }
        })
        .collect()
}

fn channel(input: &FeedInput, format: AudioFormat, items: &[String]) -> String {
    let config = input.config;
    let podcast = input.podcast;
    let base = input.base;
    let title = podcast.title.as_deref().unwrap_or(&config.title);
    let description = podcast
        .description
        .as_deref()
        .or(config.description.as_deref())
        .unwrap_or(title);
    let feed_url = format!("{base}/{}", podcast::feed_file(format));

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\" \
         xmlns:content=\"http://purl.org/rss/1.0/modules/content/\" \
         xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\" \
         xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" \
         xmlns:psc=\"http://podlove.org/simple-chapters\">\n<channel>\n",
    );
    let mut line = |text: String| {
        xml.push_str("  ");
        xml.push_str(&text);
        xml.push('\n');
    };
    // With several formats, apps and directories list each feed; the
    // format in the title tells them apart.
    let shown_title = if podcast.formats().len() > 1 {
        format!("{title} ({})", format.label())
    } else {
        title.to_string()
    };
    line(tag("title", &shown_title));
    line(tag("link", &format!("{base}/")));
    line(tag("description", description));
    line(tag("language", &config.language));
    line(tag("generator", "q.e.d."));
    if let Some(copyright) = &podcast.copyright {
        line(tag("copyright", copyright));
    }
    line(format!(
        "<atom:link href=\"{}\" rel=\"self\" type=\"application/rss+xml\"/>",
        escape(&feed_url)
    ));
    if let Some(author) = &podcast.author {
        line(tag("itunes:author", author));
    }
    if podcast.owner_name.is_some() || podcast.owner_email.is_some() {
        let mut owner = String::from("<itunes:owner>");
        if let Some(name) = &podcast.owner_name {
            owner.push_str(&tag("itunes:name", name));
        }
        if let Some(email) = &podcast.owner_email {
            owner.push_str(&tag("itunes:email", email));
        }
        owner.push_str("</itunes:owner>");
        line(owner);
    }
    if let Some(cover) = podcast.cover.as_deref().and_then(|c| image_url(base, c)) {
        line(format!("<itunes:image href=\"{}\"/>", escape(&cover)));
        line(format!(
            "<image>{}{}{}</image>",
            tag("url", &cover),
            tag("title", title),
            tag("link", &format!("{base}/"))
        ));
    }
    for name in &podcast.categories {
        match podcast::category(name) {
            Some((main, Some(sub))) => line(format!(
                "<itunes:category text=\"{}\"><itunes:category text=\"{}\"/></itunes:category>",
                escape(main),
                escape(sub)
            )),
            Some((main, None)) => line(format!("<itunes:category text=\"{}\"/>", escape(main))),
            None => {}
        }
    }
    line(tag(
        "itunes:explicit",
        if podcast.explicit { "true" } else { "false" },
    ));
    line(tag(
        "itunes:type",
        match podcast.kind {
            podcast::PodcastType::Episodic => "episodic",
            podcast::PodcastType::Serial => "serial",
        },
    ));
    if let Some(guid) = &podcast.guid {
        line(tag("podcast:guid", guid));
    }
    for funding in &podcast.funding {
        line(format!(
            "<podcast:funding url=\"{}\">{}</podcast:funding>",
            escape(&funding.url),
            escape(&funding.title)
        ));
    }
    // The newest episode's date, so the same site builds the same feed.
    if let Some(date) = input
        .episodes
        .first()
        .and_then(|page| page.content.front.date.as_ref()?.date())
    {
        line(tag("lastBuildDate", &calendar::rfc822(date)));
    }
    for item in items {
        xml.push_str(item);
    }
    xml.push_str("</channel>\n</rss>\n");
    xml
}

/// One episode in the feed of `format`, or `None` if it has no audio in it.
fn item(input: &FeedInput, page: &Page, format: AudioFormat) -> Option<String> {
    let front = &page.content.front;
    let episode: &Episode = front.episode.as_ref()?;
    let audio = episode.audio_in(format)?;
    let base = input.base;
    let (url, length) = match audio.local() {
        Some(file) => (
            format!("{base}/{}/{file}", podcast::AUDIO_URL_DIR),
            (input.local_length)(file).or(audio.length),
        ),
        None => (audio.file.trim().to_string(), audio.length),
    };
    let link = format!("{base}/{}/", page.path);

    let mut xml = String::from("  <item>\n");
    let mut line = |text: String| {
        xml.push_str("    ");
        xml.push_str(&text);
        xml.push('\n');
    };
    line(tag("title", &front.title));
    line(tag("link", &link));
    line(format!(
        "<guid isPermaLink=\"false\">{}</guid>",
        escape(&episode.guid)
    ));
    if let Some(date) = front.date.as_ref().and_then(|d| d.date()) {
        line(tag("pubDate", &calendar::rfc822(date)));
    }
    if let Some(text) = front.summary.as_deref().or(front.description.as_deref()) {
        line(tag("description", text));
        line(tag("itunes:subtitle", text));
    }
    let shownotes = (input.shownotes)(page);
    if !shownotes.trim().is_empty() {
        // CDATA keeps the HTML readable; a "]]>" inside would end it early.
        line(format!(
            "<content:encoded><![CDATA[{}]]></content:encoded>",
            shownotes.replace("]]>", "]]]]><![CDATA[>")
        ));
    }
    line(format!(
        "<enclosure url=\"{}\" length=\"{}\" type=\"{}\"/>",
        escape(&url),
        length.unwrap_or(0),
        format.mime_type()
    ));
    if let Some(duration) = episode.duration.as_deref().and_then(podcast::start_millis) {
        line(tag("itunes:duration", &(duration / 1000).to_string()));
    }
    if !front.authors.is_empty() {
        line(tag("itunes:author", &front.authors.join(", ")));
    }
    if let Some(season) = episode.season {
        line(tag("itunes:season", &season.to_string()));
        line(tag("podcast:season", &season.to_string()));
    }
    if let Some(number) = episode.number {
        // Apple takes only numbers above 0; Podcasting 2.0 takes 0 too.
        if number > 0 {
            line(tag("itunes:episode", &number.to_string()));
        }
        line(tag("podcast:episode", &number.to_string()));
    }
    if episode.kind != EpisodeType::Full {
        line(tag("itunes:episodeType", episode.kind.as_str()));
    }
    if let Some(explicit) = episode.explicit {
        line(tag(
            "itunes:explicit",
            if explicit { "true" } else { "false" },
        ));
    }
    if let Some(image) = episode.image.as_deref().and_then(|i| image_url(base, i)) {
        line(format!("<itunes:image href=\"{}\"/>", escape(&image)));
    }
    if !episode.chapters.is_empty() {
        let mut chapters = String::from("<psc:chapters version=\"1.2\">");
        for chapter in &episode.chapters {
            chapters.push_str(&format!(
                "<psc:chapter start=\"{}\" title=\"{}\"",
                escape(chapter.start.trim()),
                escape(chapter.title.trim())
            ));
            if let Some(href) = chapter.link.as_deref().filter(|l| l.contains("://")) {
                chapters.push_str(&format!(" href=\"{}\"", escape(href)));
            }
            chapters.push_str("/>");
        }
        chapters.push_str("</psc:chapters>");
        line(chapters);
    }
    xml.push_str("  </item>\n");
    Some(xml)
}

/// The full address of an image: a file in `media/`, or a full address
/// as it is.
fn image_url(base: &str, image: &str) -> Option<String> {
    let image = image.trim();
    if image.is_empty() {
        None
    } else if image.contains("://") {
        Some(image.to_string())
    } else {
        let name = image.trim_start_matches('/');
        let name = name.strip_prefix("media/").unwrap_or(name);
        Some(format!("{base}/{MEDIA_DIR}/{name}"))
    }
}

/// `<name>text</name>`, the text escaped.
fn tag(name: &str, text: &str) -> String {
    format!("<{name}>{}</{name}>", escape(text))
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The episodes in a feed, newest first, without drafts and future ones;
/// the same order and rules as a Post list.
pub fn feed_episodes<'a>(published: &[&'a Page], today: Date) -> Vec<&'a Page> {
    let prefix = format!("{}/", podcast::EPISODES_PATH);
    let mut episodes: Vec<(Date, &Page)> = published
        .iter()
        .copied()
        .filter(|page| page.path.starts_with(&prefix) && page.content.front.episode.is_some())
        .filter_map(|page| {
            let date = page.content.front.date.as_ref()?.date()?;
            (date <= today).then_some((date, page))
        })
        .collect();
    episodes
        .sort_by(|(a, page_a), (b, page_b)| b.cmp(a).then_with(|| page_a.path.cmp(&page_b.path)));
    episodes.into_iter().map(|(_, page)| page).collect()
}

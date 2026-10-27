//! Videos on YouTube, Vimeo or PeerTube, for Video blocks.
//!
//! A Video block shows the video's thumbnail with a play mark and links to
//! the video: no iframe, no JavaScript, and nothing loaded from the video
//! platform until a visitor clicks. The thumbnail and the title are fetched
//! once, when the block is made, with [oEmbed](https://oembed.com), which
//! all three platforms offer; the thumbnail is kept in `media/`.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::media::{self, Library};
use crate::site::SiteError;
use crate::slug::slugify;

/// What [`fetch`] found for a video.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VideoInfo {
    pub title: Option<String>,
    /// The thumbnail's name in `media/`.
    pub image: String,
}

#[derive(Debug, thiserror::Error)]
pub enum VideoError {
    #[error("\"{0}\" isn't a web address of a video")]
    BadUrl(String),
    #[error("couldn't reach {host}: {message}")]
    Network { host: String, message: String },
    #[error("{0} didn't say which thumbnail the video has")]
    NoThumbnail(String),
    #[error("the thumbnail from {0} isn't an image")]
    NotAnImage(String),
    #[error(transparent)]
    Site(#[from] SiteError),
}

/// The platforms q.e.d. knows by name. Any other host is tried as a
/// PeerTube instance, since those run on their own domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    YouTube,
    Vimeo,
    PeerTube,
}

/// Which platform a video address belongs to.
pub fn platform(url: &str) -> Option<Platform> {
    let parsed = reqwest::Url::parse(url.trim()).ok()?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return None;
    }
    let host = parsed
        .host_str()?
        .trim_start_matches("www.")
        .trim_start_matches("m.");
    Some(match host {
        "youtube.com" | "youtu.be" | "youtube-nocookie.com" | "music.youtube.com" => {
            Platform::YouTube
        }
        "vimeo.com" | "player.vimeo.com" => Platform::Vimeo,
        _ => Platform::PeerTube,
    })
}

/// A short name for where the video is: `YouTube`, `Vimeo`, or the
/// PeerTube instance's host (`tube.example.org`).
pub fn host_label(url: &str) -> Option<String> {
    match platform(url)? {
        Platform::YouTube => Some("YouTube".to_string()),
        Platform::Vimeo => Some("Vimeo".to_string()),
        Platform::PeerTube => reqwest::Url::parse(url.trim())
            .ok()?
            .host_str()
            .map(str::to_string),
    }
}

/// The oEmbed address that describes the video.
fn oembed_url(url: &str) -> Option<reqwest::Url> {
    let video = reqwest::Url::parse(url.trim()).ok()?;
    let endpoint = match platform(url)? {
        Platform::YouTube => "https://www.youtube.com/oembed".to_string(),
        Platform::Vimeo => "https://vimeo.com/api/oembed.json".to_string(),
        Platform::PeerTube => format!("{}://{}/services/oembed", video.scheme(), video.host_str()?),
    };
    let mut oembed = reqwest::Url::parse(&endpoint).ok()?;
    oembed
        .query_pairs_mut()
        .append_pair("format", "json")
        .append_pair("url", video.as_str());
    Some(oembed)
}

/// A YouTube video's id: `xBsZDnMdvI8` in `watch?v=xBsZDnMdvI8`,
/// `youtu.be/xBsZDnMdvI8` or `/embed/xBsZDnMdvI8`.
fn youtube_id(url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(url.trim()).ok()?;
    if let Some((_, id)) = parsed.query_pairs().find(|(key, _)| key == "v") {
        return Some(id.into_owned());
    }
    let mut segments = parsed.path_segments()?;
    let first = segments.next()?;
    let id = if parsed.host_str()?.ends_with("youtu.be") {
        first
    } else if matches!(first, "embed" | "shorts" | "live") {
        segments.next()?
    } else {
        return None;
    };
    (!id.is_empty()).then(|| id.to_string())
}

#[derive(Deserialize)]
struct OEmbed {
    title: Option<String>,
    thumbnail_url: Option<String>,
}

/// Asks the platform for the video's title and thumbnail and saves the
/// thumbnail into `media/` (`video-meetup-unsubscribe.jpg`).
pub async fn fetch(root: &Path, url: &str) -> Result<VideoInfo, VideoError> {
    let bad_url = || VideoError::BadUrl(url.to_string());
    let oembed = oembed_url(url).ok_or_else(bad_url)?;
    let host = oembed.host_str().unwrap_or_default().to_string();
    let network = |error: reqwest::Error| VideoError::Network {
        host: host.clone(),
        message: error.to_string(),
    };
    // reqwest leaves the choice of crypto to the app; q.e.d. uses ring
    // (as SSH does). Installing fails harmlessly if it's there already.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let client = reqwest::Client::builder()
        .user_agent(concat!("q.e.d./", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(network)?;

    let info: OEmbed = client
        .get(oembed)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(network)?
        .json()
        .await
        .map_err(network)?;

    // YouTube's oEmbed thumbnail is 4:3 with black bars; the large one is
    // 16:9, but not every video has it.
    let mut candidates = Vec::new();
    if platform(url) == Some(Platform::YouTube)
        && let Some(id) = youtube_id(url)
    {
        candidates.push(format!("https://i.ytimg.com/vi/{id}/maxresdefault.jpg"));
    }
    candidates.extend(info.thumbnail_url.clone());
    if candidates.is_empty() {
        return Err(VideoError::NoThumbnail(host));
    }

    let mut image = None;
    for candidate in candidates {
        let Ok(response) = client.get(&candidate).send().await else {
            continue;
        };
        if !response.status().is_success() {
            continue;
        }
        let bytes = response.bytes().await.map_err(network)?;
        if let Ok(format) = image::guess_format(&bytes) {
            image = Some((bytes, format));
            break;
        }
    }
    let (bytes, format) = image.ok_or_else(|| VideoError::NotAnImage(host.clone()))?;

    let title = info
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let stem = format!(
        "video-{}",
        title
            .as_deref()
            .map(slugify)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "thumbnail".to_string())
    );
    let extension = format!(
        ".{}",
        format.extensions_str().first().copied().unwrap_or("jpg")
    );
    let name = media::write_unique(
        &root.join(Library::Media.dir()),
        &stem,
        &extension,
        &mut bytes.as_ref(),
    )?;
    Ok(VideoInfo { title, image: name })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_the_platforms() {
        assert_eq!(
            platform("https://m.youtube.com/watch?v=xBsZDnMdvI8"),
            Some(Platform::YouTube)
        );
        assert_eq!(
            platform("https://youtu.be/xBsZDnMdvI8"),
            Some(Platform::YouTube)
        );
        assert_eq!(
            platform("https://vimeo.com/76979871"),
            Some(Platform::Vimeo)
        );
        assert_eq!(
            platform("https://tube.example.org/w/9c9de5e8-0a1e-484a-b099-e80766180a6d"),
            Some(Platform::PeerTube)
        );
        assert_eq!(platform("not a link"), None);
        assert_eq!(platform("ftp://example.org/video"), None);
        assert_eq!(
            host_label("https://tube.example.org/w/abc").as_deref(),
            Some("tube.example.org")
        );
    }

    #[test]
    fn finds_youtube_ids() {
        let id = |url| youtube_id(url);
        assert_eq!(
            id("https://www.youtube.com/watch?v=s19wvVSA2C8&list=PLx").as_deref(),
            Some("s19wvVSA2C8")
        );
        assert_eq!(
            id("https://youtu.be/xBsZDnMdvI8?t=12").as_deref(),
            Some("xBsZDnMdvI8")
        );
        assert_eq!(
            id("https://www.youtube.com/embed/xBsZDnMdvI8").as_deref(),
            Some("xBsZDnMdvI8")
        );
        assert_eq!(id("https://www.youtube.com/playlist?list=PLx"), None);
    }

    #[test]
    fn asks_each_platform_its_own_way() {
        let oembed = |url| oembed_url(url).unwrap().to_string();
        assert!(oembed("https://youtu.be/abc").starts_with(
            "https://www.youtube.com/oembed?format=json&url=https%3A%2F%2Fyoutu.be%2Fabc"
        ));
        assert!(oembed("https://vimeo.com/1").starts_with("https://vimeo.com/api/oembed.json?"));
        assert!(
            oembed("https://tube.example.org/w/abc")
                .starts_with("https://tube.example.org/services/oembed?")
        );
    }
}

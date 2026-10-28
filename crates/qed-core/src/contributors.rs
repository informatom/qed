//! Contributors: the people who make a podcast, entered once and linked to
//! episodes with a role.
//!
//! Each is a file in `contributors/`, named after them when they were
//! added (`stefan-haslinger.md`): settings between `+++` lines, then the
//! bio as Markdown.
//!
//! ```text
//! +++
//! name = "Stefan Haslinger"
//! photo = "stefan.jpg"
//!
//! [[links]]
//! title = "Mastodon"
//! url = "https://fosstodon.org/@informatom"
//! +++
//!
//! Stefan asks about things he knows nothing about.
//! ```
//!
//! An episode names them in its settings:
//!
//! ```text
//! [[episode.people]]
//! contributor = "stefan-haslinger"
//! role = "host"
//! ```
//!
//! The site gets a page for each at `/contributors/<name>/`, with the
//! episodes they're in, and the feeds name them with `podcast:person`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::page_writer;
use crate::site::{self, Site, SiteError};
use crate::slug::slugify;

/// The folder that holds them, and their pages' address on the site.
pub const CONTRIBUTORS_DIR: &str = "contributors";

/// One contributor, as the UI and the generator see them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Contributor {
    /// The file name without `.md`, which episodes refer to; it stays when
    /// the name changes.
    pub id: String,
    pub name: String,
    /// A picture in `media/`, best square.
    pub photo: Option<String>,
    /// Their website, Mastodon, …
    pub links: Vec<ContributorLink>,
    /// About them, in Markdown.
    pub bio: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct ContributorLink {
    /// The link text, e.g. "Website"; the address if empty.
    pub title: Option<String>,
    pub url: String,
}

/// The settings in a contributor's file, between the `+++` lines.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContributorFront {
    name: String,
    photo: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    links: Vec<ContributorLink>,
}

/// What someone does in an episode. The names and groups are the
/// Podcasting 2.0 taxonomy's, which apps read from `podcast:person`:
/// <https://github.com/Podcastindex-org/podcast-namespace/blob/main/taxonomy.json>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    #[default]
    Host,
    CoHost,
    GuestHost,
    Guest,
    Narrator,
    Producer,
    AudioEditor,
    SoundDesigner,
    Composer,
    Transcriber,
    CoverArtDesigner,
}

impl Role {
    pub const ALL: [Role; 11] = [
        Role::Host,
        Role::CoHost,
        Role::GuestHost,
        Role::Guest,
        Role::Narrator,
        Role::Producer,
        Role::AudioEditor,
        Role::SoundDesigner,
        Role::Composer,
        Role::Transcriber,
        Role::CoverArtDesigner,
    ];

    /// The taxonomy's role and group, as `podcast:person` wants them.
    pub fn taxonomy(self) -> (&'static str, &'static str) {
        match self {
            Role::Host => ("host", "cast"),
            Role::CoHost => ("co-host", "cast"),
            Role::GuestHost => ("guest host", "cast"),
            Role::Guest => ("guest", "cast"),
            Role::Narrator => ("narrator", "cast"),
            Role::Producer => ("producer", "administration"),
            Role::AudioEditor => ("audio editor", "audio post-production"),
            Role::SoundDesigner => ("sound designer", "audio production"),
            Role::Composer => ("composer", "audio production"),
            Role::Transcriber => ("transcriber", "writing"),
            Role::CoverArtDesigner => ("cover art designer", "visuals"),
        }
    }

    /// The role for readers, in the site's language (German or English).
    pub fn label(self, language: &str) -> &'static str {
        let german = language.trim().to_lowercase().starts_with("de");
        match (self, german) {
            (Role::Host, true) => "Moderation",
            (Role::Host, false) => "Host",
            (Role::CoHost, true) => "Co-Moderation",
            (Role::CoHost, false) => "Co-host",
            (Role::GuestHost, true) => "Gastmoderation",
            (Role::GuestHost, false) => "Guest host",
            (Role::Guest, true) => "Gast",
            (Role::Guest, false) => "Guest",
            (Role::Narrator, true) => "Sprecher:in",
            (Role::Narrator, false) => "Narrator",
            (Role::Producer, true) => "Produktion",
            (Role::Producer, false) => "Producer",
            (Role::AudioEditor, true) => "Schnitt",
            (Role::AudioEditor, false) => "Audio editor",
            (Role::SoundDesigner, true) => "Sounddesign",
            (Role::SoundDesigner, false) => "Sound designer",
            (Role::Composer, true) => "Musik",
            (Role::Composer, false) => "Composer",
            (Role::Transcriber, true) => "Transkript",
            (Role::Transcriber, false) => "Transcriber",
            (Role::CoverArtDesigner, true) => "Cover",
            (Role::CoverArtDesigner, false) => "Cover art",
        }
    }
}

/// Someone in an episode, `[[episode.people]]`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(deny_unknown_fields)]
pub struct EpisodePerson {
    /// The contributor's id, their file name in `contributors/`.
    pub contributor: String,
    #[serde(default)]
    pub role: Role,
}

/// Someone listed twice in the same role, if so: several roles are fine,
/// one role twice is a slip.
pub fn duplicate(people: &[EpisodePerson]) -> Option<String> {
    people.iter().enumerate().find_map(|(index, person)| {
        people[..index].contains(person).then(|| {
            format!(
                "{} is listed twice as {}",
                person.contributor,
                person.role.taxonomy().0
            )
        })
    })
}

fn file_of(id: &str) -> PathBuf {
    Path::new(CONTRIBUTORS_DIR).join(format!("{id}.md"))
}

/// Reads every contributor in `contributors/`, sorted by name.
pub(crate) fn load(root: &Path) -> Result<Vec<Contributor>, SiteError> {
    let folder = root.join(CONTRIBUTORS_DIR);
    if !folder.is_dir() {
        return Ok(Vec::new());
    }
    let io_error = |source| SiteError::Io {
        path: folder.clone(),
        source,
    };
    let mut people = Vec::new();
    for entry in fs::read_dir(&folder).map_err(io_error)? {
        let path = entry.map_err(io_error)?.path();
        let Some(id) = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|_| path.is_file() && path.extension().is_some_and(|ext| ext == "md"))
        else {
            continue;
        };
        people.push(parse(id, &site::read(&path)?, &file_of(id))?);
    }
    people.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(people)
}

/// Splits a contributor's file into its settings and its bio.
fn split(text: &str) -> Option<(&str, &str)> {
    let rest = text
        .strip_prefix("+++\n")
        .or_else(|| text.strip_prefix("+++\r\n"))?;
    let end = rest.find("\n+++")?;
    let after = &rest[end + 4..];
    Some((&rest[..end + 1], after.strip_prefix('\n').unwrap_or(after)))
}

fn parse(id: &str, text: &str, file: &Path) -> Result<Contributor, SiteError> {
    let invalid = |message: String| SiteError::Invalid {
        path: file.to_path_buf(),
        message,
    };
    let (front, bio) = split(text)
        .ok_or_else(|| invalid("it needs settings between +++ lines at the top".to_string()))?;
    let front: ContributorFront =
        toml::from_str(front).map_err(|e| invalid(e.message().to_string()))?;
    Ok(Contributor {
        id: id.to_string(),
        name: front.name,
        photo: front.photo,
        links: front.links,
        bio: bio.trim().to_string(),
    })
}

/// Saves a contributor and returns them as saved. A new one (empty `id`)
/// gets a file named after them; an existing one keeps its file, so
/// episodes keep finding them when the name changes. Settings someone
/// wrote by hand in the file, such as comments, stay.
pub fn save(root: &Path, contributor: &Contributor) -> Result<Contributor, SiteError> {
    let name = contributor.name.trim();
    let new = contributor.id.trim().is_empty();
    let id = if new {
        slugify(name)
    } else {
        contributor.id.trim().to_string()
    };
    let file = file_of(&id);
    let invalid = |message: String| SiteError::Invalid {
        path: file.clone(),
        message,
    };
    if name.is_empty() || id.is_empty() {
        return Err(invalid(
            "a contributor needs a name with a letter or digit".to_string(),
        ));
    }
    if id.contains(['/', '\\']) || id.starts_with('.') {
        return Err(invalid(format!(
            "“{id}” can't be a contributor's file name"
        )));
    }
    if let Some(link) = contributor.links.iter().find(|l| l.url.trim().is_empty()) {
        return Err(invalid(format!(
            "the link “{}” needs an address",
            link.title.as_deref().unwrap_or_default()
        )));
    }
    let path = root.join(&file);
    if new && path.exists() {
        return Err(invalid(format!(
            "there is a contributor called {name} already"
        )));
    }

    let front = ContributorFront {
        name: name.to_string(),
        photo: contributor
            .photo
            .as_deref()
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(str::to_string),
        links: contributor
            .links
            .iter()
            .map(|link| ContributorLink {
                title: link
                    .title
                    .as_deref()
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(str::to_string),
                url: link.url.trim().to_string(),
            })
            .collect(),
    };
    let mut document = toml_edit::ser::to_document(&front).map_err(|e| invalid(e.to_string()))?;
    // The links as `[[links]]` sections, easier to read than one long line.
    if let Some(item) = document.get_mut("links") {
        let list = std::mem::take(item);
        *item = list
            .into_array_of_tables()
            .map(toml_edit::Item::ArrayOfTables)
            .unwrap_or_else(|list| list);
    }
    let original = fs::read_to_string(&path).ok();
    let front_text = match original.as_deref().and_then(split) {
        Some((old_front, _)) => page_writer::merge(old_front, document),
        None => document.to_string(),
    };
    let bio = contributor.bio.trim();
    let text = if bio.is_empty() {
        format!("+++\n{}\n+++\n", front_text.trim_end())
    } else {
        format!("+++\n{}\n+++\n\n{bio}\n", front_text.trim_end())
    };
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder).map_err(|source| SiteError::Write {
            path: folder.to_path_buf(),
            source,
        })?;
    }
    fs::write(&path, &text).map_err(|source| SiteError::Write {
        path: path.clone(),
        source,
    })?;
    parse(&id, &text, &file)
}

/// Where a contributor is listed: episodes by address, and "the podcast's
/// team".
pub fn episodes_with(site: &Site, id: &str) -> Vec<String> {
    let in_team = site
        .config
        .podcast
        .as_ref()
        .is_some_and(|podcast| podcast.people.iter().any(|p| p.contributor == id));
    let team = in_team.then(|| "the podcast's team".to_string());
    team.into_iter()
        .chain(
            site.pages
                .iter()
                .filter(|page| {
                    page.content
                        .front
                        .episode
                        .as_ref()
                        .is_some_and(|e| e.people.iter().any(|p| p.contributor == id))
                })
                .map(|page| format!("/{}", page.path)),
        )
        .collect()
}

/// Deletes a contributor, unless an episode still lists them.
pub fn delete(site: &Site, id: &str) -> Result<(), SiteError> {
    let file = file_of(id);
    let used = episodes_with(site, id);
    if !used.is_empty() {
        return Err(SiteError::Invalid {
            path: file,
            message: format!("still in {}; take them out there first", used.join(", ")),
        });
    }
    let path = site.root.join(&file);
    fs::remove_file(&path).map_err(|source| SiteError::Write { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        site::create_site(dir.path(), "Pod", "de").unwrap();
        dir
    }

    fn stefan() -> Contributor {
        Contributor {
            id: String::new(),
            name: "Stefan Haslinger".to_string(),
            photo: Some("stefan.jpg".to_string()),
            links: vec![ContributorLink {
                title: Some("Mastodon".to_string()),
                url: "https://fosstodon.org/@informatom".to_string(),
            }],
            bio: "Fragt nach.".to_string(),
        }
    }

    #[test]
    fn saves_and_reads_contributors_as_files() {
        let dir = site();
        let saved = save(dir.path(), &stefan()).unwrap();
        assert_eq!(saved.id, "stefan-haslinger");
        let text = fs::read_to_string(dir.path().join("contributors/stefan-haslinger.md")).unwrap();
        assert!(
            text.starts_with("+++\nname = \"Stefan Haslinger\"\n"),
            "{text}"
        );
        assert!(text.contains("[[links]]\ntitle = \"Mastodon\""), "{text}");
        assert!(text.ends_with("+++\n\nFragt nach.\n"), "{text}");

        // A comment written by hand stays; the file keeps its name.
        fs::write(
            dir.path().join("contributors/stefan-haslinger.md"),
            text.replace("+++\nname", "+++\n# Gründer\nname"),
        )
        .unwrap();
        let renamed = save(
            dir.path(),
            &Contributor {
                name: "Stefan H.".to_string(),
                ..saved.clone()
            },
        )
        .unwrap();
        assert_eq!(renamed.id, "stefan-haslinger");
        let text = fs::read_to_string(dir.path().join("contributors/stefan-haslinger.md")).unwrap();
        assert!(text.contains("# Gründer\nname = \"Stefan H.\""), "{text}");

        let site = site::open(dir.path()).unwrap();
        assert_eq!(site.contributors.len(), 1);
        assert_eq!(site.contributors[0].links.len(), 1);

        // A new one whose file name is taken is refused.
        assert!(save(dir.path(), &stefan()).is_err());
    }

    #[test]
    fn deletes_only_contributors_no_episode_lists() {
        let dir = site();
        let mut config = site::open(dir.path()).unwrap().config;
        config.podcast = Some(crate::podcast::Podcast::default());
        site::save_config(dir.path(), &config).unwrap();
        let saved = save(dir.path(), &stefan()).unwrap();
        let page = crate::podcast::create_episode(dir.path(), "Eins", None).unwrap();
        let mut front = page.content.front.clone();
        if let Some(episode) = front.episode.as_mut() {
            episode.people = vec![EpisodePerson {
                contributor: saved.id.clone(),
                role: Role::Guest,
            }];
        }
        site::save_front_matter(dir.path(), &page.file, front.clone()).unwrap();
        let text = fs::read_to_string(dir.path().join(&page.file)).unwrap();
        assert!(
            text.contains(
                "[[episode.people]]\ncontributor = \"stefan-haslinger\"\nrole = \"guest\""
            ),
            "{text}"
        );

        let error = delete(&site::open(dir.path()).unwrap(), &saved.id).unwrap_err();
        assert!(error.to_string().contains(&page.path), "{error}");

        if let Some(episode) = front.episode.as_mut() {
            episode.people.clear();
        }
        site::save_front_matter(dir.path(), &page.file, front).unwrap();
        delete(&site::open(dir.path()).unwrap(), &saved.id).unwrap();
        assert!(site::open(dir.path()).unwrap().contributors.is_empty());
    }

    #[test]
    fn knows_the_podcasting_roles() {
        assert_eq!(
            Role::CoverArtDesigner.taxonomy(),
            ("cover art designer", "visuals")
        );
        assert_eq!(Role::Guest.label("de-AT"), "Gast");
        assert_eq!(Role::Guest.label("en"), "Guest");
    }

    #[test]
    fn shows_contributors_on_episodes_their_pages_and_in_the_feed() {
        let dir = site();
        let mut config = site::open(dir.path()).unwrap().config;
        config.base_url = Some("https://pod.example/".to_string());
        config.podcast = Some(crate::podcast::Podcast::default());
        site::save_config(dir.path(), &config).unwrap();
        let stefan = save(dir.path(), &stefan()).unwrap();
        let hannah = save(
            dir.path(),
            &Contributor {
                id: String::new(),
                name: "Hannah".to_string(),
                photo: None,
                links: Vec::new(),
                bio: String::new(),
            },
        )
        .unwrap();

        // The team: Stefan hosts and edits.
        let mut config = site::open(dir.path()).unwrap().config;
        if let Some(podcast) = config.podcast.as_mut() {
            podcast.people = vec![
                EpisodePerson {
                    contributor: stefan.id.clone(),
                    role: Role::Host,
                },
                EpisodePerson {
                    contributor: stefan.id.clone(),
                    role: Role::AudioEditor,
                },
            ];
        }
        site::save_config(dir.path(), &config).unwrap();
        let toml = fs::read_to_string(dir.path().join(site::CONFIG_FILE)).unwrap();
        assert!(
            toml.contains("[[podcast.people]]\ncontributor = \"stefan-haslinger\""),
            "{toml}"
        );

        // A new episode starts with the team; Hannah joins as a guest.
        let page = crate::podcast::create_episode(dir.path(), "Eins", None).unwrap();
        let mut front = page.content.front.clone();
        let episode = front.episode.as_mut().unwrap();
        assert_eq!(episode.people.len(), 2);
        episode.audio = vec![crate::podcast::EpisodeAudio {
            format: crate::podcast::AudioFormat::Mp3,
            file: "https://cdn.example/eins.mp3".to_string(),
            length: None,
        }];
        episode.people.push(EpisodePerson {
            contributor: hannah.id.clone(),
            role: Role::Guest,
        });
        site::save_front_matter(dir.path(), &page.file, front.clone()).unwrap();

        // The same role twice is a slip.
        let mut twice = front.clone();
        twice.episode.as_mut().unwrap().people.push(EpisodePerson {
            contributor: hannah.id.clone(),
            role: Role::Guest,
        });
        assert!(site::save_front_matter(dir.path(), &page.file, twice).is_err());

        let site = site::open(dir.path()).unwrap();
        let out = dir.path().join("_site");
        crate::generator::build_into(&site, &out).unwrap();

        let html = fs::read_to_string(out.join(&page.path).join("index.html")).unwrap();
        assert!(
            html.contains("<h2 class=\"text-2xl font-semibold tracking-tight\">Mitwirkende</h2>"),
            "{html}"
        );
        assert!(html.contains("Moderation, Schnitt"), "{html}");
        assert!(
            html.contains(r#"href="../../contributors/hannah/""#),
            "{html}"
        );

        let person =
            fs::read_to_string(out.join("contributors/stefan-haslinger/index.html")).unwrap();
        assert!(person.contains("Fragt nach."), "{person}");
        assert!(
            person.contains(r#"href="https://fosstodon.org/@informatom""#),
            "{person}"
        );
        assert!(person.contains("Eins"), "the episodes they're in: {person}");
        let all = fs::read_to_string(out.join("contributors/index.html")).unwrap();
        assert!(
            all.contains("Hannah") && all.contains("Stefan Haslinger"),
            "{all}"
        );

        let feed = fs::read_to_string(out.join("episodes.mp3.rss")).unwrap();
        assert!(feed.contains(r#"<podcast:person role="audio editor" group="audio post-production" img="https://pod.example/media/stefan.jpg" href="https://pod.example/contributors/stefan-haslinger/">Stefan Haslinger</podcast:person>"#), "{feed}");
        assert!(feed.contains(r#"<podcast:person role="guest" group="cast" href="https://pod.example/contributors/hannah/">Hannah</podcast:person>"#), "{feed}");

        // In the team, so not to be deleted.
        let error = delete(&site, &stefan.id).unwrap_err();
        assert!(error.to_string().contains("team"), "{error}");
    }
}

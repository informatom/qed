//! qed-mcp lets an AI assistant (Claude Code, Claude Desktop, …) work on one
//! q.e.d. site through the Model Context Protocol (#105).
//!
//! The assistant starts it with the site folder, `qed-mcp /path/to/site`,
//! and talks to it over stdin and stdout. It's a second thin shell around
//! qed-core next to the app, with the app's rules: pages are checked before
//! they're written, a page that changed since the assistant read it isn't
//! overwritten, the open version is closed before deleting or moving, and
//! every change is recorded in the site's history like a save in the app.
//! It can build the site but not publish it; that stays a click in the app.
//!
//! stdout carries the protocol, so nothing else may be printed there;
//! messages for people go to stderr.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use qed_core::calendar::Date;
use qed_core::media::{self, Library};
use qed_core::page::PageDate;
use qed_core::{editing, generator, history, site, target};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, ServiceExt, schemars, tool, tool_handler, tool_router};
use serde::Deserialize;

/// What the assistant is told when it connects: the page format and the
/// rules, so it can read and write pages without guessing.
const GUIDE: &str = include_str!("guide.md");

/// A tool's answer: text for the assistant, or what went wrong, which the
/// assistant sees as a failed tool call it can react to.
type Reply = Result<String, String>;

#[derive(Clone)]
struct Qed {
    /// The site folder; nothing outside it is touched.
    root: PathBuf,
    tool_router: ToolRouter<Self>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct PageFile {
    /// The page file, relative to the site folder, as `list_pages` shows
    /// it: `pages/about.md`, `pages/blog/2026-10-29-news.md`.
    file: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct WritePage {
    /// The page file, as `list_pages` shows it.
    file: String,
    /// The page's whole new text: settings between `+++` lines, then
    /// Markdown and `:::` blocks.
    text: String,
    /// The stamp `read_page` gave with the text this change is based on.
    stamp: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct CreatePage {
    /// The address of the page it goes below: `` for the top level,
    /// `blog` for a post in the blog.
    parent: String,
    title: String,
    /// `YYYY-MM-DD` makes it a post (or an episode below `episodes`).
    date: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct MovePage {
    /// The page file, as `list_pages` shows it.
    file: String,
    /// The address of its new parent page: `` for the top level.
    parent: String,
    /// The last part of its new address: `team` for `/about/team`.
    slug: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct Files {
    /// `media` (images, the default) or `assets` (files to download).
    library: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct Import {
    /// The full path of a file on this computer to copy into the site.
    path: String,
    /// `media` (images, the default) or `assets` (files to download).
    library: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct Versions {
    /// Only the versions that changed this file (`pages/about.md`); all
    /// versions of the site if left out.
    file: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct Note {
    /// What changed, in a few words, as the site owner would say it.
    note: String,
}

#[tool_router]
impl Qed {
    fn new(root: PathBuf) -> Self {
        Qed {
            root,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "The site's title, language, theme, web address, whether it's a podcast, how many pages it has, and what changed since the last version."
    )]
    async fn site_info(&self) -> Reply {
        let site = site::open(&self.root).map_err(text)?;
        let config = &site.config;
        let status = history::status(&self.root).map_err(text)?;
        let mut out = format!(
            "Title: {}\nLanguage: {}\nTheme: {}\nWeb address: {}\nPodcast: {}\nPages: {}\nShared blocks: {}\nFooter: {}\n",
            config.title,
            config.language,
            config.theme,
            config.base_url.as_deref().unwrap_or("(none yet)"),
            if config.podcast.is_some() {
                "yes"
            } else {
                "no"
            },
            site.pages.len(),
            site.shared.len(),
            if site.footer.is_some() {
                "footer.md"
            } else {
                "none"
            },
        );
        if !status.tracked {
            out.push_str("History: none; changes aren't recorded as versions.\n");
        } else if status.open || !status.changes.is_empty() {
            out.push_str("History: saves since the last version are kept in the open version.\n");
        }
        Ok(out)
    }

    #[tool(
        description = "Every page with its address, file, title and date; drafts and menus marked. Use the file with read_page, write_page, move_page and delete_page."
    )]
    async fn list_pages(&self) -> Reply {
        let site = site::open(&self.root).map_err(text)?;
        let mut out = String::new();
        for page in &site.pages {
            let front = &page.content.front;
            let mut marks = Vec::new();
            if let Some(date) = &front.date {
                marks.push(date.to_string());
            }
            if front.draft {
                marks.push("draft".to_string());
            }
            if front.menu != qed_core::page::Menu::None {
                marks.push(format!("{:?} menu", front.menu).to_lowercase());
            }
            let marks = if marks.is_empty() {
                String::new()
            } else {
                format!(" ({})", marks.join(", "))
            };
            out.push_str(&format!(
                "/{} · {} · {}{marks}\n",
                page.path,
                page.file.display(),
                front.title
            ));
        }
        Ok(out)
    }

    #[tool(
        description = "A page file's whole text (settings between +++ lines, then Markdown and ::: blocks) and its stamp. Pass the stamp to write_page, so a change made meanwhile in the app isn't overwritten."
    )]
    async fn read_page(&self, Parameters(PageFile { file }): Parameters<PageFile>) -> Reply {
        let (content, stamp) = editing::page_text(&self.root, Path::new(&file)).map_err(text)?;
        Ok(format!("stamp: {stamp}\n\n{content}"))
    }

    #[tool(
        description = "Replaces a page file's whole text, read with read_page. It must be a valid page with a title; mistakes come back with their line. Refused if the file changed since that read: read it again then. Recorded in the site's history."
    )]
    async fn write_page(
        &self,
        Parameters(WritePage {
            file,
            text: content,
            stamp,
        }): Parameters<WritePage>,
    ) -> Reply {
        let page = editing::write_page_text(&self.root, Path::new(&file), &content, &stamp)
            .map_err(text)?;
        self.record()?;
        let (_, stamp) = editing::page_text(&self.root, &page.file).map_err(text)?;
        Ok(format!(
            "Saved /{} ({} blocks). New stamp: {stamp}",
            page.path,
            page.content.blocks.len()
        ))
    }

    #[tool(
        description = "Makes a new page below a parent page, with a title and, for a post, a date. Its file name comes from the title. Returns the new page's file, text and stamp, to fill in with write_page."
    )]
    async fn create_page(
        &self,
        Parameters(CreatePage {
            parent,
            title,
            date,
        }): Parameters<CreatePage>,
    ) -> Reply {
        let date = date
            .as_deref()
            .map(|date| parse_date(date).ok_or(format!("\"{date}\" isn't a date like 2026-10-29")))
            .transpose()?;
        let page = site::create_page(&self.root, &parent, &title, date).map_err(text)?;
        self.record()?;
        let (content, stamp) = editing::page_text(&self.root, &page.file).map_err(text)?;
        Ok(format!(
            "Made /{} in {}.\nstamp: {stamp}\n\n{content}",
            page.path,
            page.file.display()
        ))
    }

    #[tool(
        description = "Gives a page a new address: another parent, a new last part, or both. Links to it in other pages are rewritten and its old address is kept as a redirect. The version before the move stays in the history."
    )]
    async fn move_page(
        &self,
        Parameters(MovePage { file, parent, slug }): Parameters<MovePage>,
    ) -> Reply {
        let page =
            editing::move_page(&self.root, Path::new(&file), &parent, &slug).map_err(text)?;
        self.record()?;
        Ok(format!(
            "Moved to /{} ({}).",
            page.path,
            page.file.display()
        ))
    }

    #[tool(
        description = "Deletes a page, unless other pages link to it (they're listed then) or it's the start page or has pages below it. It can be brought back from the history in the app."
    )]
    async fn delete_page(&self, Parameters(PageFile { file }): Parameters<PageFile>) -> Reply {
        let site = site::open(&self.root).map_err(text)?;
        let page = site
            .pages
            .iter()
            .find(|page| page.file == Path::new(&file))
            .ok_or(format!("{file} isn't a page of this site"))?;
        let linking = site::pages_linking_to(&site, &page.path);
        if !linking.is_empty() {
            return Err(format!(
                "Not deleted: /{} is linked from {}. Change those links first.",
                page.path,
                linking.join(", ")
            ));
        }
        editing::delete_page(&self.root, Path::new(&file)).map_err(text)?;
        self.record()?;
        Ok(format!("Deleted /{}.", page.path))
    }

    #[tool(
        description = "The site's images (library media) or files to download (library assets): name, size, image size, and which pages use each. Images are used in blocks by name (image = \"team.jpg\") and in text as /media/team.jpg."
    )]
    async fn list_files(&self, Parameters(Files { library }): Parameters<Files>) -> Reply {
        let library = parse_library(library.as_deref())?;
        let site = site::open(&self.root).map_err(text)?;
        let files = media::list(&site, library).map_err(text)?;
        if files.is_empty() {
            return Ok("No files yet.".to_string());
        }
        let mut out = String::new();
        for file in files {
            let size = match (file.width, file.height) {
                (Some(width), Some(height)) => format!(", {width}×{height} px"),
                _ => String::new(),
            };
            let used = if file.used_by.is_empty() {
                "unused".to_string()
            } else {
                // Pages as addresses; the footer and the like as they are.
                let pages: Vec<String> = file
                    .used_by
                    .iter()
                    .map(|path| match path.starts_with('(') {
                        true => path.clone(),
                        false => format!("/{path}"),
                    })
                    .collect();
                format!("used by {}", pages.join(", "))
            };
            out.push_str(&format!(
                "{} · {} KB{size} · {used}\n",
                file.name,
                file.size / 1024
            ));
        }
        Ok(out)
    }

    #[tool(
        description = "Copies a file from this computer into the site's images (media) or downloads (assets), with a clean name, which it returns."
    )]
    async fn import_file(&self, Parameters(Import { path, library }): Parameters<Import>) -> Reply {
        let library = parse_library(library.as_deref())?;
        let name = media::import(&self.root, library, Path::new(&path)).map_err(text)?;
        self.record()?;
        Ok(format!("Imported as {name}."))
    }

    #[tool(
        description = "Builds the website into the site's _site folder, as the app's Build does, and reports what it made and any notes, such as links to pages that don't exist. Doesn't publish."
    )]
    async fn build_site(&self) -> Reply {
        let root = self.root.clone();
        // Building takes a while and blocks; keep it off the threads that
        // answer the assistant.
        let report = tokio::task::spawn_blocking(move || {
            let site = site::open(&root).map_err(text)?;
            generator::build(&site).map_err(text)
        })
        .await
        .map_err(text)??;
        let mut out = format!(
            "Built {} pages into {} ({} drafts left out, {} resized images).\n",
            report.pages,
            report.out_dir.display(),
            report.drafts_skipped,
            report.resized_images
        );
        for note in &report.notes {
            out.push_str(&format!("Note: {note}\n"));
        }
        Ok(out)
    }

    #[tool(
        description = "What changed since the site was last published to each of its targets. Publishing itself is done by the site owner in the app."
    )]
    async fn changes_since_publish(&self) -> Reply {
        let targets = target::load(&self.root).map_err(text)?;
        if targets.is_empty() {
            return Ok("The site has no publish targets yet.".to_string());
        }
        let mut out = String::new();
        for target in &targets {
            let since = history::since_publish(&self.root, target.name()).map_err(text)?;
            match since.published_at {
                None => out.push_str(&format!("{}: never published.\n", target.name())),
                Some(_) if since.changes.is_empty() => {
                    out.push_str(&format!("{}: up to date.\n", target.name()));
                }
                Some(_) => out.push_str(&format!(
                    "{}: {} files changed: {}\n",
                    target.name(),
                    since.changes.len(),
                    since.changes.join(", ")
                )),
            }
        }
        Ok(out)
    }

    #[tool(
        description = "The site's versions, newest first, or only those that changed one file. The newest may be the open version, which collects saves until it's closed."
    )]
    async fn list_versions(&self, Parameters(Versions { file }): Parameters<Versions>) -> Reply {
        let versions = history::versions(&self.root, file.as_deref()).map_err(text)?;
        let mut out = String::new();
        for version in versions.iter().take(50) {
            let open = if version.open { " (open)" } else { "" };
            out.push_str(&format!(
                "{} · {}{open} · {}\n",
                &version.id[..8.min(version.id.len())],
                version.summary,
                version.author
            ));
        }
        Ok(out)
    }

    #[tool(
        description = "Closes the open version with a note saying what changed, so the site owner finds it in Versions. Use it after a set of changes that belong together."
    )]
    async fn save_version(&self, Parameters(Note { note }): Parameters<Note>) -> Reply {
        match history::save_version(&self.root, &note).map_err(text)? {
            Some(version) => Ok(format!("Saved version \"{}\".", version.summary)),
            None => Ok("Nothing changed since the last version.".to_string()),
        }
    }
}

impl Qed {
    /// Records the change in the site's history, as a save in the app does.
    fn record(&self) -> Result<(), String> {
        history::record_save(&self.root).map(|_| ()).map_err(text)
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Qed {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("qed-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(GUIDE)
    }
}

fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn parse_library(name: Option<&str>) -> Result<Library, String> {
    match name.unwrap_or("media") {
        "media" => Ok(Library::Media),
        "assets" => Ok(Library::Assets),
        other => Err(format!("\"{other}\" isn't a library: use media or assets")),
    }
}

/// `2026-10-29` as a page date.
fn parse_date(text: &str) -> Option<PageDate> {
    let mut parts = text.trim().splitn(3, '-');
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.parse().ok()?;
    PageDate::on(Date { year, month, day })
}

#[tokio::main]
async fn main() -> ExitCode {
    let Some(root) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage: qed-mcp /path/to/site");
        return ExitCode::FAILURE;
    };
    // Refuse to start for anything but a site, so a wrong path in the
    // assistant's settings shows up right away.
    if let Err(error) = site::open(&root) {
        eprintln!("qed-mcp: {error}");
        return ExitCode::FAILURE;
    }
    let server = match Qed::new(root).serve(rmcp::transport::stdio()).await {
        Ok(server) => server,
        Err(error) => {
            eprintln!("qed-mcp: {error}");
            return ExitCode::FAILURE;
        }
    };
    match server.waiting().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("qed-mcp: {error}");
            ExitCode::FAILURE
        }
    }
}

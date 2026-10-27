//! Turning the links people write into links that work on the built site.
//!
//! The built site uses relative links only (`../about/`), so it works at the
//! root of a domain, in a subfolder such as a GitHub Pages project site, and
//! in the local preview.

use std::path::{Component, Path, PathBuf};

use crate::site::{ASSETS_DIR, MEDIA_DIR, PAGES_DIR};

/// Where links are resolved from: the page being rendered.
#[derive(Debug, Clone)]
pub struct LinkContext {
    /// The page's URL path, `""` for the start page.
    page_path: String,
    /// The page's file relative to the site folder, e.g. `pages/blog/post.md`.
    page_file: PathBuf,
    /// A fixed way to the site root (`/`) instead of a relative one, for a
    /// page shown at any address: the 404 page.
    root: Option<String>,
    /// The site's own web address (`base_url`): full links to it are
    /// links inside the site.
    own_site: Option<String>,
}

impl LinkContext {
    pub fn new(page_path: &str, page_file: &Path) -> Self {
        LinkContext {
            page_path: page_path.to_string(),
            page_file: page_file.to_path_buf(),
            root: None,
            own_site: None,
        }
    }

    /// Links from the site root at `root` (`/`, or `/blog/` for a site in a
    /// subfolder), for a page the server shows at any address.
    pub fn rooted(root: &str, page_file: &Path) -> Self {
        LinkContext {
            page_path: String::new(),
            page_file: page_file.to_path_buf(),
            root: Some(root.to_string()),
            own_site: None,
        }
    }

    /// Treats full links to `base_url` (with or without `www.`, over
    /// http or https) as links inside the site, so they work in the
    /// preview too and don't count as leading elsewhere.
    pub fn with_own_site(mut self, base_url: Option<&str>) -> Self {
        self.own_site = base_url.map(str::to_string);
        self
    }

    /// `/blog/post#top` for `https://www.example.com/blog/post#top` when
    /// the site is at `https://example.com/`.
    fn own_path(&self, url: &str) -> Option<String> {
        fn host_and_path(url: &str) -> Option<(String, &str)> {
            let rest = url
                .strip_prefix("https://")
                .or_else(|| url.strip_prefix("http://"))?;
            let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
            let host = rest[..end].to_ascii_lowercase();
            let host = host.strip_prefix("www.").unwrap_or(&host).to_string();
            Some((host, &rest[end..]))
        }
        let (own_host, own_path) = host_and_path(self.own_site.as_deref()?)?;
        let (host, path) = host_and_path(url)?;
        if host != own_host {
            return None;
        }
        let rest = path.strip_prefix(own_path.trim_end_matches('/'))?;
        match rest.chars().next() {
            None => Some("/".to_string()),
            Some('/') => Some(rest.to_string()),
            Some('?' | '#') => Some(format!("/{rest}")),
            _ => None,
        }
    }

    /// The URL path of the page links are resolved from.
    pub fn page_path(&self) -> &str {
        &self.page_path
    }

    /// The way from this page's folder up to the site root: `""` on the start
    /// page, `"../"` on `/about/`, `"../../"` on `/about/team/`.
    pub fn to_root(&self) -> String {
        if let Some(root) = &self.root {
            return root.clone();
        }
        if self.page_path.is_empty() {
            String::new()
        } else {
            "../".repeat(self.page_path.split('/').count())
        }
    }

    /// A relative link to the page with the given URL path.
    pub fn page(&self, path: &str) -> String {
        let root = self.to_root();
        match (root.is_empty(), path.is_empty()) {
            (true, true) => "./".to_string(),
            (_, true) => root,
            _ => format!("{root}{path}/"),
        }
    }

    /// A relative link to a file in the site's `media/` folder.
    pub fn media(&self, file_name: &str) -> String {
        format!(
            "{}{MEDIA_DIR}/{}",
            self.to_root(),
            file_name.trim_start_matches('/')
        )
    }

    /// A relative link to a file in the site's `assets/` folder.
    pub fn asset(&self, file_name: &str) -> String {
        format!(
            "{}{ASSETS_DIR}/{}",
            self.to_root(),
            file_name.trim_start_matches('/')
        )
    }

    /// Rewrites a link as written in a page:
    ///
    /// - `https://…`, `mailto:…`, `#top`: unchanged
    /// - `/about` or `/about/`: the page at that path
    /// - `/media/logo.png`, `/assets/menu.pdf`: that file
    /// - `other-post.md`, `../about.md#team`: the page in that file
    /// - anything else: unchanged
    pub fn resolve(&self, url: &str) -> String {
        if let Some(path) = self.own_path(url) {
            return self.resolve(&path);
        }
        if is_external(url) || url.starts_with('#') || url.is_empty() {
            return url.to_string();
        }

        let (target, fragment) = match url.find(['#', '?']) {
            Some(index) => url.split_at(index),
            None => (url, ""),
        };

        if let Some(site_path) = target.strip_prefix('/') {
            let site_path = site_path.trim_end_matches('/');
            let link = if looks_like_file(site_path) {
                format!("{}{site_path}", self.to_root())
            } else {
                self.page(site_path)
            };
            return format!("{link}{fragment}");
        }

        if target.ends_with(".md")
            && let Some(path) = self.markdown_target(target)
        {
            return format!("{}{fragment}", self.page(&path));
        }

        url.to_string()
    }

    /// Whether a link written in this page leads out of the site folder: a
    /// relative path that climbs higher than the folder, such as
    /// `../../../src/lib.rs` in `pages/blog/post.md`. The website holds
    /// only what's inside the folder, so such a link can't work there.
    pub fn leaves_site(&self, url: &str) -> bool {
        if is_external(url) || url.starts_with(['#', '/']) || url.is_empty() {
            return false;
        }
        let target = url.split(['#', '?']).next().unwrap_or_default();
        let folder = self.page_file.parent().unwrap_or(Path::new(""));
        normalize(&folder.join(target)).is_none()
    }

    /// The URL path of the page a link points to (`/about`, `about.md`,
    /// `../index.md#top`), or `None` for links to files and other sites.
    pub fn page_path_of(&self, url: &str) -> Option<String> {
        if is_external(url) || url.starts_with('#') || url.is_empty() {
            return None;
        }
        let target = url.split(['#', '?']).next().unwrap_or_default();
        if let Some(site_path) = target.strip_prefix('/') {
            let site_path = site_path.trim_end_matches('/');
            return (!looks_like_file(site_path)).then(|| site_path.to_string());
        }
        if target.ends_with(".md") {
            return self.markdown_target(target);
        }
        None
    }

    /// Resolves a relative `.md` link against this page's file and returns
    /// the target page's URL path, if it lands inside `pages/`.
    fn markdown_target(&self, target: &str) -> Option<String> {
        let folder = self.page_file.parent()?;
        let joined = normalize(&folder.join(target))?;
        let inside = joined.strip_prefix(PAGES_DIR).ok()?.with_extension("");
        let mut segments: Vec<String> = inside
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        if segments.last().is_some_and(|last| last == "index") {
            segments.pop();
        }
        Some(segments.join("/"))
    }
}

fn is_external(url: &str) -> bool {
    url.starts_with("//")
        || url.split_once(':').is_some_and(|(scheme, _)| {
            !scheme.is_empty()
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        })
}

/// `logo.png` is a file, `about` and `blog/post` are pages.
fn looks_like_file(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|last| last.contains('.'))
}

/// Resolves `.` and `..` without touching the file system. Returns `None` if
/// the path climbs out of the site folder.
fn normalize(path: &Path) -> Option<PathBuf> {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => result.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !result.pop() {
                    return None;
                }
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(path: &str, file: &str) -> LinkContext {
        LinkContext::new(path, Path::new(file))
    }

    #[test]
    fn full_links_to_the_own_site_stay_inside() {
        let about = context("about", "pages/about.md").with_own_site(Some("https://example.com/"));
        assert_eq!(
            about.resolve("https://www.example.com/blog/post"),
            "../blog/post/"
        );
        assert_eq!(about.resolve("http://EXAMPLE.com"), "../");
        assert_eq!(about.resolve("https://example.com/#top"), "../#top");
        assert_eq!(
            about.resolve("https://example.org/blog"),
            "https://example.org/blog"
        );

        let in_folder =
            context("", "pages/index.md").with_own_site(Some("https://example.com/podcast/"));
        assert_eq!(
            in_folder.resolve("https://example.com/podcast/notes"),
            "notes/"
        );
        assert_eq!(
            in_folder.resolve("https://example.com/podcaster"),
            "https://example.com/podcaster"
        );
        assert_eq!(
            context("", "pages/index.md").resolve("https://example.com/x"),
            "https://example.com/x",
            "no base_url, nothing is own"
        );
    }

    #[test]
    fn walks_up_to_the_root() {
        assert_eq!(context("", "pages/index.md").to_root(), "");
        assert_eq!(context("about", "pages/about.md").to_root(), "../");
        assert_eq!(
            context("about/team", "pages/about/team.md").to_root(),
            "../../"
        );
    }

    #[test]
    fn links_to_pages_relatively() {
        let home = context("", "pages/index.md");
        assert_eq!(home.page(""), "./");
        assert_eq!(home.page("about"), "about/");

        let team = context("about/team", "pages/about/team.md");
        assert_eq!(team.page(""), "../../");
        assert_eq!(team.page("blog/first"), "../../blog/first/");
    }

    #[test]
    fn finds_links_that_leave_the_site_folder() {
        let post = context("blog/post", "pages/blog/post.md");
        assert!(post.leaves_site("../../../src/lib/site.ts"));
        assert!(post.leaves_site("../../../README.md#top"));
        assert!(
            !post.leaves_site("../../media/logo.png"),
            "media/ is inside"
        );
        assert!(!post.leaves_site("../about.md"));
        assert!(!post.leaves_site("/about"));
        assert!(!post.leaves_site("https://example.com/../x"));
        assert!(!post.leaves_site("mailto:a@b.at"));
        let footer = context("(footer)", "footer.md");
        assert!(footer.leaves_site("../other/file.pdf"));
        assert!(!footer.leaves_site("pages/about.md"));
    }

    #[test]
    fn leaves_external_links_and_anchors_alone() {
        let page = context("about", "pages/about.md");
        for url in [
            "https://example.com",
            "mailto:hi@example.com",
            "//cdn.example.com/x.js",
            "#team",
            "",
        ] {
            assert_eq!(page.resolve(url), url);
        }
    }

    #[test]
    fn resolves_site_absolute_links() {
        let page = context("blog/post", "pages/blog/post.md");
        assert_eq!(page.resolve("/"), "../../");
        assert_eq!(page.resolve("/about"), "../../about/");
        assert_eq!(page.resolve("/about/#team"), "../../about/#team");
        assert_eq!(page.resolve("/media/logo.png"), "../../media/logo.png");
    }

    #[test]
    fn resolves_markdown_file_links() {
        let page = context("blog/post", "pages/blog/post.md");
        assert_eq!(page.resolve("other-post.md"), "../../blog/other-post/");
        assert_eq!(page.resolve("../about.md#team"), "../../about/#team");
        assert_eq!(page.resolve("index.md"), "../../blog/");
        assert_eq!(page.resolve("../index.md"), "../../");
    }

    #[test]
    fn leaves_links_outside_the_pages_alone() {
        let page = context("blog/post", "pages/blog/post.md");
        assert_eq!(page.resolve("../../../src/main.rs"), "../../../src/main.rs");
        assert_eq!(page.resolve("../../notes/x.md"), "../../notes/x.md");
    }

    #[test]
    fn tells_which_page_a_link_points_to() {
        let post = context("blog/first", "pages/blog/first.md");
        assert_eq!(post.page_path_of("/about/").as_deref(), Some("about"));
        assert_eq!(
            post.page_path_of("../about.md#team").as_deref(),
            Some("about")
        );
        assert_eq!(
            post.page_path_of("second.md").as_deref(),
            Some("blog/second")
        );
        assert_eq!(post.page_path_of("../index.md").as_deref(), Some(""));
        assert_eq!(post.page_path_of("/media/a.png"), None);
        assert_eq!(post.page_path_of("https://example.com/about"), None);
        assert_eq!(post.page_path_of("#top"), None);
    }

    #[test]
    fn links_media_files() {
        assert_eq!(
            context("", "pages/index.md").media("team.jpg"),
            "media/team.jpg"
        );
        assert_eq!(
            context("about", "pages/about.md").media("team.jpg"),
            "../media/team.jpg"
        );
    }
}

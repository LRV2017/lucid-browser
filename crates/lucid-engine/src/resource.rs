//! Lucid-owned network resource loading and document resource discovery.
//!
//! This is the first layer of the independent engine's resource pipeline. It
//! fetches bounded HTTP(S) resources and discovers common external resources;
//! it does not yet parse CSS, execute scripts, decode media, or render pages.

use std::io::Read;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use reqwest::redirect::Policy;
use reqwest::Url;

use crate::{parse_stylesheet, Document, Element, Node, Stylesheet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    Stylesheet,
    Script,
    Image,
    Font,
    Video,
    Audio,
    Track,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceReference {
    pub url: Url,
    pub kind: ResourceKind,
}

#[derive(Clone, Debug)]
pub struct FetchedResource {
    pub final_url: Url,
    pub kind: ResourceKind,
    pub mime_type: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum ResourceError {
    InvalidUrl(String),
    UnsupportedScheme(String),
    Request(reqwest::Error),
    HttpStatus(reqwest::StatusCode),
    TooLarge { limit: usize },
    Io(std::io::Error),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl(url) => write!(f, "invalid resource URL: {url}"),
            Self::UnsupportedScheme(scheme) => write!(f, "unsupported resource URL scheme: {scheme}"),
            Self::Request(error) => write!(f, "resource request failed: {error}"),
            Self::HttpStatus(status) => write!(f, "resource server returned HTTP {status}"),
            Self::TooLarge { limit } => write!(f, "resource exceeded the {limit}-byte limit"),
            Self::Io(error) => write!(f, "could not read resource body: {error}"),
        }
    }
}

impl std::error::Error for ResourceError {}

impl From<reqwest::Error> for ResourceError {
    fn from(value: reqwest::Error) -> Self {
        Self::Request(value)
    }
}

/// A bounded HTTP(S) loader owned by the Lucid engine.
///
/// The default per-resource cap is intentionally conservative. Streaming media
/// and large downloads will use a separate streaming pipeline in a later stage.
pub struct ResourceLoader {
    client: Client,
    max_resource_bytes: usize,
}

impl ResourceLoader {
    pub fn new(max_resource_bytes: usize) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .user_agent("LucidBrowser/0.0.2 (LucidEngine)")
            .timeout(Duration::from_secs(25))
            .connect_timeout(Duration::from_secs(10))
            .redirect(Policy::limited(10))
            .build()?;
        Ok(Self { client, max_resource_bytes: max_resource_bytes.max(1) })
    }

    pub fn max_resource_bytes(&self) -> usize {
        self.max_resource_bytes
    }

    pub fn resolve(base: &Url, reference: &str) -> Result<Url, ResourceError> {
        let trimmed = reference.trim();
        if trimmed.is_empty() {
            return Err(ResourceError::InvalidUrl(reference.to_owned()));
        }
        let url = base.join(trimmed).map_err(|_| ResourceError::InvalidUrl(reference.to_owned()))?;
        match url.scheme() {
            "http" | "https" => Ok(url),
            scheme => Err(ResourceError::UnsupportedScheme(scheme.to_owned())),
        }
    }

    pub fn fetch(&self, reference: &ResourceReference) -> Result<FetchedResource, ResourceError> {
        match reference.url.scheme() {
            "http" | "https" => {}
            scheme => return Err(ResourceError::UnsupportedScheme(scheme.to_owned())),
        }

        let mut response = self.client.get(reference.url.clone()).send()?;
        if !response.status().is_success() {
            return Err(ResourceError::HttpStatus(response.status()));
        }

        if response.content_length().is_some_and(|length| length > self.max_resource_bytes as u64) {
            return Err(ResourceError::TooLarge { limit: self.max_resource_bytes });
        }

        let mime_type = response.headers().get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.split(';').next().unwrap_or(value).trim().to_ascii_lowercase());

        let final_url = response.url().clone();
        let mut bytes = Vec::new();
        response.by_ref().take(self.max_resource_bytes as u64 + 1).read_to_end(&mut bytes)
            .map_err(ResourceError::Io)?;
        if bytes.len() > self.max_resource_bytes {
            return Err(ResourceError::TooLarge { limit: self.max_resource_bytes });
        }

        Ok(FetchedResource {
            final_url,
            kind: reference.kind,
            mime_type,
            bytes,
        })
    }
}

/// Fetch and parse linked external stylesheets using Lucid's current CSS parser.
/// Rules are appended in document order. The current parser is intentionally a
/// small CSS subset; this does not imply full CSS specification support.
pub fn load_document_stylesheets(
    document: &Document,
    base: &Url,
    loader: &ResourceLoader,
) -> Result<Stylesheet, ResourceError> {
    let mut combined = Stylesheet::default();
    for reference in discover_document_resources(document, base)
        .into_iter()
        .filter(|resource| resource.kind == ResourceKind::Stylesheet)
    {
        let fetched = loader.fetch(&reference)?;
        let css = String::from_utf8_lossy(&fetched.bytes);
        combined.rules.extend(parse_stylesheet(&css).rules);
    }
    Ok(combined)
}

/// Discover resources referenced by CSS url() and @import declarations.
/// URL classification is extension-based until CSS MIME-aware resource loading
/// and font-face parsing are connected to the style system.
pub fn discover_css_resources(css: &str, base: &Url) -> Vec<ResourceReference> {
    let mut output = Vec::new();
    let lower = css.to_ascii_lowercase();
    let mut cursor = 0;

    while let Some(relative) = lower[cursor..].find("url(") {
        let start = cursor + relative + 4;
        let Some(close_relative) = css[start..].find(')') else { break };
        let end = start + close_relative;
        let raw = css[start..end].trim().trim_matches('"').trim_matches('\'');
        push_css_reference(raw, base, &mut output);
        cursor = end + 1;
    }

    let mut cursor = 0;
    while let Some(relative) = lower[cursor..].find("@import") {
        let start = cursor + relative + "@import".len();
        let rest = css[start..].trim_start();
        if !rest.to_ascii_lowercase().starts_with("url(") {
            let raw = rest.split([';', ' ', '\n', '\r', '\t'])
                .next().unwrap_or("").trim_end_matches(';').trim_matches('"').trim_matches('\'');
            push_css_reference(raw, base, &mut output);
        }
        cursor = start;
    }

    output
}

fn push_css_reference(raw: &str, base: &Url, output: &mut Vec<ResourceReference>) {
    if raw.is_empty() || raw.starts_with("data:") || raw.starts_with('#') {
        return;
    }
    let Ok(url) = ResourceLoader::resolve(base, raw) else { return };
    let path = url.path().to_ascii_lowercase();
    let kind = if [".woff2", ".woff", ".ttf", ".otf", ".eot"].iter().any(|ext| path.ends_with(ext)) {
        ResourceKind::Font
    } else if [".mp4", ".webm", ".m4v", ".mov"].iter().any(|ext| path.ends_with(ext)) {
        ResourceKind::Video
    } else if [".mp3", ".ogg", ".wav", ".m4a", ".aac"].iter().any(|ext| path.ends_with(ext)) {
        ResourceKind::Audio
    } else if [".css"].iter().any(|ext| path.ends_with(ext)) {
        ResourceKind::Stylesheet
    } else {
        ResourceKind::Image
    };
    let item = ResourceReference { url, kind };
    if !output.contains(&item) {
        output.push(item);
    }
}

/// Discover external files referenced directly by HTML elements.
/// Inline style blocks and CSS url()/@import references are handled by the
/// stylesheet stage; this pass covers common HTML resource-bearing elements.
pub fn discover_document_resources(document: &Document, base: &Url) -> Vec<ResourceReference> {
    fn visit(nodes: &[Node], base: &Url, output: &mut Vec<ResourceReference>) {
        for node in nodes {
            let Node::Element(element) = node else { continue };
            discover_element(element, base, output);
            visit(&element.children, base, output);
        }
    }

    let mut output = Vec::new();
    visit(&document.children, base, &mut output);
    output
}

fn discover_element(element: &Element, base: &Url, output: &mut Vec<ResourceReference>) {
    let mut add = |value: Option<&str>, kind: ResourceKind| {
        if let Some(value) = value {
            if let Ok(url) = ResourceLoader::resolve(base, value) {
                let item = ResourceReference { url, kind };
                if !output.contains(&item) {
                    output.push(item);
                }
            }
        }
    };

    match element.tag.as_str() {
        "link" => {
            let rel = element.attr("rel").unwrap_or("").to_ascii_lowercase();
            if rel.split_whitespace().any(|token| token == "stylesheet") {
                add(element.attr("href"), ResourceKind::Stylesheet);
            } else if rel.split_whitespace().any(|token| token == "icon" || token == "shortcut") {
                add(element.attr("href"), ResourceKind::Image);
            } else if rel.split_whitespace().any(|token| token == "preload") {
                let kind = match element.attr("as").unwrap_or("") {
                    "style" => ResourceKind::Stylesheet,
                    "script" => ResourceKind::Script,
                    "font" => ResourceKind::Font,
                    "image" => ResourceKind::Image,
                    "video" => ResourceKind::Video,
                    "audio" => ResourceKind::Audio,
                    _ => ResourceKind::Other,
                };
                add(element.attr("href"), kind);
            }
        }
        "script" => add(element.attr("src"), ResourceKind::Script),
        "img" => {
            add(element.attr("src"), ResourceKind::Image);
            if let Some(srcset) = element.attr("srcset") {
                for candidate in srcset.split(',') {
                    add(candidate.split_whitespace().next(), ResourceKind::Image);
                }
            }
        }
        "video" => {
            add(element.attr("src"), ResourceKind::Video);
            add(element.attr("poster"), ResourceKind::Image);
        }
        "audio" => add(element.attr("src"), ResourceKind::Audio),
        "source" => {
            let kind = match element.attr("type").unwrap_or("").split('/').next().unwrap_or("") {
                "video" => ResourceKind::Video,
                "audio" => ResourceKind::Audio,
                "image" => ResourceKind::Image,
                _ => ResourceKind::Other,
            };
            add(element.attr("src"), kind);
            if let Some(srcset) = element.attr("srcset") {
                for candidate in srcset.split(',') {
                    add(candidate.split_whitespace().next(), ResourceKind::Image);
                }
            }
        }
        "track" => add(element.attr("src"), ResourceKind::Track),
        "object" => add(element.attr("data"), ResourceKind::Other),
        "input" if element.attr("type") == Some("image") => add(element.attr("src"), ResourceKind::Image),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_html;

    #[test]
    fn resolves_relative_and_root_relative_urls() {
        let base = Url::parse("https://example.com/articles/page.html").unwrap();
        assert_eq!(
            ResourceLoader::resolve(&base, "../site.css").unwrap().as_str(),
            "https://example.com/site.css"
        );
        assert_eq!(
            ResourceLoader::resolve(&base, "/assets/app.js").unwrap().as_str(),
            "https://example.com/assets/app.js"
        );
    }

    #[test]
    fn rejects_non_network_schemes() {
        let base = Url::parse("https://example.com/").unwrap();
        assert!(matches!(
            ResourceLoader::resolve(&base, "file:///etc/passwd"),
            Err(ResourceError::UnsupportedScheme(_))
        ));
        assert!(matches!(
            ResourceLoader::resolve(&base, "javascript:alert(1)"),
            Err(ResourceError::UnsupportedScheme(_))
        ));
    }

    #[test]
    fn discovers_external_styles_scripts_images_and_media() {
        let document = parse_html(
            r#"<link rel="stylesheet" href="/site.css">
               <script src="app.js"></script>
               <img src="hero.webp">
               <video poster="poster.jpg"><source src="movie.mp4" type="video/mp4"></video>
               <audio src="sound.ogg"></audio>
               <track src="captions.vtt">"#
        );
        let base = Url::parse("https://example.com/pages/home.html").unwrap();
        let resources = discover_document_resources(&document, &base);
        assert!(resources.iter().any(|r| r.kind == ResourceKind::Stylesheet && r.url.path() == "/site.css"));
        assert!(resources.iter().any(|r| r.kind == ResourceKind::Script && r.url.path() == "/pages/app.js"));
        assert!(resources.iter().any(|r| r.kind == ResourceKind::Image && r.url.path() == "/pages/hero.webp"));
        assert!(resources.iter().any(|r| r.kind == ResourceKind::Video && r.url.path() == "/pages/movie.mp4"));
        assert!(resources.iter().any(|r| r.kind == ResourceKind::Audio && r.url.path() == "/pages/sound.ogg"));
        assert!(resources.iter().any(|r| r.kind == ResourceKind::Track && r.url.path() == "/pages/captions.vtt"));
    }

    #[test]
    fn discovers_font_image_and_import_urls_in_css() {
        let base = Url::parse("https://example.com/css/site.css").unwrap();
        let resources = discover_css_resources(
            "@font-face { src: url('../fonts/ui.woff2'); } .hero { background: url(\"../img/hero.webp\"); } @import \"theme.css\";",
            &base,
        );
        assert!(resources.iter().any(|r| r.kind == ResourceKind::Font && r.url.path() == "/fonts/ui.woff2"));
        assert!(resources.iter().any(|r| r.kind == ResourceKind::Image && r.url.path() == "/img/hero.webp"));
        assert!(resources.iter().any(|r| r.kind == ResourceKind::Stylesheet && r.url.path() == "/css/theme.css"));
    }

    #[test]
    fn deduplicates_identical_resource_urls() {
        let document = parse_html(r#"<img src="/same.png"><img src="/same.png">"#);
        let base = Url::parse("https://example.com/").unwrap();
        let resources = discover_document_resources(&document, &base);
        assert_eq!(resources.iter().filter(|r| r.url.path() == "/same.png").count(), 1);
    }
}

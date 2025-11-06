// Embedded Web UI static files

#[cfg(feature = "server")]
use rust_embed::RustEmbed;

// Skip embedding web assets when building docs.rs to avoid missing directory error
#[cfg(all(feature = "server", not(docsrs)))]
#[derive(RustEmbed)]
#[folder = "web/out/"]
pub struct WebAssets;

// Provide a stub implementation for docs.rs builds
#[cfg(all(feature = "server", docsrs))]
pub struct WebAssets;

#[cfg(all(feature = "server", docsrs))]
impl WebAssets {
    /// Stub method for docs.rs - not functional
    pub fn get(_file_path: &str) -> Option<rust_embed::EmbeddedFile> {
        None
    }

    /// Stub method for docs.rs - not functional
    pub fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> {
        std::iter::empty()
    }
}

#[cfg(feature = "server")]
pub fn get_content_type(path: &str) -> &'static str {
    let path = path.to_lowercase();
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else if path.ends_with(".txt") {
        "text/plain; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}

#[cfg(feature = "server")]
pub fn resolve_path(path: &str) -> String {
    let path = path.trim_start_matches('/');

    // If path is empty or root, serve index.html
    if path.is_empty() || path == "/" {
        return "index.html".to_string();
    }

    // If path doesn't have an extension, try appending index.html
    if !path.contains('.') {
        let with_trailing_slash = if path.ends_with('/') {
            format!("{}index.html", path)
        } else {
            format!("{}/index.html", path)
        };

        // Check if the path with index.html exists
        if WebAssets::get(&with_trailing_slash).is_some() {
            return with_trailing_slash;
        }

        // Also try .html extension
        let with_html = format!("{}.html", path.trim_end_matches('/'));
        if WebAssets::get(&with_html).is_some() {
            return with_html;
        }
    }

    path.to_string()
}

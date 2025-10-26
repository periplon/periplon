// Static file handler for embedded web UI

#[cfg(feature = "server")]
use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::Response,
};

#[cfg(feature = "server")]
use crate::server::web_ui::{get_content_type, resolve_path, WebAssets};

#[cfg(feature = "server")]
pub async fn serve_static(Path(path): Path<String>) -> Response {
    serve_file(&path).await
}

#[cfg(feature = "server")]
pub async fn serve_index() -> Response {
    serve_file("").await
}

#[cfg(feature = "server")]
async fn serve_file(path: &str) -> Response {
    let resolved_path = resolve_path(path);

    match WebAssets::get(&resolved_path) {
        Some(content) => {
            let content_type = get_content_type(&resolved_path);
            let body = content.data.into_owned();

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CACHE_CONTROL, "public, max-age=31536000")
                .body(body.into())
                .unwrap()
        }
        None => {
            // For SPA routing, serve index.html for non-file paths
            if !resolved_path.contains('.') {
                if let Some(index) = WebAssets::get("index.html") {
                    let body = index.data.into_owned();
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                        .header(header::CACHE_CONTROL, "no-cache")
                        .body(body.into())
                        .unwrap();
                }
            }

            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "text/plain")
                .body("Not Found".into())
                .unwrap()
        }
    }
}

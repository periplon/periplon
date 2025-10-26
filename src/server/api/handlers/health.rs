// Health check handlers

#[cfg(feature = "server")]
use axum::{response::IntoResponse, Json};
#[cfg(feature = "server")]
use serde_json::json;

#[cfg(feature = "server")]
pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[cfg(feature = "server")]
pub async fn version() -> impl IntoResponse {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": env!("CARGO_PKG_NAME")
    }))
}

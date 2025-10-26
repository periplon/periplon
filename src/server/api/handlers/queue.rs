// Queue handlers

#[cfg(feature = "server")]
use axum::{http::StatusCode, response::IntoResponse, Json};
#[cfg(feature = "server")]
use serde_json::json;

#[cfg(feature = "server")]
pub async fn get_queue_stats() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "pending": 0,
            "processing": 0,
            "completed": 0,
            "failed": 0
        })),
    )
}

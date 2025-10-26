// Monitoring and metrics handlers

#[cfg(feature = "server")]
use crate::server::storage::WorkflowFilter;
#[cfg(feature = "server")]
use crate::server::{Storage, WorkQueue};
#[cfg(feature = "server")]
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
#[cfg(feature = "server")]
use serde_json::json;
#[cfg(feature = "server")]
use std::sync::Arc;

/// Health check endpoint
/// Returns basic health status of the service
#[cfg(feature = "server")]
pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })),
    )
}

/// Readiness check endpoint
/// Checks if the service is ready to accept requests
#[cfg(feature = "server")]
pub async fn readiness_check(
    Extension(storage): Extension<Arc<dyn Storage>>,
    queue: Option<Extension<Arc<dyn WorkQueue>>>,
) -> impl IntoResponse {
    let mut checks = serde_json::Map::new();
    let mut all_ok = true;

    // Check storage connectivity by attempting to list workflows
    let filter = WorkflowFilter {
        name: None,
        tags: vec![],
        created_by: None,
        is_active: None,
        created_after: None,
        created_before: None,
        limit: Some(1),
        offset: None,
    };
    let storage_status = match storage.list_workflows(&filter).await {
        Ok(_) => "ok",
        Err(_) => {
            all_ok = false;
            "error"
        }
    };
    checks.insert("storage".to_string(), json!(storage_status));

    // Check queue if available
    if let Some(Extension(queue)) = queue {
        let queue_status = match queue.stats().await {
            Ok(_) => "ok",
            Err(_) => {
                all_ok = false;
                "error"
            }
        };
        checks.insert("queue".to_string(), json!(queue_status));
    } else {
        checks.insert("queue".to_string(), json!("not_configured"));
    }

    let status_code = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(json!({
            "status": if all_ok { "ready" } else { "not_ready" },
            "checks": checks,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })),
    )
}

/// Liveness check endpoint
/// Checks if the service is alive
#[cfg(feature = "server")]
pub async fn liveness_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "alive"
        })),
    )
}

/// Prometheus-compatible metrics endpoint
/// Returns metrics in Prometheus text format
#[cfg(feature = "server")]
pub async fn metrics() -> impl IntoResponse {
    let metrics = "# HELP workflow_executions_total Total number of workflow executions\n\
         # TYPE workflow_executions_total counter\n\
         workflow_executions_total 0\n\
         \n\
         # HELP workflow_executions_active Currently active workflow executions\n\
         # TYPE workflow_executions_active gauge\n\
         workflow_executions_active 0\n\
         \n\
         # HELP workflow_executions_duration_seconds Workflow execution duration\n\
         # TYPE workflow_executions_duration_seconds histogram\n\
         workflow_executions_duration_seconds_bucket{le=\"1\"} 0\n\
         workflow_executions_duration_seconds_bucket{le=\"5\"} 0\n\
         workflow_executions_duration_seconds_bucket{le=\"10\"} 0\n\
         workflow_executions_duration_seconds_bucket{le=\"30\"} 0\n\
         workflow_executions_duration_seconds_bucket{le=\"60\"} 0\n\
         workflow_executions_duration_seconds_bucket{le=\"120\"} 0\n\
         workflow_executions_duration_seconds_bucket{le=\"300\"} 0\n\
         workflow_executions_duration_seconds_bucket{le=\"+Inf\"} 0\n\
         workflow_executions_duration_seconds_sum 0\n\
         workflow_executions_duration_seconds_count 0\n\
         \n\
         # HELP queue_jobs_total Total number of jobs in the queue\n\
         # TYPE queue_jobs_total gauge\n\
         queue_jobs_total{status=\"pending\"} 0\n\
         queue_jobs_total{status=\"processing\"} 0\n\
         queue_jobs_total{status=\"completed\"} 0\n\
         queue_jobs_total{status=\"failed\"} 0\n\
         \n\
         # HELP http_requests_total Total number of HTTP requests\n\
         # TYPE http_requests_total counter\n\
         http_requests_total 0\n\
         \n\
         # HELP http_request_duration_seconds HTTP request duration\n\
         # TYPE http_request_duration_seconds histogram\n\
         http_request_duration_seconds_bucket{le=\"0.005\"} 0\n\
         http_request_duration_seconds_bucket{le=\"0.01\"} 0\n\
         http_request_duration_seconds_bucket{le=\"0.025\"} 0\n\
         http_request_duration_seconds_bucket{le=\"0.05\"} 0\n\
         http_request_duration_seconds_bucket{le=\"0.1\"} 0\n\
         http_request_duration_seconds_bucket{le=\"0.25\"} 0\n\
         http_request_duration_seconds_bucket{le=\"0.5\"} 0\n\
         http_request_duration_seconds_bucket{le=\"1\"} 0\n\
         http_request_duration_seconds_bucket{le=\"2.5\"} 0\n\
         http_request_duration_seconds_bucket{le=\"5\"} 0\n\
         http_request_duration_seconds_bucket{le=\"10\"} 0\n\
         http_request_duration_seconds_bucket{le=\"+Inf\"} 0\n\
         http_request_duration_seconds_sum 0\n\
         http_request_duration_seconds_count 0\n\
         \n\
         # HELP system_memory_usage_bytes Memory usage in bytes\n\
         # TYPE system_memory_usage_bytes gauge\n\
         system_memory_usage_bytes 0\n\
         \n\
         # HELP system_cpu_usage_percent CPU usage percentage\n\
         # TYPE system_cpu_usage_percent gauge\n\
         system_cpu_usage_percent 0\n\
         ".to_string();

    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        metrics,
    )
}

/// Stats endpoint - JSON format statistics
/// Returns various statistics about the system
#[cfg(feature = "server")]
pub async fn stats() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "workflows": {
                "total": 0,
                "active": 0
            },
            "executions": {
                "total": 0,
                "running": 0,
                "completed": 0,
                "failed": 0
            },
            "queue": {
                "pending": 0,
                "processing": 0,
                "completed": 0,
                "failed": 0
            },
            "system": {
                "uptime_seconds": 0,
                "memory_usage_bytes": 0,
                "cpu_usage_percent": 0.0
            }
        })),
    )
}

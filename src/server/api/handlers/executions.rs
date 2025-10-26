// Execution handlers

#[cfg(feature = "server")]
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use serde_json::json;
#[cfg(feature = "server")]
use std::sync::Arc;
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use crate::server::auth::jwt::Claims;
#[cfg(feature = "server")]
use crate::server::queue::{Job, WorkQueue};
#[cfg(feature = "server")]
use crate::server::{
    storage::{Execution, ExecutionFilter, ExecutionLog, ExecutionStatus},
    Storage,
};

// Request/Response types
#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct ListExecutionsQuery {
    pub workflow_id: Option<Uuid>,
    pub status: Option<String>, // Parse to ExecutionStatus
    pub triggered_by: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
pub struct ExecutionResponse {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_version: String,
    pub status: ExecutionStatus,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub triggered_by: Option<String>,
    pub trigger_type: String,
    pub input_params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub retry_count: u32,
    pub parent_execution_id: Option<Uuid>,
}

#[cfg(feature = "server")]
impl From<Execution> for ExecutionResponse {
    fn from(execution: Execution) -> Self {
        Self {
            id: execution.id,
            workflow_id: execution.workflow_id,
            workflow_version: execution.workflow_version,
            status: execution.status,
            started_at: execution.started_at.map(|dt| dt.to_rfc3339()),
            completed_at: execution.completed_at.map(|dt| dt.to_rfc3339()),
            created_at: execution.created_at.to_rfc3339(),
            triggered_by: execution.triggered_by,
            trigger_type: execution.trigger_type,
            input_params: execution.input_params,
            result: execution.result,
            error: execution.error,
            retry_count: execution.retry_count,
            parent_execution_id: execution.parent_execution_id,
        }
    }
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct CreateExecutionRequest {
    pub workflow_id: Uuid,
    pub input_params: Option<serde_json::Value>,
    pub trigger_type: Option<String>,
    pub priority: Option<i32>,
    pub parent_execution_id: Option<Uuid>,
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct GetLogsQuery {
    pub limit: Option<usize>,
    pub level: Option<String>,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
pub struct ExecutionLogResponse {
    pub id: Option<i64>,
    pub execution_id: Uuid,
    pub task_execution_id: Option<Uuid>,
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

#[cfg(feature = "server")]
impl From<ExecutionLog> for ExecutionLogResponse {
    fn from(log: ExecutionLog) -> Self {
        Self {
            id: log.id,
            execution_id: log.execution_id,
            task_execution_id: log.task_execution_id,
            timestamp: log.timestamp.to_rfc3339(),
            level: log.level,
            message: log.message,
            metadata: log.metadata,
        }
    }
}

// Handler implementations

/// List executions with filtering and pagination
#[cfg(feature = "server")]
pub async fn list_executions(
    Query(query): Query<ListExecutionsQuery>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    // Parse status if provided
    let status = query.status.and_then(|s| match s.to_lowercase().as_str() {
        "queued" => Some(ExecutionStatus::Queued),
        "running" => Some(ExecutionStatus::Running),
        "completed" => Some(ExecutionStatus::Completed),
        "failed" => Some(ExecutionStatus::Failed),
        "cancelled" => Some(ExecutionStatus::Cancelled),
        "paused" => Some(ExecutionStatus::Paused),
        _ => None,
    });

    let filter = ExecutionFilter {
        workflow_id: query.workflow_id,
        status,
        triggered_by: query.triggered_by,
        started_after: None,
        started_before: None,
        completed_after: None,
        completed_before: None,
        limit: query.limit,
        offset: query.offset,
    };

    match storage.list_executions(&filter).await {
        Ok(executions) => {
            let responses: Vec<ExecutionResponse> = executions
                .into_iter()
                .map(ExecutionResponse::from)
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "executions": responses,
                    "total": responses.len(),
                    "offset": filter.offset.unwrap_or(0),
                    "limit": filter.limit.unwrap_or(100),
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to list executions",
                "message": e.to_string()
            })),
        ),
    }
}

/// Create a new execution and queue it for processing
#[cfg(feature = "server")]
pub async fn create_execution(
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(queue): Extension<Arc<dyn WorkQueue>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateExecutionRequest>,
) -> impl IntoResponse {
    // Get workflow to validate it exists and get version
    let workflow = match storage.get_workflow(payload.workflow_id).await {
        Ok(Some((workflow, _metadata))) => workflow,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Workflow not found",
                    "workflow_id": payload.workflow_id
                })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get workflow",
                    "message": e.to_string()
                })),
            );
        }
    };

    // Create execution record
    let execution = Execution {
        id: Uuid::new_v4(),
        workflow_id: payload.workflow_id,
        workflow_version: workflow.version.clone(),
        status: ExecutionStatus::Queued,
        started_at: None,
        completed_at: None,
        created_at: Utc::now(),
        triggered_by: Some(claims.sub.clone()),
        trigger_type: payload.trigger_type.unwrap_or_else(|| "manual".to_string()),
        input_params: payload.input_params.clone(),
        result: None,
        error: None,
        retry_count: 0,
        parent_execution_id: payload.parent_execution_id,
    };

    // Store execution
    let execution_id = match storage.store_execution(&execution).await {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to create execution",
                    "message": e.to_string()
                })),
            );
        }
    };

    // Create job for queue
    let job_payload = json!({
        "workflow": workflow,
        "input_params": payload.input_params,
    });

    let mut job = Job::new(payload.workflow_id, execution_id, job_payload);
    if let Some(priority) = payload.priority {
        job = job.with_priority(priority);
    }

    // Enqueue job
    match queue.enqueue(job).await {
        Ok(_job_id) => (
            StatusCode::CREATED,
            Json(json!({
                "id": execution_id,
                "workflow_id": payload.workflow_id,
                "status": "queued",
                "message": "Execution queued successfully"
            })),
        ),
        Err(e) => {
            // Job queuing failed, but execution record exists
            // Mark execution as failed
            let mut failed_execution = execution;
            failed_execution.status = ExecutionStatus::Failed;
            failed_execution.error = Some(format!("Failed to queue job: {}", e));
            let _ = storage
                .update_execution(execution_id, &failed_execution)
                .await;

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to queue execution",
                    "message": e.to_string()
                })),
            )
        }
    }
}

/// Get a specific execution by ID
#[cfg(feature = "server")]
pub async fn get_execution(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    match storage.get_execution(id).await {
        Ok(Some(execution)) => {
            let response = ExecutionResponse::from(execution);
            (StatusCode::OK, Json(json!(response)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Execution not found",
                "id": id
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to get execution",
                "message": e.to_string()
            })),
        ),
    }
}

/// Cancel a running or queued execution
#[cfg(feature = "server")]
pub async fn cancel_execution(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    // Get existing execution
    let mut execution = match storage.get_execution(id).await {
        Ok(Some(execution)) => execution,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Execution not found",
                    "id": id
                })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get execution",
                    "message": e.to_string()
                })),
            );
        }
    };

    // Check if execution can be cancelled
    match execution.status {
        ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Cannot cancel execution",
                    "message": format!("Execution is already in {} state",
                        match execution.status {
                            ExecutionStatus::Completed => "completed",
                            ExecutionStatus::Failed => "failed",
                            ExecutionStatus::Cancelled => "cancelled",
                            _ => "unknown"
                        }),
                    "status": execution.status
                })),
            );
        }
        _ => {}
    }

    // Update execution status to cancelled
    execution.status = ExecutionStatus::Cancelled;
    execution.completed_at = Some(Utc::now());

    match storage.update_execution(id, &execution).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "id": id,
                "status": "cancelled",
                "message": "Execution cancelled successfully",
                "completed_at": execution.completed_at.map(|dt| dt.to_rfc3339())
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to cancel execution",
                "message": e.to_string()
            })),
        ),
    }
}

/// Get execution logs
#[cfg(feature = "server")]
pub async fn get_execution_logs(
    Path(id): Path<Uuid>,
    Query(query): Query<GetLogsQuery>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    // Check if execution exists
    match storage.get_execution(id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Execution not found",
                    "id": id
                })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to check execution",
                    "message": e.to_string()
                })),
            );
        }
    }

    // Get logs
    match storage.get_execution_logs(id, query.limit).await {
        Ok(logs) => {
            // Filter by level if specified
            let filtered_logs: Vec<ExecutionLogResponse> = logs
                .into_iter()
                .filter(|log| {
                    query
                        .level
                        .as_ref()
                        .map(|level| log.level.eq_ignore_ascii_case(level))
                        .unwrap_or(true)
                })
                .map(ExecutionLogResponse::from)
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "execution_id": id,
                    "logs": filtered_logs,
                    "total": filtered_logs.len()
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to get execution logs",
                "message": e.to_string()
            })),
        ),
    }
}

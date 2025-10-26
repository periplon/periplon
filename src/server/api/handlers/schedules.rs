// Schedule handlers

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
    storage::{Schedule, ScheduleFilter},
    Storage,
};

// Request/Response types
#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct ListSchedulesQuery {
    pub workflow_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
pub struct ScheduleResponse {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub cron_expression: String,
    pub timezone: String,
    pub is_active: bool,
    pub input_params: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub description: Option<String>,
}

#[cfg(feature = "server")]
impl From<Schedule> for ScheduleResponse {
    fn from(schedule: Schedule) -> Self {
        Self {
            id: schedule.id,
            workflow_id: schedule.workflow_id,
            cron_expression: schedule.cron_expression,
            timezone: schedule.timezone,
            is_active: schedule.is_active,
            input_params: schedule.input_params,
            created_at: schedule.created_at.to_rfc3339(),
            updated_at: schedule.updated_at.to_rfc3339(),
            created_by: schedule.created_by,
            last_run_at: schedule.last_run_at.map(|dt| dt.to_rfc3339()),
            next_run_at: schedule.next_run_at.map(|dt| dt.to_rfc3339()),
            description: schedule.description,
        }
    }
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub workflow_id: Uuid,
    pub cron_expression: String,
    pub timezone: Option<String>,
    pub input_params: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    pub cron_expression: Option<String>,
    pub timezone: Option<String>,
    pub is_active: Option<bool>,
    pub input_params: Option<serde_json::Value>,
    pub description: Option<String>,
}

// Handler implementations

/// List schedules with filtering and pagination
#[cfg(feature = "server")]
pub async fn list_schedules(
    Query(query): Query<ListSchedulesQuery>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    let filter = ScheduleFilter {
        workflow_id: query.workflow_id,
        is_active: query.is_active,
        created_by: None,
        limit: query.limit,
        offset: query.offset,
    };

    match storage.list_schedules(&filter).await {
        Ok(schedules) => {
            let responses: Vec<ScheduleResponse> =
                schedules.into_iter().map(ScheduleResponse::from).collect();

            (
                StatusCode::OK,
                Json(json!({
                    "schedules": responses,
                    "total": responses.len(),
                    "offset": filter.offset.unwrap_or(0),
                    "limit": filter.limit.unwrap_or(100),
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to list schedules",
                "message": e.to_string()
            })),
        ),
    }
}

/// Create a new schedule
#[cfg(feature = "server")]
pub async fn create_schedule(
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateScheduleRequest>,
) -> impl IntoResponse {
    // Validate workflow exists
    match storage.get_workflow(payload.workflow_id).await {
        Ok(Some(_)) => {}
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
                    "error": "Failed to validate workflow",
                    "message": e.to_string()
                })),
            );
        }
    }

    // TODO: Validate cron expression
    // For now, just accept any string

    let schedule = Schedule {
        id: Uuid::new_v4(),
        workflow_id: payload.workflow_id,
        cron_expression: payload.cron_expression,
        timezone: payload.timezone.unwrap_or_else(|| "UTC".to_string()),
        is_active: true,
        input_params: payload.input_params,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: Some(claims.sub.clone()),
        last_run_at: None,
        next_run_at: None, // TODO: Calculate next run time from cron expression
        description: payload.description,
    };

    match storage.store_schedule(&schedule).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({
                "id": id,
                "workflow_id": schedule.workflow_id,
                "message": "Schedule created successfully"
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to create schedule",
                "message": e.to_string()
            })),
        ),
    }
}

/// Get a specific schedule by ID
#[cfg(feature = "server")]
pub async fn get_schedule(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    match storage.get_schedule(id).await {
        Ok(Some(schedule)) => {
            let response = ScheduleResponse::from(schedule);
            (StatusCode::OK, Json(json!(response)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Schedule not found",
                "id": id
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to get schedule",
                "message": e.to_string()
            })),
        ),
    }
}

/// Update an existing schedule
#[cfg(feature = "server")]
pub async fn update_schedule(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
    Json(payload): Json<UpdateScheduleRequest>,
) -> impl IntoResponse {
    // Get existing schedule
    let mut schedule = match storage.get_schedule(id).await {
        Ok(Some(schedule)) => schedule,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Schedule not found",
                    "id": id
                })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get schedule",
                    "message": e.to_string()
                })),
            );
        }
    };

    // Update fields
    if let Some(cron_expression) = payload.cron_expression {
        // TODO: Validate cron expression
        schedule.cron_expression = cron_expression;
        schedule.next_run_at = None; // TODO: Recalculate next run time
    }

    if let Some(timezone) = payload.timezone {
        schedule.timezone = timezone;
        schedule.next_run_at = None; // TODO: Recalculate next run time
    }

    if let Some(is_active) = payload.is_active {
        schedule.is_active = is_active;
    }

    if let Some(input_params) = payload.input_params {
        schedule.input_params = Some(input_params);
    }

    if let Some(description) = payload.description {
        schedule.description = Some(description);
    }

    schedule.updated_at = Utc::now();

    match storage.update_schedule(id, &schedule).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "id": id,
                "message": "Schedule updated successfully",
                "updated_at": schedule.updated_at.to_rfc3339()
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to update schedule",
                "message": e.to_string()
            })),
        ),
    }
}

/// Delete a schedule
#[cfg(feature = "server")]
pub async fn delete_schedule(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    // Check if schedule exists
    match storage.get_schedule(id).await {
        Ok(Some(_)) => {
            // Schedule exists, proceed with deletion
            match storage.delete_schedule(id).await {
                Ok(_) => (StatusCode::NO_CONTENT, Json(json!({}))),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to delete schedule",
                        "message": e.to_string()
                    })),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Schedule not found",
                "id": id
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to check schedule",
                "message": e.to_string()
            })),
        ),
    }
}

/// Manually trigger a scheduled workflow
#[cfg(feature = "server")]
pub async fn trigger_schedule(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(queue): Extension<Arc<dyn WorkQueue>>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    // Get schedule
    let schedule = match storage.get_schedule(id).await {
        Ok(Some(schedule)) => schedule,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Schedule not found",
                    "id": id
                })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get schedule",
                    "message": e.to_string()
                })),
            );
        }
    };

    // Get workflow
    let (workflow, _) = match storage.get_workflow(schedule.workflow_id).await {
        Ok(Some(wf)) => wf,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Workflow not found",
                    "workflow_id": schedule.workflow_id
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

    // Create execution record (similar to create_execution handler)
    use crate::server::storage::{Execution, ExecutionStatus};

    let execution = Execution {
        id: Uuid::new_v4(),
        workflow_id: schedule.workflow_id,
        workflow_version: workflow.version.clone(),
        status: ExecutionStatus::Queued,
        started_at: None,
        completed_at: None,
        created_at: Utc::now(),
        triggered_by: Some(claims.sub.clone()),
        trigger_type: "scheduled_manual".to_string(),
        input_params: schedule.input_params.clone(),
        result: None,
        error: None,
        retry_count: 0,
        parent_execution_id: None,
    };

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
        "input_params": schedule.input_params,
    });

    let job = Job::new(schedule.workflow_id, execution_id, job_payload);

    // Enqueue job
    match queue.enqueue(job).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({
                "schedule_id": id,
                "execution_id": execution_id,
                "workflow_id": schedule.workflow_id,
                "status": "queued",
                "message": "Schedule triggered successfully"
            })),
        ),
        Err(e) => {
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

// Workflow handlers

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
use crate::dsl::schema::DSLWorkflow;
#[cfg(feature = "server")]
use crate::server::auth::jwt::Claims;
#[cfg(feature = "server")]
use crate::server::{
    storage::{WorkflowFilter, WorkflowMetadata},
    Storage,
};

// Request/Response types
#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct ListWorkflowsQuery {
    pub name: Option<String>,
    pub tags: Option<String>, // Comma-separated tags
    pub created_by: Option<String>,
    pub is_active: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    pub tags: Vec<String>,
    pub is_active: bool,
    pub definition: DSLWorkflow,
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub definition: DSLWorkflow,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub definition: Option<DSLWorkflow>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// Handler implementations

/// List workflows with filtering and pagination
#[cfg(feature = "server")]
pub async fn list_workflows(
    Query(query): Query<ListWorkflowsQuery>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    // Parse tags if provided
    let tags = query
        .tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let filter = WorkflowFilter {
        name: query.name,
        tags,
        created_by: query.created_by,
        is_active: query.is_active,
        created_after: None,
        created_before: None,
        limit: query.limit,
        offset: query.offset,
    };

    match storage.list_workflows(&filter).await {
        Ok(workflows) => {
            let responses: Vec<WorkflowResponse> = workflows
                .into_iter()
                .map(|(workflow, metadata)| WorkflowResponse {
                    id: metadata.id,
                    name: metadata.name,
                    version: metadata.version,
                    description: metadata.description,
                    created_at: metadata.created_at.to_rfc3339(),
                    updated_at: metadata.updated_at.to_rfc3339(),
                    created_by: metadata.created_by,
                    tags: metadata.tags,
                    is_active: metadata.is_active,
                    definition: workflow,
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "workflows": responses,
                    "total": responses.len(),
                    "offset": filter.offset.unwrap_or(0),
                    "limit": filter.limit.unwrap_or(100),
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to list workflows",
                "message": e.to_string()
            })),
        ),
    }
}

/// Create a new workflow
#[cfg(feature = "server")]
pub async fn create_workflow(
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateWorkflowRequest>,
) -> impl IntoResponse {
    // Validate workflow first
    if let Err(error) = crate::dsl::validator::validate_workflow(&payload.definition) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Workflow validation failed",
                "message": error.to_string()
            })),
        );
    }

    let metadata = WorkflowMetadata {
        id: Uuid::new_v4(),
        name: payload.definition.name.clone(),
        version: payload.definition.version.clone(),
        description: payload.description,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: Some(claims.sub.clone()),
        tags: payload.tags.unwrap_or_default(),
        is_active: true,
    };

    match storage.store_workflow(&payload.definition, &metadata).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({
                "id": id,
                "name": metadata.name,
                "version": metadata.version,
                "message": "Workflow created successfully"
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to create workflow",
                "message": e.to_string()
            })),
        ),
    }
}

/// Get a specific workflow by ID
#[cfg(feature = "server")]
pub async fn get_workflow(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    match storage.get_workflow(id).await {
        Ok(Some((workflow, metadata))) => {
            let response = WorkflowResponse {
                id: metadata.id,
                name: metadata.name,
                version: metadata.version,
                description: metadata.description,
                created_at: metadata.created_at.to_rfc3339(),
                updated_at: metadata.updated_at.to_rfc3339(),
                created_by: metadata.created_by,
                tags: metadata.tags,
                is_active: metadata.is_active,
                definition: workflow,
            };

            (StatusCode::OK, Json(json!(response)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Workflow not found",
                "id": id
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to get workflow",
                "message": e.to_string()
            })),
        ),
    }
}

/// Update an existing workflow
#[cfg(feature = "server")]
pub async fn update_workflow(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
    Json(payload): Json<UpdateWorkflowRequest>,
) -> impl IntoResponse {
    // Get existing workflow
    let (mut workflow, mut metadata) = match storage.get_workflow(id).await {
        Ok(Some(wf)) => wf,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Workflow not found",
                    "id": id
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

    // Update fields
    if let Some(new_definition) = payload.definition {
        // Validate new definition
        if let Err(error) = crate::dsl::validator::validate_workflow(&new_definition) {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Workflow validation failed",
                    "message": error.to_string()
                })),
            );
        }
        workflow = new_definition;
    }

    if let Some(desc) = payload.description {
        metadata.description = Some(desc);
    }

    if let Some(tags) = payload.tags {
        metadata.tags = tags;
    }

    if let Some(is_active) = payload.is_active {
        metadata.is_active = is_active;
    }

    metadata.updated_at = Utc::now();

    match storage.update_workflow(id, &workflow, &metadata).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "id": id,
                "message": "Workflow updated successfully",
                "updated_at": metadata.updated_at.to_rfc3339()
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to update workflow",
                "message": e.to_string()
            })),
        ),
    }
}

/// Delete a workflow
#[cfg(feature = "server")]
pub async fn delete_workflow(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    // Check if workflow exists
    match storage.get_workflow(id).await {
        Ok(Some(_)) => {
            // Workflow exists, proceed with deletion
            match storage.delete_workflow(id).await {
                Ok(_) => (StatusCode::NO_CONTENT, Json(json!({}))),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to delete workflow",
                        "message": e.to_string()
                    })),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Workflow not found",
                "id": id
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to check workflow",
                "message": e.to_string()
            })),
        ),
    }
}

/// Validate a workflow without storing it
#[cfg(feature = "server")]
pub async fn validate_workflow(Json(workflow): Json<DSLWorkflow>) -> impl IntoResponse {
    match crate::dsl::validator::validate_workflow(&workflow) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!(ValidationResponse {
                valid: true,
                errors: vec![],
                warnings: vec![],
            })),
        ),
        Err(error) => {
            (
                StatusCode::OK, // Return 200 with validation results
                Json(json!(ValidationResponse {
                    valid: false,
                    errors: vec![error.to_string()],
                    warnings: vec![],
                })),
            )
        }
    }
}

/// Get workflow versions
#[cfg(feature = "server")]
pub async fn list_workflow_versions(
    Path(id): Path<Uuid>,
    Extension(_storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    // For now, return empty list - full version management would require
    // additional storage layer support
    (
        StatusCode::OK,
        Json(json!({
            "workflow_id": id,
            "versions": []
        })),
    )
}

/// Get a specific workflow version
#[cfg(feature = "server")]
pub async fn get_workflow_version(
    Path((id, version)): Path<(Uuid, String)>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    match storage.get_workflow_version(id, &version).await {
        Ok(Some((workflow, metadata))) => {
            let response = WorkflowResponse {
                id: metadata.id,
                name: metadata.name,
                version: metadata.version,
                description: metadata.description,
                created_at: metadata.created_at.to_rfc3339(),
                updated_at: metadata.updated_at.to_rfc3339(),
                created_by: metadata.created_by,
                tags: metadata.tags,
                is_active: metadata.is_active,
                definition: workflow,
            };

            (StatusCode::OK, Json(json!(response)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Workflow version not found",
                "id": id,
                "version": version
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to get workflow version",
                "message": e.to_string()
            })),
        ),
    }
}

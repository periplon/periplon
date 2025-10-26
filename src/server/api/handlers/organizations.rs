// Organization and Team handlers

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
use crate::server::{
    storage::{Organization, OrganizationFilter, Team, TeamFilter, TeamMember},
    Storage,
};

// Organization Request/Response types
#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct ListOrganizationsQuery {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub is_active: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
pub struct OrganizationResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub plan: String,
    pub settings: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
    pub is_active: bool,
}

#[cfg(feature = "server")]
impl From<Organization> for OrganizationResponse {
    fn from(org: Organization) -> Self {
        Self {
            id: org.id,
            name: org.name,
            slug: org.slug,
            description: org.description,
            logo_url: org.logo_url,
            plan: org.plan,
            settings: org.settings,
            created_at: org.created_at.to_rfc3339(),
            updated_at: org.updated_at.to_rfc3339(),
            is_active: org.is_active,
        }
    }
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub plan: Option<String>,
    pub is_active: Option<bool>,
}

// Team Request/Response types
#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct ListTeamsQuery {
    pub organization_id: Option<Uuid>,
    pub name: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(feature = "server")]
impl From<Team> for TeamResponse {
    fn from(team: Team) -> Self {
        Self {
            id: team.id,
            organization_id: team.organization_id,
            name: team.name,
            description: team.description,
            created_at: team.created_at.to_rfc3339(),
            updated_at: team.updated_at.to_rfc3339(),
        }
    }
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct UpdateTeamRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct AddTeamMemberRequest {
    pub user_id: Uuid,
    pub role: Option<String>,
}

// Organization Handlers
#[cfg(feature = "server")]
pub async fn list_organizations(
    Query(query): Query<ListOrganizationsQuery>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    let filter = OrganizationFilter {
        name: query.name,
        slug: query.slug,
        is_active: query.is_active,
        limit: query.limit,
        offset: query.offset,
    };

    match storage.list_organizations(&filter).await {
        Ok(orgs) => {
            let responses: Vec<OrganizationResponse> = orgs.into_iter().map(Into::into).collect();
            (
                StatusCode::OK,
                Json(json!({
                    "organizations": responses,
                    "total": responses.len(),
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to list organizations", "message": e.to_string()})),
        ),
    }
}

#[cfg(feature = "server")]
pub async fn create_organization(
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
    Json(payload): Json<CreateOrganizationRequest>,
) -> impl IntoResponse {
    let organization = Organization {
        id: Uuid::new_v4(),
        name: payload.name,
        slug: payload.slug,
        description: payload.description,
        logo_url: payload.logo_url,
        plan: "free".to_string(),
        settings: json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_active: true,
    };

    match storage.store_organization(&organization).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({"id": id, "message": "Organization created"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to create organization", "message": e.to_string()})),
        ),
    }
}

#[cfg(feature = "server")]
pub async fn get_organization(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    match storage.get_organization(id).await {
        Ok(Some(org)) => (StatusCode::OK, Json(json!(OrganizationResponse::from(org)))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Organization not found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to get organization", "message": e.to_string()})),
        ),
    }
}

#[cfg(feature = "server")]
pub async fn update_organization(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
    Json(payload): Json<UpdateOrganizationRequest>,
) -> impl IntoResponse {
    let mut org = match storage.get_organization(id).await {
        Ok(Some(org)) => org,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Organization not found"})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get organization", "message": e.to_string()})),
            )
        }
    };

    if let Some(name) = payload.name {
        org.name = name;
    }
    if let Some(desc) = payload.description {
        org.description = Some(desc);
    }
    if let Some(logo) = payload.logo_url {
        org.logo_url = Some(logo);
    }
    if let Some(plan) = payload.plan {
        org.plan = plan;
    }
    if let Some(active) = payload.is_active {
        org.is_active = active;
    }
    org.updated_at = Utc::now();

    match storage.update_organization(id, &org).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Organization updated"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to update organization", "message": e.to_string()})),
        ),
    }
}

#[cfg(feature = "server")]
pub async fn delete_organization(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    match storage.delete_organization(id).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(json!({}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to delete organization", "message": e.to_string()})),
        ),
    }
}

// Team Handlers
#[cfg(feature = "server")]
pub async fn list_teams(
    Query(query): Query<ListTeamsQuery>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    let filter = TeamFilter {
        organization_id: query.organization_id,
        name: query.name,
        limit: query.limit,
        offset: query.offset,
    };

    match storage.list_teams(&filter).await {
        Ok(teams) => {
            let responses: Vec<TeamResponse> = teams.into_iter().map(Into::into).collect();
            (
                StatusCode::OK,
                Json(json!({"teams": responses, "total": responses.len()})),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to list teams", "message": e.to_string()})),
        ),
    }
}

#[cfg(feature = "server")]
pub async fn create_team(
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
    Json(payload): Json<CreateTeamRequest>,
) -> impl IntoResponse {
    let team = Team {
        id: Uuid::new_v4(),
        organization_id: payload.organization_id,
        name: payload.name,
        description: payload.description,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match storage.store_team(&team).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({"id": id, "message": "Team created"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to create team", "message": e.to_string()})),
        ),
    }
}

#[cfg(feature = "server")]
pub async fn get_team(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    match storage.get_team(id).await {
        Ok(Some(team)) => (StatusCode::OK, Json(json!(TeamResponse::from(team)))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Team not found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to get team", "message": e.to_string()})),
        ),
    }
}

#[cfg(feature = "server")]
pub async fn update_team(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
    Json(payload): Json<UpdateTeamRequest>,
) -> impl IntoResponse {
    let mut team = match storage.get_team(id).await {
        Ok(Some(team)) => team,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Team not found"})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get team", "message": e.to_string()})),
            )
        }
    };

    if let Some(name) = payload.name {
        team.name = name;
    }
    if let Some(desc) = payload.description {
        team.description = Some(desc);
    }
    team.updated_at = Utc::now();

    match storage.update_team(id, &team).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Team updated"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to update team", "message": e.to_string()})),
        ),
    }
}

#[cfg(feature = "server")]
pub async fn delete_team(
    Path(id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    match storage.delete_team(id).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(json!({}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to delete team", "message": e.to_string()})),
        ),
    }
}

#[cfg(feature = "server")]
pub async fn add_team_member(
    Path(team_id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AddTeamMemberRequest>,
) -> impl IntoResponse {
    let member = TeamMember {
        id: Uuid::new_v4(),
        team_id,
        user_id: payload.user_id,
        role: payload.role.unwrap_or_else(|| "member".to_string()),
        added_at: Utc::now(),
        added_by: Some(
            Uuid::parse_str(&claims.sub)
                .ok()
                .unwrap_or_else(Uuid::new_v4),
        ),
    };

    match storage.add_team_member(&member).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({"id": id, "message": "Member added"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to add member", "message": e.to_string()})),
        ),
    }
}

#[cfg(feature = "server")]
pub async fn remove_team_member(
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    match storage.remove_team_member(team_id, user_id).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(json!({}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to remove member", "message": e.to_string()})),
        ),
    }
}

#[cfg(feature = "server")]
pub async fn get_team_members(
    Path(team_id): Path<Uuid>,
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(_claims): Extension<Claims>,
) -> impl IntoResponse {
    match storage.get_team_members(team_id).await {
        Ok(members) => (StatusCode::OK, Json(json!({"members": members}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to get members", "message": e.to_string()})),
        ),
    }
}

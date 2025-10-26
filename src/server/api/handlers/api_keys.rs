// API Key management handlers

#[cfg(feature = "server")]
use axum::{
    extract::{Extension, Path},
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
use crate::server::{
    auth::jwt::Claims,
    storage::{ApiKey, ApiKeyFilter, Storage},
};

// Request/Response types

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub expires_in_days: Option<i64>,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub key: String, // Full key only returned once
    pub key_prefix: String,
    pub name: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scopes: Option<Vec<String>>,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub key_prefix: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}

#[cfg(feature = "server")]
impl From<ApiKey> for ApiKeyResponse {
    fn from(key: ApiKey) -> Self {
        Self {
            id: key.id,
            key_prefix: key.key_prefix,
            name: key.name,
            description: key.description,
            scopes: key.scopes,
            created_at: key.created_at,
            expires_at: key.expires_at,
            last_used_at: key.last_used_at,
            is_active: key.is_active,
        }
    }
}

// Handlers

/// List user's API keys
#[cfg(feature = "server")]
pub async fn list_api_keys(
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid user ID"})),
            )
        }
    };

    let filter = ApiKeyFilter {
        user_id: Some(user_id),
        is_active: Some(true),
        ..Default::default()
    };

    match storage.list_api_keys(&filter).await {
        Ok(keys) => {
            let response: Vec<ApiKeyResponse> = keys.into_iter().map(|k| k.into()).collect();
            (StatusCode::OK, Json(json!(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

/// Create a new API key
#[cfg(feature = "server")]
pub async fn create_api_key(
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid user ID"})),
            )
        }
    };

    // Generate random API key
    let key_bytes: [u8; 32] = rand::random();
    use base64::{engine::general_purpose, Engine as _};
    let key = format!("sk_{}", general_purpose::URL_SAFE_NO_PAD.encode(key_bytes));

    // Hash the key for storage
    let key_hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let key_prefix = key.chars().take(12).collect::<String>();

    let expires_at = payload
        .expires_in_days
        .map(|days| Utc::now() + chrono::Duration::days(days));

    let api_key = ApiKey {
        id: Uuid::new_v4(),
        user_id,
        key_hash,
        key_prefix: key_prefix.clone(),
        name: payload.name.clone(),
        description: payload.description,
        scopes: payload.scopes.clone(),
        created_at: Utc::now(),
        expires_at,
        last_used_at: None,
        is_active: true,
    };

    match storage.store_api_key(&api_key).await {
        Ok(id) => {
            let response = CreateApiKeyResponse {
                id,
                key, // Full key only shown once
                key_prefix,
                name: payload.name,
                scopes: payload.scopes,
                expires_at,
            };
            (StatusCode::CREATED, Json(json!(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

/// Get API key details
#[cfg(feature = "server")]
pub async fn get_api_key(
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid user ID"})),
            )
        }
    };

    match storage.get_api_key(id).await {
        Ok(Some(key)) => {
            // Verify ownership
            if key.user_id != user_id {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "Access denied"})),
                );
            }

            let response: ApiKeyResponse = key.into();
            (StatusCode::OK, Json(json!(response)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "API key not found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

/// Update API key
#[cfg(feature = "server")]
pub async fn update_api_key(
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateApiKeyRequest>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid user ID"})),
            )
        }
    };

    // Get existing key
    let mut api_key = match storage.get_api_key(id).await {
        Ok(Some(key)) => key,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "API key not found"})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        }
    };

    // Verify ownership
    if api_key.user_id != user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Access denied"})),
        );
    }

    // Update fields
    if let Some(name) = payload.name {
        api_key.name = Some(name);
    }
    if let Some(description) = payload.description {
        api_key.description = Some(description);
    }
    if let Some(scopes) = payload.scopes {
        api_key.scopes = scopes;
    }

    match storage.update_api_key(id, &api_key).await {
        Ok(_) => {
            let response: ApiKeyResponse = api_key.into();
            (StatusCode::OK, Json(json!(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

/// Revoke API key (soft delete by setting is_active = false)
#[cfg(feature = "server")]
pub async fn revoke_api_key(
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid user ID"})),
            )
        }
    };

    // Verify ownership
    match storage.get_api_key(id).await {
        Ok(Some(key)) => {
            if key.user_id != user_id {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "Access denied"})),
                );
            }
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "API key not found"})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        }
    }

    match storage.revoke_api_key(id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "API key revoked"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

/// Rotate API key (generate new key, revoke old one)
#[cfg(feature = "server")]
pub async fn rotate_api_key(
    Extension(storage): Extension<Arc<dyn Storage>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid user ID"})),
            )
        }
    };

    // Get existing key
    let old_key = match storage.get_api_key(id).await {
        Ok(Some(key)) => key,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "API key not found"})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        }
    };

    // Verify ownership
    if old_key.user_id != user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Access denied"})),
        );
    }

    // Generate new key
    let key_bytes: [u8; 32] = rand::random();
    use base64::{engine::general_purpose, Engine as _};
    let new_key = format!("sk_{}", general_purpose::URL_SAFE_NO_PAD.encode(key_bytes));

    let new_key_hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(new_key.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let new_key_prefix = new_key.chars().take(12).collect::<String>();

    // Create new API key with same metadata
    let new_api_key = ApiKey {
        id: Uuid::new_v4(),
        user_id,
        key_hash: new_key_hash,
        key_prefix: new_key_prefix.clone(),
        name: old_key.name.clone(),
        description: old_key.description.clone(),
        scopes: old_key.scopes.clone(),
        created_at: Utc::now(),
        expires_at: old_key.expires_at,
        last_used_at: None,
        is_active: true,
    };

    // Store new key
    match storage.store_api_key(&new_api_key).await {
        Ok(new_id) => {
            // Revoke old key
            if let Err(e) = storage.revoke_api_key(id).await {
                // Log error but continue - new key is already created
                eprintln!("Failed to revoke old API key {}: {}", id, e);
            }

            let response = CreateApiKeyResponse {
                id: new_id,
                key: new_key, // Full key only shown once
                key_prefix: new_key_prefix,
                name: new_api_key.name,
                scopes: new_api_key.scopes,
                expires_at: new_api_key.expires_at,
            };
            (StatusCode::OK, Json(json!(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

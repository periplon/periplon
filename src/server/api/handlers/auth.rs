// Authentication handlers

#[cfg(feature = "server")]
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
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
use crate::server::auth::{Claims, JwtManager};
#[cfg(feature = "server")]
use crate::server::storage::{password, User, UserStorage};

// Note: get_current_user and refresh_token require authentication middleware
// to populate the Claims extension. They will fail without it.

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
}

/// Login endpoint
/// Authenticates user with email and password
#[cfg(feature = "server")]
pub async fn login(
    Extension(user_storage): Extension<Arc<dyn UserStorage>>,
    Extension(jwt_manager): Extension<Arc<JwtManager>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Get user by email
    let user = match user_storage.get_user_by_email(&payload.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Invalid email or password"
                })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Database error: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Check if user is active
    if !user.is_active {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "User account is disabled"
            })),
        )
            .into_response();
    }

    // Verify password
    match password::verify_password(&payload.password, &user.password_hash) {
        Ok(true) => {
            // Password is correct, update last login
            if let Err(e) = user_storage.update_last_login(user.id).await {
                eprintln!("Failed to update last login: {}", e);
            }

            // Generate JWT token
            match jwt_manager.generate_token(&user.id.to_string(), &user.email, user.roles.clone())
            {
                Ok(token) => {
                    let response = AuthResponse {
                        token,
                        user: UserInfo {
                            id: user.id.to_string(),
                            email: user.email,
                            name: user.name,
                            roles: user.roles,
                        },
                    };
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": format!("Failed to generate token: {}", e)
                    })),
                )
                    .into_response(),
            }
        }
        Ok(false) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Invalid email or password"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Password verification error: {}", e)
            })),
        )
            .into_response(),
    }
}

/// Register endpoint
/// Creates a new user account
#[cfg(feature = "server")]
pub async fn register(
    Extension(user_storage): Extension<Arc<dyn UserStorage>>,
    Extension(jwt_manager): Extension<Arc<JwtManager>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Validate email format (basic check)
    if !payload.email.contains('@') || !payload.email.contains('.') {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid email format"
            })),
        )
            .into_response();
    }

    // Validate password strength (basic check)
    if payload.password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Password must be at least 8 characters"
            })),
        )
            .into_response();
    }

    // Check if user already exists
    match user_storage.email_exists(&payload.email).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "User with this email already exists"
                })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Database error: {}", e)
                })),
            )
                .into_response();
        }
        Ok(false) => {}
    }

    // Hash password
    let password_hash = match password::hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Password hashing error: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Create user
    let now = Utc::now();
    let user = User {
        id: Uuid::new_v4(),
        email: payload.email.to_lowercase(),
        name: payload.name,
        password_hash,
        roles: vec!["user".to_string()], // Default role
        is_active: true,
        email_verified: false, // Require email verification in production
        created_at: now,
        updated_at: now,
        last_login_at: None,
    };

    // Save user
    let user_id = match user_storage.create_user(&user).await {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Failed to create user: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Generate JWT token
    match jwt_manager.generate_token(&user_id.to_string(), &user.email, user.roles.clone()) {
        Ok(token) => {
            let response = AuthResponse {
                token,
                user: UserInfo {
                    id: user_id.to_string(),
                    email: user.email,
                    name: user.name,
                    roles: user.roles,
                },
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to generate token: {}", e)
            })),
        )
            .into_response(),
    }
}

/// Get current user info from JWT claims
/// Note: This requires authentication middleware to be enabled
#[cfg(feature = "server")]
pub async fn get_current_user(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "id": claims.sub,
            "email": claims.email,
            "roles": claims.roles,
        })),
    )
}

/// Refresh JWT token
/// Note: This requires authentication middleware to be enabled
#[cfg(feature = "server")]
pub async fn refresh_token(
    Extension(jwt_manager): Extension<Arc<JwtManager>>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    // Generate new token with same claims
    match jwt_manager.generate_token(&claims.sub, &claims.email, claims.roles.clone()) {
        Ok(token) => (
            StatusCode::OK,
            Json(json!({
                "token": token
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to refresh token: {}", e)
            })),
        )
            .into_response(),
    }
}

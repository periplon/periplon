// Authentication middleware for Axum

#[cfg(feature = "server")]
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
#[cfg(feature = "server")]
use serde_json::json;
#[cfg(feature = "server")]
use std::sync::Arc;

#[cfg(feature = "server")]
use super::jwt::{Claims, JwtManager};

/// Shared authentication state
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AuthLayer {
    jwt_manager: Arc<JwtManager>,
    public_paths: Vec<String>,
}

#[cfg(feature = "server")]
impl AuthLayer {
    pub fn new(jwt_manager: Arc<JwtManager>) -> Self {
        Self {
            jwt_manager,
            public_paths: vec![
                "/health".to_string(),
                "/ready".to_string(),
                "/live".to_string(),
                "/version".to_string(),
                "/metrics".to_string(),
                "/api/v1/auth/login".to_string(),
                "/api/v1/auth/register".to_string(),
            ],
        }
    }

    pub fn with_public_paths(mut self, paths: Vec<String>) -> Self {
        self.public_paths = paths;
        self
    }
}

/// Extract JWT token from Authorization header
#[cfg(feature = "server")]
fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer ").map(|s| s.to_string()))
}

/// Authentication middleware
#[cfg(feature = "server")]
pub async fn auth_middleware(
    auth_layer: AuthLayer,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    // Check if path is public
    if auth_layer.public_paths.iter().any(|p| path.starts_with(p)) {
        return Ok(next.run(request).await);
    }

    // Extract token from header
    let token = extract_token(request.headers()).ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token
    let claims = auth_layer
        .jwt_manager
        .validate_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check if token is expired
    if auth_layer.jwt_manager.is_token_expired(&claims) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Store claims in request extensions for use in handlers
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Error response for authentication failures
#[cfg(feature = "server")]
pub fn auth_error_response(message: &str) -> impl IntoResponse {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "Unauthorized",
            "message": message
        })),
    )
}

/// Role-based authorization middleware
#[cfg(feature = "server")]
pub async fn require_role(
    required_role: &str,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| auth_error_response("Not authenticated").into_response())?;

    if !JwtManager::has_role(claims, required_role) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Forbidden",
                "message": format!("Requires role: {}", required_role)
            })),
        )
            .into_response());
    }

    Ok(next.run(request).await)
}

/// Multiple roles authorization middleware (user must have ANY of the roles)
#[cfg(feature = "server")]
pub async fn require_any_role(
    required_roles: &[&str],
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| auth_error_response("Not authenticated").into_response())?;

    if !JwtManager::has_any_role(claims, required_roles) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Forbidden",
                "message": format!("Requires one of these roles: {:?}", required_roles)
            })),
        )
            .into_response());
    }

    Ok(next.run(request).await)
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer test_token_123".parse().unwrap());

        let token = extract_token(&headers);
        assert_eq!(token, Some("test_token_123".to_string()));

        // Test without Bearer prefix
        headers.insert("Authorization", "test_token_123".parse().unwrap());
        let token = extract_token(&headers);
        assert_eq!(token, None);

        // Test missing header
        headers.remove("Authorization");
        let token = extract_token(&headers);
        assert_eq!(token, None);
    }
}

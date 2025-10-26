// API routes definition

#[cfg(feature = "server")]
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Extension, Router,
};

#[cfg(feature = "server")]
use super::handlers;

#[cfg(feature = "server")]
use crate::server::auth::{middleware as auth_middleware, JwtManager};
#[cfg(feature = "server")]
use crate::server::config::{CorsConfig, RateLimitConfig};
#[cfg(feature = "server")]
use crate::server::middleware::{rate_limit_middleware, RateLimiter};
#[cfg(feature = "server")]
use crate::server::queue::WorkQueue;
#[cfg(feature = "server")]
use crate::server::storage::UserStorage;
#[cfg(feature = "server")]
use crate::server::Storage;
#[cfg(feature = "server")]
use std::sync::Arc;
#[cfg(feature = "server")]
use tower_http::cors::{Any, CorsLayer};

#[cfg(feature = "server")]
pub fn create_router(
    user_storage: Arc<dyn UserStorage>,
    jwt_manager: Arc<JwtManager>,
    storage: Arc<dyn Storage>,
    queue: Arc<dyn WorkQueue>,
    cors_config: Option<CorsConfig>,
    rate_limit_config: RateLimitConfig,
) -> Router {
    // Create auth layer for protected routes
    let auth_layer = auth_middleware::AuthLayer::new(Arc::clone(&jwt_manager));

    // Create rate limiter
    let rate_limiter = RateLimiter::new(rate_limit_config);

    // Create CORS layer if configured
    let cors_layer = if let Some(cors) = cors_config {
        let mut layer = CorsLayer::new();

        // Configure allowed origins
        if cors.allowed_origins.is_empty() || cors.allowed_origins.contains(&"*".to_string()) {
            layer = layer.allow_origin(Any);
        } else {
            use axum::http::HeaderValue;
            let origins: Vec<HeaderValue> = cors
                .allowed_origins
                .iter()
                .filter_map(|o| o.parse().ok())
                .collect();
            layer = layer.allow_origin(origins);
        }

        // Configure allowed methods
        if cors.allowed_methods.is_empty() {
            layer = layer.allow_methods(Any);
        } else {
            use axum::http::Method;
            let methods: Vec<Method> = cors
                .allowed_methods
                .iter()
                .filter_map(|m| m.parse().ok())
                .collect();
            layer = layer.allow_methods(methods);
        }

        // Configure credentials
        if cors.allow_credentials {
            layer = layer.allow_credentials(true);
        }

        // Allow common headers
        layer = layer.allow_headers(Any);

        Some(layer)
    } else {
        None
    };

    // Public routes (no authentication required)
    let public_routes = Router::new()
        // Health and monitoring endpoints
        .route("/health", get(handlers::monitoring::health_check))
        .route("/ready", get(handlers::monitoring::readiness_check))
        .route("/live", get(handlers::monitoring::liveness_check))
        .route("/version", get(handlers::health::version))
        .route("/metrics", get(handlers::monitoring::metrics))
        .route("/stats", get(handlers::monitoring::stats))
        // Authentication endpoints
        .route("/api/v1/auth/login", post(handlers::auth::login))
        .route("/api/v1/auth/register", post(handlers::auth::register));

    // Protected routes (require valid JWT)
    let protected_routes = Router::new()
        // Auth endpoints that need authentication
        .route("/api/v1/auth/me", get(handlers::auth::get_current_user))
        .route("/api/v1/auth/refresh", post(handlers::auth::refresh_token))
        // Workflow endpoints
        .route(
            "/api/v1/workflows",
            get(handlers::workflows::list_workflows),
        )
        .route(
            "/api/v1/workflows",
            post(handlers::workflows::create_workflow),
        )
        .route(
            "/api/v1/workflows/:id",
            get(handlers::workflows::get_workflow),
        )
        .route(
            "/api/v1/workflows/:id",
            put(handlers::workflows::update_workflow),
        )
        .route(
            "/api/v1/workflows/:id",
            delete(handlers::workflows::delete_workflow),
        )
        // Workflow version management
        .route(
            "/api/v1/workflows/:id/versions",
            get(handlers::workflows::list_workflow_versions),
        )
        .route(
            "/api/v1/workflows/:id/versions/:version",
            get(handlers::workflows::get_workflow_version),
        )
        // Workflow validation (no auth needed for this, move to public if needed)
        .route(
            "/api/v1/workflows/validate",
            post(handlers::workflows::validate_workflow),
        )
        // Execution endpoints
        .route(
            "/api/v1/executions",
            get(handlers::executions::list_executions),
        )
        .route(
            "/api/v1/executions",
            post(handlers::executions::create_execution),
        )
        .route(
            "/api/v1/executions/:id",
            get(handlers::executions::get_execution),
        )
        .route(
            "/api/v1/executions/:id/cancel",
            post(handlers::executions::cancel_execution),
        )
        .route(
            "/api/v1/executions/:id/logs",
            get(handlers::executions::get_execution_logs),
        )
        // Schedule endpoints
        .route(
            "/api/v1/schedules",
            get(handlers::schedules::list_schedules),
        )
        .route(
            "/api/v1/schedules",
            post(handlers::schedules::create_schedule),
        )
        .route(
            "/api/v1/schedules/:id",
            get(handlers::schedules::get_schedule),
        )
        .route(
            "/api/v1/schedules/:id",
            put(handlers::schedules::update_schedule),
        )
        .route(
            "/api/v1/schedules/:id",
            delete(handlers::schedules::delete_schedule),
        )
        .route(
            "/api/v1/schedules/:id/trigger",
            post(handlers::schedules::trigger_schedule),
        )
        // Organization endpoints
        .route(
            "/api/v1/organizations",
            get(handlers::organizations::list_organizations),
        )
        .route(
            "/api/v1/organizations",
            post(handlers::organizations::create_organization),
        )
        .route(
            "/api/v1/organizations/:id",
            get(handlers::organizations::get_organization),
        )
        .route(
            "/api/v1/organizations/:id",
            put(handlers::organizations::update_organization),
        )
        .route(
            "/api/v1/organizations/:id",
            delete(handlers::organizations::delete_organization),
        )
        // Team endpoints
        .route("/api/v1/teams", get(handlers::organizations::list_teams))
        .route("/api/v1/teams", post(handlers::organizations::create_team))
        .route("/api/v1/teams/:id", get(handlers::organizations::get_team))
        .route(
            "/api/v1/teams/:id",
            put(handlers::organizations::update_team),
        )
        .route(
            "/api/v1/teams/:id",
            delete(handlers::organizations::delete_team),
        )
        .route(
            "/api/v1/teams/:id/members",
            post(handlers::organizations::add_team_member),
        )
        .route(
            "/api/v1/teams/:id/members/:user_id",
            delete(handlers::organizations::remove_team_member),
        )
        .route(
            "/api/v1/teams/:id/members",
            get(handlers::organizations::get_team_members),
        )
        // API Key endpoints
        .route("/api/v1/api-keys", get(handlers::api_keys::list_api_keys))
        .route("/api/v1/api-keys", post(handlers::api_keys::create_api_key))
        .route("/api/v1/api-keys/:id", get(handlers::api_keys::get_api_key))
        .route(
            "/api/v1/api-keys/:id",
            put(handlers::api_keys::update_api_key),
        )
        .route(
            "/api/v1/api-keys/:id",
            delete(handlers::api_keys::revoke_api_key),
        )
        .route(
            "/api/v1/api-keys/:id/rotate",
            post(handlers::api_keys::rotate_api_key),
        )
        // WebSocket endpoints for real-time streaming
        .route(
            "/api/v1/executions/:id/stream",
            get(handlers::websocket::execution_stream),
        )
        // Queue stats
        .route("/api/v1/queue/stats", get(handlers::queue::get_queue_stats))
        // Apply authentication middleware to protected routes
        .route_layer(middleware::from_fn(move |req, next| {
            auth_middleware::auth_middleware(auth_layer.clone(), req, next)
        }));

    // Web UI routes (serve embedded static files)
    let web_ui_routes = Router::new()
        .route("/", get(handlers::static_files::serve_index))
        .route("/*path", get(handlers::static_files::serve_static));

    // Combine all routes
    let mut router = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(web_ui_routes)
        // Add storage as state for WebSocket handlers
        .with_state(Arc::clone(&storage))
        // Add global extensions
        .layer(Extension(storage))
        .layer(Extension(queue))
        .layer(Extension(user_storage))
        .layer(Extension(jwt_manager));

    // Apply rate limiting middleware
    router = router.route_layer(middleware::from_fn(move |req, next| {
        rate_limit_middleware(rate_limiter.clone(), req, next)
    }));

    // Apply CORS layer if configured
    if let Some(cors) = cors_layer {
        router = router.layer(cors);
    }

    router
}

// Example of how to enable authentication middleware (currently disabled):
//
// use axum::middleware;
// use crate::server::auth::{AuthLayer, JwtManager};
//
// pub fn create_router_with_auth(jwt_secret: &str) -> Router {
//     let jwt_manager = JwtManager::new(jwt_secret, 24); // 24 hour expiration
//     let auth_layer = AuthLayer::new(jwt_manager.clone());
//
//     Router::new()
//         // Public routes (no auth required)
//         .route("/health", get(handlers::monitoring::health_check))
//         .route("/api/v1/auth/login", post(handlers::auth::login))
//         .route("/api/v1/auth/register", post(handlers::auth::register))
//
//         // Protected routes (auth required)
//         .route("/api/v1/workflows", get(handlers::workflows::list_workflows))
//         // ... other protected routes
//         .layer(middleware::from_fn(move |req, next| {
//             crate::server::auth::middleware::auth_middleware(auth_layer.clone(), req, next)
//         }))
//         .layer(Extension(jwt_manager))
// }

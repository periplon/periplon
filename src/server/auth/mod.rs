// Authentication and authorization module

#[cfg(feature = "server")]
pub mod jwt;

#[cfg(feature = "server")]
pub mod middleware;

#[cfg(feature = "server")]
pub mod authorization;

#[cfg(feature = "server")]
pub use jwt::{Claims, JwtManager};

#[cfg(feature = "server")]
pub use middleware::AuthLayer;

#[cfg(feature = "server")]
pub use authorization::{
    check_permission, check_resource_access, AuthorizationService, PostgresAuthorizationService,
};

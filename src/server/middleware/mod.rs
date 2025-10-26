// Middleware module

#[cfg(feature = "server")]
pub mod rate_limit;

#[cfg(feature = "server")]
pub use rate_limit::{rate_limit_middleware, RateLimiter};

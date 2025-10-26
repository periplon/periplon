// REST API module

#[cfg(feature = "server")]
pub mod routes;

#[cfg(feature = "server")]
pub mod handlers;

// Re-exports will be added as handlers are implemented

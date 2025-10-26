// API handlers

#[cfg(feature = "server")]
pub mod health;

#[cfg(feature = "server")]
pub mod workflows;

#[cfg(feature = "server")]
pub mod executions;

#[cfg(feature = "server")]
pub mod schedules;

#[cfg(feature = "server")]
pub mod organizations;

#[cfg(feature = "server")]
pub mod queue;

#[cfg(feature = "server")]
pub mod monitoring;

#[cfg(feature = "server")]
pub mod auth;

#[cfg(feature = "server")]
pub mod websocket;

#[cfg(feature = "server")]
pub mod api_keys;

#[cfg(feature = "server")]
pub mod static_files;

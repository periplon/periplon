//! Testing Utilities
//!
//! This module provides comprehensive testing utilities for building tests
//! that interact with the Periplon SDK. It includes mock implementations of
//! all major service interfaces and builder patterns for common test data.
//!
//! # Modules
//!
//! - `MockMcpServer`: Mock MCP server for testing tool integrations
//! - `MockPermissionService`: Mock permission service for testing authorization flows
//! - `MockHookService`: Mock hook service for testing lifecycle events
//! - `MessageBuilder`, `NotificationBuilder`, etc.: Builders for test data
//!
//! # Examples
//!
//! ```
//! use periplon_sdk::testing::{MockMcpServer, MockPermissionService};
//! use serde_json::json;
//!
//! # tokio_test::block_on(async {
//! // Create a mock MCP server
//! let mut server = MockMcpServer::new("test-server");
//! server.with_static_tool("ping", "Pings", json!({}), json!({"pong": true}));
//!
//! // Create a mock permission service
//! let service = MockPermissionService::allow_all();
//! # });
//! ```

mod mock_hook_service;
mod mock_mcp_server;
mod mock_permission_service;
mod test_helpers;

#[cfg(feature = "server")]
mod mock_auth_service;
#[cfg(feature = "server")]
mod mock_queue;
#[cfg(feature = "server")]
mod mock_storage;

pub use mock_hook_service::*;
pub use mock_mcp_server::*;
pub use mock_permission_service::*;
pub use test_helpers::*;

#[cfg(feature = "server")]
pub use mock_auth_service::*;
#[cfg(feature = "server")]
pub use mock_queue::*;
#[cfg(feature = "server")]
pub use mock_storage::*;

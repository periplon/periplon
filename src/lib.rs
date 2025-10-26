//! Periplon SDK - Multi-agent AI workflow orchestration
//!
//! This SDK provides a Rust interface for building and executing multi-agent AI workflows.
//! It communicates with the CLI via stdin/stdout using newline-delimited JSON (NDJSON).
//!
//! # Architecture
//!
//! The SDK follows **Hexagonal Architecture** (Ports and Adapters pattern):
//!
//! - **Domain Core**: Pure business logic (message types, sessions, permissions, hooks, control)
//! - **Primary Ports**: Inbound interfaces (AgentService, SessionManager, ControlProtocol)
//! - **Secondary Ports**: Outbound interfaces (Transport, PermissionService, HookService, McpServer)
//! - **Primary Adapters**: Implementations driving the application (query function, PeriplonSDKClient)
//! - **Secondary Adapters**: Implementations connecting to external systems (SubprocessCLITransport, MockTransport)
//! - **Application Services**: Orchestration layer (Query)
//!
//! # Examples
//!
//! ## Simple Query
//!
//! ```no_run
//! use periplon_sdk::{query, Message, ContentBlock};
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut stream = query("What is 2 + 2?", None).await?;
//!
//!     while let Some(msg) = stream.next().await {
//!         match msg {
//!             Message::Assistant(assistant_msg) => {
//!                 for block in assistant_msg.message.content {
//!                     if let ContentBlock::Text { text } = block {
//!                         println!("Assistant: {}", text);
//!                     }
//!                 }
//!             }
//!             Message::Result(result_msg) => {
//!                 println!("Cost: ${:.4}", result_msg.total_cost_usd.unwrap_or(0.0));
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Interactive Client
//!
//! ```no_run
//! use periplon_sdk::{PeriplonSDKClient, AgentOptions};
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let options = AgentOptions {
//!         allowed_tools: vec!["Read".to_string(), "Bash".to_string()],
//!         permission_mode: Some("acceptEdits".to_string()),
//!         ..Default::default()
//!     };
//!
//!     let mut client = PeriplonSDKClient::new(options);
//!     client.connect(None).await?;
//!
//!     // First query
//!     client.query("List files in current directory").await?;
//!     {
//!         let stream = client.receive_response()?;
//!         futures::pin_mut!(stream);
//!         while let Some(msg) = stream.next().await {
//!             println!("{:?}", msg);
//!         }
//!     }
//!
//!     // Follow-up query
//!     client.query("Create a README.md file").await?;
//!     {
//!         let stream = client.receive_response()?;
//!         futures::pin_mut!(stream);
//!         while let Some(msg) = stream.next().await {
//!             println!("{:?}", msg);
//!         }
//!     }
//!
//!     client.disconnect().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod adapters;
pub mod application;
pub mod data_fetcher;
pub mod domain;
pub mod dsl;
pub mod error;
pub mod options;
pub mod ports;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "tui")]
pub mod tui;

// Re-export commonly used types
pub use adapters::primary::{query, PeriplonSDKClient};
pub use data_fetcher::{
    DataFetcher, FetchError, FileMetadata, HttpMethod, HttpRequest, HttpResponse,
};
pub use domain::{ContentBlock, ContentValue, Message, PermissionResult, ToolPermissionContext};
pub use dsl::{parse_workflow, parse_workflow_file, validate_workflow, DSLExecutor, DSLWorkflow};
pub use error::{Error, Result};
pub use options::{AgentOptions, SystemPromptConfig};

#[cfg(feature = "tui")]
pub use tui::TuiApp;

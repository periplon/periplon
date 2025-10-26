# Periplon

A Rust SDK for building multi-agent AI workflows and automation.

[![Crates.io](https://img.shields.io/crates/v/periplon.svg)](https://crates.io/crates/periplon)
[![Documentation](https://docs.rs/periplon/badge.svg)](https://docs.rs/periplon)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Tools](#tools)
  - [Executor CLI](#executor-cli)
  - [TUI (Terminal Interface)](#tui-terminal-interface)
- [Documentation](#documentation)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Overview

Periplon provides a powerful Rust interface for building and executing multi-agent AI workflows. It enables seamless orchestration of AI agents through a clean, type-safe API and a comprehensive DSL system for complex automation tasks.

**Key Capabilities:**
- 🦀 **Type-Safe Rust SDK** - Strong typing with compile-time guarantees
- 🔄 **Multi-Agent Workflows** - Orchestrate complex AI agent interactions
- 📝 **Powerful DSL** - YAML-based workflow definition language
- 🏗️ **Hexagonal Architecture** - Clean separation of concerns
- ⚡ **Async I/O** - Non-blocking operations throughout
- 🔌 **Extensible** - Plugin architecture with ports and adapters

## Features

### Rust SDK

- **Hexagonal Architecture**: Clean separation with ports and adapters pattern
- **Type Safety**: Strong Rust types with compile-time guarantees
- **Async I/O**: Non-blocking async/await using tokio
- **Stream-Based**: Efficient streaming without buffering
- **Error Handling**: Rich error types with context
- **Testability**: Mock adapters for isolated testing

### DSL System

- **Multi-Agent Workflows**: Orchestrate complex AI agent interactions
- **Natural Language Generation**: Create workflows from plain English
- **State Management**: Checkpoint and resume execution
- **Dependency Resolution**: DAG-based task execution
- **Variable System**: Scoped variables with interpolation
- **Loop Support**: ForEach, While, RepeatUntil, Repeat patterns
- **Validation**: Comprehensive workflow validation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
periplon = "0.1.0"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

For detailed installation instructions, see [Installation Guide](./docs/guides/installation.md).

## Quick Start

### Simple Query

```rust
use periplon_sdk::{query, Message, ContentBlock};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = query("What is 2 + 2?", None).await?;

    while let Some(msg) = stream.next().await {
        match msg {
            Message::Assistant(assistant_msg) => {
                for block in assistant_msg.message.content {
                    if let ContentBlock::Text { text } = block {
                        println!("Assistant: {}", text);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
```

### Interactive Client

```rust
use periplon_sdk::{PeriplonSDKClient, AgentOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = AgentOptions {
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        permission_mode: Some("acceptEdits".to_string()),
        ..Default::default()
    };

    let mut client = PeriplonSDKClient::new(options);
    client.connect(None).await?;

    client.query("List files in current directory").await?;
    // Process response...

    client.disconnect().await?;
    Ok(())
}
```

See the [Quick Start Guide](./docs/guides/quick-start.md) for more examples.

## Tools

### Executor CLI

A complete workflow orchestration platform with zero configuration:

- **🚀 Zero-Config Server**: Start instantly with no database required
- **🎨 Embedded Web UI**: Full Next.js interface built into the binary
- **⚡ Production Ready**: API server and web interface in one executable
- **🔧 Developer Friendly**: Hot reload, validation, natural language generation

```bash
# Build the CLI
cargo build --release --features full

# Start server with embedded web UI
./target/release/periplon-executor server --port 8080 --workers

# Access web UI at http://localhost:8080
```

**Documentation:** [CLI Usage](./CLI_USAGE.md) | [Embedded Web UI](./EMBEDDED_WEB_UI.md)

### TUI (Terminal Interface)

Interactive terminal interface for workflow management:

- **📁 Workflow Browser**: Browse and manage workflow files
- **✏️ Smart Editor**: YAML editor with syntax highlighting
- **🤖 AI Generation**: Create workflows from natural language
- **📊 Execution Monitor**: Real-time progress tracking
- **💾 State Management**: Save and resume executions
- **⌨️ Keyboard-Driven**: Full keyboard navigation

```bash
# Build the TUI
cargo build --release --bin periplon-tui --features tui

# Launch TUI
./target/release/periplon-tui

# Launch with custom workflow directory
./target/release/periplon-tui --workflow-dir ./my-workflows
```

**Documentation:**
- [User Guide](./docs/tui/user-guide.md)
- [Keyboard Shortcuts](./docs/tui/shortcuts.md)
- [Architecture](./docs/tui/architecture.md)
- [Troubleshooting](./docs/tui/troubleshooting.md)

## Documentation

### Getting Started
- [Installation Guide](./docs/guides/installation.md) - Setup and requirements
- [Quick Start Guide](./docs/guides/quick-start.md) - Get up and running
- [Configuration Guide](./docs/guides/configuration.md) - Agent options and settings

### Core Concepts
- [Architecture Guide](./docs/guides/architecture.md) - Hexagonal architecture overview
- [Message Types](./docs/api/message-types.md) - Message type reference
- [Error Handling](./docs/guides/error-handling.md) - Error types and patterns
- [Testing Guide](./docs/guides/testing.md) - Comprehensive testing (166+ tests)

### DSL System
- [DSL Overview](./docs/guides/dsl-overview.md) - Introduction to the DSL
- [Loop Patterns Guide](./docs/loop-patterns.md) - Comprehensive loop reference
- [Loop Cookbook](./docs/loop-cookbook.md) - 25 production-ready patterns
- [Loop Tutorial](./docs/loop-tutorial.md) - Step-by-step guide
- [DSL Implementation](./docs/DSL_IMPLEMENTATION.md) - Technical details
- [Natural Language Generation](./docs/DSL_NL_GENERATION.md) - NL workflow generation
- [HTTP Collections](./docs/HTTP_COLLECTION_SUMMARY.md) - HTTP/HTTPS integration
- [Security Audit](./docs/SECURITY_AUDIT.md) - Safety analysis

### API Reference
- [Rust API Documentation](https://docs.rs/periplon)
- [Example Workflows](./examples/dsl_workflows/)

## Examples

Run the included examples:

```bash
# Simple query
cargo run --example simple_query

# Interactive client
cargo run --example interactive_client

# DSL executor
cargo run --example dsl_executor_example
```

### DSL Loop Examples
- [ForEach Demo](./examples/foreach_demo.rs) - Process collections
- [While Demo](./examples/while_demo.rs) - Polling pattern
- [Polling Demo](./examples/polling_demo.rs) - API polling
- [Parallel Demo](./examples/parallel_foreach_demo.rs) - Concurrent execution
- [HTTP Collection Demo](./examples/http_collection_demo.rs) - Fetch from APIs
- [Checkpoint Demo](./examples/checkpoint_resume_demo.rs) - Resume capability

See [examples/](./examples/) for more examples.

## Requirements

- **Minimum CLI version**: 2.0.0
- **Rust**: 1.70 or later
- **Tokio runtime**: Required for async operations

## Testing

The SDK includes comprehensive test coverage with 166+ integration tests:

```bash
# Run all tests with server features
cargo test --lib --tests --features server

# Run specific test suite
cargo test --test execution_api_tests --features server

# Run tests with output
cargo test --features server -- --nocapture
```

**Test Suites:**
- Authentication & Authorization (26 tests)
- Queue Backend Operations (22 tests)
- Storage Backend Operations (21 tests)
- Schedule Management API (22 tests)
- Execution Management API (22 tests)
- WebSocket Streaming (21 tests)
- Workflow API Integration (32 tests)

See [Testing Guide](./docs/guides/testing.md) for comprehensive documentation and examples.

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test --lib --tests --features server`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. Documentation is updated

See [CLAUDE.md](./CLAUDE.md) for development guidelines.

## License

MIT OR Apache-2.0

## Resources

- [Project Documentation](./docs/)
- [API Reference](https://docs.rs/periplon)
- [Examples](./examples/)
- [Change Log](./CHANGELOG.md)

---

**Built with ❤️ in Rust**

# Periplon

A powerful DSL and Rust SDK for building multi-agent AI workflows and automation.

[![Crates.io](https://img.shields.io/crates/v/periplon.svg)](https://crates.io/crates/periplon)
[![Documentation](https://docs.rs/periplon/badge.svg)](https://docs.rs/periplon)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Quick Start - DSL](#quick-start---dsl)
- [Tools](#tools)
  - [Executor CLI](#executor-cli)
  - [TUI (Terminal Interface)](#tui-terminal-interface)
- [SDK Usage](#sdk-usage)
- [Documentation](#documentation)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Overview

Periplon provides a comprehensive DSL (Domain-Specific Language) for orchestrating multi-agent AI workflows with zero configuration. Define complex automation tasks in YAML, and let Periplon handle the execution, state management, and agent coordination.

**Key Capabilities:**
- 📝 **Powerful DSL** - YAML-based workflow definition language
- 🔄 **Multi-Agent Workflows** - Orchestrate complex AI agent interactions
- 🚀 **Zero Configuration** - Start instantly with embedded web UI and API server
- 🤖 **Natural Language Generation** - Create workflows from plain English
- 💾 **State Management** - Checkpoint and resume execution
- 🔌 **Extensible** - Plugin architecture with ports and adapters
- 🦀 **Type-Safe Rust SDK** - Strong typing with compile-time guarantees for advanced use cases

## Features

### DSL System

The Periplon DSL is the primary interface for building workflows:

- **Multi-Agent Workflows**: Define and orchestrate multiple specialized AI agents
- **Natural Language Generation**: Create workflows from plain English descriptions
- **State Management**: Automatic checkpoint and resume execution
- **Dependency Resolution**: DAG-based task execution with automatic ordering
- **Variable System**: Scoped variables with `${variable}` interpolation
- **Loop Support**: ForEach, While, RepeatUntil, Repeat patterns for iterative tasks
- **Validation**: Comprehensive workflow validation before execution
- **Tool Filtering**: Control which tools each agent can access
- **Permission Modes**: Fine-grained control over agent capabilities
- **Lifecycle Hooks**: on_start, on_complete, on_error event handlers
- **HTTP Collections**: Fetch data from REST APIs with authentication

### Rust SDK

For advanced programmatic usage:

- **Hexagonal Architecture**: Clean separation with ports and adapters pattern
- **Type Safety**: Strong Rust types with compile-time guarantees
- **Async I/O**: Non-blocking async/await using tokio
- **Stream-Based**: Efficient streaming without buffering
- **Error Handling**: Rich error types with context
- **Testability**: Mock adapters for isolated testing

## Installation

### For DSL Users

Build the executor CLI and TUI:

```bash
# Build the executor CLI with full features
cargo build --release --features full

# Build the TUI
cargo build --release --bin periplon-tui --features tui
```

Binaries will be available at:
- `./target/release/periplon-executor`
- `./target/release/periplon-tui`

### For SDK Users

Add to your `Cargo.toml`:

```toml
[dependencies]
periplon = "0.1.0"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

For detailed installation instructions, see [Installation Guide](./docs/guides/installation.md).

## Quick Start - DSL

### Creating Your First Workflow

Create a simple workflow file `hello-world.yaml`:

```yaml
name: "Hello World Workflow"
version: "1.0.0"
description: "A simple workflow demonstrating multi-agent coordination"

agents:
  greeter:
    description: "Generate friendly greetings"
    model: "claude-sonnet-4-5"
    permissions:
      mode: "default"

  writer:
    description: "Save greetings to a file"
    model: "claude-sonnet-4-5"
    tools: [Write]
    permissions:
      mode: "acceptEdits"

tasks:
  generate_greeting:
    description: "Generate a friendly greeting message"
    agent: "greeter"

  save_greeting:
    description: "Save the greeting to greeting.txt"
    agent: "writer"
    depends_on: [generate_greeting]
```

### Running the Workflow

```bash
# Validate the workflow
./target/release/periplon-executor validate hello-world.yaml

# Run the workflow
./target/release/periplon-executor run hello-world.yaml
```

### Generate Workflow from Natural Language

```bash
# Generate a workflow from description
./target/release/periplon-executor generate \
  "Create a workflow that analyzes a codebase, finds todos, and generates a report" \
  -o analyze-todos.yaml

# Run the generated workflow
./target/release/periplon-executor run analyze-todos.yaml
```

### Advanced Example: Variable Interpolation

```yaml
name: "Project Analysis"
version: "1.0.0"

inputs:
  project_name:
    type: string
    required: true
    default: "MyProject"

  output_dir:
    type: string
    required: true
    default: "./reports"

agents:
  analyzer:
    description: "Analyze code for ${workflow.project_name}"
    model: "claude-sonnet-4-5"
    tools: [Read, Grep, Glob]
    inputs:
      target_dir:
        type: string
        required: true

tasks:
  scan_codebase:
    description: "Scan ${workflow.project_name} codebase for issues"
    agent: "analyzer"
    inputs:
      target_dir: "./src"
    outputs:
      report:
        source:
          type: file
          path: "${workflow.output_dir}/analysis.json"
```

See the [DSL Overview](./docs/guides/dsl-overview.md) for comprehensive documentation.

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

**Documentation:** [CLI Usage](./docs/guides/CLI_USAGE.md) | [Embedded Web UI](./docs/features/EMBEDDED_WEB_UI.md)

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

## SDK Usage

For advanced programmatic control, use the Rust SDK directly:

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

See the [Quick Start Guide](./docs/guides/quick-start.md) for more SDK examples.

## Documentation

### DSL System
- [DSL Overview](./docs/guides/dsl-overview.md) - Introduction to the DSL
- [Loop Patterns Guide](./docs/features/loop-patterns.md) - Comprehensive loop reference
- [Loop Cookbook](./docs/features/loop-cookbook.md) - 25 production-ready patterns
- [Loop Tutorial](./docs/features/loop-tutorial.md) - Step-by-step guide
- [DSL Implementation](./docs/api/DSL_IMPLEMENTATION.md) - Technical details
- [Natural Language Generation](./docs/api/DSL_NL_GENERATION.md) - NL workflow generation
- [HTTP Collections](./docs/features/HTTP_COLLECTION_SUMMARY.md) - HTTP/HTTPS integration
- [Security Audit](./docs/api/SECURITY_AUDIT.md) - Safety analysis

### Getting Started
- [Installation Guide](./docs/guides/installation.md) - Setup and requirements
- [Quick Start Guide](./docs/guides/quick-start.md) - Get up and running
- [Configuration Guide](./docs/guides/configuration.md) - Agent options and settings

### SDK Reference
- [Rust API Documentation](https://docs.rs/periplon)
- [Architecture Guide](./docs/guides/architecture.md) - Hexagonal architecture overview
- [Message Types](./docs/api/message-types.md) - Message type reference
- [Error Handling](./docs/guides/error-handling.md) - Error types and patterns
- [Testing Guide](./docs/guides/testing.md) - Comprehensive testing (166+ tests)

### Example Workflows
- [Example Workflows](./examples/dsl_workflows/)

## Examples

### DSL Workflow Examples

Run the included DSL examples:

```bash
# DSL executor example
cargo run --example dsl_executor_example
```

**Loop Pattern Examples:**
- [ForEach Demo](./examples/sdk/foreach_demo.rs) - Process collections
- [While Demo](./examples/sdk/while_demo.rs) - Polling pattern
- [Polling Demo](./examples/sdk/polling_demo.rs) - API polling
- [Parallel Demo](./examples/sdk/parallel_foreach_demo.rs) - Concurrent execution
- [HTTP Collection Demo](./examples/sdk/http_collection_demo.rs) - Fetch from APIs
- [Checkpoint Demo](./examples/sdk/checkpoint_resume_demo.rs) - Resume capability

### SDK Examples

Run the included SDK examples:

```bash
# Simple query
cargo run --example simple_query

# Interactive client
cargo run --example interactive_client
```

See [examples/](./examples/) for all examples.

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

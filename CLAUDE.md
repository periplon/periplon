# CLAUDE.md

This file provides guidance when working with code in this repository.

## Project Overview

Periplon - A Rust SDK for building multi-agent AI workflows and automation. Provides a type-safe, async Rust interface for orchestrating AI agents via NDJSON communication. Includes a powerful DSL system for building complex multi-agent workflows.

## Development Commands

### Building
```bash
# Build library and binaries
cargo build

# Build in release mode
cargo build --release

# Build DSL executor binary specifically
cargo build --release --bin periplon-executor

# Build DSL TUI binary specifically
cargo build --release --bin periplon-tui --features tui
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output visible
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests (requires CLI installed)
cargo test --test integration_tests
```

### Code Quality
```bash
# Format code
cargo fmt

# Check for warnings
cargo clippy

# Run benchmarks
cargo bench --bench dsl_benchmarks
```

### Running Examples
```bash
# Simple query example
cargo run --example simple_query

# Interactive client
cargo run --example interactive_client

# DSL executor example
cargo run --example dsl_executor_example
```

### DSL Executor CLI
```bash
# Generate DSL template
./target/release/periplon-executor template > template.yaml

# Generate workflow from natural language
./target/release/periplon-executor generate "Description" -o workflow.yaml

# Validate workflow
./target/release/periplon-executor validate workflow.yaml

# Run workflow
./target/release/periplon-executor run workflow.yaml
```

### DSL TUI
```bash
# Launch TUI with default workflow directory
./target/release/periplon-tui

# Launch with custom workflow directory
./target/release/periplon-tui --workflow-dir ./my-workflows

# Launch with specific workflow
./target/release/periplon-tui --workflow ./workflow.yaml

# Launch in readonly mode
./target/release/periplon-tui --readonly

# Launch with custom theme and debug logging
./target/release/periplon-tui --theme dark --debug
```

## Architecture

### Hexagonal Architecture (Ports & Adapters)

The codebase follows strict hexagonal architecture with clear separation:

**Domain Core** (`src/domain/`):
- Pure business logic with zero external dependencies
- `message.rs`: Message types (User, Assistant, System, Result, StreamEvent)
- `session.rs`: Session management and state
- `permission.rs`: Permission evaluation logic
- `control.rs`: Control protocol state machine
- `hook.rs`: Hook type definitions

**Primary Ports** (`src/ports/primary/`):
- Inbound interfaces used by external actors
- `agent_service.rs`: Query execution and message handling traits
- `session_manager.rs`: Session lifecycle management
- `control_protocol.rs`: Control flow management

**Secondary Ports** (`src/ports/secondary/`):
- Outbound interfaces for external systems
- `transport.rs`: CLI communication trait
- `permission_service.rs`: Permission evaluation trait
- `hook_service.rs`: Hook execution trait
- `mcp_server.rs`: MCP server integration trait

**Primary Adapters** (`src/adapters/primary/`):
- Drive the application from outside
- `query_fn.rs`: Simple `query()` function for one-shot queries
- `sdk_client.rs`: `PeriplonSDKClient` for interactive multi-turn conversations

**Secondary Adapters** (`src/adapters/secondary/`):
- Implement connections to external systems
- `subprocess_transport.rs`: CLI subprocess communication via stdin/stdout
- `mock_transport.rs`: Testing adapter with canned responses
- `callback_permission.rs`: Callback-based permission service
- `callback_hook.rs`: Callback-based hook service

**Application Services** (`src/application/`):
- Orchestration layer coordinating domain and adapters
- `query.rs`: Core Query orchestration logic

### DSL System Architecture

Located in `src/dsl/`, implements a complete workflow engine:

**Core Components**:
- `schema.rs`: Type definitions for workflows, agents, tasks, tools, permissions
- `parser.rs`: YAML deserialization and initial validation
- `validator.rs`: Semantic validation (agent references, dependencies, cycles, variables)
- `executor.rs`: Main execution engine coordinating agents and tasks
- `task_graph.rs`: Hierarchical task management, dependency resolution, DAG traversal
- `message_bus.rs`: Inter-agent communication channels
- `state.rs`: Workflow state persistence and resumption
- `hooks.rs`: Lifecycle hooks (on_start, on_complete, on_error)
- `variables.rs`: Variable context management and interpolation
- `nl_generator.rs`: Natural language to DSL conversion
- `template.rs`: Auto-generated template generation
- `notifications.rs`: Multi-channel notification delivery system

**Key Features**:
- Hierarchical task decomposition with parent-child relationships
- Dependency-based execution order (topological sort)
- Agent specialization via tool filtering and permission modes
- State checkpointing for resume capability
- Natural language workflow generation
- **Variable system** with scoped input/output variables and interpolation
- **Notification system** with multi-channel support and MCP integration

**Variable System** (`src/dsl/variables.rs`):
- Scoped variables at workflow, agent, task, and subflow levels
- Variable interpolation using `${scope.variable}` or `${variable}` syntax
- Runtime variable resolution with proper scope hierarchy
- Input variables with type definitions, defaults, and validation
- Output variables sourced from files, state, or task results
- Validator checks for undefined variable references
- Example usage:
  ```yaml
  inputs:
    project_name:
      type: string
      required: true
      default: "MyProject"

  agents:
    researcher:
      description: "Research ${workflow.project_name}"
      inputs:
        api_key:
          type: string
          required: true

  tasks:
    analyze:
      description: "Analyze project ${workflow.project_name}"
      inputs:
        config: "${workflow.project_name}/config.yaml"
      outputs:
        result:
          source:
            type: file
            path: "./analysis.json"
  ```

## CLI Integration

The SDK communicates with the CLI via subprocess:

1. **Discovery**: Finds CLI binary via PATH, symlinks, shell aliases, or common install locations
2. **Communication**: Bidirectional NDJSON over stdin/stdout
3. **Protocol**: Alternating request/response with control messages
4. **Version Check**: Validates minimum CLI version (2.0.0+)

Skip version check: `export PERIPLON_SKIP_VERSION_CHECK=1`

## Message Flow

```
User -> PeriplonSDKClient -> Query -> Transport (CLI subprocess)
                                         |
                                         v
User <- PeriplonSDKClient <- Query <- Transport (NDJSON stream)
```

Messages are strongly typed enums:
- `Message::User`: User prompts
- `Message::Assistant`: AI responses with content blocks
- `Message::System`: System-level information
- `Message::Result`: Query completion with costs/metadata
- `Message::StreamEvent`: Real-time streaming events

Content blocks:
- `Text`: Regular text responses
- `Thinking`: Extended thinking with signature
- `ToolUse`: Tool invocation requests
- `ToolResult`: Tool execution results

## Permission System

Permission modes control auto-approval behavior:
- `default`: Prompt for dangerous operations
- `acceptEdits`: Auto-approve file edits
- `plan`: Planning mode, no execution
- `bypassPermissions`: Skip all checks (dangerous)

Permissions can be overridden via callbacks in `AgentOptions`.

## DSL Workflow Structure

```yaml
name: "Workflow Name"
version: "1.0.0"
description: "Optional description"

# Provider selection (workflow-level default)
provider: claude  # claude (default) or codex
model: "claude-sonnet-4-5"  # optional workflow-level model

agents:
  agent_id:
    description: "What this agent does"
    provider: claude  # optional, overrides workflow-level provider
    model: "claude-sonnet-4-5"  # optional, overrides workflow-level model
    tools: [Read, Write, WebSearch]  # tool allowlist
    permissions:
      mode: "acceptEdits"
      max_turns: 10  # optional limit

tasks:
  task_id:
    description: "Task description"
    agent: "agent_id"
    depends_on: [other_task_id]  # optional dependencies
    subtasks: [child_task_id]    # optional hierarchy
    output: "output.md"           # optional output file
    inputs:
      key: "value"                # optional task inputs
```

### Provider Selection

Periplon supports multiple AI providers:

**Claude (Anthropic)**: Default provider
- CLI: `claude` (requires `@anthropics/claude` npm package)
- Models: `claude-sonnet-4-5`, `claude-sonnet-4`, `claude-opus-4`, `claude-haiku-4`
- Command: `claude --output-format stream-json --verbose`

**Codex (OpenAI)**: Alternative provider with automatic sandbox bypass
- CLI: `codex` (requires separate Codex CLI installation)
- Models: `gpt-5-codex`, `gpt-5`
- Command: `codex --output-format stream-json --verbose --dangerously-bypass-approvals-and-sandbox`

Provider selection cascades:
1. Agent-level `provider` field (highest priority)
2. Workflow-level `provider` field
3. Default: `claude`

Model selection cascades:
1. Agent-level `model` field (highest priority)
2. Workflow-level `model` field
3. Provider default (`claude-sonnet-4-5` for Claude, `gpt-5-codex` for Codex)

Example with mixed providers:
```yaml
name: "Mixed Provider Workflow"
version: "1.0.0"
provider: claude  # default for all agents
model: "claude-sonnet-4-5"

agents:
  researcher:
    description: "Research using Claude"
    # inherits workflow provider and model

  coder:
    description: "Code generation using Codex"
    provider: codex  # override to use Codex
    model: "gpt-5-codex"
```

## Key Implementation Patterns

### Async Streams
All message flows use `async_stream::stream!` for non-blocking iteration:
```rust
let stream = query("prompt", None).await?;
while let Some(msg) = stream.next().await {
    // Process message
}
```

### Error Handling
Uses `thiserror` for structured errors:
- `CliNotFound`: CLI binary not available
- `NotConnected`: Client not connected
- `InvalidMessage`: Malformed NDJSON
- `TransportError`: Communication failure
- `ValidationError`: DSL validation failure

### Transport Abstraction
All CLI communication goes through the `Transport` trait, enabling:
- Real subprocess communication (`SubprocessCLITransport`)
- Mock testing (`MockTransport`)
- Future transport implementations (HTTP, gRPC, etc.)

## Testing Strategy

1. **Unit Tests**: Domain logic and message parsing
2. **Integration Tests**: Real CLI communication (requires CLI installed)
3. **Mock Tests**: Use `MockTransport` for deterministic testing
4. **Benchmarks**: Performance tracking via Criterion

## Binary Targets

- `periplon-executor`: Standalone DSL workflow executor CLI
  - Template generation
  - Natural language workflow creation
  - Validation
  - Execution

- `periplon-tui`: Interactive TUI for DSL workflow management
  - Workflow browsing and management
  - Interactive YAML editor with syntax highlighting
  - Real-time execution monitoring
  - State persistence and resume capability
  - AI-powered workflow generation
  - Context-sensitive help system

## Commit Conventions

This project uses **Conventional Commits** for automatic semantic versioning and changelog generation.

### Commit Message Format

```
<type>(<scope>): <subject>

[optional body]

[optional footer(s)]
```

### Commit Types

- **feat**: A new feature (triggers MINOR version bump)
- **fix**: A bug fix (triggers PATCH version bump)
- **docs**: Documentation only changes (triggers PATCH version bump)
- **style**: Code style changes (formatting, missing semi-colons, etc.) (triggers PATCH version bump)
- **refactor**: Code refactoring without changing functionality (triggers PATCH version bump)
- **perf**: Performance improvements (triggers PATCH version bump)
- **test**: Adding or updating tests (triggers PATCH version bump)
- **build**: Changes to build system or dependencies (triggers PATCH version bump)
- **ci**: Changes to CI configuration (triggers PATCH version bump)
- **chore**: Other changes that don't modify src or test files (triggers PATCH version bump)
- **revert**: Reverts a previous commit (triggers PATCH version bump)

### Breaking Changes

To indicate a breaking change, add `!` after the type or include `BREAKING CHANGE:` in the footer:

```
feat!: remove deprecated API endpoints

BREAKING CHANGE: The /v1/old-endpoint has been removed. Use /v2/new-endpoint instead.
```

Breaking changes trigger a MAJOR version bump.

### Examples

```bash
# Feature (minor version bump)
feat(dsl): add support for parallel task execution

# Bug fix (patch version bump)
fix(executor): resolve race condition in task completion

# Breaking change (major version bump)
feat(api)!: redesign message protocol

BREAKING CHANGE: Message format has changed from JSON to NDJSON.
Client applications must be updated to handle the new format.

# Documentation (patch version bump)
docs(readme): add installation instructions

# Chore (patch version bump)
chore(deps): update tokio to 1.42
```

### Automated Releases

The project uses GitHub Actions to automatically:
1. Analyze commits on `main` branch
2. Determine version bump based on conventional commits
3. Update `Cargo.toml` version
4. Create and push git tag (e.g., `v0.2.0`)
5. Generate changelog from commit messages
6. Trigger release workflow to build and publish binaries

**No manual version bumping or tagging is required** - just push conventional commits to `main`.

## Important Development Notes

- **Never buffer entire responses**: Use streaming throughout
- **Maintain hexagonal boundaries**: Domain must not depend on adapters/infrastructure
- **Type safety over runtime checks**: Leverage Rust's type system
- **Async everywhere**: All I/O must be async
- **Structured errors**: Always use specific error variants, never strings
- **DSL validation**: Validate workflows before execution (cycles, references, etc.)
- **CLI version compatibility**: Ensure minimum version 2.0.0
- **ALWAYS use Rust best practices and idioms for safety, performance, and maintainability**
- **ALWAYS use conventional commits for all changes** to enable automated releases 
- make sure just ci passes without errors before any commit

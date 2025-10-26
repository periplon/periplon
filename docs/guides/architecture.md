# Architecture Guide

## Hexagonal Architecture

Periplon follows the **Hexagonal Architecture** pattern (also known as Ports and Adapters), providing clean separation of concerns and testability.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          EXTERNAL WORLD                                  │
│  (User Applications, CLI Process, External MCP Servers)                  │
└──────────────────────┬──────────────────────────────────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
┌───────▼──────┐  ┌────▼─────┐  ┌────▼────────┐
│  Primary     │  │ Primary  │  │  Secondary  │
│  Adapters    │  │  Ports   │  │  Ports      │
│  (Drivers)   │  │          │  │             │
├──────────────┤  ├──────────┤  ├─────────────┤
│              │  │          │  │             │
│ query()      │─▶│ Agent    │  │ Transport   │
│ function     │  │ Service  │  │ trait       │
│              │  │          │  │             │
│ ClaudeSDK    │─▶│ Message  │  │ Permission  │
│ Client       │  │ Handler  │  │ Service     │
│              │  │          │  │             │
└──────────────┘  └─────┬────┘  └──────▲──────┘
                        │               │
                  ┌─────▼───────────────┴──────┐
                  │    DOMAIN CORE              │
                  ├─────────────────────────────┤
                  │ - Message types & parsing   │
                  │ - Permission logic          │
                  │ - Hook orchestration        │
                  │ - Session management        │
                  │ - Control protocol state    │
                  └─────────────────────────────┘
```

## Core Components

### 1. Domain Core (`src/domain/`)

Pure business logic with zero external dependencies:

- **`message.rs`**: Message types (User, Assistant, System, Result, StreamEvent)
- **`session.rs`**: Session management and state
- **`permission.rs`**: Permission evaluation logic
- **`control.rs`**: Control protocol state machine
- **`hook.rs`**: Hook type definitions

**Key Principle**: Domain code never imports from adapters or infrastructure.

### 2. Primary Ports (`src/ports/primary/`)

Inbound interfaces used by external actors:

- **`agent_service.rs`**: Query execution and message handling traits
- **`session_manager.rs`**: Session lifecycle management
- **`control_protocol.rs`**: Control flow management

**Purpose**: Define what the application can do.

### 3. Secondary Ports (`src/ports/secondary/`)

Outbound interfaces for external systems:

- **`transport.rs`**: CLI communication trait
- **`permission_service.rs`**: Permission evaluation trait
- **`hook_service.rs`**: Hook execution trait
- **`mcp_server.rs`**: MCP server integration trait

**Purpose**: Define what the application needs from the outside world.

### 4. Primary Adapters (`src/adapters/primary/`)

Drive the application from outside:

- **`query_fn.rs`**: Simple `query()` function for one-shot queries
- **`sdk_client.rs`**: `PeriplonSDKClient` for interactive multi-turn conversations

**Purpose**: Provide convenient APIs for users.

### 5. Secondary Adapters (`src/adapters/secondary/`)

Implement connections to external systems:

- **`subprocess_transport.rs`**: CLI subprocess communication via stdin/stdout
- **`mock_transport.rs`**: Testing adapter with canned responses
- **`callback_permission.rs`**: Callback-based permission service
- **`callback_hook.rs`**: Callback-based hook service

**Purpose**: Integrate with external dependencies.

## Message Flow

```
User Application
      │
      ▼
query() or PeriplonSDKClient
      │
      ▼
Query Service (Application Layer)
      │
      ├──▶ Domain Logic
      │    ├─ Message Parsing
      │    ├─ Permission Check
      │    └─ Hook Execution
      │
      ▼
Transport Adapter
      │
      ▼
CLI Process (External)
      │
      ▼
NDJSON Stream
      │
      ▼
Message Stream back to User
```

## DSL System Architecture

Located in `src/dsl/`, implements a complete workflow engine:

### Core Components

- **`schema.rs`**: Type definitions for workflows, agents, tasks, tools, permissions
- **`parser.rs`**: YAML deserialization and initial validation
- **`validator.rs`**: Semantic validation (agent references, dependencies, cycles, variables)
- **`executor.rs`**: Main execution engine coordinating agents and tasks
- **`task_graph.rs`**: Hierarchical task management, dependency resolution, DAG traversal
- **`message_bus.rs`**: Inter-agent communication channels
- **`state.rs`**: Workflow state persistence and resumption
- **`hooks.rs`**: Lifecycle hooks (on_start, on_complete, on_error)
- **`variables.rs`**: Variable context management and interpolation
- **`nl_generator.rs`**: Natural language to DSL conversion
- **`template.rs`**: Auto-generated template generation
- **`notifications.rs`**: Multi-channel notification delivery system

### Key Features

1. **Hierarchical task decomposition** with parent-child relationships
2. **Dependency-based execution order** (topological sort)
3. **Agent specialization** via tool filtering and permission modes
4. **State checkpointing** for resume capability
5. **Natural language workflow generation**
6. **Variable system** with scoped input/output variables and interpolation
7. **Notification system** with multi-channel support and MCP integration

## Dependency Flow

```
External World
    │
    ▼
Primary Adapters ────▶ Primary Ports
                          │
                          ▼
                     Domain Core
                          │
                          ▼
Secondary Ports ─────▶ Secondary Adapters
    │
    ▼
External Systems
```

**Rules**:
- Domain never depends on adapters
- Ports are interfaces only
- Adapters implement ports
- Dependencies point inward toward domain

## Benefits of This Architecture

### 1. Testability

Mock adapters for isolated testing:

```rust
let mock_transport = Box::new(MockTransport::new(test_messages));
let query = Query::new(mock_transport, false, None, None);
```

### 2. Flexibility

Swap implementations without changing core logic:

```rust
// Use real CLI
let transport = Box::new(SubprocessCLITransport::new());

// Or use mock for testing
let transport = Box::new(MockTransport::new(responses));
```

### 3. Maintainability

Clear boundaries prevent coupling:
- Business logic isolated in domain
- Easy to locate and modify code
- Changes to adapters don't affect domain

### 4. Extensibility

Add new adapters without modifying core:
- HTTP transport adapter
- gRPC transport adapter
- Database persistence adapter

## Design Patterns

### Repository Pattern

Used in DSL state management:

```rust
pub trait StateRepository {
    async fn save(&self, state: &WorkflowState) -> Result<()>;
    async fn load(&self, workflow_id: &str) -> Result<Option<WorkflowState>>;
}
```

### Strategy Pattern

Permission evaluation uses strategy pattern:

```rust
pub trait PermissionService {
    async fn can_use_tool(&self, request: PermissionRequest) -> Result<bool>;
}
```

### Observer Pattern

Hook system implements observer pattern:

```rust
pub trait HookService {
    async fn on_event(&self, event: HookEvent) -> Result<()>;
}
```

### Factory Pattern

Transport creation:

```rust
impl Transport {
    pub fn subprocess() -> Box<dyn Transport> {
        Box::new(SubprocessCLITransport::new())
    }

    pub fn mock(responses: Vec<Value>) -> Box<dyn Transport> {
        Box::new(MockTransport::new(responses))
    }
}
```

## Code Organization

```
src/
├── domain/              # Pure business logic
│   ├── message.rs
│   ├── session.rs
│   ├── permission.rs
│   ├── control.rs
│   └── hook.rs
│
├── ports/
│   ├── primary/         # Inbound interfaces
│   │   ├── agent_service.rs
│   │   ├── session_manager.rs
│   │   └── control_protocol.rs
│   │
│   └── secondary/       # Outbound interfaces
│       ├── transport.rs
│       ├── permission_service.rs
│       ├── hook_service.rs
│       └── mcp_server.rs
│
├── adapters/
│   ├── primary/         # Application drivers
│   │   ├── query_fn.rs
│   │   └── sdk_client.rs
│   │
│   └── secondary/       # External integrations
│       ├── subprocess_transport.rs
│       ├── mock_transport.rs
│       ├── callback_permission.rs
│       └── callback_hook.rs
│
├── application/         # Orchestration layer
│   └── query.rs
│
└── dsl/                 # DSL workflow system
    ├── schema.rs
    ├── parser.rs
    ├── validator.rs
    ├── executor.rs
    └── ...
```

## Testing Strategy

### 1. Unit Tests

Test domain logic in isolation:

```rust
#[test]
fn test_message_parsing() {
    let json = r#"{"type": "user", ...}"#;
    let msg: Message = serde_json::from_str(json).unwrap();
    assert!(matches!(msg, Message::User(_)));
}
```

### 2. Integration Tests

Test with mock adapters:

```rust
#[tokio::test]
async fn test_query_flow() {
    let responses = vec![...];
    let transport = Box::new(MockTransport::new(responses));
    let mut query = Query::new(transport, false, None, None);
    // Test complete flow
}
```

### 3. E2E Tests

Test with real CLI (requires CLI installed):

```rust
#[tokio::test]
async fn test_real_cli() {
    let stream = query("test", None).await.unwrap();
    // Test with actual CLI
}
```

## Extension Points

### 1. Custom Transport

Implement the `Transport` trait:

```rust
struct HttpTransport { /* ... */ }

#[async_trait]
impl Transport for HttpTransport {
    async fn send(&mut self, msg: &Value) -> Result<()> {
        // HTTP implementation
    }

    async fn receive(&mut self) -> Result<Option<Value>> {
        // HTTP implementation
    }
}
```

### 2. Custom Permission Service

```rust
struct DatabasePermissionService { /* ... */ }

#[async_trait]
impl PermissionService for DatabasePermissionService {
    async fn can_use_tool(&self, request: PermissionRequest) -> Result<bool> {
        // Database lookup
    }
}
```

### 3. Custom Hook Service

```rust
struct LoggingHookService { /* ... */ }

#[async_trait]
impl HookService for LoggingHookService {
    async fn on_event(&self, event: HookEvent) -> Result<()> {
        // Log to file/database
    }
}
```

## Best Practices

1. **Keep domain pure**: No external dependencies in domain code
2. **Use interfaces**: Program to ports, not implementations
3. **Dependency inversion**: High-level modules don't depend on low-level modules
4. **Single responsibility**: Each component has one reason to change
5. **Test at boundaries**: Test adapters and domain separately

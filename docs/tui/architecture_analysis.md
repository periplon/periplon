# TUI REPL Architecture Analysis

**Project:** DSL TUI REPL Development
**Version:** 1.0.0
**Date:** 2025-10-21
**Purpose:** Foundation analysis for integrating TUI REPL into hexagonal architecture

---

## Executive Summary

The TUI REPL will function as a **Primary Adapter** in the hexagonal architecture, driving the application through existing Primary Ports. This analysis identifies integration points, architectural boundaries, and implementation patterns for building an interactive terminal interface to the DSL workflow system.

---

## 1. Current Architecture Overview

### 1.1 Hexagonal Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                   PRIMARY ADAPTERS                       │
│  (Drive application - User Interface Layer)             │
│                                                          │
│  • query_fn.rs         - Simple function interface      │
│  • sdk_client.rs       - Interactive multi-turn client  │
│  • dsl_executor binary - CLI workflow runner            │
│  → TUI REPL (NEW)      - Interactive DSL interface      │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                    PRIMARY PORTS                         │
│  (Inbound interfaces the application exposes)           │
│                                                          │
│  • AgentService        - Query execution                │
│  • SessionManager      - Session lifecycle              │
│  • ControlProtocol     - Control flow management        │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                  APPLICATION SERVICES                    │
│  (Orchestration layer)                                  │
│                                                          │
│  • Query              - Core orchestration logic        │
│  • DSLExecutor        - Workflow execution engine       │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                     DOMAIN CORE                          │
│  (Pure business logic - no external dependencies)       │
│                                                          │
│  • Message            - Message types & parsing         │
│  • Session            - Session state management        │
│  • Permission         - Permission evaluation           │
│  • Control            - Control protocol state machine  │
│  • Hook               - Hook type definitions           │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                   SECONDARY PORTS                        │
│  (Outbound interfaces to external systems)              │
│                                                          │
│  • Transport          - CLI communication               │
│  • PermissionService  - Permission evaluation           │
│  • HookService        - Hook execution                  │
│  • McpServer          - MCP server integration          │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│                  SECONDARY ADAPTERS                      │
│  (Implementation of external system connections)        │
│                                                          │
│  • SubprocessCLITransport  - Subprocess communication   │
│  • MockTransport           - Testing adapter            │
│  • CallbackPermission      - Callback-based permissions │
│  • CallbackHook            - Callback-based hooks       │
└─────────────────────────────────────────────────────────┘
```

### 1.2 Key Components

**Domain Core** (`src/domain/`):
- `message.rs`: Message types (User, Assistant, System, Result, StreamEvent)
- `session.rs`: Session management and state
- `permission.rs`: Permission evaluation logic
- `control.rs`: Control protocol state machine
- `hook.rs`: Hook type definitions

**Application Services** (`src/application/`):
- `query.rs`: Core Query orchestration, handles message streaming, control protocol
- DSL system (`src/dsl/`): Complete workflow engine with executor, parser, validator, state management

**Primary Adapters** (`src/adapters/primary/`):
- `query_fn.rs`: Simple one-shot query function
- `sdk_client.rs`: `PeriplonSDKClient` for multi-turn interactive conversations
- Binary: `src/bin/dsl_executor.rs` - CLI tool for workflow execution

---

## 2. Integration Points for TUI REPL

### 2.1 Primary Adapter Position

**Location:** `src/adapters/primary/tui_repl.rs` (new file)

The TUI REPL is a PRIMARY ADAPTER that:
1. Presents interactive UI to users
2. Drives the application through Primary Ports
3. Consumes domain messages and workflow state
4. Does NOT contain business logic

### 2.2 Key Integration Interfaces

#### A. PeriplonSDKClient Integration
**File:** `src/adapters/primary/sdk_client.rs:14-100`

The `PeriplonSDKClient` provides the foundation for TUI integration:

```rust
pub struct PeriplonSDKClient {
    options: AgentOptions,
    query: Option<Query>,
}

// Key methods:
async fn connect(&mut self, prompt: Option<String>) -> Result<()>
async fn query(&mut self, prompt: impl Into<String>) -> Result<()>
fn receive_messages(&self) -> Result<impl Stream<Item = Message> + '_>
fn receive_response(&self) -> Result<impl Stream<Item = Message> + '_>
```

**TUI Integration Pattern:**
- The TUI REPL can embed one or more `PeriplonSDKClient` instances
- Use `receive_messages()` to stream real-time agent responses
- Display streaming messages in TUI panels
- Handle user input through `query()` method

#### B. DSLExecutor Integration
**File:** `src/dsl/executor.rs:22-150`

The `DSLExecutor` manages workflow execution:

```rust
pub struct DSLExecutor {
    workflow: DSLWorkflow,
    agents: HashMap<String, PeriplonSDKClient>,
    task_graph: TaskGraph,
    message_bus: Arc<MessageBus>,
    state: Option<WorkflowState>,
    state_persistence: Option<StatePersistence>,
    notification_manager: Arc<NotificationManager>,
}

// Key methods for TUI:
pub fn new(workflow: DSLWorkflow) -> Result<Self>
pub async fn execute(&mut self) -> Result<()>
pub fn get_progress(&self) -> (usize, usize) // (completed, total)
```

**TUI Integration Pattern:**
- Display workflow progress in real-time
- Show task graph visualization
- Monitor agent activity across tasks
- Provide execution controls (pause, resume, cancel)

#### C. Message Stream Integration
**File:** `src/domain/message.rs`

All communication uses strongly-typed message enums:

```rust
pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
    System(SystemMessage),
    Result(ResultMessage),
    StreamEvent(StreamEventMessage),
}

pub enum ContentBlock {
    Text { text: String },
    Thinking { thinking: String, signature: Option<String> },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: Vec<ContentValue> },
}
```

**TUI Integration Pattern:**
- Subscribe to message streams
- Render different ContentBlock types appropriately
- Display thinking blocks in separate panels
- Show tool usage in real-time

---

## 3. Existing Patterns to Follow

### 3.1 Interactive Client Pattern
**Reference:** `examples/interactive_client.rs:1-90`

Key learnings:
```rust
// 1. Create client with options
let mut client = PeriplonSDKClient::new(options);

// 2. Connect once
client.connect(None).await?;

// 3. Multiple queries in same session
client.query("First query").await?;
{
    let stream = client.receive_response()?;
    futures::pin_mut!(stream);
    while let Some(msg) = stream.next().await {
        // Process messages
    }
}

// 4. Follow-up query maintains context
client.query("Follow-up query").await?;
```

**TUI Application:**
- Single connection lifecycle
- Multiple sequential queries
- Stream-based message handling
- Context preservation between queries

### 3.2 DSL Executor CLI Pattern
**Reference:** `src/bin/dsl_executor.rs:1-150`

Current CLI structure:
```rust
enum Commands {
    Run { workflow_file, state_dir, resume, clean, verbose, dry_run, json },
    Validate { workflow_file, verbose, json },
    Template { output },
    Generate { description, output },
    // ... more commands
}
```

**TUI Enhancement Opportunities:**
- Convert static CLI commands to interactive workflows
- Real-time workflow validation as user types
- Visual workflow builder/editor
- Live progress tracking during execution

### 3.3 Async Stream Pattern
**Universal pattern across codebase:**

```rust
let stream = some_async_operation();
futures::pin_mut!(stream);
while let Some(item) = stream.next().await {
    // Non-blocking iteration
}
```

**TUI Application:**
- Non-blocking message rendering
- Concurrent task monitoring
- Real-time UI updates

---

## 4. TUI Architecture Proposal

### 4.1 Component Structure

```
src/adapters/primary/tui/
├── mod.rs                  # Public API
├── app.rs                  # Main TUI application state
├── repl.rs                 # REPL logic and command parsing
├── ui/
│   ├── mod.rs
│   ├── layout.rs           # Screen layout management
│   ├── panels/
│   │   ├── chat.rs         # Chat/conversation panel
│   │   ├── workflow.rs     # Workflow visualization panel
│   │   ├── tasks.rs        # Task status panel
│   │   ├── thinking.rs     # Extended thinking panel
│   │   └── status.rs       # Status bar
│   └── widgets/
│       ├── message.rs      # Message rendering
│       ├── progress.rs     # Progress indicators
│       └── tree.rs         # Task graph visualization
├── input/
│   ├── handler.rs          # Keyboard/mouse input handling
│   └── commands.rs         # REPL command definitions
└── state/
    ├── session.rs          # Session state management
    └── view.rs             # View state (scrolling, focus, etc.)
```

### 4.2 Integration Architecture

```
┌──────────────────────────────────────────────────────────┐
│                      TUI REPL                             │
│                   (Primary Adapter)                       │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │              TUI Application State               │    │
│  │  • Active sessions                               │    │
│  │  • View state (panels, focus)                    │    │
│  │  • Input buffer                                  │    │
│  └─────────────────────────────────────────────────┘    │
│                          ↓                                │
│  ┌─────────────────────────────────────────────────┐    │
│  │           Session Manager Component              │    │
│  │  • Manages PeriplonSDKClient instances             │    │
│  │  • Manages DSLExecutor instances                 │    │
│  │  • Routes messages to UI panels                  │    │
│  └─────────────────────────────────────────────────┘    │
│                          ↓                                │
│  ┌──────────────┬──────────────┬──────────────────┐    │
│  │  Chat Panel  │ Workflow Panel│   Task Panel     │    │
│  │              │               │                  │    │
│  │  Displays:   │  Displays:    │  Displays:       │    │
│  │  • Messages  │  • Task graph │  • Task status   │    │
│  │  • Tool use  │  • Progress   │  • Dependencies  │    │
│  │  • Thinking  │  • Agents     │  • Failures      │    │
│  └──────────────┴──────────────┴──────────────────┘    │
└──────────────────────────────────────────────────────────┘
                           ↓
┌──────────────────────────────────────────────────────────┐
│              Existing Application Layer                   │
│  • PeriplonSDKClient (sdk_client.rs)                       │
│  • DSLExecutor (dsl/executor.rs)                         │
│  • Query (application/query.rs)                          │
└──────────────────────────────────────────────────────────┘
```

### 4.3 Technology Stack

**Recommended TUI Libraries:**

1. **ratatui** (formerly tui-rs)
   - Most popular Rust TUI framework
   - Widget-based architecture
   - Terminal backend abstraction
   - Active development

2. **crossterm**
   - Cross-platform terminal manipulation
   - Event handling (keyboard, mouse, resize)
   - Direct rendering control
   - Works seamlessly with ratatui

**Cargo.toml additions:**
```toml
[dependencies]
ratatui = "0.26"
crossterm = { version = "0.27", features = ["event-stream"] }
tui-textarea = "0.4"  # For multi-line input
```

---

## 5. Key Design Decisions

### 5.1 Respect Hexagonal Boundaries

**✅ Correct:**
- TUI components call `PeriplonSDKClient` methods
- TUI renders `Message` domain types
- TUI observes `DSLWorkflow` state
- TUI uses `AgentOptions` for configuration

**❌ Incorrect:**
- TUI directly manipulating subprocess transport
- TUI parsing NDJSON messages
- TUI containing workflow execution logic
- TUI bypassing Primary Ports

### 5.2 Message Flow Architecture

```
User Input → TUI Input Handler → REPL Command Parser
                                        ↓
                              Command Router
                                        ↓
                    ┌──────────────────┴──────────────────┐
                    ↓                                      ↓
            PeriplonSDKClient.query()            DSLExecutor.execute()
                    ↓                                      ↓
            Message Stream                         Task Events
                    ↓                                      ↓
            TUI Message Renderer              TUI Workflow Renderer
                    ↓                                      ↓
            Terminal Display                   Terminal Display
```

### 5.3 State Management

**Separation of Concerns:**
- **Domain State** (Session, WorkflowState): Managed by application layer
- **View State** (scroll position, focus, panel size): Managed by TUI
- **TUI observes domain state**, never mutates it directly

### 5.4 Async Coordination

Use Tokio channels for async communication:
```rust
// Message channel from application to TUI
let (msg_tx, msg_rx) = mpsc::unbounded_channel::<Message>();

// Input channel from TUI to application
let (input_tx, input_rx) = mpsc::unbounded_channel::<String>();

// Event loop:
tokio::select! {
    Some(msg) = msg_rx.recv() => {
        // Update UI with new message
    }
    Some(event) = event_stream.next() => {
        // Handle keyboard/mouse input
    }
    _ = tick_interval.tick() => {
        // Periodic UI refresh
    }
}
```

---

## 6. Implementation Roadmap

### Phase 1: Foundation (TUI Adapter)
1. Create `src/adapters/primary/tui/` module structure
2. Implement basic ratatui application skeleton
3. Add crossterm event handling
4. Create simple text display panel

### Phase 2: Client Integration
1. Embed `PeriplonSDKClient` in TUI app state
2. Implement message stream subscription
3. Render `Message::Assistant` content blocks
4. Handle user input → `client.query()` flow

### Phase 3: REPL Commands
1. Define REPL command grammar (slash commands)
2. Implement command parser
3. Add workflow commands (load, validate, run)
4. Add session commands (clear, history, export)

### Phase 4: Multi-Panel Layout
1. Split screen into panels (chat, tasks, thinking)
2. Implement panel focus/switching
3. Add scrolling within panels
4. Implement status bar

### Phase 5: Workflow Visualization
1. Integrate `DSLExecutor`
2. Render task graph as tree
3. Show real-time task progress
4. Display agent activity

### Phase 6: Advanced Features
1. Workflow editor (inline YAML editing)
2. Syntax highlighting
3. Auto-completion
4. Help system
5. Configuration management

---

## 7. Critical Constraints

### 7.1 CLI Communication
**Constraint:** All CLI communication happens through `SubprocessCLITransport`

**Implication for TUI:**
- Cannot modify transport layer
- Must work within subprocess stdin/stdout model
- Single active query per client instance
- Must handle subprocess lifecycle

### 7.2 Message Streaming
**Constraint:** Messages arrive as async streams, not synchronous responses

**Implication for TUI:**
- UI must be non-blocking
- Use async event loop with `tokio::select!`
- Buffer incomplete messages
- Handle stream completion

### 7.3 Session State
**Constraint:** Session state managed by `Query` and `PeriplonSDKClient`

**Implication for TUI:**
- TUI cannot directly access session history
- Must maintain separate view-level history
- Display state derived from message stream
- No direct session mutation

### 7.4 Tool Permissions
**Constraint:** Permission callbacks configured in `AgentOptions`

**Implication for TUI:**
- TUI must provide permission callback handler
- Display permission requests in UI
- Collect user approval/denial
- Pass decision back through callback

---

## 8. Example Integration Code

### 8.1 TUI Application Skeleton

```rust
// src/adapters/primary/tui/app.rs

use crate::adapters::primary::PeriplonSDKClient;
use crate::domain::Message;
use crate::options::AgentOptions;
use crossterm::event::{Event, KeyCode};
use ratatui::prelude::*;
use tokio::sync::mpsc;

pub struct TuiApp {
    client: Option<PeriplonSDKClient>,
    messages: Vec<Message>,
    input_buffer: String,
    should_quit: bool,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            client: None,
            messages: Vec::new(),
            input_buffer: String::new(),
            should_quit: false,
        }
    }

    pub async fn connect(&mut self, options: AgentOptions) -> Result<()> {
        let mut client = PeriplonSDKClient::new(options);
        client.connect(None).await?;
        self.client = Some(client);
        Ok(())
    }

    pub async fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                if let Some(client) = &mut self.client {
                    let query = self.input_buffer.clone();
                    self.input_buffer.clear();

                    client.query(query).await?;

                    // Spawn task to collect messages
                    let stream = client.receive_response()?;
                    tokio::spawn(async move {
                        futures::pin_mut!(stream);
                        while let Some(msg) = stream.next().await {
                            // Send to UI channel
                        }
                    });
                }
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            _ => {}
        }
        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame) {
        // Render UI panels
    }
}
```

### 8.2 REPL Command Parser

```rust
// src/adapters/primary/tui/input/commands.rs

#[derive(Debug, Clone)]
pub enum ReplCommand {
    // Query commands
    Query(String),

    // Workflow commands
    LoadWorkflow(PathBuf),
    ValidateWorkflow,
    RunWorkflow,
    StopWorkflow,

    // Session commands
    ClearHistory,
    SaveSession(PathBuf),
    LoadSession(PathBuf),

    // Control commands
    Help,
    Quit,
}

pub fn parse_command(input: &str) -> Result<ReplCommand> {
    if input.starts_with('/') {
        // Slash command
        let parts: Vec<&str> = input.trim_start_matches('/').split_whitespace().collect();
        match parts[0] {
            "load" => Ok(ReplCommand::LoadWorkflow(PathBuf::from(parts[1]))),
            "validate" => Ok(ReplCommand::ValidateWorkflow),
            "run" => Ok(ReplCommand::RunWorkflow),
            "help" => Ok(ReplCommand::Help),
            "quit" => Ok(ReplCommand::Quit),
            _ => Err(Error::InvalidCommand(input.to_string())),
        }
    } else {
        // Regular query
        Ok(ReplCommand::Query(input.to_string()))
    }
}
```

---

## 9. Testing Strategy

### 9.1 Unit Tests
- Test REPL command parsing
- Test message rendering logic
- Test state transitions
- Test input handling

### 9.2 Integration Tests
- Test TUI + `PeriplonSDKClient` integration
- Test TUI + `DSLExecutor` integration
- Use `MockTransport` for deterministic testing

### 9.3 Manual Testing
- Test with real CLI subprocess
- Test multi-turn conversations
- Test workflow execution
- Test error handling

---

## 10. References

### 10.1 Key Files to Study

**Primary Adapters:**
- `src/adapters/primary/sdk_client.rs:14-100` - Client pattern
- `src/adapters/primary/query_fn.rs` - Simple query pattern
- `src/bin/dsl_executor.rs:1-150` - CLI structure

**Application Layer:**
- `src/application/query.rs:14-100` - Message orchestration
- `src/dsl/executor.rs:22-150` - Workflow execution

**Domain:**
- `src/domain/message.rs` - Message type definitions
- `src/dsl/schema.rs:1-150` - DSL workflow schema

**Examples:**
- `examples/interactive_client.rs:1-90` - Multi-turn pattern
- `examples/dsl_executor_example.rs` - Workflow execution

### 10.2 External Documentation

**Hexagonal Architecture:**
- Ports and Adapters pattern
- Dependency inversion principle
- Separation of concerns

**TUI Libraries:**
- ratatui documentation: https://ratatui.rs
- crossterm documentation: https://docs.rs/crossterm
- TUI patterns and best practices

---

## 11. Conclusion

The TUI REPL will integrate cleanly as a Primary Adapter, consuming existing Primary Ports without violating hexagonal architecture boundaries. Key integration points are:

1. **PeriplonSDKClient** for interactive agent queries
2. **DSLExecutor** for workflow execution and monitoring
3. **Message streams** for real-time UI updates
4. **AgentOptions** for configuration

The implementation follows established patterns from `interactive_client.rs` and `dsl_executor.rs`, ensuring consistency with the existing codebase.

**Next Steps:**
1. Set up ratatui + crossterm dependencies
2. Create basic TUI application skeleton
3. Integrate PeriplonSDKClient for message streaming
4. Implement REPL command parser
5. Build out multi-panel UI layout

---

**End of Architecture Analysis**

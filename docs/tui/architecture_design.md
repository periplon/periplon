# TUI REPL Comprehensive Architecture Design

**Project:** DSL TUI REPL Development
**Version:** 1.0.0
**Date:** 2025-10-21
**Purpose:** Detailed architectural design following hexagonal patterns
**Based On:** architecture_analysis.md

---

## Table of Contents

1. [Design Principles](#1-design-principles)
2. [Component Hierarchy](#2-component-hierarchy)
3. [Module Structure](#3-module-structure)
4. [State Management Architecture](#4-state-management-architecture)
5. [Event Handling System](#5-event-handling-system)
6. [Data Flow Patterns](#6-data-flow-patterns)
7. [View Layer Architecture](#7-view-layer-architecture)
8. [Extension Points](#8-extension-points)
9. [Implementation Specifications](#9-implementation-specifications)
10. [Testing Strategy](#10-testing-strategy)

---

## 1. Design Principles

### 1.1 Core Principles

**Hexagonal Architecture Compliance:**
- TUI is a **Primary Adapter** - drives application, never contains business logic
- All domain interaction through **Primary Ports** (AgentService, SessionManager, ControlProtocol)
- Clear separation: View State vs. Domain State
- No direct dependency on Secondary Adapters

**Async-First Design:**
- Non-blocking message rendering
- Concurrent task monitoring
- Streaming data integration
- Tokio-based coordination

**Modularity & Extensibility:**
- Plugin architecture for custom panels
- Configurable layouts
- Theme system for customization
- Command extensibility

**Performance:**
- Lazy rendering (only visible content)
- Message buffering and pagination
- Efficient screen updates
- Memory-bounded history

### 1.2 Quality Attributes

| Attribute | Target | Strategy |
|-----------|--------|----------|
| Responsiveness | <16ms UI updates | Async event loop, buffered rendering |
| Reliability | 99.9% uptime | Error boundaries, graceful degradation |
| Maintainability | <200 LOC/module | Clear separation of concerns, trait-based design |
| Testability | >80% coverage | Mock adapters, component isolation |
| Extensibility | Plugin support | Trait-based panels, event bus |

---

## 2. Component Hierarchy

### 2.1 Top-Level Architecture

```
┌────────────────────────────────────────────────────────────┐
│                      TUI Application                        │
│              (Primary Adapter Layer)                        │
└────────────────────────────────────────────────────────────┘
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
          ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   Runtime    │  │     State    │  │      UI      │
│   Manager    │  │   Manager    │  │   Renderer   │
└──────────────┘  └──────────────┘  └──────────────┘
          │                 │                 │
          │                 │                 │
          ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Event Loop   │  │  Session     │  │   Layout     │
│ Coordinator  │  │  Context     │  │   Manager    │
└──────────────┘  └──────────────┘  └──────────────┘
```

### 2.2 Component Responsibilities

#### A. Runtime Manager
**Responsibility:** Lifecycle management and async coordination

Components:
- **Event Loop Coordinator:** Main tokio::select! loop
- **Message Router:** Routes domain messages to UI components
- **Task Spawner:** Manages background tasks
- **Shutdown Handler:** Graceful cleanup

#### B. State Manager
**Responsibility:** Unified state management

Components:
- **Session Context:** Active sessions (PeriplonSDKClient, DSLExecutor instances)
- **View State:** UI-specific state (scroll, focus, visibility)
- **History Manager:** Message and command history
- **Configuration Store:** User preferences and settings

#### C. UI Renderer
**Responsibility:** Terminal rendering and layout

Components:
- **Layout Manager:** Panel positioning and sizing
- **Panel Registry:** Available panels and their instances
- **Widget Library:** Reusable UI components
- **Theme Engine:** Color schemes and styling

---

## 3. Module Structure

### 3.1 Directory Layout

```
src/adapters/primary/tui/
├── mod.rs                          # Public API exports
├── lib.rs                          # Library entry point
│
├── runtime/                        # Runtime management
│   ├── mod.rs
│   ├── app.rs                      # Main TuiApp struct
│   ├── event_loop.rs               # Tokio event loop coordinator
│   ├── message_router.rs           # Message routing logic
│   └── lifecycle.rs                # Startup/shutdown logic
│
├── state/                          # State management
│   ├── mod.rs
│   ├── manager.rs                  # Unified state manager
│   ├── session.rs                  # Session context (clients, executors)
│   ├── view.rs                     # View state (scroll, focus, etc.)
│   ├── history.rs                  # Message/command history
│   └── config.rs                   # Configuration persistence
│
├── ui/                             # UI layer
│   ├── mod.rs
│   ├── renderer.rs                 # Main rendering logic
│   ├── layout/
│   │   ├── mod.rs
│   │   ├── manager.rs              # Layout management
│   │   ├── presets.rs              # Predefined layouts
│   │   └── builder.rs              # Layout DSL
│   ├── panels/                     # Panel implementations
│   │   ├── mod.rs
│   │   ├── traits.rs               # Panel trait definition
│   │   ├── registry.rs             # Panel registration
│   │   ├── chat.rs                 # Chat/conversation panel
│   │   ├── workflow.rs             # Workflow visualization
│   │   ├── tasks.rs                # Task status panel
│   │   ├── thinking.rs             # Extended thinking display
│   │   ├── logs.rs                 # System logs panel
│   │   ├── help.rs                 # Help/documentation panel
│   │   └── status_bar.rs           # Status bar at bottom
│   ├── widgets/                    # Reusable widgets
│   │   ├── mod.rs
│   │   ├── message.rs              # Message rendering
│   │   ├── progress.rs             # Progress bars/spinners
│   │   ├── tree.rs                 # Task tree visualization
│   │   ├── input.rs                # Input field
│   │   ├── table.rs                # Data tables
│   │   └── modal.rs                # Modal dialogs
│   └── theme/
│       ├── mod.rs
│       ├── engine.rs               # Theme system
│       ├── default.rs              # Default theme
│       └── colors.rs               # Color definitions
│
├── input/                          # Input handling
│   ├── mod.rs
│   ├── handler.rs                  # Event handler
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── parser.rs               # Command parsing
│   │   ├── registry.rs             # Command registration
│   │   ├── builtin.rs              # Built-in commands
│   │   └── workflow.rs             # Workflow commands
│   ├── keybindings.rs              # Keyboard shortcuts
│   └── autocomplete.rs             # Command completion
│
├── domain/                         # Domain integration
│   ├── mod.rs
│   ├── client_adapter.rs           # PeriplonSDKClient wrapper
│   ├── executor_adapter.rs         # DSLExecutor wrapper
│   ├── message_adapter.rs          # Message type adapters
│   └── permission_handler.rs       # Permission UI integration
│
├── channels/                       # Inter-component communication
│   ├── mod.rs
│   ├── types.rs                    # Channel message types
│   ├── message_channel.rs          # Domain message channel
│   ├── event_channel.rs            # UI event channel
│   └── command_channel.rs          # Command channel
│
├── extensions/                     # Extension system
│   ├── mod.rs
│   ├── traits.rs                   # Extension traits
│   ├── loader.rs                   # Plugin loader
│   └── registry.rs                 # Extension registry
│
└── utils/                          # Utilities
    ├── mod.rs
    ├── terminal.rs                 # Terminal utilities
    ├── formatting.rs               # Text formatting helpers
    └── logging.rs                  # TUI-specific logging
```

### 3.2 Module Dependencies

```
runtime → state, ui, input, domain, channels
state → domain, channels
ui → state, channels, widgets, theme
input → state, channels, commands
domain → (application layer - PeriplonSDKClient, DSLExecutor)
channels → (pure types, no dependencies)
extensions → ui, state, input
```

**Dependency Rules:**
- **No circular dependencies**
- **Channels module** is dependency-free (pure types)
- **Domain module** is the only bridge to application layer
- **UI module** never imports domain module directly

---

## 4. State Management Architecture

### 4.1 State Hierarchy

```
┌─────────────────────────────────────────────────────────┐
│                    Unified State                         │
│                  (Single Source of Truth)                │
└─────────────────────────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│   Session    │ │     View     │ │   History    │
│   State      │ │    State     │ │    State     │
└──────────────┘ └──────────────┘ └──────────────┘
```

### 4.2 State Type Definitions

```rust
// src/adapters/primary/tui/state/manager.rs

use tokio::sync::RwLock;
use std::sync::Arc;

/// Unified application state
pub struct AppState {
    /// Session state (domain-related)
    pub session: Arc<RwLock<SessionState>>,

    /// View state (UI-related)
    pub view: Arc<RwLock<ViewState>>,

    /// History state (messages, commands)
    pub history: Arc<RwLock<HistoryState>>,

    /// Configuration
    pub config: Arc<RwLock<TuiConfig>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            session: Arc::new(RwLock::new(SessionState::default())),
            view: Arc::new(RwLock::new(ViewState::default())),
            history: Arc::new(RwLock::new(HistoryState::default())),
            config: Arc::new(RwLock::new(TuiConfig::load())),
        }
    }
}
```

### 4.3 Session State

```rust
// src/adapters/primary/tui/state/session.rs

use crate::adapters::primary::PeriplonSDKClient;
use crate::dsl::DSLExecutor;
use std::collections::HashMap;

/// Session state - manages domain layer connections
pub struct SessionState {
    /// Active chat sessions
    pub clients: HashMap<SessionId, ClientContext>,

    /// Active workflow executions
    pub workflows: HashMap<WorkflowId, WorkflowContext>,

    /// Current active session
    pub active_session: Option<SessionId>,

    /// Global agent options
    pub agent_options: AgentOptions,
}

pub struct ClientContext {
    pub id: SessionId,
    pub client: PeriplonSDKClient,
    pub status: SessionStatus,
    pub created_at: Instant,
    pub last_activity: Instant,
}

pub struct WorkflowContext {
    pub id: WorkflowId,
    pub executor: DSLExecutor,
    pub workflow: DSLWorkflow,
    pub status: WorkflowStatus,
    pub progress: WorkflowProgress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Connecting,
    Active,
    Waiting,
    Disconnected,
    Error,
}

pub struct WorkflowProgress {
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub current_tasks: Vec<String>,
    pub failed_tasks: Vec<String>,
}
```

### 4.4 View State

```rust
// src/adapters/primary/tui/state/view.rs

/// View state - UI-specific state
pub struct ViewState {
    /// Current layout preset
    pub layout: LayoutPreset,

    /// Panel states
    pub panels: HashMap<PanelId, PanelState>,

    /// Focus management
    pub focus: FocusState,

    /// Visibility toggles
    pub visibility: VisibilityState,

    /// Terminal size
    pub terminal_size: (u16, u16),
}

pub struct PanelState {
    pub id: PanelId,
    pub visible: bool,
    pub scroll_offset: usize,
    pub size: PanelSize,
    pub position: PanelPosition,
}

pub struct FocusState {
    pub active_panel: Option<PanelId>,
    pub focus_history: Vec<PanelId>,
}

pub struct VisibilityState {
    pub show_thinking: bool,
    pub show_system_messages: bool,
    pub show_tool_details: bool,
    pub show_timestamps: bool,
}

#[derive(Debug, Clone)]
pub enum LayoutPreset {
    Default,           // Chat + Status
    Workflow,          // Chat + Workflow + Tasks + Status
    FullWorkflow,      // All panels visible
    Minimal,           // Chat only
    Custom(Layout),    // User-defined
}
```

### 4.5 History State

```rust
// src/adapters/primary/tui/state/history.rs

use std::collections::VecDeque;

/// History state - bounded message and command history
pub struct HistoryState {
    /// Messages per session (bounded)
    pub messages: HashMap<SessionId, MessageHistory>,

    /// Command history (global)
    pub commands: CommandHistory,

    /// Search history
    pub searches: VecDeque<String>,
}

pub struct MessageHistory {
    /// Circular buffer of messages
    pub messages: VecDeque<DisplayMessage>,

    /// Maximum messages to retain
    pub max_size: usize,

    /// Filter settings
    pub filter: MessageFilter,
}

pub struct CommandHistory {
    /// Previous commands
    pub commands: VecDeque<String>,

    /// Current position in history (for up/down navigation)
    pub position: usize,

    /// Max history size
    pub max_size: usize,
}

#[derive(Clone)]
pub struct DisplayMessage {
    pub id: MessageId,
    pub timestamp: Instant,
    pub message: Message,        // Domain Message type
    pub rendered: Option<String>, // Cached rendering
}
```

### 4.6 State Update Pattern

**Immutable Updates with Arc<RwLock>:**

```rust
// Reading state
let view_state = app_state.view.read().await;
let active_panel = view_state.focus.active_panel;

// Updating state
{
    let mut view_state = app_state.view.write().await;
    view_state.focus.active_panel = Some(PanelId::Chat);
}

// Atomic updates with helper methods
impl AppState {
    pub async fn set_active_panel(&self, panel: PanelId) {
        let mut view = self.view.write().await;
        view.focus.active_panel = Some(panel);
        view.focus.focus_history.push(panel);
    }

    pub async fn add_message(&self, session_id: SessionId, msg: Message) {
        let mut history = self.history.write().await;
        let session_history = history.messages
            .entry(session_id)
            .or_insert_with(MessageHistory::new);

        session_history.push(DisplayMessage::from(msg));
    }
}
```

---

## 5. Event Handling System

### 5.1 Event Architecture

```
┌────────────────────────────────────────────────────────┐
│                    Event Sources                        │
└────────────────────────────────────────────────────────┘
    │                  │                  │
    │ Keyboard         │ Domain           │ Timers
    │ Events           │ Messages         │
    │                  │                  │
    ▼                  ▼                  ▼
┌────────────────────────────────────────────────────────┐
│                 Event Loop (tokio::select!)             │
└────────────────────────────────────────────────────────┘
    │                  │                  │
    │                  │                  │
    ▼                  ▼                  ▼
┌─────────┐     ┌──────────┐      ┌──────────┐
│ Input   │     │ Message  │      │  Timer   │
│ Handler │     │ Handler  │      │ Handler  │
└─────────┘     └──────────┘      └──────────┘
    │                  │                  │
    └──────────────────┼──────────────────┘
                       ▼
            ┌──────────────────┐
            │   State Update   │
            └──────────────────┘
                       ▼
            ┌──────────────────┐
            │    UI Render     │
            └──────────────────┘
```

### 5.2 Event Types

```rust
// src/adapters/primary/tui/channels/types.rs

use crossterm::event::{KeyEvent, MouseEvent};
use crate::domain::Message;

/// All events that can occur in the TUI
#[derive(Debug, Clone)]
pub enum TuiEvent {
    /// Keyboard input
    Key(KeyEvent),

    /// Mouse input
    Mouse(MouseEvent),

    /// Terminal resize
    Resize(u16, u16),

    /// Domain message received
    DomainMessage {
        session_id: SessionId,
        message: Message,
    },

    /// Workflow event
    WorkflowEvent {
        workflow_id: WorkflowId,
        event: WorkflowEventType,
    },

    /// Command execution result
    CommandResult {
        command: String,
        result: Result<(), String>,
    },

    /// Timer tick (for periodic updates)
    Tick,

    /// Shutdown signal
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum WorkflowEventType {
    Started,
    TaskStarted(String),
    TaskCompleted(String),
    TaskFailed { task: String, error: String },
    ProgressUpdate { completed: usize, total: usize },
    Completed,
    Failed(String),
}
```

### 5.3 Event Loop Implementation

```rust
// src/adapters/primary/tui/runtime/event_loop.rs

use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use crossterm::event::{EventStream, Event};
use futures::StreamExt;

pub struct EventLoop {
    /// Event receiver
    event_rx: mpsc::UnboundedReceiver<TuiEvent>,

    /// Application state
    state: Arc<AppState>,

    /// Terminal backend
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl EventLoop {
    pub async fn run(mut self) -> Result<()> {
        // Create event sources
        let mut crossterm_events = EventStream::new();
        let mut tick_interval = interval(Duration::from_millis(250));

        loop {
            tokio::select! {
                // Crossterm events (keyboard, mouse, resize)
                Some(Ok(event)) = crossterm_events.next() => {
                    match event {
                        Event::Key(key) => self.handle_key(key).await?,
                        Event::Mouse(mouse) => self.handle_mouse(mouse).await?,
                        Event::Resize(w, h) => self.handle_resize(w, h).await?,
                        _ => {}
                    }
                }

                // Domain messages from application layer
                Some(event) = self.event_rx.recv() => {
                    match event {
                        TuiEvent::DomainMessage { session_id, message } => {
                            self.handle_domain_message(session_id, message).await?;
                        }
                        TuiEvent::WorkflowEvent { workflow_id, event } => {
                            self.handle_workflow_event(workflow_id, event).await?;
                        }
                        TuiEvent::CommandResult { command, result } => {
                            self.handle_command_result(command, result).await?;
                        }
                        TuiEvent::Shutdown => {
                            break;
                        }
                        _ => {}
                    }
                }

                // Periodic tick for UI updates
                _ = tick_interval.tick() => {
                    self.render().await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Delegate to input handler
        input::handle_key_event(key, &self.state).await
    }

    async fn handle_domain_message(
        &mut self,
        session_id: SessionId,
        message: Message
    ) -> Result<()> {
        // Add to history
        self.state.add_message(session_id, message.clone()).await;

        // Trigger render
        self.render().await
    }

    async fn render(&mut self) -> Result<()> {
        let state = self.state.clone();

        self.terminal.draw(|frame| {
            // Render UI with current state
            ui::render(frame, &state);
        })?;

        Ok(())
    }
}
```

### 5.4 Message Channel Architecture

```rust
// src/adapters/primary/tui/channels/message_channel.rs

use tokio::sync::mpsc;

/// Channel for domain messages from PeriplonSDKClient
pub struct MessageChannel {
    tx: mpsc::UnboundedSender<TuiEvent>,
}

impl MessageChannel {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<TuiEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { tx }, rx)
    }

    /// Send a domain message to the TUI
    pub fn send_message(&self, session_id: SessionId, message: Message) {
        let _ = self.tx.send(TuiEvent::DomainMessage {
            session_id,
            message,
        });
    }

    /// Send a workflow event
    pub fn send_workflow_event(&self, workflow_id: WorkflowId, event: WorkflowEventType) {
        let _ = self.tx.send(TuiEvent::WorkflowEvent {
            workflow_id,
            event,
        });
    }
}
```

---

## 6. Data Flow Patterns

### 6.1 Query Flow

```
User Input
    │
    ▼
┌─────────────────┐
│ Input Handler   │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Command Parser  │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Session State   │  ← Get active PeriplonSDKClient
└─────────────────┘
    │
    ▼
┌─────────────────────────────────┐
│ PeriplonSDKClient.query(prompt)   │  ← Application layer
└─────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────┐
│ Spawn message listener task      │
└──────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────┐
│ Stream messages via channel      │
└──────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────┐
│ Event Loop receives TuiEvent     │
└──────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────┐
│ Update History State             │
└──────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────┐
│ Trigger UI Render                │
└──────────────────────────────────┘
```

### 6.2 Workflow Execution Flow

```
User Command: /run workflow.yaml
    │
    ▼
┌─────────────────────────┐
│ Parse workflow file     │
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ Create DSLExecutor      │
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ Add to Session State    │
└─────────────────────────┘
    │
    ▼
┌──────────────────────────────────┐
│ Spawn executor.execute() task    │
└──────────────────────────────────┘
    │
    ├─ Task events → WorkflowEvent channel
    │
    └─ Progress updates → WorkflowEvent channel
    │
    ▼
┌──────────────────────────────────┐
│ Event Loop receives events       │
└──────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────┐
│ Update Workflow State            │
└──────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────┐
│ Render Workflow Panel            │
└──────────────────────────────────┘
```

### 6.3 Permission Request Flow

```
ToolUse message received
    │
    ▼
┌─────────────────────────┐
│ Permission callback     │
│ (from AgentOptions)     │
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ Show permission modal   │
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ Await user input        │
│ (y/n/always/never)      │
└─────────────────────────┘
    │
    ▼
┌─────────────────────────┐
│ Return PermissionResult │
└─────────────────────────┘
    │
    ▼
Application layer continues
```

---

## 7. View Layer Architecture

### 7.1 Panel System

```rust
// src/adapters/primary/tui/ui/panels/traits.rs

use ratatui::layout::Rect;
use ratatui::Frame;

/// Trait that all panels must implement
pub trait Panel: Send + Sync {
    /// Unique panel identifier
    fn id(&self) -> PanelId;

    /// Human-readable panel name
    fn name(&self) -> &str;

    /// Render the panel
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState);

    /// Handle panel-specific input
    fn handle_input(&mut self, key: KeyEvent, state: &AppState) -> Result<InputResult>;

    /// Update panel state (called on tick)
    fn update(&mut self, state: &AppState) -> Result<()>;

    /// Can this panel receive focus?
    fn can_focus(&self) -> bool {
        true
    }

    /// Minimum size requirements
    fn min_size(&self) -> (u16, u16) {
        (20, 5)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    Chat,
    Workflow,
    Tasks,
    Thinking,
    Logs,
    Help,
    StatusBar,
}

pub enum InputResult {
    Handled,
    NotHandled,
    SwitchPanel(PanelId),
    ExecuteCommand(String),
}
```

### 7.2 Panel Implementations

#### Chat Panel

```rust
// src/adapters/primary/tui/ui/panels/chat.rs

pub struct ChatPanel {
    id: PanelId,
    scroll_offset: usize,
    auto_scroll: bool,
}

impl Panel for ChatPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Get messages from history
        let history = state.history.blocking_read();
        let active_session = state.session.blocking_read().active_session;

        if let Some(session_id) = active_session {
            if let Some(messages) = history.messages.get(&session_id) {
                // Render messages
                let message_widgets = messages
                    .messages
                    .iter()
                    .skip(self.scroll_offset)
                    .map(|msg| self.render_message(msg))
                    .collect::<Vec<_>>();

                let list = List::new(message_widgets)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title("Chat"));

                frame.render_widget(list, area);
            }
        }
    }

    fn handle_input(&mut self, key: KeyEvent, state: &AppState) -> Result<InputResult> {
        match key.code {
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                Ok(InputResult::Handled)
            }
            KeyCode::Down => {
                self.scroll_offset += 1;
                Ok(InputResult::Handled)
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                Ok(InputResult::Handled)
            }
            KeyCode::PageDown => {
                self.scroll_offset += 10;
                Ok(InputResult::Handled)
            }
            _ => Ok(InputResult::NotHandled)
        }
    }
}
```

#### Workflow Panel

```rust
// src/adapters/primary/tui/ui/panels/workflow.rs

pub struct WorkflowPanel {
    id: PanelId,
    selected_task: Option<String>,
}

impl Panel for WorkflowPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        let session = state.session.blocking_read();

        // Get active workflow
        if let Some((workflow_id, context)) = session.workflows.iter().next() {
            // Render task tree
            let tree = self.build_task_tree(&context.executor);

            let tree_widget = Tree::new(tree)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Workflow: {}", context.workflow.name)))
                .highlight_style(Style::default().fg(Color::Yellow));

            frame.render_widget(tree_widget, area);

            // Render progress bar
            let progress_area = Rect {
                y: area.y + area.height - 3,
                height: 3,
                ..area
            };

            let progress = context.progress.completed_tasks as f64
                / context.progress.total_tasks as f64;

            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green))
                .percent((progress * 100.0) as u16)
                .label(format!(
                    "{}/{} tasks",
                    context.progress.completed_tasks,
                    context.progress.total_tasks
                ));

            frame.render_widget(gauge, progress_area);
        }
    }
}
```

### 7.3 Layout System

```rust
// src/adapters/primary/tui/ui/layout/manager.rs

pub struct LayoutManager {
    current_layout: LayoutPreset,
}

impl LayoutManager {
    pub fn compute_layout(&self, terminal_size: Rect) -> HashMap<PanelId, Rect> {
        match self.current_layout {
            LayoutPreset::Default => self.layout_default(terminal_size),
            LayoutPreset::Workflow => self.layout_workflow(terminal_size),
            LayoutPreset::FullWorkflow => self.layout_full_workflow(terminal_size),
            LayoutPreset::Minimal => self.layout_minimal(terminal_size),
            LayoutPreset::Custom(ref layout) => self.layout_custom(terminal_size, layout),
        }
    }

    fn layout_workflow(&self, area: Rect) -> HashMap<PanelId, Rect> {
        use ratatui::layout::{Constraint, Direction, Layout};

        let mut map = HashMap::new();

        // Main vertical split
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),      // Main content
                Constraint::Length(3),    // Status bar
            ])
            .split(area);

        // Horizontal split for main content
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),  // Chat
                Constraint::Percentage(50),  // Workflow + Tasks
            ])
            .split(chunks[0]);

        // Vertical split for right side
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70),  // Workflow
                Constraint::Percentage(30),  // Tasks
            ])
            .split(main_chunks[1]);

        map.insert(PanelId::Chat, main_chunks[0]);
        map.insert(PanelId::Workflow, right_chunks[0]);
        map.insert(PanelId::Tasks, right_chunks[1]);
        map.insert(PanelId::StatusBar, chunks[1]);

        map
    }
}
```

---

## 8. Extension Points

### 8.1 Extension System Architecture

```
┌──────────────────────────────────────────────────────┐
│                  Extension API                        │
└──────────────────────────────────────────────────────┘
              │               │               │
              ▼               ▼               ▼
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │    Panel     │  │   Command    │  │    Theme     │
    │  Extension   │  │  Extension   │  │  Extension   │
    └──────────────┘  └──────────────┘  └──────────────┘
```

### 8.2 Panel Extension Trait

```rust
// src/adapters/primary/tui/extensions/traits.rs

/// Extension trait for custom panels
pub trait PanelExtension: Panel {
    /// Extension metadata
    fn metadata(&self) -> ExtensionMetadata;

    /// Initialize extension with application state
    fn initialize(&mut self, state: &AppState) -> Result<()>;

    /// Cleanup on unload
    fn cleanup(&mut self) -> Result<()>;

    /// Extension configuration
    fn configure(&mut self, config: serde_json::Value) -> Result<()>;
}

pub struct ExtensionMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub dependencies: Vec<String>,
}
```

### 8.3 Command Extension

```rust
// src/adapters/primary/tui/extensions/traits.rs

/// Extension trait for custom commands
pub trait CommandExtension: Send + Sync {
    /// Command name (e.g., "export", "analyze")
    fn name(&self) -> &str;

    /// Command description for help
    fn description(&self) -> &str;

    /// Command usage string
    fn usage(&self) -> &str;

    /// Execute the command
    fn execute(
        &self,
        args: Vec<String>,
        state: &AppState,
    ) -> Pin<Box<dyn Future<Output = Result<CommandResult>> + Send>>;

    /// Autocomplete suggestions
    fn autocomplete(&self, partial: &str) -> Vec<String> {
        vec![]
    }
}

pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

### 8.4 Extension Registry

```rust
// src/adapters/primary/tui/extensions/registry.rs

pub struct ExtensionRegistry {
    panels: HashMap<String, Box<dyn PanelExtension>>,
    commands: HashMap<String, Box<dyn CommandExtension>>,
    themes: HashMap<String, Theme>,
}

impl ExtensionRegistry {
    pub fn register_panel(&mut self, panel: Box<dyn PanelExtension>) -> Result<()> {
        let name = panel.metadata().name.clone();
        self.panels.insert(name, panel);
        Ok(())
    }

    pub fn register_command(&mut self, command: Box<dyn CommandExtension>) -> Result<()> {
        let name = command.name().to_string();
        self.commands.insert(name, command);
        Ok(())
    }

    pub fn get_panel(&self, name: &str) -> Option<&dyn PanelExtension> {
        self.panels.get(name).map(|p| p.as_ref())
    }

    pub fn get_command(&self, name: &str) -> Option<&dyn CommandExtension> {
        self.commands.get(name).map(|c| c.as_ref())
    }
}
```

### 8.5 Extension Loading

```rust
// src/adapters/primary/tui/extensions/loader.rs

/// Load extensions from directory
pub async fn load_extensions(
    path: &Path,
    registry: &mut ExtensionRegistry,
) -> Result<usize> {
    let mut loaded = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
            // Load WASM extension
            load_wasm_extension(&path, registry).await?;
            loaded += 1;
        } else if path.extension().and_then(|s| s.to_str()) == Some("so") {
            // Load native extension (unsafe)
            load_native_extension(&path, registry)?;
            loaded += 1;
        }
    }

    Ok(loaded)
}
```

---

## 9. Implementation Specifications

### 9.1 Public API

```rust
// src/adapters/primary/tui/lib.rs

pub use runtime::TuiApp;
pub use state::{AppState, SessionState, ViewState};
pub use ui::panels::{Panel, PanelId};
pub use input::commands::{Command, CommandRegistry};
pub use extensions::{PanelExtension, CommandExtension, ExtensionRegistry};

/// Entry point for TUI REPL
pub async fn run_tui(options: TuiOptions) -> Result<()> {
    let app = TuiApp::new(options)?;
    app.run().await
}

pub struct TuiOptions {
    /// Agent options for PeriplonSDKClient
    pub agent_options: AgentOptions,

    /// Initial layout
    pub layout: LayoutPreset,

    /// Theme
    pub theme: String,

    /// Extension directory
    pub extensions_dir: Option<PathBuf>,

    /// Configuration file
    pub config_file: Option<PathBuf>,
}
```

### 9.2 Main Application Structure

```rust
// src/adapters/primary/tui/runtime/app.rs

pub struct TuiApp {
    /// Unified state
    state: Arc<AppState>,

    /// Extension registry
    extensions: ExtensionRegistry,

    /// Panel registry
    panels: PanelRegistry,

    /// Command registry
    commands: CommandRegistry,

    /// Event channel
    event_tx: mpsc::UnboundedSender<TuiEvent>,
    event_rx: mpsc::UnboundedReceiver<TuiEvent>,

    /// Options
    options: TuiOptions,
}

impl TuiApp {
    pub fn new(options: TuiOptions) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let mut app = Self {
            state: Arc::new(AppState::new()),
            extensions: ExtensionRegistry::new(),
            panels: PanelRegistry::new(),
            commands: CommandRegistry::new(),
            event_tx,
            event_rx,
            options,
        };

        // Register built-in panels
        app.register_builtin_panels()?;

        // Register built-in commands
        app.register_builtin_commands()?;

        // Load extensions
        if let Some(ext_dir) = &app.options.extensions_dir {
            load_extensions(ext_dir, &mut app.extensions).await?;
        }

        Ok(app)
    }

    pub async fn run(mut self) -> Result<()> {
        // Setup terminal
        let mut terminal = setup_terminal()?;

        // Create event loop
        let event_loop = EventLoop::new(
            self.event_rx,
            self.state.clone(),
            terminal,
        );

        // Run event loop
        event_loop.run().await?;

        // Cleanup
        cleanup_terminal()?;

        Ok(())
    }
}
```

### 9.3 Configuration System

```rust
// src/adapters/primary/tui/state/config.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// Default layout
    pub default_layout: LayoutPreset,

    /// Theme name
    pub theme: String,

    /// Keybindings
    pub keybindings: HashMap<String, KeyBinding>,

    /// History settings
    pub history: HistoryConfig,

    /// UI preferences
    pub ui: UiConfig,

    /// Extension settings
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    pub max_messages: usize,
    pub max_commands: usize,
    pub persist: bool,
    pub persist_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub show_timestamps: bool,
    pub show_thinking: bool,
    pub show_tool_details: bool,
    pub auto_scroll: bool,
    pub tick_rate_ms: u64,
}

impl TuiConfig {
    pub fn load() -> Self {
        // Load from config file or use defaults
        Self::from_file("~/.config/tui-repl/config.toml")
            .unwrap_or_default()
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let path = shellexpand::tilde(path);
        let contents = fs::read_to_string(path.as_ref())?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = "~/.config/tui-repl/config.toml";
        let path = shellexpand::tilde(path);

        // Create directory if needed
        if let Some(parent) = Path::new(path.as_ref()).parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(path.as_ref(), contents)?;

        Ok(())
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            default_layout: LayoutPreset::Default,
            theme: "default".to_string(),
            keybindings: default_keybindings(),
            history: HistoryConfig {
                max_messages: 1000,
                max_commands: 100,
                persist: true,
                persist_path: None,
            },
            ui: UiConfig {
                show_timestamps: true,
                show_thinking: true,
                show_tool_details: true,
                auto_scroll: true,
                tick_rate_ms: 250,
            },
            extensions: HashMap::new(),
        }
    }
}
```

---

## 10. Testing Strategy

### 10.1 Unit Testing

**State Management Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_state_management() {
        let state = AppState::new();

        // Create session
        let session_id = SessionId::new();
        let client = PeriplonSDKClient::new(AgentOptions::default());

        // Add to state
        {
            let mut session = state.session.write().await;
            session.clients.insert(session_id, ClientContext {
                id: session_id,
                client,
                status: SessionStatus::Active,
                created_at: Instant::now(),
                last_activity: Instant::now(),
            });
        }

        // Verify
        let session = state.session.read().await;
        assert!(session.clients.contains_key(&session_id));
    }

    #[tokio::test]
    async fn test_message_history_bounded() {
        let mut history = MessageHistory::new();
        history.max_size = 10;

        // Add 20 messages
        for i in 0..20 {
            history.push(create_test_message(i));
        }

        // Should only retain 10
        assert_eq!(history.messages.len(), 10);

        // Should be most recent 10
        assert_eq!(history.messages[0].id.0, 10);
    }
}
```

**Panel Tests:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_chat_panel_scroll() {
        let mut panel = ChatPanel::new();
        panel.scroll_offset = 5;

        // Scroll up
        let result = panel.handle_input(
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            &mock_state(),
        ).unwrap();

        assert_eq!(panel.scroll_offset, 4);
        assert_eq!(result, InputResult::Handled);
    }
}
```

### 10.2 Integration Testing

**Client Integration:**
```rust
#[tokio::test]
async fn test_query_flow() {
    let app = TuiApp::new(TuiOptions::default()).unwrap();

    // Create session
    let session_id = app.create_session().await.unwrap();

    // Send query
    app.send_query(session_id, "What is 2+2?").await.unwrap();

    // Wait for response
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify message in history
    let history = app.state.history.read().await;
    let messages = &history.messages[&session_id];

    assert!(messages.messages.len() > 0);
}
```

### 10.3 UI Testing

**Snapshot Testing:**
```rust
#[test]
fn test_chat_panel_rendering() {
    let mut terminal = setup_test_terminal();
    let panel = ChatPanel::new();
    let state = create_test_state_with_messages();

    terminal.draw(|frame| {
        panel.render(frame, frame.size(), &state);
    }).unwrap();

    // Compare with snapshot
    let buffer = terminal.backend().buffer().clone();
    insta::assert_snapshot!(buffer_to_string(&buffer));
}
```

---

## Conclusion

This comprehensive architecture design provides:

1. **Clear component hierarchy** following hexagonal architecture principles
2. **Robust state management** with separated concerns (Session, View, History)
3. **Flexible event handling** using tokio::select! and channels
4. **Extensible panel system** with trait-based design
5. **Plugin architecture** for custom panels, commands, and themes
6. **Well-defined data flows** for queries, workflows, and permissions
7. **Testable design** with mockable components and clear boundaries

**Key Strengths:**
- ✅ Respects hexagonal boundaries (TUI as Primary Adapter)
- ✅ Async-first with non-blocking message streams
- ✅ Memory-bounded history for long-running sessions
- ✅ Extensible through trait-based plugin system
- ✅ Configurable layouts, themes, and keybindings
- ✅ Comprehensive testing strategy

**Next Steps:**
1. Implement core runtime (event loop, state manager)
2. Build basic panels (Chat, Status)
3. Integrate PeriplonSDKClient
4. Add REPL command system
5. Implement workflow visualization
6. Add extension system

---

**End of Architecture Design**

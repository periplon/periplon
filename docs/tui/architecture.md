# DSL TUI Architecture

Comprehensive architecture documentation for the DSL TUI (Text User Interface) system.

## Table of Contents

1. [Overview](#overview)
2. [Architectural Principles](#architectural-principles)
3. [System Architecture](#system-architecture)
4. [Component Breakdown](#component-breakdown)
5. [Data Flow](#data-flow)
6. [Extension Points](#extension-points)
7. [Testing Strategy](#testing-strategy)

## Overview

The DSL TUI is an interactive terminal application built using the **Hexagonal Architecture** (Ports & Adapters) pattern. It provides a rich user interface for creating, managing, and executing AI workflow definitions.

### Key Features

- **Interactive Workflow Management**: Browse, create, edit, and execute workflows
- **Real-time Execution Monitoring**: Live progress tracking with detailed logs
- **State Persistence**: Save and resume workflow executions
- **AI-Powered Generation**: Create workflows from natural language descriptions
- **Syntax Highlighting**: YAML editor with validation and auto-completion
- **Hexagonal Design**: Clean separation of concerns with testable components

### Technology Stack

- **Language**: Rust 2021 edition
- **UI Framework**: Ratatui (terminal UI)
- **Event Handling**: Crossterm (cross-platform terminal)
- **Async Runtime**: Tokio
- **Parsing**: serde_yaml, serde_json
- **Styling**: Custom theme system

## Architectural Principles

### 1. Hexagonal Architecture (Ports & Adapters)

The TUI follows strict hexagonal architecture:

```
┌─────────────────────────────────────────────────────────┐
│                    Primary Adapters                      │
│                  (User Interaction)                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │   CLI    │  │   TUI    │  │  Config  │             │
│  │  Args    │  │  Events  │  │  Loader  │             │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘             │
│       │             │             │                     │
├───────┼─────────────┼─────────────┼─────────────────────┤
│       │             │             │                     │
│       ▼             ▼             ▼                     │
│  ┌────────────────────────────────────────┐            │
│  │          Primary Ports                  │            │
│  │  (Application Services Interface)       │            │
│  └────────────────┬───────────────────────┘            │
│                   │                                     │
│       ┌───────────┴──────────┐                         │
│       ▼                      ▼                          │
│  ┌──────────┐          ┌──────────┐                    │
│  │  Domain  │          │ Business │                    │
│  │  Models  │◄────────►│  Logic   │                    │
│  └──────────┘          └──────────┘                    │
│       │                      │                          │
│       └──────────┬───────────┘                         │
│                  │                                      │
│  ┌───────────────┴────────────────────┐               │
│  │        Secondary Ports              │               │
│  │  (Infrastructure Interface)         │               │
│  └────────┬────────────────────────────┘               │
│           │                                             │
├───────────┼─────────────────────────────────────────────┤
│           │                                             │
│           ▼                                             │
│  ┌─────────────────────────────────────────┐          │
│  │       Secondary Adapters                 │          │
│  │    (External Systems)                    │          │
│  │  ┌──────┐  ┌──────┐  ┌──────┐          │          │
│  │  │ File │  │ DSL  │  │ MCP  │          │          │
│  │  │System│  │Engine│  │Server│          │          │
│  │  └──────┘  └──────┘  └──────┘          │          │
│  └─────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────┘
```

### 2. Separation of Concerns

Each layer has a single responsibility:

- **Domain Layer**: Pure business logic (workflows, tasks, agents)
- **Application Layer**: Use case orchestration
- **Adapter Layer**: External system integration
- **Presentation Layer**: UI rendering and event handling

### 3. Dependency Inversion

All dependencies point inward toward the domain:

```
Presentation → Application → Domain
     ↓              ↓
Infrastructure ←────┘
```

### 4. Testability

Each layer is independently testable:

- **Domain**: Pure unit tests
- **Application**: Service tests with mock ports
- **Adapters**: Integration tests
- **Presentation**: UI snapshot tests

## System Architecture

### Directory Structure

```
src/tui/
├── mod.rs                    # TUI module root, public API
├── app.rs                    # Main application state and orchestration
├── config.rs                 # Configuration loading and validation
│
├── domain/                   # Pure business logic (no dependencies)
│   ├── mod.rs
│   ├── models.rs            # Core data structures
│   ├── events.rs            # Domain events
│   └── validation.rs        # Business rules and validation
│
├── ports/                    # Interface definitions
│   ├── mod.rs
│   ├── primary/             # Inbound ports (driven by external actors)
│   │   ├── mod.rs
│   │   ├── app_service.rs   # Main application service interface
│   │   └── event_handler.rs # Event handling interface
│   │
│   └── secondary/           # Outbound ports (drive external systems)
│       ├── mod.rs
│       ├── workflow_repo.rs # Workflow persistence interface
│       ├── executor.rs      # Workflow execution interface
│       ├── generator.rs     # AI generation interface
│       └── notifier.rs      # Notification interface
│
├── application/             # Use case implementations
│   ├── mod.rs
│   ├── services/           # Application services
│   │   ├── mod.rs
│   │   ├── workflow_manager.rs
│   │   ├── execution_manager.rs
│   │   ├── state_manager.rs
│   │   └── generator_service.rs
│   │
│   └── handlers/           # Command/query handlers
│       ├── mod.rs
│       ├── workflow_handlers.rs
│       ├── execution_handlers.rs
│       └── state_handlers.rs
│
├── adapters/               # External system implementations
│   ├── mod.rs
│   ├── primary/           # Driving adapters
│   │   ├── mod.rs
│   │   ├── cli.rs         # CLI argument parsing
│   │   └── tui_events.rs  # Terminal event adapter
│   │
│   └── secondary/         # Driven adapters
│       ├── mod.rs
│       ├── file_workflow_repo.rs  # File-based workflow storage
│       ├── dsl_executor.rs        # DSL execution engine adapter
│       ├── ai_generator.rs        # AI generation adapter
│       └── notification_adapter.rs # Notification delivery
│
├── ui/                     # Presentation layer
│   ├── mod.rs
│   ├── renderer.rs        # Main rendering coordinator
│   ├── theme.rs          # Color schemes and styling
│   ├── components/       # Reusable UI components
│   │   ├── mod.rs
│   │   ├── list.rs
│   │   ├── editor.rs
│   │   ├── status_bar.rs
│   │   ├── dialog.rs
│   │   └── progress.rs
│   │
│   └── views/            # Full-screen views
│       ├── mod.rs
│       ├── workflow_browser.rs
│       ├── workflow_editor.rs
│       ├── execution_monitor.rs
│       ├── state_browser.rs
│       ├── help_viewer.rs
│       └── viewer.rs     # Workflow details viewer
│
├── state/                # Application state management
│   ├── mod.rs
│   ├── app_state.rs     # Global application state
│   ├── view_state.rs    # Per-view state
│   └── transitions.rs   # State transition logic
│
└── help/                # Documentation and help content
    ├── mod.rs
    ├── content.rs       # Help text content
    ├── search.rs        # Help search functionality
    └── docs/            # Embedded documentation
        ├── user_guide.md
        ├── shortcuts.md
        └── troubleshooting.md
```

### Module Responsibilities

#### `mod.rs` - Public API
- Exports public types and functions
- Provides main entry point: `run_tui()`
- Feature flag coordination

#### `app.rs` - Application Orchestration
- Main application loop
- Event routing to handlers
- View lifecycle management
- State coordination

#### `config.rs` - Configuration
- CLI argument parsing
- Config file loading
- Environment variable handling
- Configuration validation

## Component Breakdown

### Domain Layer

#### Models (`domain/models.rs`)

Core data structures with no external dependencies:

```rust
/// Represents a workflow file in the system
pub struct WorkflowFile {
    pub path: PathBuf,
    pub name: String,
    pub content: Option<Workflow>,
    pub metadata: FileMetadata,
}

/// Execution state information
pub struct ExecutionState {
    pub id: ExecutionId,
    pub workflow: WorkflowId,
    pub status: ExecutionStatus,
    pub progress: ExecutionProgress,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Saved workflow state for resumption
pub struct SavedState {
    pub id: StateId,
    pub workflow: WorkflowId,
    pub status: StateStatus,
    pub completed_tasks: Vec<TaskId>,
    pub variables: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
```

#### Events (`domain/events.rs`)

Domain events representing business-significant occurrences:

```rust
pub enum DomainEvent {
    // Workflow events
    WorkflowCreated { id: WorkflowId, name: String },
    WorkflowUpdated { id: WorkflowId },
    WorkflowDeleted { id: WorkflowId },

    // Execution events
    ExecutionStarted { id: ExecutionId, workflow: WorkflowId },
    ExecutionPaused { id: ExecutionId },
    ExecutionResumed { id: ExecutionId },
    ExecutionCompleted { id: ExecutionId, success: bool },

    // State events
    StateCreated { id: StateId, workflow: WorkflowId },
    StateRestored { id: StateId },
    StateDeleted { id: StateId },
}
```

#### Validation (`domain/validation.rs`)

Business rules and validation logic:

```rust
pub trait Validator<T> {
    fn validate(&self, value: &T) -> Result<(), ValidationError>;
}

pub struct WorkflowValidator;
impl Validator<Workflow> for WorkflowValidator {
    fn validate(&self, workflow: &Workflow) -> Result<(), ValidationError> {
        // Validate workflow structure, dependencies, etc.
    }
}
```

### Ports Layer

#### Primary Ports (`ports/primary/`)

Inbound interfaces driven by external actors:

```rust
#[async_trait]
pub trait AppService {
    async fn load_workflow(&mut self, path: &Path) -> Result<WorkflowId>;
    async fn save_workflow(&mut self, id: WorkflowId) -> Result<()>;
    async fn execute_workflow(&mut self, id: WorkflowId) -> Result<ExecutionId>;
    async fn pause_execution(&mut self, id: ExecutionId) -> Result<()>;
}

pub trait EventHandler {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<EventResult>;
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<EventResult>;
}
```

#### Secondary Ports (`ports/secondary/`)

Outbound interfaces for external systems:

```rust
#[async_trait]
pub trait WorkflowRepository {
    async fn find_all(&self, dir: &Path) -> Result<Vec<WorkflowFile>>;
    async fn find_by_id(&self, id: WorkflowId) -> Result<Option<WorkflowFile>>;
    async fn save(&self, workflow: &WorkflowFile) -> Result<()>;
    async fn delete(&self, id: WorkflowId) -> Result<()>;
}

#[async_trait]
pub trait WorkflowExecutor {
    async fn execute(&self, workflow: &Workflow) -> Result<ExecutionHandle>;
    async fn pause(&self, handle: ExecutionHandle) -> Result<()>;
    async fn resume(&self, handle: ExecutionHandle) -> Result<()>;
    async fn stop(&self, handle: ExecutionHandle) -> Result<()>;
}

#[async_trait]
pub trait WorkflowGenerator {
    async fn generate(&self, description: &str) -> Result<Workflow>;
    async fn generate_template(&self) -> Result<Workflow>;
}
```

### Application Layer

#### Services (`application/services/`)

Orchestration of business logic:

```rust
pub struct WorkflowManager {
    repository: Arc<dyn WorkflowRepository>,
    validator: Arc<WorkflowValidator>,
}

impl WorkflowManager {
    pub async fn create_workflow(
        &self,
        name: String,
        template: Option<WorkflowId>,
    ) -> Result<WorkflowId> {
        // 1. Load template if provided
        // 2. Create new workflow
        // 3. Validate
        // 4. Save to repository
        // 5. Emit WorkflowCreated event
    }

    pub async fn update_workflow(
        &self,
        id: WorkflowId,
        content: String,
    ) -> Result<()> {
        // 1. Parse content
        // 2. Validate
        // 3. Update repository
        // 4. Emit WorkflowUpdated event
    }
}
```

#### Handlers (`application/handlers/`)

Command and query handlers:

```rust
pub struct ExecutionHandlers {
    executor: Arc<dyn WorkflowExecutor>,
    state_repo: Arc<dyn StateRepository>,
}

impl ExecutionHandlers {
    pub async fn handle_start_execution(
        &self,
        cmd: StartExecutionCommand,
    ) -> Result<ExecutionId> {
        // 1. Load workflow
        // 2. Validate inputs
        // 3. Start execution
        // 4. Create initial state
        // 5. Return execution ID
    }
}
```

### Adapters Layer

#### Primary Adapters (`adapters/primary/`)

Adapt external input to application interface:

```rust
pub struct TUIEventAdapter {
    app_service: Arc<dyn AppService>,
}

impl TUIEventAdapter {
    pub fn handle_crossterm_event(&mut self, event: CrosstermEvent) -> Result<()> {
        match event {
            CrosstermEvent::Key(key) => {
                self.app_service.handle_key_event(key)?;
            }
            CrosstermEvent::Mouse(mouse) => {
                self.app_service.handle_mouse_event(mouse)?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

#### Secondary Adapters (`adapters/secondary/`)

Implement connections to external systems:

```rust
pub struct FileWorkflowRepository {
    base_dir: PathBuf,
}

#[async_trait]
impl WorkflowRepository for FileWorkflowRepository {
    async fn find_all(&self, dir: &Path) -> Result<Vec<WorkflowFile>> {
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry?;
            if entry.path().extension() == Some("yaml".as_ref()) {
                files.push(self.load_workflow(entry.path()).await?);
            }
        }
        Ok(files)
    }

    async fn save(&self, workflow: &WorkflowFile) -> Result<()> {
        let yaml = serde_yaml::to_string(&workflow.content)?;
        tokio::fs::write(&workflow.path, yaml).await?;
        Ok(())
    }
}
```

### Presentation Layer

#### UI Components (`ui/components/`)

Reusable UI widgets:

```rust
pub struct List<'a, T> {
    items: &'a [T],
    selected: Option<usize>,
    title: &'a str,
    style: Style,
}

impl<'a, T: Display> Widget for List<'a, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render list with highlighting, scrolling, etc.
    }
}

pub struct Editor<'a> {
    content: &'a str,
    cursor: (usize, usize),
    scroll: usize,
    language: Option<&'a str>,
}

impl<'a> Widget for Editor<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render editor with syntax highlighting
    }
}
```

#### Views (`ui/views/`)

Full-screen views implementing specific use cases:

```rust
pub struct WorkflowBrowser {
    file_manager: FileManager,
    selected_index: usize,
    scroll_offset: usize,
}

impl WorkflowBrowser {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // 1. Render file list
        // 2. Render preview pane
        // 3. Render status bar
    }

    pub fn handle_event(&mut self, event: &Event) -> Result<EventResult> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => self.move_selection_up(),
                KeyCode::Down => self.move_selection_down(),
                KeyCode::Enter => self.open_selected(),
                // ...
            },
            _ => Ok(EventResult::Ignored),
        }
    }
}
```

### State Management

#### Application State (`state/app_state.rs`)

Global application state:

```rust
pub struct AppState {
    pub current_view: View,
    pub workflows: HashMap<WorkflowId, WorkflowFile>,
    pub executions: HashMap<ExecutionId, ExecutionState>,
    pub states: HashMap<StateId, SavedState>,
    pub config: Config,
}

impl AppState {
    pub fn transition_to(&mut self, view: View) -> Result<()> {
        // Validate transition
        // Save current view state
        // Switch to new view
        // Load new view state
    }
}
```

#### View State (`state/view_state.rs`)

Per-view state persistence:

```rust
pub enum ViewState {
    WorkflowBrowser(WorkflowBrowserState),
    WorkflowEditor(EditorState),
    ExecutionMonitor(ExecutionMonitorState),
    StateBrowser(StateBrowserState),
    Help(HelpState),
}

pub struct WorkflowBrowserState {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub current_dir: PathBuf,
    pub filter: Option<String>,
}
```

## Data Flow

### User Interaction Flow

```
User Input → Terminal → Crossterm Event
                ↓
        TUIEventAdapter (Primary Adapter)
                ↓
        EventHandler (Primary Port)
                ↓
        ApplicationService (Application Layer)
                ↓
        DomainService (Domain Layer)
                ↓
        Repository/Executor (Secondary Port)
                ↓
        FileSystem/DSL Engine (Secondary Adapter)
                ↓
        Domain Event Emission
                ↓
        View Update
                ↓
        Terminal Rendering
```

### Workflow Execution Flow

```
1. User selects workflow in browser
2. Browser emits ExecuteWorkflow command
3. ApplicationService receives command
4. WorkflowManager loads workflow from repository
5. WorkflowValidator validates workflow
6. ExecutionHandler starts execution
7. DSLExecutor adapter runs workflow
8. ExecutionMonitor receives progress events
9. StateManager persists execution state
10. View updates with progress
11. User sees real-time feedback
```

### State Persistence Flow

```
1. Execution reaches checkpoint (task completion)
2. Executor emits StateCheckpoint event
3. StateManager handles event
4. StateRepository saves state to filesystem
5. StateBrowser refreshes state list
6. User can resume from any checkpoint
```

## Extension Points

### Adding New Views

1. Create view struct in `ui/views/`
2. Implement `View` trait with render and handle_event methods
3. Add view to `View` enum in `state/app_state.rs`
4. Add navigation in `app.rs`

Example:

```rust
// ui/views/my_view.rs
pub struct MyView {
    // View state
}

impl MyView {
    pub fn new() -> Self {
        Self { /* ... */ }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Render logic
    }

    pub fn handle_event(&mut self, event: &Event) -> Result<EventResult> {
        // Event handling
    }
}

// state/app_state.rs
pub enum View {
    // ...existing views...
    MyView,
}

// app.rs
impl App {
    fn render_current_view(&mut self, frame: &mut Frame) {
        match self.state.current_view {
            // ...existing views...
            View::MyView => self.my_view.render(frame, frame.size()),
        }
    }
}
```

### Adding New Workflows Operations

1. Define port interface in `ports/secondary/`
2. Implement adapter in `adapters/secondary/`
3. Add service method in `application/services/`
4. Wire up in `app.rs`

### Adding New Themes

1. Add theme definition in `ui/theme.rs`
2. Define color palette
3. Update theme loader in `config.rs`

```rust
// ui/theme.rs
pub struct Theme {
    pub name: String,
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    // ...
}

impl Theme {
    pub fn monokai() -> Self {
        Self {
            name: "monokai".into(),
            primary: Color::Rgb(249, 38, 114),
            secondary: Color::Rgb(102, 217, 239),
            success: Color::Rgb(166, 226, 46),
            error: Color::Rgb(249, 38, 114),
            warning: Color::Rgb(253, 151, 31),
        }
    }
}
```

## Testing Strategy

### Unit Tests

Test domain logic in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_validation() {
        let workflow = Workflow {
            name: "test".into(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
        };

        let validator = WorkflowValidator;
        assert!(validator.validate(&workflow).is_ok());
    }
}
```

### Integration Tests

Test adapter implementations:

```rust
#[tokio::test]
async fn test_file_repository() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = FileWorkflowRepository::new(temp_dir.path());

    let workflow = create_test_workflow();
    repo.save(&workflow).await.unwrap();

    let loaded = repo.find_by_id(workflow.id).await.unwrap();
    assert_eq!(loaded, Some(workflow));
}
```

### UI Tests

Test view rendering and interactions:

```rust
#[test]
fn test_workflow_browser_navigation() {
    let mut browser = WorkflowBrowser::new();
    browser.add_workflow(create_test_workflow());

    browser.handle_event(&Event::Key(KeyCode::Down)).unwrap();
    assert_eq!(browser.selected_index(), 1);
}
```

### End-to-End Tests

Test complete user workflows:

```rust
#[tokio::test]
async fn test_create_and_execute_workflow() {
    let mut app = App::new(Config::default()).await.unwrap();

    // Create workflow
    app.create_workflow("test").await.unwrap();

    // Execute workflow
    let execution_id = app.execute_workflow("test").await.unwrap();

    // Wait for completion
    app.wait_for_execution(execution_id).await.unwrap();

    // Verify results
    let state = app.get_execution_state(execution_id).await.unwrap();
    assert_eq!(state.status, ExecutionStatus::Completed);
}
```

## Performance Considerations

### Rendering Optimization

- **Incremental Rendering**: Only redraw changed regions
- **Layout Caching**: Cache layout calculations
- **Lazy Loading**: Load workflow content on demand
- **Virtual Scrolling**: Only render visible items

### Memory Management

- **Streaming Logs**: Don't keep entire log history in memory
- **Workflow Unloading**: Unload unused workflow content
- **State Cleanup**: Periodically clean up old states
- **Resource Limits**: Enforce maximum concurrent executions

### Async Operations

- **Non-blocking I/O**: All file operations are async
- **Background Tasks**: Long-running operations don't block UI
- **Cancellation**: Support cancelling async operations
- **Backpressure**: Handle slow consumers gracefully

## Security Considerations

- **Path Validation**: Prevent directory traversal attacks
- **Input Sanitization**: Validate all user input
- **Permission Checks**: Verify file permissions before operations
- **Resource Limits**: Prevent resource exhaustion
- **Secure Defaults**: Safe default configurations

## Future Enhancements

- **Plugin System**: Load custom views and components
- **Remote Execution**: Execute workflows on remote servers
- **Collaborative Editing**: Multi-user workflow editing
- **Version Control**: Git integration for workflows
- **Advanced Debugging**: Breakpoints and step-through execution
- **Performance Profiling**: Built-in profiling tools
- **Metrics Dashboard**: Real-time metrics and analytics

## References

- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Ratatui Documentation](https://ratatui.rs/)
- [Tokio Async Runtime](https://tokio.rs/)

---

**Version**: 1.0.0
**Last Updated**: 2025-10-21

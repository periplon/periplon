# Debugging Infrastructure Architecture

## Overview

This document outlines the architecture for adding comprehensive debugging capabilities to the Periplon DSL executor, including time-travel debugging, breakpoints, step execution, and side-effect tracking.

## Core Components

### 1. Execution Pointer

**Purpose**: Track current execution position in the workflow

```rust
pub struct ExecutionPointer {
    /// Current task being executed
    current_task: Option<String>,

    /// Current loop iteration (if inside a loop)
    loop_iteration: Option<LoopPosition>,

    /// Stack of subtask execution contexts
    execution_stack: Vec<ExecutionFrame>,

    /// Execution history for time-travel
    history: Vec<ExecutionSnapshot>,

    /// Current position in history (for back/forward navigation)
    history_index: usize,
}

pub struct LoopPosition {
    task_id: String,
    iteration: usize,
}

pub struct ExecutionFrame {
    task_id: String,
    parent_task: Option<String>,
    depth: usize,
}

pub struct ExecutionSnapshot {
    timestamp: Instant,
    pointer: ExecutionPointer,
    state_checkpoint: StateCheckpoint,
    side_effects: Vec<SideEffect>,
}
```

### 2. Breakpoint System

**Purpose**: Pause execution at specific points for inspection

```rust
pub struct BreakpointManager {
    /// Task breakpoints (break when task starts)
    task_breakpoints: HashSet<String>,

    /// Conditional breakpoints (break when condition is true)
    conditional_breakpoints: Vec<ConditionalBreakpoint>,

    /// Loop iteration breakpoints
    loop_breakpoints: HashMap<String, Vec<usize>>, // task_id -> iterations

    /// Variable watch breakpoints (break when variable changes)
    watch_breakpoints: HashMap<String, WatchCondition>,
}

pub struct ConditionalBreakpoint {
    id: String,
    condition: BreakCondition,
    enabled: bool,
}

pub enum BreakCondition {
    /// Break when task status matches
    TaskStatus { task_id: String, status: TaskStatus },

    /// Break when variable equals value
    VariableEquals { var_name: String, value: serde_json::Value },

    /// Break when expression evaluates to true
    Expression(String),

    /// Break on any error
    OnError,
}

pub struct WatchCondition {
    variable: String,
    old_value: Option<serde_json::Value>,
}
```

### 3. Side-Effect Journal

**Purpose**: Track all side effects for undo/redo capability

```rust
pub struct SideEffectJournal {
    /// All recorded side effects
    effects: Vec<SideEffect>,

    /// Compensation handlers for undo
    compensations: HashMap<usize, Box<dyn CompensationStrategy>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SideEffect {
    /// File system operations
    FileCreated { path: PathBuf, timestamp: SystemTime },
    FileModified {
        path: PathBuf,
        original_content: Vec<u8>,
        new_content: Vec<u8>,
        timestamp: SystemTime,
    },
    FileDeleted {
        path: PathBuf,
        original_content: Vec<u8>,
        timestamp: SystemTime,
    },

    /// State modifications
    StateChanged {
        field: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },

    /// Variable assignments
    VariableSet {
        scope: VariableScope,
        name: String,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
    },

    /// Task status changes
    TaskStatusChanged {
        task_id: String,
        old_status: TaskStatus,
        new_status: TaskStatus,
    },

    /// External command execution
    CommandExecuted {
        command: String,
        exit_code: i32,
        stdout: String,
        stderr: String,
    },

    /// Network operations (for future LLM API calls)
    NetworkRequest {
        url: String,
        method: String,
        response_status: u16,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableScope {
    Workflow,
    Agent(String),
    Task(String),
    Loop { task_id: String, iteration: usize },
}

pub trait CompensationStrategy: Send + Sync {
    fn compensate(&self) -> Result<()>;
    fn description(&self) -> String;
}
```

### 4. Debugger State Machine

**Purpose**: Control execution flow during debugging

```rust
pub struct DebuggerState {
    mode: DebugMode,
    execution_pointer: ExecutionPointer,
    breakpoints: BreakpointManager,
    side_effects: SideEffectJournal,
    step_mode: StepMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugMode {
    /// Normal execution
    Running,

    /// Paused at breakpoint
    Paused,

    /// Single-stepping
    Stepping,

    /// Stepping through time (navigating history)
    TimeTraveling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    /// Execute one task
    StepTask,

    /// Step into subtasks
    StepInto,

    /// Step over subtasks (complete them without pausing)
    StepOver,

    /// Step out of current subtask context
    StepOut,

    /// Step one loop iteration
    StepIteration,

    /// Continue until breakpoint or completion
    Continue,
}
```

### 5. Inspection API

**Purpose**: Inspect runtime state, variables, and execution context

```rust
pub struct Inspector {
    executor: Arc<Mutex<DSLExecutor>>,
    debugger: Arc<Mutex<DebuggerState>>,
}

impl Inspector {
    /// Get current execution position
    pub fn current_position(&self) -> ExecutionPosition {
        // Returns current task, loop iteration, call stack
    }

    /// Inspect all variables in scope
    pub fn inspect_variables(&self, scope: Option<VariableScope>) -> VariableSnapshot {
        // Returns all variables with their current values
    }

    /// Get task execution details
    pub fn inspect_task(&self, task_id: &str) -> TaskInspection {
        // Returns task status, inputs, outputs, duration, etc.
    }

    /// Get execution call stack
    pub fn call_stack(&self) -> Vec<ExecutionFrame> {
        // Returns current execution stack (for nested tasks)
    }

    /// Get side effect history
    pub fn side_effects(&self, filter: Option<SideEffectFilter>) -> Vec<SideEffect> {
        // Returns recorded side effects, optionally filtered
    }

    /// Get execution timeline
    pub fn timeline(&self) -> ExecutionTimeline {
        // Returns chronological execution events
    }
}

pub struct VariableSnapshot {
    pub workflow_vars: HashMap<String, serde_json::Value>,
    pub agent_vars: HashMap<String, HashMap<String, serde_json::Value>>,
    pub task_vars: HashMap<String, HashMap<String, serde_json::Value>>,
    pub loop_vars: HashMap<String, HashMap<String, serde_json::Value>>,
}

pub struct TaskInspection {
    pub task_id: String,
    pub status: TaskStatus,
    pub inputs: HashMap<String, serde_json::Value>,
    pub outputs: Option<TaskOutput>,
    pub duration: Option<Duration>,
    pub error: Option<String>,
    pub subtasks: Vec<String>,
    pub dependencies: Vec<String>,
}
```

### 6. REPL Command Structure

**Purpose**: Interactive debugging interface

```rust
pub enum ReplCommand {
    // Execution control
    Continue,
    Step(StepMode),
    StepBack,
    Restart,

    // Breakpoints
    Break { target: BreakTarget },
    DeleteBreak { id: String },
    ListBreaks,
    EnableBreak { id: String },
    DisableBreak { id: String },

    // Inspection
    Inspect(InspectTarget),
    Print { expression: String },
    Vars { scope: Option<VariableScope> },
    CallStack,
    Timeline,

    // Modification
    Set { var: String, value: serde_json::Value },
    EditSpec { modification: SpecModification },
    ChangeProvider { provider: String, model: Option<String> },

    // Navigation
    Goto { target: GotoTarget },
    Back { steps: usize },
    Forward { steps: usize },

    // AI assistance
    Generate { prompt: String },
    Suggest,

    // Utility
    Help { command: Option<String> },
    Quit,
}

pub enum BreakTarget {
    Task(String),
    Iteration { task: String, iteration: usize },
    Condition(BreakCondition),
    Watch(String),
}

pub enum InspectTarget {
    Task(String),
    Variable(String),
    State,
    SideEffects,
}

pub enum GotoTarget {
    Task(String),
    Iteration { task: String, iteration: usize },
    Snapshot(usize),
}

pub enum SpecModification {
    AddTask { id: String, spec: TaskSpec },
    ModifyTask { id: String, spec: TaskSpec },
    DeleteTask { id: String },
    AddAgent { id: String, spec: AgentSpec },
    ModifyAgent { id: String, spec: AgentSpec },
}
```

## Integration with Existing Executor

### Modified DSLExecutor

```rust
pub struct DSLExecutor {
    // Existing fields
    workflow: DSLWorkflow,
    agents: HashMap<String, PeriplonSDKClient>,
    task_graph: TaskGraph,
    message_bus: Arc<MessageBus>,
    state: Option<WorkflowState>,
    state_persistence: Option<StatePersistence>,
    resolved_inputs: HashMap<String, serde_json::Value>,
    notification_manager: Arc<NotificationManager>,
    workflow_start_time: Option<Instant>,
    json_output: bool,

    // NEW: Debugging infrastructure
    debugger: Option<Arc<Mutex<DebuggerState>>>,
    inspector: Option<Arc<Inspector>>,
}

impl DSLExecutor {
    /// Enable debugging mode
    pub fn with_debugger(mut self) -> Self {
        let debugger = Arc::new(Mutex::new(DebuggerState::new()));
        let inspector = Arc::new(Inspector::new(debugger.clone()));
        self.debugger = Some(debugger);
        self.inspector = Some(inspector);
        self
    }

    /// Execute with debugging support
    pub async fn execute_debug(&mut self) -> Result<()> {
        // Execution loop with breakpoint checking and step control
    }

    /// Record side effect
    fn record_side_effect(&mut self, effect: SideEffect) {
        if let Some(debugger) = &self.debugger {
            debugger.lock().await.side_effects.record(effect);
        }
    }

    /// Check for breakpoints
    async fn check_breakpoints(&self, task_id: &str) -> bool {
        if let Some(debugger) = &self.debugger {
            debugger.lock().await.breakpoints.should_break(task_id)
        } else {
            false
        }
    }
}
```

## TUI Architecture (Ratatui-based)

### Pane Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ Workflow Viewer (Tree)        │ YAML Editor                     │
│                                │                                 │
│ ▼ workflow_name                │ tasks:                          │
│   ├─ ▶ agents                  │   analyze:                      │
│   │   ├─ researcher            │     description: "Analyze..."   │
│   │   └─ coder                 │     agent: researcher           │
│   ├─ ▼ tasks [3/5]             │     depends_on: []              │
│   │   ├─ ✓ research            │                                 │
│   │   ├─ ► analyze (paused)    │                                 │
│   │   ├─ ○ code                │                                 │
│   │   ├─ ○ test                │                                 │
│   │   └─ ○ deploy              │                                 │
│   └─ ▶ variables               │                                 │
├────────────────────────────────┼─────────────────────────────────┤
│ Variables & State              │ Execution Timeline              │
│                                │                                 │
│ Workflow Variables:            │ 10:23:45  ✓ research completed  │
│   project_name: "myapp"        │ 10:24:12  ⏸ analyze paused      │
│                                │                                 │
│ Task Variables (analyze):      │ Side Effects:                   │
│   input_file: "data.json"      │   FileCreated: analysis.txt     │
│   threshold: 0.85              │   VarSet: result = {...}        │
├────────────────────────────────┴─────────────────────────────────┤
│ REPL> break task:code                                            │
│ Breakpoint #3 set on task 'code'                                 │
│ REPL> step                                                        │
└─────────────────────────────────────────────────────────────────┘
```

### Key Components

```rust
pub struct DebuggerTUI {
    /// Terminal backend
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,

    /// Application state
    app_state: Arc<Mutex<AppState>>,

    /// Active panes
    panes: PaneManager,

    /// Input handler
    input: InputHandler,

    /// Executor reference
    executor: Arc<Mutex<DSLExecutor>>,
}

pub struct AppState {
    /// Current focus
    focused_pane: PaneId,

    /// Workflow tree state
    tree_state: TreeState,

    /// YAML editor state
    editor_state: EditorState,

    /// REPL state
    repl_state: ReplState,

    /// Execution state
    execution_state: ExecutionState,
}

pub enum PaneId {
    WorkflowTree,
    YamlEditor,
    Variables,
    Timeline,
    Repl,
}

pub struct PaneManager {
    panes: HashMap<PaneId, Box<dyn Pane>>,
    layout: PaneLayout,
}

pub trait Pane {
    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool);
    fn handle_input(&mut self, event: KeyEvent) -> PaneAction;
    fn update(&mut self, state: &AppState);
}
```

### Keyboard Shortcuts

```
Global:
  Tab       - Switch pane focus
  Shift+Tab - Reverse switch pane
  Ctrl+Q    - Quit
  Ctrl+H    - Help overlay

Workflow Tree Pane:
  ↑/↓       - Navigate tree
  Enter     - Expand/collapse node
  Space     - Select node for editing
  E         - Edit selected node
  A         - Add child node
  D         - Delete node
  G         - Generate with AI

YAML Editor Pane:
  Standard vim-like editing
  Ctrl+S    - Save changes
  Ctrl+Z    - Undo
  Ctrl+Y    - Redo

Execution Control (anywhere):
  F5        - Continue execution
  F10       - Step over
  F11       - Step into
  Shift+F11 - Step out
  F9        - Toggle breakpoint
  Ctrl+←    - Step back
  Ctrl+→    - Step forward

Variables Pane:
  ↑/↓       - Navigate variables
  Enter     - Edit value
  W         - Add watch
```

## Implementation Plan

### Phase 1: Core Debugging Infrastructure
1. Implement `ExecutionPointer` and snapshot system
2. Add `SideEffectJournal` with basic file tracking
3. Create `BreakpointManager` with task breakpoints
4. Modify executor to integrate debugger

### Phase 2: Execution Control
1. Implement step modes (step, step into, step over, step out)
2. Add breakpoint checking to execution loop
3. Implement time-travel (back/forward navigation)
4. Add compensation strategies for undo

### Phase 3: Inspection API
1. Create `Inspector` for state inspection
2. Add variable snapshot capability
3. Implement timeline generation
4. Add task inspection details

### Phase 4: REPL Interface
1. Build command parser
2. Implement core commands (step, break, inspect)
3. Add variable modification
4. Implement spec modification on-the-fly

### Phase 5: TUI Implementation
1. Set up Ratatui framework
2. Implement pane system
3. Build workflow tree viewer
4. Add YAML editor integration
5. Create variables and timeline panes
6. Integrate REPL pane

### Phase 6: AI Integration
1. Add multi-provider support for generation
2. Implement block generation from prompts
3. Add suggestion system
4. Allow on-the-fly provider/model switching

## File Structure

```
src/dsl/
├── debugger/
│   ├── mod.rs              # Public API
│   ├── pointer.rs          # ExecutionPointer, snapshots
│   ├── breakpoints.rs      # BreakpointManager
│   ├── side_effects.rs     # SideEffectJournal, compensations
│   ├── state.rs            # DebuggerState
│   ├── inspector.rs        # Inspector API
│   └── compensation/       # Compensation strategies
│       ├── mod.rs
│       ├── file.rs         # File operation compensation
│       └── state.rs        # State compensation
├── repl/
│   ├── mod.rs              # REPL core
│   ├── commands.rs         # Command definitions
│   ├── parser.rs           # Command parser
│   ├── executor.rs         # Command executor
│   └── completions.rs      # Tab completion
└── tui/
    ├── mod.rs              # TUI main
    ├── app.rs              # AppState
    ├── panes/
    │   ├── mod.rs
    │   ├── tree.rs         # Workflow tree pane
    │   ├── editor.rs       # YAML editor pane
    │   ├── variables.rs    # Variables pane
    │   ├── timeline.rs     # Timeline pane
    │   └── repl.rs         # REPL pane
    ├── layout.rs           # Pane layout manager
    ├── input.rs            # Input handling
    └── ai_assist.rs        # AI generation integration
```

## Dependencies

Additional crates needed:
```toml
[dependencies]
# TUI
ratatui = "0.28"
crossterm = "0.28"

# YAML editing
tree-sitter = "0.20"
tree-sitter-yaml = "0.5"

# Text editing
ropey = "1.6"  # Rope-based text buffer

# Command parsing
nom = "7.1"    # Parser combinator

# AI integration (already present)
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
```

## Next Steps

1. Review and approve architecture
2. Begin Phase 1 implementation
3. Create unit tests for each component
4. Integration testing with existing executor
5. Documentation and examples

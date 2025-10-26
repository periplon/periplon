# TUI Core Application Implementation

## Overview

This document describes the core TUI application structure implemented for the DSL workflow REPL. The implementation follows hexagonal architecture principles, treating the TUI as a primary adapter that drives the DSL application layer.

## Architecture

### Module Structure

```
src/tui/
├── mod.rs              # Module definition and re-exports
├── app.rs              # Core App struct, event loop, view router
├── state.rs            # Application state management
├── events.rs           # Event handling and input processing
├── theme.rs            # Color schemes and styling
└── ui/                 # View components
    ├── mod.rs          # UI module definition
    ├── workflow_list.rs    # Workflow list view (stub)
    ├── editor.rs           # Workflow editor view (stub)
    ├── execution_monitor.rs # Execution monitor view (stub)
    ├── help.rs             # Help screen view (stub)
    └── modal.rs            # Modal dialogs (stub)
```

## Core Components

### 1. TuiApp (`app.rs`)

The main application struct that orchestrates the entire TUI experience.

**Key Features:**
- **Event Loop**: Async event processing with crossterm integration
- **View Router**: Routes rendering to appropriate view based on state
- **Modal System**: Overlay modals for confirmations, inputs, errors
- **State Management**: Centralized application state
- **Terminal Management**: Proper initialization and cleanup

**View Modes:**
- `WorkflowList`: Browse and select workflows
- `Editor`: Edit workflow YAML with syntax highlighting
- `ExecutionMonitor`: Watch workflow execution in real-time
- `Help`: Keyboard shortcuts and usage instructions

**Modal Types:**
- `Confirm`: Yes/No confirmation dialogs
- `Error`: Error message display
- `Input`: Text input dialogs
- `Info`: Information messages

### 2. AppState (`state.rs`)

Manages all application state with clear separation of concerns.

**State Components:**
- **View Mode**: Current active view
- **Modal State**: Active modal (if any)
- **Workflows**: List of available workflows with metadata
- **Current Workflow**: Loaded workflow for editing
- **Execution State**: Real-time execution tracking
- **Search**: Workflow filtering

**Key Types:**
- `WorkflowEntry`: Workflow metadata (name, path, version, modified time)
- `ExecutionState`: Execution status, current task, logs
- `ExecutionStatus`: Running, Paused, Completed, Failed, Cancelled
- `LogEntry`: Timestamped log messages with levels

### 3. EventHandler (`events.rs`)

Async event handling with clean separation of terminal and application events.

**Event Types:**
- `Key`: Keyboard input with modifiers
- `Resize`: Terminal resize events
- `Tick`: Periodic updates (250ms default)
- `ExecutionUpdate`: Workflow execution updates
- `Error`: Error notifications
- `Quit`: Application exit

**Features:**
- Non-blocking event polling
- Async event channel (mpsc unbounded)
- Keyboard modifier detection (Ctrl, Alt, Shift)
- Execution update streaming

### 4. Theme (`theme.rs`)

Consistent color scheme and styling throughout the application.

**Colors:**
- Primary: Cyan (main UI elements)
- Secondary: Blue (supporting elements)
- Accent: Magenta (highlights)
- Success: Green (successful operations)
- Warning: Yellow (warnings)
- Error: Red (errors)
- Muted: Dark Gray (disabled/dimmed)

**Styles:**
- Title: Bold primary
- Subtitle: Italic secondary
- Highlight: Inverted with bold
- Modal: Custom background/border

## Event Flow

```
Terminal Events → EventHandler → AppEvent Channel
                                       ↓
                                  TuiApp::handle_event()
                                       ↓
                        ┌──────────────┴────────────┐
                        ↓                           ↓
                   Modal Active?              Route to View
                        ↓                           ↓
              handle_modal_key()       handle_<view>_key()
                        ↓                           ↓
                   Modal Actions              View Actions
                        └────────────┬────────────┘
                                     ↓
                               Update State
                                     ↓
                                  Render
```

## Key Keyboard Shortcuts

### Global
- `Ctrl+Q`: Quit application (with confirmation)
- `?`: Show help screen
- `Esc`: Go back / Close modal

### Workflow List
- `Up/k`: Move selection up
- `Down/j`: Move selection down
- `Enter`: Open selected workflow
- `Ctrl+N`: Create new workflow
- `Ctrl+D`: Delete selected workflow
- `/`: Search workflows

### Editor
- `Ctrl+S`: Save workflow
- `Ctrl+R`: Run workflow
- `Esc`: Return to workflow list

### Execution Monitor
- `Ctrl+S`: Stop execution
- `Esc`: Return to workflow list (if not running)

### Modals
- `Enter`: Confirm
- `Y/y`: Yes (for confirmations)
- `N/n`: No (for confirmations)
- `Esc`: Cancel/Close

## State Machine

```
[WorkflowList] ──Enter──→ [Editor] ──Ctrl+R──→ [ExecutionMonitor]
      ↑                        ↓                        ↓
      └────────Esc─────────────┴────────Esc────────────┘
              (with confirmation if changes)

[Any View] ──?──→ [Help] ──Esc──→ [Previous View]
```

## Error Handling

All operations use the `Result<()>` pattern with proper error propagation:

1. **Terminal Errors**: Automatically cleanup on panic/drop
2. **I/O Errors**: Display error modal to user
3. **Workflow Errors**: Show validation/execution errors
4. **Graceful Shutdown**: Proper terminal restoration

## Async Integration

The TUI integrates async operations seamlessly:

1. **Event Polling**: Non-blocking with tokio
2. **Workflow Execution**: Background tasks with progress updates
3. **File Operations**: Async file I/O
4. **Execution Updates**: Channel-based streaming

## Terminal Management

Proper terminal lifecycle management:

1. **Initialization**:
   - Enter alternate screen
   - Enable raw mode
   - Hide cursor

2. **Runtime**:
   - Event loop
   - Rendering
   - State updates

3. **Cleanup** (via Drop):
   - Leave alternate screen
   - Disable raw mode
   - Restore cursor
   - Clear screen

## Next Steps

The following components need full implementation:

1. **UI Views** (stubs created):
   - `workflow_list.rs`: Workflow browsing with search
   - `editor.rs`: YAML editor with syntax highlighting
   - `execution_monitor.rs`: Real-time execution display
   - `help.rs`: Help screen with keyboard shortcuts
   - `modal.rs`: Modal rendering

2. **Workflow Operations**:
   - Load workflows from directory
   - Create/delete workflows
   - Save workflow changes
   - Execute workflows with monitoring

3. **Advanced Features**:
   - Syntax highlighting (via syntect)
   - Auto-completion
   - Validation feedback
   - Execution history
   - Workflow templates

## Testing

Unit tests included for:
- Event modifier detection
- Event handler creation
- State filtering

Integration tests needed for:
- Full event loop
- View transitions
- Modal interactions
- Workflow operations

## Dependencies

- `ratatui`: TUI framework (v0.29)
- `crossterm`: Terminal manipulation (v0.28)
- `tui-textarea`: Text editing (v0.7)
- `syntect`: Syntax highlighting (v5.2)
- `tokio`: Async runtime
- `serde_yaml`: Workflow serialization

## File References

- Main app: `src/tui/app.rs:1`
- State management: `src/tui/state.rs:1`
- Event handling: `src/tui/events.rs:1`
- Theme: `src/tui/theme.rs:1`
- Module definition: `src/tui/mod.rs:1`
- Library integration: `src/lib.rs:105`

## Feature Flag

The TUI is behind the `tui` feature flag in `Cargo.toml`:

```toml
[features]
tui = ["ratatui", "crossterm", "tui-textarea", "syntect"]
```

Build with: `cargo build --features tui`

---

**Status**: Core foundation complete, UI views stubbed for next phase
**Next Task**: Implement workflow list view rendering

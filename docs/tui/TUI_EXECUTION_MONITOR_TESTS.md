# TUI Execution Monitor Testing Suite

Comprehensive test coverage for the Execution Monitor screen rendering, keyboard handling, and real-time workflow execution tracking operations.

## Overview

The execution monitor testing suite consists of two main test files that provide complete coverage of the real-time workflow execution monitoring functionality:

1. **`tests/tui_execution_monitor_rendering_tests.rs`** - Rendering and layout tests (42 tests)
2. **`tests/tui_execution_monitor_keyboard_tests.rs`** - Keyboard interaction and state management tests (41 tests)

**Total: 83 tests, all passing**

## Test Files

### Execution Monitor Rendering Tests (`tui_execution_monitor_rendering_tests.rs`)

Tests the visual rendering, layout, progress tracking, and status display using ratatui's `TestBackend`.

#### Test Categories

**Basic Rendering (3 tests)**
- Basic execution monitor screen rendering
- Layout structure (header, content, statistics, shortcuts)
- Minimum terminal size handling (80x24)

**Header Rendering (6 tests)** - All execution statuses
- Workflow info display (name, version)
- Running status with progress
- Paused status indicator
- Completed status with summary
- Failed status with error count
- Cancelled status indicator

**Task List Rendering (5 tests)** - Task status display
- Pending task (○ icon)
- Running task with progress (⟳ icon)
- Completed task (✓ icon)
- Failed task with error (✗ icon)
- Multiple tasks with mixed statuses

**Log Output Rendering (6 tests)** - All log levels
- Empty log output
- Info log entries (ℹ icon)
- Error log entries (✗ icon, red)
- Warning log entries (⚠ icon, yellow)
- Debug log entries (🔍 icon, gray)
- Multiple log entries with scrolling

**Statistics Panel (4 tests)**
- Initial state (0 tasks, 0 cost)
- Task count statistics (total, completed, failed, running, pending, skipped)
- Failure tracking with error counts
- Cost tracking and token usage display

**Panel Focus (3 tests)**
- Task list panel focus
- Log output panel focus
- Task details panel focus

**Progress Tracking (3 tests)**
- Zero percent progress
- Fifty percent progress (mid-execution)
- Complete (100%) progress

**Error Display (2 tests)**
- Single failed task with error message
- Multiple failed tasks with error aggregation

**Token Usage (2 tests)**
- Per-task token usage tracking
- Total token statistics (input, output, cache)

**Theme Tests (1 test)**
- Rendering with all themes (dark, light, monokai, solarized)

**Edge Cases (7 tests)**
- No tasks (empty workflow)
- Many tasks (50+ tasks)
- Many logs (100+ entries)
- Long error message (500+ chars)
- Large terminal (200x100)
- Auto-scroll logs enabled
- Auto-scroll logs disabled

### Keyboard Handling Tests (`tui_execution_monitor_keyboard_tests.rs`)

Tests keyboard event processing for monitoring controls, navigation, and execution management.

#### Test Categories

**Navigation (2 tests)**
- Escape returns to workflow list
- Escape works in any execution status

**Execution Control (4 tests)**
- Ctrl+S triggers stop confirmation
- Ctrl+S when running
- Ctrl+S when paused
- Ctrl+S when completed (ignored)

**State Management (5 tests)**
- Panel focus state tracking
- Auto-scroll state toggle
- Task details visibility toggle
- Scroll state tracking (log and task scrolls)
- Selected task tracking

**Execution Status (5 tests)**
- Status: Running
- Status: Paused
- Status: Completed
- Status: Failed
- Status: Cancelled

**Task State (5 tests)**
- Task status: Pending
- Task status: Running
- Task status: Completed
- Task status: Failed
- Task status: Skipped

**Statistics Tracking (4 tests)**
- Statistics initialization (all zeros)
- Statistics tracking during execution
- Cost accumulation tracking
- Token usage accumulation

**Ignored Keys (10 tests)**
- Regular characters (a-z, 0-9) ignored
- Arrow keys ignored
- Function keys (F1-F12) ignored
- Page Up/Down ignored
- Home/End ignored
- Tab ignored
- Enter ignored
- Backspace ignored
- Delete ignored
- Other special keys ignored

**Edge Cases (3 tests)**
- Rapid escape key presses
- Rapid Ctrl+S presses
- Mixed key sequence handling

**State Consistency (3 tests)**
- State defaults validation
- ExecutionStatus enum values
- TaskStatus enum values
- MonitorPanel enum values

## Key Implementation Details

### Execution Statuses

**Running**
- Active workflow execution
- Progress bar animating
- Real-time log updates
- Task status updates

**Paused**
- Execution temporarily halted
- Progress preserved
- Can resume or stop

**Completed**
- All tasks finished successfully
- Final statistics displayed
- Summary available

**Failed**
- One or more tasks failed
- Error messages displayed
- Partial results available

**Cancelled**
- User-initiated cancellation
- Cleanup performed
- Partial results available

### Task Statuses

- **Pending**: Not yet started (○ icon)
- **Running**: Currently executing (⟳ icon)
- **Completed**: Finished successfully (✓ icon)
- **Failed**: Execution error (✗ icon)
- **Skipped**: Dependency failure (⊘ icon)

### Panel Focus Management

**TaskList Panel**
- Shows all workflow tasks
- Status icons and progress
- Timing information
- Dependency visualization

**LogOutput Panel**
- Real-time log streaming
- Log level filtering
- Auto-scroll support
- Search capability

**TaskDetails Panel**
- Detailed task information
- Agent assignment
- Input/output variables
- Cost and token metrics

### Layout Structure

```
┌─────────────────────────────────────┐
│ Header (5 lines)                    │  - Workflow name/version
│ - Workflow info                     │  - Status indicator
│ - Execution status                  │  - Progress bar
│ - Progress bar                      │  - Timing information
├─────────────────────────────────────┤
│ Task List       │ Log Output        │  - Task statuses
│ (30%)           │ (70%)             │  - Real-time logs
│                 │                   │  - Error messages
│                 │                   │
├─────────────────────────────────────┤
│ Statistics Panel (4 lines)          │  - Task counts
│ - Task counts, timing, costs        │  - Execution time
│ - Token usage                       │  - Cost tracking
├─────────────────────────────────────┤
│ Shortcuts Bar (1 line)              │  - Context-sensitive help
└─────────────────────────────────────┘
```

### Log Levels and Icons

- **Debug**: 🔍 Gray, verbose information
- **Info**: ℹ Blue, general information
- **Warning**: ⚠ Yellow, potential issues
- **Error**: ✗ Red, execution failures

### Execution Statistics

**Task Metrics**:
- Total tasks count
- Completed tasks count
- Failed tasks count
- Running tasks count
- Pending tasks count
- Skipped tasks count

**Resource Metrics**:
- Total cost ($)
- Total input tokens
- Total output tokens
- Cache read tokens
- Cache write tokens

**Timing Metrics**:
- Execution start time
- Current/end time
- Paused duration
- Total elapsed time
- Per-task timing

### Keyboard Shortcuts

- **Esc**: Return to workflow list
- **Ctrl+S**: Confirm stop execution

## Test Utilities

### Rendering Utilities

```rust
/// Create test terminal
fn create_terminal(width: u16, height: u16) -> Terminal<TestBackend>

/// Render execution monitor and return terminal
fn render_monitor(
    state: &ExecutionMonitorState,
    theme: &Theme,
    width: u16,
    height: u16,
) -> Terminal<TestBackend>

/// Check if buffer contains text
fn buffer_contains(terminal: &Terminal<TestBackend>, text: &str) -> bool

/// Create test workflow
fn create_test_workflow() -> DSLWorkflow

/// Create task execution state
fn create_task_state(
    task_id: &str,
    status: TaskStatus,
    progress: u8,
) -> TaskExecutionState

/// Create log entry
fn create_log_entry(level: LogLevel, message: &str) -> LogEntry
```

### Keyboard Utilities

```rust
/// Create KeyEvent
fn key(code: KeyCode) -> KeyEvent

/// Create KeyEvent with Ctrl
fn ctrl_key(code: KeyCode) -> KeyEvent

/// Simulated keyboard action results
enum MonitorAction {
    None,
    Exit,
    ConfirmStop,
}

/// Simulate execution monitor key handling
fn handle_execution_monitor_key_simulation(
    state: &mut ExecutionMonitorState,
    key_event: KeyEvent,
) -> MonitorAction

/// Create test workflow
fn create_test_workflow() -> DSLWorkflow

/// Create task execution state
fn create_task_state(task_id: &str, status: TaskStatus) -> TaskExecutionState
```

## Running the Tests

```bash
# Run all execution monitor tests
cargo test --features tui --test tui_execution_monitor_rendering_tests --test tui_execution_monitor_keyboard_tests

# Run only rendering tests
cargo test --features tui --test tui_execution_monitor_rendering_tests

# Run only keyboard tests
cargo test --features tui --test tui_execution_monitor_keyboard_tests

# Run specific test
cargo test --features tui --test tui_execution_monitor_keyboard_tests test_escape_returns_to_workflow_list

# Run with output visible
cargo test --features tui --test tui_execution_monitor_rendering_tests -- --nocapture
```

## Test Coverage Summary

| Category | Tests | Status |
|----------|-------|--------|
| **RENDERING TESTS** | **42** | **✅** |
| Basic Rendering | 3 | ✅ All Pass |
| Header Rendering | 6 | ✅ All Pass |
| Task List | 5 | ✅ All Pass |
| Log Output | 6 | ✅ All Pass |
| Statistics Panel | 4 | ✅ All Pass |
| Panel Focus | 3 | ✅ All Pass |
| Progress Tracking | 3 | ✅ All Pass |
| Error Display | 2 | ✅ All Pass |
| Token Usage | 2 | ✅ All Pass |
| Theme Tests | 1 | ✅ All Pass |
| Edge Cases | 7 | ✅ All Pass |
| **KEYBOARD TESTS** | **41** | **✅** |
| Navigation | 2 | ✅ All Pass |
| Execution Control | 4 | ✅ All Pass |
| State Management | 5 | ✅ All Pass |
| Execution Status | 5 | ✅ All Pass |
| Task State | 5 | ✅ All Pass |
| Statistics Tracking | 4 | ✅ All Pass |
| Ignored Keys | 10 | ✅ All Pass |
| Edge Cases | 3 | ✅ All Pass |
| State Consistency | 3 | ✅ All Pass |
| **TOTAL** | **83** | **✅ 100%** |

## Architecture Compliance

### Hexagonal Architecture

**Domain**: `ExecutionMonitorState`, `ExecutionStatus`, `TaskExecutionState`, `TaskStatus`
- Pure state and business logic
- No UI dependencies
- Execution tracking logic

**Primary Port**: Execution monitor rendering interface
- `render()` function in `views/execution_monitor`
- Real-time state updates
- Statistics calculation

**Rendering**: Pure presentation
- Layout calculations (task list 30%, logs 70%)
- Progress visualization
- Log level highlighting
- Visual feedback (status icons, progress bars)
- No state mutation

**State Management**: Clean separation
- Keyboard handlers update state
- Rendering reads state
- Execution events update statistics
- Real-time log streaming

## Features Tested

✅ Real-time task status monitoring
✅ Execution status tracking (Running, Paused, Completed, Failed, Cancelled)
✅ Task status display (Pending, Running, Completed, Failed, Skipped)
✅ Log output streaming with levels (Debug, Info, Warning, Error)
✅ Progress tracking (overall and per-task)
✅ Statistics panel (tasks, timing, costs, tokens)
✅ Panel focus management (TaskList, LogOutput, TaskDetails)
✅ Auto-scroll logs functionality
✅ Error message display
✅ Token usage tracking
✅ Cost accumulation
✅ Keyboard shortcuts (Esc, Ctrl+S)
✅ Theme support (all themes)
✅ Layout responsiveness
✅ Edge case handling (no tasks, many tasks, many logs, long errors)

## Known Issues

None currently. All 83 tests pass successfully.

## Future Enhancements

Potential areas for additional testing:

1. **Real-time Updates**: Test live log streaming simulation
2. **Task Dependencies**: Test dependency visualization
3. **Pause/Resume**: Test execution pause and resume cycles
4. **Log Filtering**: Test log level filtering functionality
5. **Search**: Test search within logs
6. **Export**: Test execution report export
7. **Performance**: Benchmark with 1000+ tasks and 10k+ log entries
8. **Visual Regression**: Snapshot testing for rendering
9. **Integration**: Full workflow execution → monitoring → completion scenarios
10. **Error Recovery**: Test error handling and recovery mechanisms

## Related Files

- `src/tui/views/execution_monitor.rs` - Execution monitor rendering implementation (850+ lines)
- `src/tui/ui/execution_monitor.rs` - Execution monitor component wrapper
- `src/tui/state.rs` - ExecutionMonitorState definition
- `src/tui/app.rs` - Execution monitor keyboard handling (handle_execution_monitor_key)
- `src/tui/theme.rs` - Theme definitions
- `src/dsl/executor.rs` - DSL executor integration
- `src/dsl/state.rs` - Workflow state management
- `tests/tui_unit_tests.rs` - General TUI component tests
- `docs/TUI_EDITOR_TESTS.md` - Editor testing documentation
- `docs/TUI_MODAL_TESTS.md` - Modal testing documentation
- `docs/TUI_GENERATOR_TESTS.md` - Generator testing documentation

## Testing Best Practices

1. **Use TestBackend**: Always use ratatui's TestBackend for rendering tests
2. **Test State Isolation**: Each test starts with fresh ExecutionMonitorState
3. **Real-time Simulation**: Test log streaming and status updates
4. **State Consistency**: Verify statistics match actual task states
5. **Panel Context**: Test focus-dependent behavior
6. **Status Validation**: Verify execution status transitions
7. **Keyboard Modifiers**: Test Ctrl shortcuts separately from regular keys
8. **Edge Cases**: Test empty execution, very large task lists, long logs
9. **Resource Tracking**: Verify cost and token accumulation accuracy
10. **Documentation**: Keep test documentation synchronized with implementation

---

**Test Suite Version**: 1.0
**Last Updated**: 2025-10-25
**Maintainer**: TUI Team
**Coverage**: 83/83 tests passing (100%), 0 tests skipped

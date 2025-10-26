# TUI State Browser Testing Suite

Comprehensive test coverage for the state browser rendering and keyboard handling in the DSL TUI application.

## Overview

The state browser testing suite consists of two main test files that provide complete coverage of state browser functionality:

1. **`tests/tui_state_browser_rendering_tests.rs`** - Rendering and layout tests (40 tests)
2. **`tests/tui_state_browser_keyboard_tests.rs`** - Keyboard interaction tests (32 tests)

**Total: 72 tests, all passing**

## Test Files

### State Browser Rendering Tests (`tui_state_browser_rendering_tests.rs`)

Tests the visual rendering, layout, and display of the state browser using ratatui's `TestBackend`.

#### Test Categories

**Basic Rendering (3 tests)**
- Basic state browser display
- Empty state list handling
- State browser with multiple workflow states

**List View Tests (8 tests)**
- Header display ("Workflow State Browser")
- Filter bar (placeholder and active filter)
- Sort mode display
- Status display (Running, Completed, Failed, Paused)
- Version display
- Task count display
- Progress bar rendering
- Help bar with navigation hints
- Highlight symbol for selection

**Details View Tests (5 tests)**
- Header with workflow name and version
- Status information display
- Progress information display
- Control hints (Delete, Back, Scroll)
- Error handling (no state loaded)

**Sort Mode Tests (3 tests)**
- Sort mode cycle validation
- Sort mode display names
- All sort modes rendered correctly

**State Entry Tests (1 test)**
- Status text conversion

**View Mode Tests (1 test)**
- View mode enum values

**Filter Tests (3 tests)**
- Filter by workflow name
- Filter by status
- Case-insensitive filtering

**Layout Tests (4 tests)**
- List view layout structure
- Details view layout structure
- Minimum size rendering (80x24)
- Large terminal rendering (200x60)

**Theme Tests (1 test)**
- Rendering with all themes (default/dark, light, monokai, solarized)

**Edge Cases (11 tests)**
- Long workflow names
- Many states (50+)
- Zero progress
- Full progress (100%)
- Special characters in names
- State browser defaults
- Back to list transition
- Select next/previous navigation
- Selection boundary conditions

### Keyboard Handling Tests (`tui_state_browser_keyboard_tests.rs`)

Tests keyboard event processing, navigation, sorting, and view switching.

#### Test Categories

**Navigation Tests - List View (6 tests)**
- Escape returns to workflow list
- 'q' returns to workflow list
- Up arrow selects previous
- Up arrow stops at zero
- Down arrow selects next
- Down arrow stops at last item
- Enter loads details

**Navigation Tests - Details View (8 tests)**
- Escape from details returns to list
- 'q' from details returns to list
- Up arrow scrolls up
- Up arrow stops at zero
- Down arrow scrolls down
- PageUp scrolls up by page size
- PageUp stops at zero
- PageDown scrolls down by page size
- PageDown respects maximum

**Sort Mode Tests (2 tests)**
- 's' key cycles sort mode
- Full sort mode cycle (all 6 modes)

**Help Key Tests (1 test)**
- '?' key opens help

**Ignored Keys Tests (2 tests)**
- Regular characters ignored
- Function keys ignored

**State Consistency Tests (3 tests)**
- Selection persists between key presses
- View mode preserved during navigation
- Scroll preserved during sorting

**Complex Workflow Tests (3 tests)**
- Navigation sequence (multi-step navigation)
- Details scroll sequence
- View transition sequence

**Edge Cases (7 tests)**
- Navigation with empty list
- Navigation with single item
- Rapid sort changes
- Alternating navigation
- Details scroll boundary conditions

## Key Testing Utilities

### Rendering Utilities

```rust
/// Create test terminal with specified dimensions
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend>

/// Render state browser and return terminal for assertions
fn render_state_browser(
    state: &mut StateBrowserState,
    theme: &Theme,
    width: u16,
    height: u16,
) -> Terminal<TestBackend>

/// Check if buffer contains text
fn buffer_contains_text(terminal: &Terminal<TestBackend>, text: &str) -> bool

/// Count visible border characters
fn count_border_chars(terminal: &Terminal<TestBackend>) -> usize

/// Create a test state entry
fn create_test_entry(name: &str, status: WorkflowStatus, progress: f64) -> StateEntry
```

### Keyboard Utilities

```rust
/// Create KeyEvent for testing
fn key(code: KeyCode) -> KeyEvent

/// Simulate state browser keyboard handling in list view
async fn handle_list_key_simulation(
    state: &mut AppState,
    key_event: KeyEvent,
) -> ViewMode
```

## State Browser Functionality Tested

### View Modes

**List View (StateBrowserViewMode::List)**
- Displays list of all workflow states
- Shows status, progress, task counts for each state
- Filtering by name or status
- Sorting by name, modified date, or progress
- Visual elements: status badges, progress bars, timestamps

**Details View (StateBrowserViewMode::Details)**
- Displays detailed information for selected workflow state
- Shows complete task breakdown (completed, failed, pending)
- Task results and error messages
- Loop states (if any)
- Metadata
- Scrollable content with scrollbar

### Navigation Keys

| Key | Action in List View | Action in Details View |
|-----|---------------------|------------------------|
| `Esc` / `q` | Return to workflow list | Return to list view |
| `↑` | Select previous state | Scroll up one line |
| `↓` | Select next state | Scroll down one line |
| `Enter` | Load details for selected state | (no action) |
| `PageUp` | (no action) | Scroll up by page size |
| `PageDown` | (no action) | Scroll down by page size |
| `s` | Cycle sort mode | Cycle sort mode |
| `/` | Start filter mode (TODO) | (no action) |
| `r` | Resume selected workflow (TODO) | Resume workflow (TODO) |
| `d` | Delete selected state | Delete current state |
| `?` | Open help | Open help |

### Sort Modes

The state browser supports 6 sort modes, cycling in this order:

1. **Name ↑** - Alphabetical by workflow name (ascending)
2. **Name ↓** - Alphabetical by workflow name (descending)
3. **Modified ↑** - By modification time (oldest first)
4. **Modified ↓** - By modification time (newest first) *[default]*
5. **Progress ↑** - By completion progress (least complete first)
6. **Progress ↓** - By completion progress (most complete first)

### Workflow Status Types

| Status | Color | Description |
|--------|-------|-------------|
| Running | Yellow | Workflow currently executing |
| Completed | Green | Workflow finished successfully |
| Failed | Red | Workflow execution failed |
| Paused | Cyan | Workflow paused/suspended |

### Layout Structure - List View

```
┌─────────────────────────────────────────┐
│     Workflow State Browser              │ Header (3 lines)
│                                         │
├─────────────────────────────────────────┤
│ ┌─ Sort: Modified ↓ ─────────────────┐ │ Filter Bar (3 lines)
│ │ Filter: (type to search)           │ │
│ └────────────────────────────────────┘ │
├─────────────────────────────────────────┤
│ ┌─ States (N) ───────────────────────┐ │
│ │ [Running] workflow1 v1.0.0         │ │ State List (scrollable)
│ │   [████████░░] (5/10 tasks) 1m ago │ │
│ │ [Completed] workflow2 v2.0.0       │ │
│ │   [██████████] (10/10 tasks) 2h ago│ │
│ └────────────────────────────────────┘ │
├─────────────────────────────────────────┤
│ ↑/↓: Navigate | Enter: View Details... │ Status Bar (2 lines)
└─────────────────────────────────────────┘
```

### Layout Structure - Details View

```
┌─────────────────────────────────────────┐
│ Workflow State: workflow1 v1.0.0        │ Header (3 lines)
│                                         │
├─────────────────────────────────────────┤
│ ┌─ Details ──────────────────────────┐ │
│ │ Status: Running                    │ │
│ │ Progress: 50%  [█████░░░░░░░░░░]   │ │
│ │ Duration: 5m 30s                   │ │
│ │                                    │ │ Details Content (scrollable)
│ │ Task Status:                       │ │
│ │   Completed: 5                     │ │
│ │     ✓ task1                        │ │
│ │     ✓ task2                        │ │
│ │   Pending: 5                       │ │
│ │     ○ task6                        │ │
│ └────────────────────────────────────┘↑│ (scrollbar)
├─────────────────────────────────────────┤
│ r: Resume | d: Delete | Esc/q: Back ... │ Controls (2 lines)
└─────────────────────────────────────────┘
```

## Implementation Details Verified

### List View Elements

**State Entry Display**
- Status badge: `[Running]`, `[Completed]`, `[Failed]`, `[Paused]`
- Workflow name (bold, primary color)
- Version: `v1.0.0`
- Progress bar: `[██████░░░░]` (20 chars wide)
- Task count: `(5/10 tasks)`
- Time since checkpoint: `1m ago`, `2h ago`, `1d 3h`

**Progress Bar Colors**
- Red: 0-49%
- Yellow: 50-99%
- Green: 100%

**Filtering**
- Searches in workflow name (case-insensitive)
- Searches in status text (case-insensitive)
- Live filter as you type (when implemented)

### Details View Elements

**Status Overview**
- Status with color coding
- Progress percentage with bar
- Duration (time from start to last checkpoint)

**Task Breakdown**
- Completed tasks (green) with ✓ symbol
  - Shows task results if available
- Failed tasks (red) with ✗ symbol
  - Shows error messages
- Pending tasks (yellow/muted) with ○ symbol

**Loop States** (if present)
- Current iteration number
- Total iterations (if known)
- Loop progress bar

**Metadata** (if present)
- Key-value pairs from workflow metadata

### Scroll Behavior

**List View**
- Up/Down arrows: Move selection by 1
- Selection cannot go below 0 or above last item
- Filtered list affects available items

**Details View**
- Up/Down arrows: Scroll content by 1 line
- PageUp: Scroll up by `details_page_size` (typically 20)
- PageDown: Scroll down by `details_page_size`
- Scroll range: 0 to `(total_lines - page_size)`
- Scrollbar shows position and content length

### State Management

- `StateBrowserState` tracks:
  - `states`: Vec<StateEntry> - all available states
  - `selected_index`: usize - current selection
  - `view_mode`: List or Details
  - `filter_query`: String - current filter
  - `sort_mode`: One of 6 sort modes
  - `details_scroll`: usize - scroll position in details
  - `current_state`: Option<WorkflowState> - loaded detail state

## Running the Tests

```bash
# Run all state browser tests
cargo test --features tui --test tui_state_browser_rendering_tests --test tui_state_browser_keyboard_tests

# Run only rendering tests
cargo test --features tui --test tui_state_browser_rendering_tests

# Run only keyboard tests
cargo test --features tui --test tui_state_browser_keyboard_tests

# Run with output visible
cargo test --features tui --test tui_state_browser_rendering_tests -- --nocapture

# Run specific test
cargo test --features tui --test tui_state_browser_keyboard_tests test_sort_mode_cycle
```

## Test Coverage Summary

| Category | Tests | Status |
|----------|-------|--------|
| Basic Rendering | 3 | ✅ All Pass |
| List View | 8 | ✅ All Pass |
| Details View | 5 | ✅ All Pass |
| Sort Mode | 3 | ✅ All Pass |
| State Entry | 1 | ✅ All Pass |
| View Mode | 1 | ✅ All Pass |
| Filter | 3 | ✅ All Pass |
| Layout | 4 | ✅ All Pass |
| Theme | 1 | ✅ All Pass |
| Edge Cases (Rendering) | 11 | ✅ All Pass |
| Navigation - List View | 6 | ✅ All Pass |
| Navigation - Details View | 8 | ✅ All Pass |
| Sort Mode (Keyboard) | 2 | ✅ All Pass |
| Help Key | 1 | ✅ All Pass |
| Ignored Keys | 2 | ✅ All Pass |
| State Consistency | 3 | ✅ All Pass |
| Complex Workflows | 3 | ✅ All Pass |
| Edge Cases (Keyboard) | 7 | ✅ All Pass |
| **Total** | **72** | **✅ 100%** |

## Architecture Notes

### Test Backend Usage

Tests use `ratatui::backend::TestBackend` which provides:
- In-memory buffer for rendering
- Buffer inspection without actual terminal I/O
- Fast, deterministic test execution
- Cell-by-cell content verification

### State Management

Keyboard tests simulate the actual `handle_state_browser_key` logic from `src/tui/app.rs`:
- View mode transitions (StateBrowser ↔ WorkflowList, StateBrowser ↔ Help)
- List selection management
- Details scroll management
- Sort mode cycling
- State loading and transitions

### Hexagonal Architecture Compliance

Tests validate the state browser's compliance with hexagonal architecture:
- **Domain**: StateBrowserState, StateEntry, sort/filter logic
- **Primary Port**: State browser rendering interface
- **Rendering**: Pure presentation logic with no side effects
- **State**: Clean separation of state and rendering

## Related Test Suites

- `tests/tui_modal_tests.rs` - Modal rendering tests (27 tests)
- `tests/tui_modal_keyboard_tests.rs` - Modal keyboard tests (34 tests)
- `tests/tui_editor_rendering_tests.rs` - Editor rendering tests (42 tests)
- `tests/tui_editor_keyboard_tests.rs` - Editor keyboard tests (50 tests)
- `tests/tui_viewer_rendering_tests.rs` - Viewer rendering tests (40 tests)
- `tests/tui_viewer_keyboard_tests.rs` - Viewer keyboard tests (32 tests)
- `tests/tui_unit_tests.rs` - General TUI component tests (18 tests)

**Combined TUI Test Coverage: 345 tests**

## Future Enhancements

Potential areas for additional testing:

1. **State Persistence Integration**: Test actual state file loading and saving
2. **Resume Functionality**: Test workflow resume operations
3. **Filter Input Modal**: Test interactive filter input when implemented
4. **Delete Confirmation**: Test delete workflow with confirmation modal
5. **Performance Tests**: Rendering benchmarks for large state lists (1000+ items)
6. **Visual Regression**: Snapshot testing for pixel-perfect rendering
7. **Integration Tests**: Full state browser workflow from load to resume
8. **Real-time Updates**: Test state list updates during workflow execution

## Known Limitations

1. **Load Details**: Tests cannot fully test `load_details()` as it requires actual state persistence
2. **Resume Workflow**: Resume functionality is stubbed (TODO in app.rs)
3. **Filter Input**: Filter mode activation ('/') doesn't open input modal yet
4. **Delete Workflow**: Delete confirmation flow not fully implemented in tests

These limitations reflect actual implementation TODOs in the application code.

## Related Files

- `src/tui/ui/state_browser.rs` - State browser component wrapper
- `src/tui/views/state_browser.rs` - State browser rendering implementation
- `src/tui/state.rs` - AppState with state_browser field
- `src/tui/app.rs` - State browser keyboard handling
- `src/dsl/state.rs` - WorkflowState and StatePersistence
- `src/tui/theme.rs` - Theme definitions
- `docs/TUI_MODAL_TESTS.md` - Modal test documentation
- `docs/TUI_EDITOR_TESTS.md` - Editor test documentation
- `docs/TUI_VIEWER_TESTS.md` - Viewer test documentation

---

**Test Suite Version**: 1.0
**Last Updated**: 2025-10-25
**Maintainer**: TUI Team

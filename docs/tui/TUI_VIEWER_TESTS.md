# TUI Viewer Testing Suite

Comprehensive test coverage for the viewer screen rendering and keyboard handling in the DSL TUI application.

## Overview

The viewer testing suite consists of two main test files that provide complete coverage of viewer functionality:

1. **`tests/tui_viewer_rendering_tests.rs`** - Rendering and layout tests (40 tests)
2. **`tests/tui_viewer_keyboard_tests.rs`** - Keyboard interaction tests (32 tests)

**Total: 72 tests, all passing**

## Test Files

### Viewer Rendering Tests (`tui_viewer_rendering_tests.rs`)

Tests the visual rendering, layout, and display of the workflow viewer using ratatui's `TestBackend`.

#### Test Categories

**Basic Rendering (3 tests)**
- Workflow display with borders and basic structure
- Empty workflow handling
- Complete workflow with agents and tasks

**Header Rendering (3 tests)**
- Workflow name display
- Version information
- View mode indicator (Condensed/Full)

**Condensed View Tests (12 tests)**
- Metadata section (name, version, cwd)
- Section headers with proper formatting
- Agents section with symbols (◆) and details
- Agent count display
- Agent detail rendering (description, model, tools)
- Tasks section with symbols (▶) and details
- Task dependencies display
- Input variables section
- Output variables section
- Empty agents and tasks handling

**Full YAML View Tests (3 tests)**
- YAML key rendering (name:, version:)
- YAML structure for complete workflows
- Nested structure display (agents:, tasks:, cwd:)

**Status Bar Tests (3 tests)**
- Navigation hints display
- Condensed mode status
- Full mode status

**ViewerState Tests (3 tests)**
- Default state initialization
- State reset functionality
- View mode toggle

**Scroll Tests (3 tests)**
- Zero scroll offset
- Non-zero scroll offset
- Scrollbar with long content

**Layout and Sizing Tests (3 tests)**
- Viewer layout structure (header, content, status)
- Minimum size handling (80x24)
- Large terminal adaptation (200x60)

**Theme Tests (1 test)**
- Rendering with all themes (dark, light, monokai, solarized)

**Edge Cases and Robustness (6 tests)**
- Long workflow names
- Special characters in workflow content
- Unicode support

### Keyboard Handling Tests (`tui_viewer_keyboard_tests.rs`)

Tests keyboard event processing, navigation, scrolling, and mode switching.

#### Test Categories

**Navigation Tests (4 tests)**
- Escape key returns to workflow list
- Escape resets viewer state
- 'e' key switches to editor mode
- Tab key toggles view mode

**Scrolling Tests - Arrow Keys (4 tests)**
- Up arrow scrolls up
- Up arrow stops at zero
- Down arrow scrolls down
- Down arrow respects maximum lines

**Scrolling Tests - Vim Keys (4 tests)**
- 'k' key scrolls up
- 'j' key scrolls down
- Multiple k presses
- Multiple j presses

**Page Navigation Tests (4 tests)**
- PageUp scrolls up by 10 lines
- PageUp stops at zero
- PageDown scrolls down by 10 lines
- PageDown from near maximum

**Home/End Tests (3 tests)**
- Home key jumps to top
- End key jumps to bottom
- Home from zero stays at zero

**Ignored Keys Tests (3 tests)**
- Regular characters ignored (a, x, z)
- Ctrl keys ignored (Ctrl+s, Ctrl+r)
- Function keys ignored (F1, F12)

**State Consistency Tests (3 tests)**
- Scroll state persists between key presses
- View mode persists during scrolling
- Workflow preserved during navigation

**Complex Workflow Tests (3 tests)**
- Scroll navigation sequence
- View mode toggle sequence
- Vim and arrow keys interchangeable

**Edge Cases (4 tests)**
- Rapid key presses (100 down arrows)
- Alternating directions
- Escape from different scroll positions

## Key Testing Utilities

### Rendering Utilities

```rust
/// Create test terminal with specified dimensions
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend>

/// Render viewer and return terminal for assertions
fn render_viewer(
    workflow: &DSLWorkflow,
    state: &ViewerState,
    theme: &Theme,
    width: u16,
    height: u16,
) -> Terminal<TestBackend>

/// Check if buffer contains text
fn buffer_contains_text(terminal: &Terminal<TestBackend>, text: &str) -> bool

/// Count visible border characters
fn count_border_chars(terminal: &Terminal<TestBackend>) -> usize

/// Create a minimal test workflow
fn create_minimal_workflow() -> DSLWorkflow

/// Create a complete test workflow with agents and tasks
fn create_complete_workflow() -> DSLWorkflow
```

### Keyboard Utilities

```rust
/// Create KeyEvent for testing
fn key(code: KeyCode) -> KeyEvent

/// Create KeyEvent with Control modifier
fn ctrl_key(code: KeyCode) -> KeyEvent

/// Simulate viewer keyboard handling
async fn handle_viewer_key_simulation(
    state: &mut AppState,
    key_event: KeyEvent,
) -> ViewMode
```

## Viewer Functionality Tested

### View Modes

**Condensed View (WorkflowViewMode::Condensed)**
- Displays workflow summary in structured sections
- Sections: Metadata, Agents, Tasks, Inputs, Outputs
- Compact representation with symbols and key information
- Visual: Section headers (━━━), agent symbols (◆), task symbols (▶)

**Full View (WorkflowViewMode::Full)**
- Displays complete YAML representation
- Syntax highlighting for YAML elements
- Shows all workflow configuration
- Useful for detailed inspection

### Navigation Keys

| Key | Action |
|-----|--------|
| `Esc` | Return to workflow list and reset viewer state |
| `Tab` | Toggle between Condensed and Full view modes |
| `e` | Switch to Editor mode |
| `↑` / `k` | Scroll up one line |
| `↓` / `j` | Scroll down one line |
| `PageUp` | Scroll up by 10 lines |
| `PageDown` | Scroll down by 10 lines |
| `Home` | Jump to top |
| `End` | Jump to bottom |

### Layout Structure

```
┌─────────────────────────────────────┐
│ Header (3 lines)                    │
│ - Workflow name                     │
│ - Version                           │
│ - View mode indicator               │
├─────────────────────────────────────┤
│                                     │
│ Content Area                        │
│ (Scrollable)                        │
│                                     │
│ - Condensed view sections OR        │
│ - Full YAML view                    │
│                                     │
├─────────────────────────────────────┤
│ Status Bar (1 line)                 │
│ - Navigation hints                  │
│ - Current view mode                 │
└─────────────────────────────────────┘
```

## Implementation Details Verified

### Condensed View Sections

1. **Metadata Section**
   - Workflow name
   - Version
   - CWD (if set)
   - Section header: `━━━ Metadata ━━━`

2. **Agents Section**
   - Agent count
   - Each agent prefixed with ◆ symbol
   - Agent details: description, model, tools
   - Section header: `━━━ Agents (N) ━━━`

3. **Tasks Section**
   - Task count
   - Each task prefixed with ▶ symbol
   - Task details: description, agent, dependencies
   - Section header: `━━━ Tasks (N) ━━━`

4. **Inputs Section** (if present)
   - Input variable names and types
   - Required/optional indicators
   - Section header: `━━━ Inputs ━━━`

5. **Outputs Section** (if present)
   - Output variable names
   - Output sources
   - Section header: `━━━ Outputs ━━━`

### Full View YAML

- Complete YAML serialization of DSLWorkflow
- Syntax highlighting:
  - Comments: gray/dimmed
  - Keys: cyan/blue
  - Values: white/default
  - Lists: yellow/special
- Fields with default values may be omitted (e.g., `dsl_version: "1.0.0"`)

### Scroll Behavior

- **Up/k**: Decrements scroll by 1, stops at 0
- **Down/j**: Increments scroll by 1, enforces max on some keys
- **PageUp**: Decrements scroll by 10, stops at 0 (uses `saturating_sub`)
- **PageDown**: Increments scroll by 10 (no max enforcement in implementation)
- **Home**: Sets scroll to 0
- **End**: Sets scroll to max_lines
- Scroll offset affects which content lines are visible in viewport

### State Management

- `ViewerState` tracks:
  - `scroll`: Current scroll position (u16)
  - `view_mode`: Condensed or Full
  - Additional fields: `section`, `expanded`
- Resetting viewer state sets scroll to 0
- View mode toggle preserves scroll position
- Workflow data preserved during all navigation

## Running the Tests

```bash
# Run all viewer tests
cargo test --features tui --test tui_viewer_rendering_tests --test tui_viewer_keyboard_tests

# Run only rendering tests
cargo test --features tui --test tui_viewer_rendering_tests

# Run only keyboard tests
cargo test --features tui --test tui_viewer_keyboard_tests

# Run with output visible
cargo test --features tui --test tui_viewer_rendering_tests -- --nocapture

# Run specific test
cargo test --features tui --test tui_viewer_keyboard_tests test_tab_toggles_view_mode
```

## Test Coverage Summary

| Category | Tests | Status |
|----------|-------|--------|
| Basic Rendering | 3 | ✅ All Pass |
| Header Rendering | 3 | ✅ All Pass |
| Condensed View | 12 | ✅ All Pass |
| Full YAML View | 3 | ✅ All Pass |
| Status Bar | 3 | ✅ All Pass |
| ViewerState | 3 | ✅ All Pass |
| Scroll Display | 3 | ✅ All Pass |
| Layout & Sizing | 3 | ✅ All Pass |
| Theme Tests | 1 | ✅ All Pass |
| Edge Cases (Rendering) | 6 | ✅ All Pass |
| Navigation | 4 | ✅ All Pass |
| Arrow Key Scrolling | 4 | ✅ All Pass |
| Vim Key Scrolling | 4 | ✅ All Pass |
| Page Navigation | 4 | ✅ All Pass |
| Home/End | 3 | ✅ All Pass |
| Ignored Keys | 3 | ✅ All Pass |
| State Consistency | 3 | ✅ All Pass |
| Complex Workflows | 3 | ✅ All Pass |
| Edge Cases (Keyboard) | 4 | ✅ All Pass |
| **Total** | **72** | **✅ 100%** |

## Architecture Notes

### Test Backend Usage

Tests use `ratatui::backend::TestBackend` which provides:
- In-memory buffer for rendering
- Buffer inspection without actual terminal I/O
- Fast, deterministic test execution
- Cell-by-cell content verification

### State Management

Keyboard tests simulate the actual `handle_viewer_key` logic from `src/tui/app.rs`:
- View mode transitions (Viewer ↔ WorkflowList, Viewer → Editor)
- Scroll position management
- View mode toggle (Condensed ↔ Full)
- State reset on Esc

### Hexagonal Architecture Compliance

Tests validate the viewer system's compliance with hexagonal architecture:
- **Domain**: ViewerState, WorkflowViewMode enums
- **Primary Port**: Viewer rendering interface
- **Rendering**: Pure presentation logic with no side effects
- **State**: Clean separation of state and rendering

## Related Test Suites

- `tests/tui_modal_tests.rs` - Modal rendering tests (27 tests)
- `tests/tui_modal_keyboard_tests.rs` - Modal keyboard tests (34 tests)
- `tests/tui_editor_rendering_tests.rs` - Editor rendering tests (42 tests)
- `tests/tui_editor_keyboard_tests.rs` - Editor keyboard tests (50 tests)
- `tests/tui_unit_tests.rs` - General TUI component tests (18 tests)

**Combined TUI Test Coverage: 273 tests**

## Future Enhancements

Potential areas for additional testing:

1. **Workflow-Specific Scroll Max**: Calculate actual max lines based on workflow content
2. **Section Expansion**: Test collapsible sections in condensed view
3. **Search/Filter**: Test workflow content search if implemented
4. **Performance Tests**: Rendering benchmarks for large workflows
5. **Visual Regression**: Snapshot testing for pixel-perfect rendering
6. **Integration Tests**: Full viewer workflow from load to navigation to edit

## Related Files

- `src/tui/ui/viewer.rs` - Viewer component wrapper
- `src/tui/views/viewer.rs` - Viewer rendering implementation
- `src/tui/state.rs` - ViewerState and WorkflowViewMode definitions
- `src/tui/app.rs` - Viewer keyboard handling
- `src/tui/theme.rs` - Theme definitions
- `docs/TUI_MODAL_TESTS.md` - Modal test documentation
- `docs/TUI_EDITOR_TESTS.md` - Editor test documentation

---

**Test Suite Version**: 1.0
**Last Updated**: 2025-10-25
**Maintainer**: TUI Team

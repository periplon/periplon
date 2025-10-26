# TUI Workflow List Testing Suite

Comprehensive test coverage for the Workflow List screen rendering, keyboard handling, and workflow management operations.

## Overview

The workflow list testing suite consists of two main test files that provide complete coverage of the main workflow list view functionality:

1. **`tests/tui_workflow_list_rendering_tests.rs`** - Rendering and layout tests (25 tests)
2. **`tests/tui_workflow_list_keyboard_tests.rs`** - Keyboard interaction tests (33 tests)

**Total: 58 tests, all passing**

## Test Files

### Workflow List Rendering Tests (`tui_workflow_list_rendering_tests.rs`)

Tests the visual rendering, layout, workflow display, and empty state handling using ratatui's `TestBackend`.

#### Test Categories

**Basic Rendering (3 tests)**
- Basic workflow list screen rendering
- Layout structure (header, list, footer)
- Minimum terminal size handling (80x24)

**Workflow Display (3 tests)**
- Single workflow display
- Multiple workflows display (3+ items)
- Long workflow names handling

**Empty State (2 tests)**
- Empty workflow list with helpful message
- Keybinding hints for creation and quit

**Selection (4 tests)**
- First item selected
- Second item selected
- Last item selected
- Selection bounds validation (out of range handling)

**Footer/Keybindings (2 tests)**
- Footer keybindings display (Select, View, Edit, Generate, New, States, Help, Quit)
- Navigation key hints (↑↓, Enter)

**Theme Tests (1 test)**
- Rendering with all themes (default, light, monokai, solarized)

**Edge Cases (6 tests)**
- Many workflows (50+ workflows)
- Large terminal (200x100)
- Workflow with special characters
- Single line terminal (very small height)
- Narrow terminal (40 width)

**Content Validation (4 tests)**
- Header centered alignment
- Borders present (box drawing)
- Workflow list title display
- Selection marker presence/absence

### Keyboard Handling Tests (`tui_workflow_list_keyboard_tests.rs`)

Tests keyboard event processing for navigation, workflow actions, and view transitions.

#### Test Categories

**Navigation (4 tests)**
- Up arrow navigates up
- Down arrow navigates down
- Vim 'k' navigates up
- Vim 'j' navigates down

**Workflow Actions (4 tests)**
- Enter opens viewer
- 'e' opens editor
- Ctrl+D deletes workflow
- 'n' creates new workflow

**View Transitions (3 tests)**
- '?' shows help
- 'g' shows generator
- 's' shows states browser

**Search (1 test)**
- '/' starts search mode

**Quit (1 test)**
- 'q' confirms quit

**Ignored Keys (9 tests)**
- Regular characters (a-z, 0-9) ignored
- Function keys (F1-F12) ignored
- Page Up/Down ignored
- Home/End ignored
- Tab ignored
- Backspace ignored
- Delete ignored
- Escape ignored (workflow list is root view)
- Left/Right arrows ignored

**Modifier Combinations (2 tests)**
- 'd' without Ctrl ignored
- Ctrl combinations without 'd' ignored

**Edge Cases (3 tests)**
- Rapid navigation key presses
- Mixed navigation keys (arrows and vim keys)
- Rapid action key presses

**Action Priority (3 tests)**
- All workflow actions tested
- All view transitions tested
- All navigation keys tested

**Case Sensitivity (1 test)**
- Uppercase keys behavior validation

**Comprehensive Coverage (2 tests)**
- All defined actions have keys
- No unintended action triggers

## Key Implementation Details

### Layout Structure

```
┌───────────────────────────────────────┐
│      DSL Workflow Manager             │  Header (3 lines)
│                                       │  - Centered title
│                                       │  - Bordered box
├───────────────────────────────────────┤
│ Workflows                             │  List (min 5 lines)
│ ▶ workflow-1                          │  - Selected item (▶)
│   workflow-2                          │  - Unselected items
│   workflow-3                          │  - Highlighted selection
│                                       │
├───────────────────────────────────────┤
│ ↑↓ Select │ Enter View │ e Edit │... │  Footer (2 lines)
│ ... g Generate │ n New │ s States... │  - Keybindings
└───────────────────────────────────────┘
```

### Empty State Display

When no workflows are found:

```
No workflows found in current directory

Press n to create a new workflow
Press q to quit
```

### Workflow Entry Structure

```rust
pub struct WorkflowEntry {
    pub name: String,              // Display name
    pub path: PathBuf,             // Full file path
    pub description: Option<String>, // Optional description
    pub version: Option<String>,   // Workflow version
    pub valid: bool,               // Validation status
    pub errors: Vec<String>,       // Validation errors
}
```

### Keyboard Shortcuts

**Navigation**:
- **↑** / **k**: Move selection up
- **↓** / **j**: Move selection down

**Workflow Actions**:
- **Enter**: View selected workflow
- **e**: Edit selected workflow in editor
- **n**: Create new workflow
- **Ctrl+D**: Delete selected workflow (with confirmation)
- **/**: Search workflows

**View Transitions**:
- **?**: Show help screen
- **g**: Open workflow generator
- **s**: Open states browser

**System**:
- **q**: Quit application (with confirmation)

### Selection Behavior

- Selection wraps at bounds (can't go above first or below last)
- Selected item highlighted with different background color
- Selection marker (▶) shown on selected item
- Unselected items have no marker (just spacing)

## Test Utilities

### Rendering Utilities

```rust
/// Create test terminal
fn create_terminal(width: u16, height: u16) -> Terminal<TestBackend>

/// Render workflow list and return terminal
fn render_workflow_list(
    workflows: &[WorkflowEntry],
    selected: usize,
    theme: &Theme,
    width: u16,
    height: u16,
) -> Terminal<TestBackend>

/// Check if buffer contains text
fn buffer_contains(terminal: &Terminal<TestBackend>, text: &str) -> bool

/// Create sample workflow entry
fn create_workflow_entry(name: &str) -> WorkflowEntry
```

### Keyboard Utilities

```rust
/// Create KeyEvent
fn key(code: KeyCode) -> KeyEvent

/// Create KeyEvent with Ctrl
fn ctrl_key(code: KeyCode) -> KeyEvent

/// Simulated keyboard action results
enum WorkflowListAction {
    None,
    SelectUp,
    SelectDown,
    ViewWorkflow,
    EditWorkflow,
    DeleteWorkflow,
    CreateWorkflow,
    ShowHelp,
    ShowGenerator,
    ShowStates,
    StartSearch,
    ConfirmQuit,
}

/// Simulate workflow list key handling
fn handle_workflow_list_key_simulation(key_event: KeyEvent) -> WorkflowListAction
```

## Running the Tests

```bash
# Run all workflow list tests
cargo test --features tui --test tui_workflow_list_rendering_tests --test tui_workflow_list_keyboard_tests

# Run only rendering tests
cargo test --features tui --test tui_workflow_list_rendering_tests

# Run only keyboard tests
cargo test --features tui --test tui_workflow_list_keyboard_tests

# Run specific test
cargo test --features tui --test tui_workflow_list_keyboard_tests test_up_arrow_navigates_up

# Run with output visible
cargo test --features tui --test tui_workflow_list_rendering_tests -- --nocapture
```

## Test Coverage Summary

| Category | Tests | Status |
|----------|-------|--------|
| **RENDERING TESTS** | **25** | **✅** |
| Basic Rendering | 3 | ✅ All Pass |
| Workflow Display | 3 | ✅ All Pass |
| Empty State | 2 | ✅ All Pass |
| Selection | 4 | ✅ All Pass |
| Footer/Keybindings | 2 | ✅ All Pass |
| Theme Tests | 1 | ✅ All Pass |
| Edge Cases | 6 | ✅ All Pass |
| Content Validation | 4 | ✅ All Pass |
| **KEYBOARD TESTS** | **33** | **✅** |
| Navigation | 4 | ✅ All Pass |
| Workflow Actions | 4 | ✅ All Pass |
| View Transitions | 3 | ✅ All Pass |
| Search | 1 | ✅ All Pass |
| Quit | 1 | ✅ All Pass |
| Ignored Keys | 9 | ✅ All Pass |
| Modifier Combinations | 2 | ✅ All Pass |
| Edge Cases | 3 | ✅ All Pass |
| Action Priority | 3 | ✅ All Pass |
| Case Sensitivity | 1 | ✅ All Pass |
| Comprehensive Coverage | 2 | ✅ All Pass |
| **TOTAL** | **58** | **✅ 100%** |

## Architecture Compliance

### Hexagonal Architecture

**Domain**: Workflow management logic
- `WorkflowEntry` structure (state)
- Workflow list state management
- Selection tracking

**Primary Port**: Workflow list rendering interface
- `WorkflowListView::render()` function
- Pure presentation logic
- No state mutation

**Rendering**: Pure presentation
- Layout calculations (header 3, list min 5, footer 2)
- Selection highlighting
- Empty state messaging
- Visual feedback (▶ marker, highlighting)

**State Management**: Clean separation
- Keyboard handlers update state
- Rendering reads state
- No circular dependencies

## Features Tested

✅ Basic workflow list display
✅ Multiple workflows rendering
✅ Empty state with helpful messaging
✅ Selection highlighting (background color + marker)
✅ Keyboard navigation (arrows and vim keys)
✅ Workflow actions (View, Edit, Delete, Create)
✅ View transitions (Help, Generator, States)
✅ Search functionality
✅ Quit confirmation
✅ Theme support (all 4 themes)
✅ Layout responsiveness
✅ Edge case handling (many workflows, long names, small terminals)
✅ Border rendering
✅ Footer keybinding display
✅ Selection bounds validation

## Known Issues

None currently. All 58 tests pass successfully.

## Future Enhancements

Potential areas for additional testing:

1. **Search Filtering**: Test search functionality with results filtering
2. **Sort Options**: Test workflow sorting (name, date, etc.)
3. **Workflow Validation**: Test invalid workflow display (errors field)
4. **Pagination**: Test workflow list pagination with 100+ workflows
5. **Workflow Metadata**: Test description and version display
6. **Selection Persistence**: Test selection retention across view transitions
7. **Performance**: Benchmark with 1000+ workflows
8. **Visual Regression**: Snapshot testing for consistent rendering
9. **Integration**: Full workflow lifecycle (create → edit → view → delete)
10. **Error States**: Test error handling for missing/corrupted workflows

## Related Files

- `src/tui/ui/workflow_list.rs` - Workflow list rendering implementation (139 lines)
- `src/tui/state.rs` - WorkflowEntry and app state definitions
- `src/tui/app.rs` - Workflow list keyboard handling (handle_workflow_list_key)
- `src/tui/theme.rs` - Theme definitions
- `tests/tui_unit_tests.rs` - General TUI component tests
- `docs/TUI_VIEWER_TESTS.md` - Viewer testing documentation
- `docs/TUI_EDITOR_TESTS.md` - Editor testing documentation
- `docs/TUI_EXECUTION_MONITOR_TESTS.md` - Execution monitor testing documentation

## Testing Best Practices

1. **Use TestBackend**: Always use ratatui's TestBackend for rendering tests
2. **Test State Isolation**: Each test starts with fresh workflow list
3. **Selection Validation**: Verify selection bounds and highlighting
4. **Empty State**: Test both empty and populated states
5. **Theme Independence**: Test with all available themes
6. **Keyboard Modifiers**: Test Ctrl shortcuts separately from regular keys
7. **Edge Cases**: Test with 0 workflows, 1 workflow, many workflows
8. **Layout Flexibility**: Test with various terminal sizes
9. **Visual Elements**: Verify borders, markers, and alignment
10. **Documentation**: Keep test documentation synchronized with implementation

---

**Test Suite Version**: 1.0
**Last Updated**: 2025-10-25
**Maintainer**: TUI Team
**Coverage**: 58/58 tests passing (100%), 0 tests skipped

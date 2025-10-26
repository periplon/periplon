# TUI Editor Testing Suite

Comprehensive test coverage for the workflow editor screen rendering, keyboard handling, and text editing operations.

## Overview

The editor testing suite consists of two main test files that provide complete coverage of editor functionality:

1. **`tests/tui_editor_rendering_tests.rs`** - Rendering and layout tests (42 tests)
2. **`tests/tui_editor_keyboard_tests.rs`** - Keyboard interaction and text editing tests (50 tests)

**Total: 92 tests, all passing**

## Test Files

### Editor Rendering Tests (`tui_editor_rendering_tests.rs`)

Tests the visual rendering, layout, syntax highlighting, and validation feedback display using ratatui's `TestBackend`.

#### Test Categories

**Basic Editor Rendering (3 tests)**
- Basic rendering with content and borders
- Empty content handling
- File path display

**Header Rendering (4 tests)**
- Mode display (Text/Form)
- Validation status (valid/invalid with error count)
- Modified indicator
- File name in header

**Text Editor Mode Rendering (4 tests)**
- Line numbers with separator
- YAML syntax highlighting
- YAML Editor title
- Inline error/warning markers (❌/⚠️)

**Form Editor Mode Rendering (3 tests)**
- Form Editor title
- Valid workflow parsing and display (Metadata, Agents, Tasks sections)
- Invalid YAML error message

**Validation Panel (6 tests)**
- No errors message
- Error list with line numbers
- Warning list with line numbers
- Mixed errors and warnings
- Panel title
- Error/warning count display

**Status Bar (3 tests)**
- Text mode keyboard shortcuts
- Form mode keyboard shortcuts
- Modified indicator in status bar

**Layout Tests (3 tests)**
- Four-section layout structure (Header, Editor, Validation, Status)
- Minimum size handling (60x20)
- Large terminal adaptation (200x60)

**Content Rendering (4 tests)**
- Multiline content
- Long lines (200+ chars)
- Special characters and Unicode
- Empty lines

**Theme Tests (1 test)**
- Rendering with all themes (dark, light, monokai, solarized)

**Cursor Position Tests (3 tests)**
- Cursor at beginning (0,0)
- Cursor at end
- Cursor line highlighting

**Scroll Tests (2 tests)**
- Scroll offset handling
- Scrollbar with long content (↑/↓ indicators)

**Validation Feedback (2 tests)**
- Helper methods (is_valid, error_count, warning_count)
- Timestamp tracking

**Edge Cases (4 tests)**
- Very long error messages
- Many errors (50+)
- No file path handling
- EditorMode enum validation

### Keyboard Handling Tests (`tui_editor_keyboard_tests.rs`)

Tests keyboard event processing, text editing, cursor movement, and editor state management.

#### Test Categories

**Text Entry (6 tests)**
- Basic text entry ("hello")
- Multiline text entry with Enter
- Number entry
- Special characters (!@#$%^&*())
- Spaces
- Unicode characters (日本語)

**Backspace Tests (5 tests)**
- Basic backspace
- Backspace at line start (merges lines)
- Backspace on empty line
- Backspace at document start (no-op)
- Unicode character deletion

**Delete Key Tests (3 tests)**
- Basic delete
- Delete at line end (merges with next line)
- Delete at document end (no-op)

**Enter Key Tests (3 tests)**
- Enter at line end
- Enter middle of line (splits line)
- Enter at line start

**Cursor Movement (12 tests)**
- Left/Right movement
- Left at start (no-op)
- Right at end (no-op)
- Up/Down movement
- Up at top (no-op)
- Down at bottom (no-op)
- Up/Down with column adjustment for shorter lines
- Home key (move to line start)
- End key (move to line end)

**Keyboard Shortcuts (3 tests)**
- Ctrl+S (Save)
- Ctrl+R (Run)
- Escape (Exit)

**Scroll Tests (3 tests)**
- Page Up
- Page Up at top
- Page Down

**Modified Flag (4 tests)**
- Set on text entry
- Set on backspace
- Set on delete
- Not set on cursor movement

**Complex Editing Scenarios (4 tests)**
- Typing complete YAML workflow
- Insert text middle of line
- Delete word with multiple backspaces
- Navigate and edit across lines

**Edge Cases (3 tests)**
- Empty content operations
- Backspace on empty content
- Cursor movement on empty lines
- Very long line navigation (1000 chars)

**State Consistency (1 test)**
- Cursor position matches content after operations

**Mode Tests (2 tests)**
- EditorMode enum values
- EditorState defaults

## Key Implementation Details

### Editor Modes

**Text Mode**
- YAML syntax highlighting
  - Comments (gray italic)
  - Keys (bold accent color)
  - List items (accent dash)
  - Document separators
- Line numbers with separator (│)
- Inline validation markers
- Cursor highlighting (inverted colors on cursor position)
- Real-time error/warning display

**Form Mode**
- Structured field display
- Workflow metadata section
- Agents section with descriptions
- Tasks section with agent references
- Parse error handling

### Layout Structure

```
┌─────────────────────────────────────┐
│ Header (3 lines)                    │  - File name, mode, validation status
│ - File path                         │
│ - Mode indicator                    │
│ - Validation status                 │
├─────────────────────────────────────┤
│                                     │
│ Editor Content (flexible)           │  - Text or Form mode
│                                     │  - Syntax highlighting
│                                     │  - Line numbers
│                                     │  - Inline errors
│                                     │
├─────────────────────────────────────┤
│ Validation Panel (10 lines)         │  - Error list
│ - Errors with line numbers          │  - Warning list
│ - Warnings with line numbers        │  - Success message
├─────────────────────────────────────┤
│ Status Bar (1 line)                 │  - Keyboard shortcuts
└─────────────────────────────────────┘  - Modified indicator
```

### Syntax Highlighting Rules

- **Comments**: `# ...` → Gray + Italic
- **Keys**: `key:` → Bold + Accent color
- **List items**: `- item` → Accent colored dash
- **Values**: After `:` → Normal foreground
- **Document separators**: `---` / `...` → Border color

### Validation Display

**Inline Markers** (in Text Mode):
```
  5 │ tasks:
    │ ❌ Missing required field 'agent'
  6 │   test_task:
    │ ⚠️  Consider adding description
```

**Validation Panel**:
```
┌─ Validation ─────────────┐
│ Errors (2)               │
│   ❌ Line 5: Missing...  │
│   ❌ Line 12: Invalid... │
│                          │
│ Warnings (1)             │
│   ⚠️  Line 6: Consider... │
└──────────────────────────┘
```

### Text Editing Behavior

**Character Insertion**:
- Multi-byte character support (Unicode)
- Character-based cursor positioning
- Auto-extends lines as needed

**Line Operations**:
- Enter splits current line at cursor
- Backspace at line start merges with previous
- Delete at line end merges with next
- Proper handling of empty lines

**Cursor Movement**:
- Column adjustment when moving to shorter lines
- Bounds checking for all movements
- Home/End snap to line boundaries

**Scroll Management**:
- Page Up/Down by 10 lines
- Automatic scroll to keep cursor visible
- Scrollbar indicators for long content

## Test Utilities

### Rendering Utilities

```rust
/// Create test terminal
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend>

/// Render editor and return terminal
fn render_editor(
    state: &EditorState,
    feedback: &ValidationFeedback,
    theme: &Theme,
    width: u16,
    height: u16,
) -> Terminal<TestBackend>

/// Check if buffer contains text
fn buffer_contains_text(terminal: &Terminal<TestBackend>, text: &str) -> bool

/// Count border characters
fn count_border_chars(terminal: &Terminal<TestBackend>) -> usize
```

### Keyboard Utilities

```rust
/// Create KeyEvent
fn key(code: KeyCode) -> KeyEvent

/// Create KeyEvent with Ctrl
fn ctrl_key(code: KeyCode) -> KeyEvent

/// Simulate editor key handling
fn handle_editor_key_simulation(
    state: &mut EditorState,
    key_event: KeyEvent,
) -> EditorAction
```

## Running the Tests

```bash
# Run all editor tests
cargo test --features tui --test tui_editor_rendering_tests --test tui_editor_keyboard_tests

# Run only rendering tests
cargo test --features tui --test tui_editor_rendering_tests

# Run only keyboard tests
cargo test --features tui --test tui_editor_keyboard_tests

# Run specific test
cargo test --features tui --test tui_editor_keyboard_tests test_text_entry_unicode

# Run with output visible
cargo test --features tui --test tui_editor_rendering_tests -- --nocapture
```

## Test Coverage Summary

| Category | Tests | Status |
|----------|-------|--------|
| **RENDERING TESTS** | **42** | **✅** |
| Basic Editor Rendering | 3 | ✅ All Pass |
| Header Rendering | 4 | ✅ All Pass |
| Text Editor Mode | 4 | ✅ All Pass |
| Form Editor Mode | 3 | ✅ All Pass |
| Validation Panel | 6 | ✅ All Pass |
| Status Bar | 3 | ✅ All Pass |
| Layout Tests | 3 | ✅ All Pass |
| Content Rendering | 4 | ✅ All Pass |
| Theme Tests | 1 | ✅ All Pass |
| Cursor Position | 3 | ✅ All Pass |
| Scroll Tests | 2 | ✅ All Pass |
| Validation Feedback | 2 | ✅ All Pass |
| Edge Cases | 4 | ✅ All Pass |
| **KEYBOARD TESTS** | **50** | **✅** |
| Text Entry | 6 | ✅ All Pass |
| Backspace | 5 | ✅ All Pass |
| Delete Key | 3 | ✅ All Pass |
| Enter Key | 3 | ✅ All Pass |
| Cursor Movement | 12 | ✅ All Pass |
| Keyboard Shortcuts | 3 | ✅ All Pass |
| Scroll Tests | 3 | ✅ All Pass |
| Modified Flag | 4 | ✅ All Pass |
| Complex Scenarios | 4 | ✅ All Pass |
| Edge Cases | 4 | ✅ All Pass |
| State Consistency | 1 | ✅ All Pass |
| Mode Tests | 2 | ✅ All Pass |
| **TOTAL** | **92** | **✅ 100%** |

## Architecture Compliance

### Hexagonal Architecture

**Domain**: `EditorState`, `EditorMode`, `ValidationFeedback`
- Pure state and business logic
- No UI dependencies

**Primary Port**: Editor rendering interface
- `render()` function in `views/editor`
- `validate_and_get_feedback()`
- `get_autocomplete_suggestions()`

**Rendering**: Pure presentation
- Layout calculations
- Syntax highlighting
- Visual feedback
- No state mutation

**State Management**: Clean separation
- Keyboard handlers update state
- Rendering reads state
- Validation is separate service

## Features Tested

✅ Text Mode YAML editing with syntax highlighting
✅ Form Mode structured viewing
✅ Real-time validation feedback
✅ Inline error markers
✅ Line numbers
✅ Cursor positioning and highlighting
✅ Keyboard shortcuts (Ctrl+S, Ctrl+R, Esc)
✅ Text entry (ASCII, Unicode, special chars)
✅ Line editing (Insert, Delete, Backspace, Enter)
✅ Cursor movement (Arrow keys, Home, End, PgUp/PgDn)
✅ Modified flag tracking
✅ Scroll management
✅ Theme support
✅ Layout responsiveness
✅ Edge case handling

## Future Enhancements

Potential areas for additional testing:

1. **Auto-completion**: Test suggestion generation and selection
2. **Undo/Redo**: Test edit history management
3. **Search/Replace**: Test find and replace operations
4. **Multi-cursor**: Test simultaneous edits
5. **Selection**: Test text selection and operations
6. **Copy/Paste**: Test clipboard integration
7. **Syntax Error Recovery**: Test resilient parsing
8. **Performance**: Benchmark with very large files (10k+ lines)
9. **Visual Regression**: Snapshot testing for rendering
10. **Integration**: Full workflow editing scenarios

## Related Files

- `src/tui/views/editor.rs` - Editor rendering implementation
- `src/tui/ui/editor.rs` - Editor component wrapper
- `src/tui/state.rs` - EditorState definition
- `src/tui/app.rs` - Editor keyboard handling
- `src/tui/theme.rs` - Theme definitions
- `tests/tui_unit_tests.rs` - General TUI component tests
- `docs/TUI_MODAL_TESTS.md` - Modal testing documentation

## Known Limitations

1. **Implicit Empty Lines**: Content ending with `\n` creates an implicit empty line that `lines()` doesn't count. Tests account for this with cursor position validation.

2. **Multi-byte Characters**: Cursor positioning uses character counts, not byte offsets, to properly handle Unicode.

3. **Scrollbar**: Only shown when content exceeds viewport height.

4. **Form Mode**: Requires valid YAML; shows error message for parse failures.

## Testing Best Practices

1. **Use TestBackend**: Always use ratatui's TestBackend for rendering tests
2. **Test State Isolation**: Each test starts with fresh EditorState
3. **Unicode Support**: Include Unicode tests for international users
4. **Edge Cases**: Test boundaries, empty content, very long content
5. **State Consistency**: Verify cursor position matches content after edits
6. **Modified Flag**: Test that flag is set/cleared appropriately
7. **Keyboard Modifiers**: Test Ctrl shortcuts separately from regular keys

---

**Test Suite Version**: 1.0
**Last Updated**: 2025-10-25
**Maintainer**: TUI Team
**Coverage**: 92/92 tests passing (100%)

# TUI Test Utilities

Shared test utilities, helpers, and mock backends for TUI component testing. This module reduces code duplication and provides consistent testing patterns across all TUI tests.

## Overview

The `tests/common/` module provides reusable utilities for TUI testing:

- **Terminal utilities**: Terminal creation, rendering, buffer inspection
- **Keyboard utilities**: Keyboard event creation with various modifiers
- **Assertions**: Common test assertions for UI validation
- **Fixtures**: Test data builders and standard fixtures
- **Mocks**: Mock implementations for isolated testing

## Module Structure

```
tests/common/
├── mod.rs           # Module root with re-exports
├── terminal.rs      # Terminal utilities
├── keyboard.rs      # Keyboard event creation
├── assertions.rs    # Test assertions
├── fixtures.rs      # Test data and fixtures
└── mocks.rs         # Mock implementations
```

## Usage

### Importing Utilities

```rust
// In your test file
#![cfg(feature = "tui")]

// Import common utilities
use common::{
    create_terminal, buffer_contains,
    key, ctrl_key,
    assert_buffer_contains,
    create_all_themes,
};

// Or use the full path
use common::terminal::render_with_terminal;
use common::keyboard::alt_key;
```

## Terminal Utilities (`terminal.rs`)

### Creating Test Terminals

```rust
use common::terminal::create_terminal;

// Create a terminal with specific dimensions
let terminal = create_terminal(80, 24);
let terminal = create_terminal(120, 40);
```

### Rendering Components

```rust
use common::terminal::render_with_terminal;

// Render a component and return the terminal
let terminal = render_with_terminal(120, 40, |f| {
    MyView::render(f, f.area(), &state, &theme);
});
```

### Buffer Inspection

```rust
use common::terminal::{buffer_contains, buffer_content, buffer_lines};

// Check if buffer contains text
assert!(buffer_contains(&terminal, "Hello World"));

// Get full buffer content
let content = buffer_content(&terminal);

// Get buffer as lines
let lines = buffer_lines(&terminal);
assert!(lines[0].contains("Header"));

// Count occurrences
use common::terminal::count_in_buffer;
assert_eq!(count_in_buffer(&terminal, "task"), 3);

// Check specific line
use common::terminal::buffer_line_contains;
assert!(buffer_line_contains(&terminal, 5, "Status"));

// Get terminal size
use common::terminal::terminal_size;
let (width, height) = terminal_size(&terminal);
```

## Keyboard Utilities (`keyboard.rs`)

### Creating Key Events

```rust
use common::keyboard::{key, ctrl_key, alt_key, shift_key};

// Simple keys
let enter = key(KeyCode::Enter);
let esc = key(KeyCode::Esc);
let char_a = key(KeyCode::Char('a'));

// With modifiers
let ctrl_c = ctrl_key(KeyCode::Char('c'));
let alt_x = alt_key(KeyCode::Char('x'));
let shift_tab = shift_key(KeyCode::Tab);

// Multiple modifiers
use common::keyboard::{ctrl_shift_key, key_with_modifiers};
let ctrl_shift_z = ctrl_shift_key(KeyCode::Char('z'));

// Custom modifiers
let custom = key_with_modifiers(
    KeyCode::Char('a'),
    KeyModifiers::CONTROL | KeyModifiers::ALT
);
```

### Modifier Checking

```rust
use common::keyboard::{is_ctrl, is_alt, is_shift};

let event = ctrl_key(KeyCode::Char('c'));
assert!(is_ctrl(&event));
assert!(!is_alt(&event));
```

## Assertions (`assertions.rs`)

### Buffer Assertions

```rust
use common::assertions::{
    assert_buffer_contains,
    assert_buffer_not_contains,
    assert_buffer_contains_all,
    assert_buffer_contains_any,
    assert_buffer_contains_in_order,
};

// Single text assertion
assert_buffer_contains(&terminal, "Expected text");
assert_buffer_not_contains(&terminal, "Unexpected text");

// Multiple texts
assert_buffer_contains_all(&terminal, &["First", "Second", "Third"]);
assert_buffer_contains_any(&terminal, &["Option1", "Option2", "Option3"]);

// Ordered text
assert_buffer_contains_in_order(&terminal, &["Header", "Content", "Footer"]);
```

### Terminal Size Assertions

```rust
use common::assertions::{
    assert_terminal_size,
    assert_terminal_width,
    assert_terminal_height,
};

assert_terminal_size(&terminal, 80, 24);
assert_terminal_width(&terminal, 120);
assert_terminal_height(&terminal, 40);
```

## Fixtures (`fixtures.rs`)

### Theme Fixtures

```rust
use common::fixtures::{create_test_theme, create_all_themes, theme_by_name};

// Get default test theme
let theme = create_test_theme();

// Get all themes for iteration
let themes = create_all_themes();
for theme in themes {
    // Test with each theme
}

// Get specific theme by name
let light = theme_by_name("light");
let monokai = theme_by_name("monokai");
```

### Terminal Size Fixtures

```rust
use common::fixtures::terminal_sizes;

// Standard sizes
let (width, height) = terminal_sizes::MIN;        // 80x24
let (width, height) = terminal_sizes::STANDARD;   // 120x40
let (width, height) = terminal_sizes::LARGE;      // 200x100
let (width, height) = terminal_sizes::SMALL;      // 40x12
let (width, height) = terminal_sizes::NARROW;     // 60x24
let (width, height) = terminal_sizes::TALL;       // 80x60

// Test with all sizes
for (width, height) in terminal_sizes::all() {
    let terminal = create_terminal(width, height);
    // Test at each size
}
```

## Mocks (`mocks.rs`)

### Workflow Entry Builder

```rust
use common::mocks::WorkflowEntryBuilder;

// Build a workflow entry
let entry = WorkflowEntryBuilder::new("my-workflow")
    .description("Test workflow")
    .version("2.0.0")
    .build();

// Invalid workflow
let invalid = WorkflowEntryBuilder::new("broken-workflow")
    .with_error("Missing agent")
    .with_error("Invalid YAML")
    .build();

assert!(!invalid.valid);
assert_eq!(invalid.errors.len(), 2);
```

### DSL Workflow Builder

```rust
use common::mocks::DSLWorkflowBuilder;

let workflow = DSLWorkflowBuilder::new("test-workflow")
    .version("1.5.0")
    .dsl_version("2.0.0")
    .cwd(PathBuf::from("/workflows"))
    .build();
```

### Quick Helpers

```rust
use common::mocks::{
    mock_workflow_entry,
    mock_workflow_entries,
    mock_dsl_workflow,
};

// Single entry
let entry = mock_workflow_entry("quick-test");

// Multiple entries
let entries = mock_workflow_entries(&["one", "two", "three"]);

// Simple workflow
let workflow = mock_dsl_workflow("simple");
```

## Complete Example

```rust
#![cfg(feature = "tui")]

use common::{
    create_terminal, buffer_contains,
    key, ctrl_key,
    assert_buffer_contains, assert_terminal_size,
    create_all_themes, terminal_sizes,
    mock_workflow_entry,
};

#[test]
fn test_my_component() {
    // Create test data
    let workflow = mock_workflow_entry("test");
    let theme = create_all_themes()[0];

    // Create and render terminal
    let terminal = render_with_terminal(120, 40, |f| {
        MyView::render(f, f.area(), &workflow, &theme);
    });

    // Assert rendering
    assert_terminal_size(&terminal, 120, 40);
    assert_buffer_contains(&terminal, "test");
}

#[test]
fn test_keyboard_handling() {
    // Create key events
    let esc = key(KeyCode::Esc);
    let ctrl_s = ctrl_key(KeyCode::Char('s'));

    // Test keyboard handling
    let action1 = handle_key(esc);
    let action2 = handle_key(ctrl_s);

    assert_eq!(action1, Action::Exit);
    assert_eq!(action2, Action::Save);
}

#[test]
fn test_all_themes() {
    let workflow = mock_workflow_entry("test");

    for theme in create_all_themes() {
        let terminal = render_with_terminal(120, 40, |f| {
            MyView::render(f, f.area(), &workflow, &theme);
        });

        // Should render without panic
        assert_buffer_contains(&terminal, "test");
    }
}

#[test]
fn test_all_terminal_sizes() {
    let workflow = mock_workflow_entry("test");
    let theme = create_test_theme();

    for (width, height) in terminal_sizes::all() {
        let terminal = render_with_terminal(width, height, |f| {
            MyView::render(f, f.area(), &workflow, &theme);
        });

        assert_terminal_size(&terminal, width, height);
    }
}
```

## Benefits

### Reduced Duplication

Before:
```rust
// Every test file
fn create_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

fn buffer_contains(terminal: &Terminal<TestBackend>, text: &str) -> bool {
    let buffer = terminal.backend().buffer();
    let content: String = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .join("");
    content.contains(text)
}
```

After:
```rust
use common::{create_terminal, buffer_contains};
```

### Consistent Testing Patterns

All tests use the same utilities, making tests more readable and maintainable.

### Better Error Messages

Assertions provide detailed error messages with buffer content for debugging.

### Easier Test Writing

Quick helpers and builders reduce boilerplate code.

## Best Practices

1. **Use shared utilities**: Always prefer common utilities over duplicating code
2. **Import at module level**: Import utilities at the top of test files
3. **Use fixtures**: Use provided fixtures for themes and terminal sizes
4. **Use builders**: Use builders for complex test data
5. **Clear assertions**: Use specific assertions (e.g., `assert_buffer_contains` instead of `assert!(buffer_contains(...))`)
6. **Test isolation**: Each test should create its own terminal and state
7. **Theme testing**: Test with all themes using `create_all_themes()`
8. **Size testing**: Test with various sizes using `terminal_sizes::all()`

## Migration Guide

### Migrating Existing Tests

To migrate existing tests to use shared utilities:

1. **Add import**:
   ```rust
   use common::{create_terminal, buffer_contains, key, assert_buffer_contains};
   ```

2. **Remove duplicate utility functions** from test file

3. **Replace assertions**:
   ```rust
   // Before
   assert!(buffer_contains(&terminal, "text"));

   // After
   assert_buffer_contains(&terminal, "text");
   ```

4. **Use fixtures**:
   ```rust
   // Before
   let theme = Theme::default();

   // After
   use common::fixtures::create_test_theme;
   let theme = create_test_theme();
   ```

5. **Use builders**:
   ```rust
   // Before
   let entry = WorkflowEntry {
       name: "test".to_string(),
       path: PathBuf::from("/test.yaml"),
       description: None,
       version: Some("1.0.0".to_string()),
       valid: true,
       errors: Vec::new(),
   };

   // After
   use common::mocks::WorkflowEntryBuilder;
   let entry = WorkflowEntryBuilder::new("test").build();
   ```

## Testing the Utilities

The common module includes its own tests:

```bash
# Run all tests (utilities are tested inline)
cargo test --features tui

# The utilities have #[cfg(test)] blocks with their own tests
```

## API Reference

### Terminal Module

| Function | Description |
|----------|-------------|
| `create_terminal(w, h)` | Create test terminal |
| `render_with_terminal(w, h, f)` | Render and return terminal |
| `buffer_contains(t, text)` | Check if buffer contains text |
| `buffer_content(t)` | Get full buffer content |
| `buffer_lines(t)` | Get buffer as lines |
| `count_in_buffer(t, text)` | Count occurrences |
| `buffer_line_contains(t, line, text)` | Check specific line |
| `terminal_size(t)` | Get terminal dimensions |

### Keyboard Module

| Function | Description |
|----------|-------------|
| `key(code)` | Create simple key event |
| `ctrl_key(code)` | Create Ctrl+key event |
| `alt_key(code)` | Create Alt+key event |
| `shift_key(code)` | Create Shift+key event |
| `ctrl_shift_key(code)` | Create Ctrl+Shift+key |
| `alt_shift_key(code)` | Create Alt+Shift+key |
| `key_with_modifiers(code, mods)` | Create with custom modifiers |
| `is_ctrl(event)` | Check if has Ctrl |
| `is_alt(event)` | Check if has Alt |
| `is_shift(event)` | Check if has Shift |

### Assertions Module

| Function | Description |
|----------|-------------|
| `assert_buffer_contains(t, text)` | Assert text in buffer |
| `assert_buffer_not_contains(t, text)` | Assert text not in buffer |
| `assert_buffer_contains_all(t, texts)` | Assert all texts present |
| `assert_buffer_contains_any(t, texts)` | Assert any text present |
| `assert_buffer_contains_in_order(t, texts)` | Assert texts in order |
| `assert_terminal_size(t, w, h)` | Assert size |
| `assert_terminal_width(t, w)` | Assert width |
| `assert_terminal_height(t, h)` | Assert height |

### Fixtures Module

| Item | Description |
|------|-------------|
| `create_test_theme()` | Get default theme |
| `create_all_themes()` | Get all themes |
| `theme_by_name(name)` | Get specific theme |
| `theme_names()` | Get theme names |
| `terminal_sizes::MIN` | 80x24 |
| `terminal_sizes::STANDARD` | 120x40 |
| `terminal_sizes::LARGE` | 200x100 |
| `terminal_sizes::SMALL` | 40x12 |
| `terminal_sizes::NARROW` | 60x24 |
| `terminal_sizes::TALL` | 80x60 |
| `terminal_sizes::all()` | All sizes |

### Mocks Module

| Item | Description |
|------|-------------|
| `WorkflowEntryBuilder` | Build workflow entries |
| `DSLWorkflowBuilder` | Build DSL workflows |
| `mock_workflow_entry(name)` | Quick entry creation |
| `mock_workflow_entries(names)` | Multiple entries |
| `mock_dsl_workflow(name)` | Quick workflow |

---

**Version**: 1.0
**Last Updated**: 2025-10-25
**Maintainer**: TUI Team

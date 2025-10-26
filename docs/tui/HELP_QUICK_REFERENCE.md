# Help System Quick Reference

## For Users

### Opening Help
```
?          Open context-aware help
F1         Open help
Esc        Close help / go back
q          Quit help
```

### Navigation
```
↑/↓        Move up/down
k/j        Vim-style up/down
Enter      Select/Open
/          Search
Tab        Switch panels
```

### Scrolling
```
PageUp     Scroll page up
PageDown   Scroll page down
Home       Scroll to top
End        Scroll to bottom
```

## For Developers

### Basic Usage

```rust
use crate::tui::help::{HelpView, HelpViewState, HelpContext};

// Create help view
let help_view = HelpView::new();

// Create state with context
let mut help_state = HelpViewState::new(HelpContext::General);

// Render
help_view.render(frame, area, &mut help_state, &theme);
```

### Context Management

```rust
// Update context based on current view
state.update_help_context();

// Or set explicitly
state.help_state.set_context(HelpContext::Editor);
```

### Navigation API

```rust
// Browse mode
help_state.next_category();
help_state.prev_category();
help_state.next_topic();
help_state.prev_topic();
help_state.select();

// Search mode
help_state.enter_search_mode();
help_state.update_search("query".to_string());

// Topic mode
help_state.scroll_up();
help_state.scroll_down();
help_state.page_up();
help_state.page_down();
help_state.go_back();
```

### Adding Help Content

```rust
// In src/tui/help/content.rs, inside initialize_content()
self.add_topic(HelpTopic {
    id: "unique_id".to_string(),
    title: "Display Title".to_string(),
    content: r#"
# Title

Content in **markdown** format.

## Subsection

- Lists work
- Tables work
- `Code` works

```rust
// Code blocks work too
fn example() {}
```
"#.to_string(),
    related: vec!["related_id".to_string()],
    keywords: vec!["search", "keywords".to_string()],
    category: HelpCategory::GettingStarted,
});
```

### Search Examples

```rust
use crate::tui::help::{HelpContent, HelpSearchEngine};

let content = HelpContent::new();
let engine = HelpSearchEngine::new(content);

// Basic search
let results = engine.search("workflow");

// Process results
for result in results {
    println!("{}: {:.0}%", result.topic.title, result.score * 100.0);
    println!("  {}", result.excerpt);
}

// Get suggestions
let suggestions = engine.suggest("work");
```

### Markdown Rendering

```rust
use crate::tui::help::MarkdownRenderer;

let renderer = MarkdownRenderer::new();
let markdown = "# Title\n\nContent with **bold**";
let text = renderer.render(markdown);

// Use with ratatui
frame.render_widget(Paragraph::new(text), area);
```

### Custom Contexts

```rust
// Add to HelpContext enum
pub enum HelpContext {
    MyNewView,
    // ...
}

// Implement topics
impl HelpContext {
    pub fn topics(&self) -> Vec<&'static str> {
        match self {
            HelpContext::MyNewView => vec!["topic1", "topic2"],
            // ...
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            HelpContext::MyNewView => "My View Help",
            // ...
        }
    }
}
```

### Testing

```bash
# Run all help tests
cargo test --lib help

# Run specific test
cargo test test_search_engine

# With output
cargo test help -- --nocapture
```

## Help Content Categories

```rust
pub enum HelpCategory {
    GettingStarted,      // Intro and quick start
    WorkflowManagement,  // Creating/managing workflows
    Editing,             // Editor and YAML
    Execution,           // Running workflows
    KeyboardShortcuts,   // All shortcuts
    Advanced,            // Advanced features
    Troubleshooting,     // Common issues
}
```

## Available Contexts

```rust
pub enum HelpContext {
    WorkflowList,      // Workflow list view
    Viewer,            // Workflow viewer
    Editor,            // Workflow editor
    ExecutionMonitor,  // Execution monitor
    Generator,         // AI generator
    General,           // General help
}
```

## Markdown Features

### Supported
- Headers: `# H1` through `###### H6`
- Bold: `**text**`
- Italic: `*text*`
- Code: `` `code` ``
- Code blocks: ` ```...``` `
- Lists: `- item` or `1. item`
- Tables: `| col1 | col2 |`
- Blockquotes: `> quote`
- Links: `[text](url)`

### Styling
- Headers: Cyan with bold (h1-h2), Blue (h3-h6)
- Code: Green on dark gray background
- Bold: Bold modifier
- Italic: Italic modifier
- Links: Light cyan with underline
- Quotes: Gray with italic

## Performance Tips

1. **Search**: Queries < 3 chars return no results (optimization)
2. **Content**: Keep topics focused and concise
3. **Keywords**: Add relevant keywords for better search
4. **Related**: Link related topics for easy navigation
5. **Caching**: Renderer caches are automatic

## Common Patterns

### Context-Aware Help Button

```rust
// In event handler
KeyCode::F1 | KeyCode::Char('?') => {
    state.update_help_context();
    state.view_mode = ViewMode::Help;
}
```

### Search from Current View

```rust
KeyCode::Char('/') if !in_help => {
    state.update_help_context();
    state.view_mode = ViewMode::Help;
    state.help_state.enter_search_mode();
}
```

### Topic Quick Jump

```rust
// Jump to specific topic
if let Some(topic) = content.get_topic("topic_id") {
    help_state.view_topic(topic.clone());
}
```

## Troubleshooting

### Help not showing
1. Check `help_state` is initialized in `AppState::new()`
2. Verify help view is rendered in view router
3. Check `ViewMode::Help` is handled

### Search not working
1. Verify `HelpSearchEngine` initialization
2. Check topics have content and keywords
3. Try simple single-word query

### Markdown issues
1. Validate markdown syntax
2. Check for unescaped special characters
3. Test rendering in isolation

## File Locations

```
src/tui/help/
├── mod.rs           # Public API
├── content.rs       # Help database
├── markdown.rs      # Renderer
├── search.rs        # Search engine
├── view.rs          # View component
└── tests.rs         # Tests

docs/tui/
├── overview.md              # User docs
├── help-system.md           # Developer docs
└── IMPLEMENTATION_SUMMARY.md # Summary
```

## Quick Links

- Architecture: `docs/tui/help-system.md`
- User Guide: `docs/tui/overview.md`
- Implementation: `docs/tui/IMPLEMENTATION_SUMMARY.md`
- Source: `src/tui/help/`

---

**Quick Reference Version**: 1.0.0
**Last Updated**: 2025-10-21

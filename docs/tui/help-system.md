# TUI Help System Documentation

The DSL TUI includes a comprehensive, searchable help and documentation system with context-aware assistance.

## Architecture

The help system follows a modular architecture with clear separation of concerns:

```
src/tui/help/
├── mod.rs           # Public API and help context
├── content.rs       # Help content database
├── markdown.rs      # Markdown rendering engine
├── search.rs        # Full-text search engine
├── view.rs          # Interactive help view component
└── tests.rs         # Comprehensive test suite
```

## Components

### 1. Help Content Database (`content.rs`)

Structured help content with topics organized by category.

#### HelpTopic

Each help topic contains:
- **id**: Unique identifier (e.g., "getting_started")
- **title**: Display title
- **content**: Markdown-formatted documentation
- **related**: List of related topic IDs
- **keywords**: Search keywords for improved discoverability
- **category**: Topic category for organization

#### Categories

Topics are organized into categories:
- **Getting Started**: Introductory guides and quick start
- **Workflow Management**: Creating and managing workflows
- **Editing**: Workflow editing and YAML syntax
- **Execution**: Running and monitoring workflows
- **Keyboard Shortcuts**: Complete shortcut reference
- **Advanced**: Advanced features and configuration
- **Troubleshooting**: Common issues and solutions

### 2. Markdown Renderer (`markdown.rs`)

Converts markdown to styled ratatui text with support for:

- **Headers** (h1-h6) with distinct styling
- **Text formatting**: **bold**, *italic*, `inline code`
- **Code blocks** with syntax highlighting
- **Lists**: Ordered and unordered
- **Tables** with column separators
- **Blockquotes** with visual indicators
- **Links** with underlined styling

#### Example

```rust
use crate::tui::help::MarkdownRenderer;

let renderer = MarkdownRenderer::new();
let text = renderer.render("# Welcome\n\nThis is **bold** text.");
```

### 3. Search Engine (`search.rs`)

Full-text search with inverted indexing for fast lookups.

#### Features

- **Full-text indexing**: All content is indexed for fast search
- **Fuzzy matching**: Prefix matching for partial queries
- **Relevance scoring**: Results ranked by relevance (0.0 - 1.0)
- **Context excerpts**: Shows matching context from content
- **Search suggestions**: Auto-complete based on indexed terms

#### Search Algorithm

1. Tokenize query into search terms
2. Look up terms in inverted index
3. Calculate relevance scores:
   - Exact match: 1.0 point
   - Prefix match: 0.5 points
4. Normalize scores and sort by relevance
5. Extract context excerpts from matching content

#### Example

```rust
use crate::tui::help::{HelpContent, HelpSearchEngine};

let content = HelpContent::new();
let engine = HelpSearchEngine::new(content);

let results = engine.search("keyboard shortcuts");
for result in results {
    println!("{}: {:.0}%", result.topic.title, result.score * 100.0);
}
```

### 4. Help View (`view.rs`)

Interactive help view with three modes:

#### Browse Mode

Navigate topics by category:
- Left panel: Category list
- Right panel: Topics in selected category
- `Enter`: Open selected topic

#### Topic Mode

View topic content with markdown rendering:
- Breadcrumb navigation showing path
- Scrollable content
- Related topics at bottom
- `Esc`: Go back

#### Search Mode

Search all help content:
- Live search as you type
- Results with relevance scores
- Context excerpts showing matches
- `Enter`: Open selected result

### 5. Help Context (`mod.rs`)

Context-aware help based on current TUI view:

```rust
pub enum HelpContext {
    WorkflowList,      // Help for workflow list view
    Viewer,            // Help for workflow viewer
    Editor,            // Help for editor view
    ExecutionMonitor,  // Help for execution monitor
    Generator,         // Help for AI generator
    General,           // General help
}
```

Each context provides relevant topics for that view.

## Usage

### Integrating Help into TUI

The help system is integrated into `AppState`:

```rust
use crate::tui::help::{HelpContext, HelpViewState};

pub struct AppState {
    // ... other fields
    pub help_state: HelpViewState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            // ... other initialization
            help_state: HelpViewState::new(HelpContext::General),
        }
    }

    pub fn update_help_context(&mut self) {
        let context = match self.view_mode {
            ViewMode::WorkflowList => HelpContext::WorkflowList,
            ViewMode::Editor => HelpContext::Editor,
            // ... other mappings
        };
        self.help_state.set_context(context);
    }
}
```

### Rendering Help View

```rust
use crate::tui::help::HelpView;
use crate::tui::theme::Theme;

let help_view = HelpView::new();
help_view.render(frame, area, &mut state.help_state, &theme);
```

### Navigation API

```rust
// Browse mode
state.help_state.next_category();
state.help_state.prev_category();
state.help_state.next_topic();
state.help_state.prev_topic();
state.help_state.select(); // Open topic

// Search mode
state.help_state.enter_search_mode();
state.help_state.update_search("query".to_string());

// Topic mode
state.help_state.scroll_up();
state.help_state.scroll_down();
state.help_state.page_up();
state.help_state.page_down();
state.help_state.go_back();
```

## Keyboard Shortcuts

When in help view:

### Global
- `?` or `F1`: Open/close help
- `Esc`: Go back / close help
- `q`: Close help

### Browse Mode
- `↑/↓` or `k/j`: Navigate categories/topics
- `Enter`: Open selected topic
- `/`: Enter search mode
- `Tab`: Switch between categories and topics

### Topic Mode
- `↑/↓` or `k/j`: Scroll content
- `PageUp/PageDown`: Scroll by page
- `Home/End`: Scroll to top/bottom
- `Esc`: Go back to browse

### Search Mode
- Type to search
- `↑/↓` or `k/j`: Navigate results
- `Enter`: Open selected result
- `Esc`: Return to browse mode

## Adding New Help Content

To add a new help topic:

1. Open `src/tui/help/content.rs`
2. In `initialize_content()`, add a new topic:

```rust
self.add_topic(HelpTopic {
    id: "my_topic".to_string(),
    title: "My Topic".to_string(),
    content: r#"
# My Topic

Topic content in markdown...

## Section

More content...
"#.to_string(),
    related: vec!["related_topic".to_string()],
    keywords: vec!["keyword1".to_string(), "keyword2".to_string()],
    category: HelpCategory::Advanced,
});
```

3. The topic will automatically be indexed for search
4. It will appear in the appropriate category

## Performance

### Indexing
- Initial indexing: O(n) where n = total words in all topics
- Typical time: < 10ms for ~50 topics

### Search
- Query time: O(m * log n) where m = query terms, n = indexed terms
- Typical time: < 1ms for most queries
- Supports hundreds of topics efficiently

### Rendering
- Markdown parsing: O(k) where k = content length
- Typical time: < 5ms for typical help topics
- Lazy rendering for large documents

## Testing

Comprehensive test suite in `src/tui/help/tests.rs`:

```bash
# Run all help system tests
cargo test --lib help

# Run specific test
cargo test test_search_engine_basic

# Run with output
cargo test -- --nocapture
```

Test coverage includes:
- Content initialization and retrieval
- Search functionality and relevance scoring
- Markdown rendering for all supported features
- View state management and navigation
- Context switching
- Clone implementation

## Customization

### Theming

Help view uses the application theme:

```rust
pub struct Theme {
    pub accent: Color,      // Headers, highlights
    pub text: Color,        // Normal text
    pub border: Color,      // Borders
    // ... other colors
}
```

### Custom Markdown Styles

Customize markdown rendering:

```rust
let mut renderer = MarkdownRenderer::new();
// Customize styles as needed
renderer.header_styles[0] = Style::default()
    .fg(Color::Cyan)
    .add_modifier(Modifier::BOLD);
```

## Future Enhancements

Potential improvements:
- [ ] Incremental search with live updates
- [ ] History navigation (back/forward buttons)
- [ ] Bookmarking favorite topics
- [ ] Export help content to markdown files
- [ ] Interactive code examples with syntax highlighting
- [ ] Screenshot/diagram support in help content
- [ ] External documentation links
- [ ] Offline PDF export

## Troubleshooting

### Help content not showing
- Verify `docs/tui/overview.md` exists
- Check help system initialization in `AppState::new()`
- Ensure help view is properly rendered

### Search not working
- Check search index initialization
- Verify topics have keywords defined
- Test with simple single-word queries first

### Markdown rendering issues
- Verify markdown syntax is valid
- Check for special characters that need escaping
- Test rendering in isolation

## Architecture Decisions

### Why separate content database?
- Enables easy content updates without code changes
- Supports future loading from external files
- Simplifies testing and content management

### Why custom markdown renderer?
- Full control over styling for TUI
- Optimized for terminal rendering
- No external dependencies for parsing

### Why inverted index for search?
- O(1) lookup time for indexed terms
- Scales well to large help databases
- Enables fuzzy matching and suggestions

---

**Version**: 1.0.0
**Last Updated**: 2025-10-21

# DSL TUI Developer Guide

Comprehensive guide for developers working on the DSL TUI codebase.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Workflow](#development-workflow)
3. [Architecture Overview](#architecture-overview)
4. [Adding Features](#adding-features)
5. [Testing](#testing)
6. [Debugging](#debugging)
7. [Performance Optimization](#performance-optimization)
8. [Contributing](#contributing)

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo
- Basic understanding of async Rust
- Familiarity with terminal applications

### Development Setup

```bash
# Clone repository
git clone <repo-url>
cd periplon

# Build with TUI feature
cargo build --features tui

# Run in development mode
cargo run --bin periplon-tui --features tui -- --debug

# Run tests
cargo test --features tui

# Run TUI-specific tests
cargo test --test tui_unit_tests --features tui
cargo test --test tui_integration_tests --features tui
```

### Development Tools

**Recommended:**
- **rust-analyzer**: LSP for IDE integration
- **cargo-watch**: Auto-rebuild on changes
- **cargo-expand**: View macro expansions
- **cargo-flamegraph**: Performance profiling

```bash
# Install dev tools
cargo install cargo-watch cargo-expand cargo-flamegraph

# Auto-rebuild and run
cargo watch -x 'run --bin periplon-tui --features tui'

# View macro expansions
cargo expand --features tui tui::ui::components

# Profile performance
cargo flamegraph --bin periplon-tui --features tui
```

### Project Structure

```
src/tui/
├── mod.rs              # Public API and feature flags
├── app.rs              # Main application loop
├── config.rs           # Configuration management
├── domain/             # Business logic (pure)
├── ports/              # Interface definitions
├── application/        # Use case orchestration
├── adapters/           # External system integration
├── ui/                 # Presentation layer
├── state/              # State management
└── help/               # Documentation system
```

## Development Workflow

### Starting Development

1. **Create feature branch:**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make changes following architecture:**
   - Domain first (pure logic)
   - Ports second (interfaces)
   - Adapters third (implementations)
   - UI last (presentation)

3. **Test as you go:**
   ```bash
   cargo test --features tui
   ```

4. **Run the TUI:**
   ```bash
   cargo run --bin periplon-tui --features tui -- --debug
   ```

5. **Check for issues:**
   ```bash
   cargo clippy --features tui
   cargo fmt --check
   ```

### Code Style

Follow Rust conventions and project patterns:

```rust
// ✓ Good: Clear names, proper error handling
pub async fn load_workflow(
    &self,
    path: &Path,
) -> Result<Workflow, LoadError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| LoadError::FileRead {
            path: path.to_owned(),
            source: e
        })?;

    serde_yaml::from_str(&content)
        .map_err(|e| LoadError::ParseYaml {
            path: path.to_owned(),
            source: e
        })
}

// ✗ Bad: Unclear names, unwraps, generic errors
pub fn load(p: &Path) -> Result<Workflow, Box<dyn Error>> {
    let s = std::fs::read_to_string(p).unwrap();
    Ok(serde_yaml::from_str(&s)?)
}
```

### Commit Messages

Use conventional commits:

```
feat: add workflow state persistence
fix: resolve rendering race condition
docs: update architecture documentation
test: add integration tests for file manager
refactor: extract editor component
perf: optimize log rendering
```

## Architecture Overview

### Hexagonal Architecture

The TUI strictly follows hexagonal architecture (ports & adapters):

```
External → Primary Adapter → Primary Port → Application → Domain
                                                    ↓
External ← Secondary Adapter ← Secondary Port ←────┘
```

**Key Principles:**

1. **Domain is Pure**: No external dependencies
2. **Ports Define Contracts**: Interfaces, not implementations
3. **Adapters are Pluggable**: Easy to swap implementations
4. **Dependencies Point Inward**: Toward domain

### Layer Responsibilities

#### Domain Layer (`domain/`)

Pure business logic with zero dependencies:

```rust
// domain/models.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowFile {
    pub id: WorkflowId,
    pub path: PathBuf,
    pub name: String,
    pub metadata: FileMetadata,
}

impl WorkflowFile {
    pub fn new(path: PathBuf) -> Self {
        // Pure logic, no I/O
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        // Business rules only
    }
}
```

#### Ports Layer (`ports/`)

Interface definitions (traits):

```rust
// ports/secondary/workflow_repo.rs
#[async_trait]
pub trait WorkflowRepository {
    async fn find_all(&self, dir: &Path) -> Result<Vec<WorkflowFile>>;
    async fn find_by_id(&self, id: WorkflowId) -> Result<Option<WorkflowFile>>;
    async fn save(&self, workflow: &WorkflowFile) -> Result<()>;
    async fn delete(&self, id: WorkflowId) -> Result<()>;
}
```

#### Application Layer (`application/`)

Use case orchestration:

```rust
// application/services/workflow_manager.rs
pub struct WorkflowManager {
    repository: Arc<dyn WorkflowRepository>,
    validator: Arc<dyn Validator<Workflow>>,
}

impl WorkflowManager {
    pub async fn create_workflow(
        &self,
        name: String,
    ) -> Result<WorkflowId> {
        // 1. Create domain object
        // 2. Validate
        // 3. Save via repository
        // 4. Emit event
    }
}
```

#### Adapters Layer (`adapters/`)

External system implementations:

```rust
// adapters/secondary/file_workflow_repo.rs
pub struct FileWorkflowRepository {
    base_dir: PathBuf,
}

#[async_trait]
impl WorkflowRepository for FileWorkflowRepository {
    async fn find_all(&self, dir: &Path) -> Result<Vec<WorkflowFile>> {
        // File system I/O implementation
    }
}
```

#### UI Layer (`ui/`)

Presentation and rendering:

```rust
// ui/views/workflow_browser.rs
pub struct WorkflowBrowser {
    state: BrowserState,
}

impl WorkflowBrowser {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Ratatui rendering
    }

    pub fn handle_event(&mut self, event: Event) -> EventResult {
        // User input handling
    }
}
```

## Adding Features

### Adding a New View

1. **Create view file:**

```rust
// ui/views/my_view.rs
use ratatui::{Frame, layout::Rect};
use crossterm::event::{Event, KeyCode, KeyEvent};

pub struct MyView {
    // View state
    selected_index: usize,
    items: Vec<String>,
}

impl MyView {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            items: Vec::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Render logic using ratatui widgets
        use ratatui::widgets::{Block, Borders, List, ListItem};

        let items: Vec<ListItem> = self.items
            .iter()
            .map(|i| ListItem::new(i.as_str()))
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title("My View")
                .borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn handle_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Up => {
                    self.selected_index = self.selected_index.saturating_sub(1);
                    EventResult::Handled
                }
                KeyCode::Down => {
                    self.selected_index = (self.selected_index + 1)
                        .min(self.items.len().saturating_sub(1));
                    EventResult::Handled
                }
                KeyCode::Enter => {
                    // Handle selection
                    EventResult::Handled
                }
                _ => EventResult::Ignored,
            },
            _ => EventResult::Ignored,
        }
    }
}
```

2. **Register in app.rs:**

```rust
// app.rs
pub enum View {
    WorkflowBrowser,
    WorkflowEditor,
    ExecutionMonitor,
    StateBrowser,
    Help,
    MyView,  // Add new view
}

pub struct App {
    // ...existing fields...
    my_view: MyView,  // Add view instance
}

impl App {
    fn render(&mut self, frame: &mut Frame) {
        match self.current_view {
            // ...existing views...
            View::MyView => self.my_view.render(frame, frame.size()),
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<EventResult> {
        match self.current_view {
            // ...existing views...
            View::MyView => self.my_view.handle_event(event),
        }
    }
}
```

3. **Add navigation:**

```rust
// app.rs - in handle_event
KeyCode::Tab => {
    self.next_view();  // Add MyView to rotation
    Ok(EventResult::Handled)
}
```

### Adding a New Secondary Port

1. **Define port trait:**

```rust
// ports/secondary/my_service.rs
#[async_trait]
pub trait MyService: Send + Sync {
    async fn do_something(&self, input: &str) -> Result<String>;
}
```

2. **Implement adapter:**

```rust
// adapters/secondary/my_service_impl.rs
pub struct MyServiceImpl {
    config: Config,
}

#[async_trait]
impl MyService for MyServiceImpl {
    async fn do_something(&self, input: &str) -> Result<String> {
        // Implementation
        Ok(format!("Processed: {}", input))
    }
}
```

3. **Wire up in app:**

```rust
// app.rs
pub struct App {
    my_service: Arc<dyn MyService>,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let my_service = Arc::new(MyServiceImpl::new(config.clone()));
        // ...
    }
}
```

### Adding a New Component

1. **Create component:**

```rust
// ui/components/my_component.rs
use ratatui::{widgets::Widget, buffer::Buffer, layout::Rect};

pub struct MyComponent<'a> {
    title: &'a str,
    content: &'a str,
}

impl<'a> MyComponent<'a> {
    pub fn new(title: &'a str, content: &'a str) -> Self {
        Self { title, content }
    }
}

impl<'a> Widget for MyComponent<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Rendering logic
    }
}
```

2. **Use in view:**

```rust
// ui/views/some_view.rs
use super::super::components::my_component::MyComponent;

pub fn render(&self, frame: &mut Frame, area: Rect) {
    let component = MyComponent::new("Title", "Content");
    frame.render_widget(component, area);
}
```

## Testing

### Unit Tests

Test domain logic and pure functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_validation() {
        let workflow = Workflow {
            name: "test".into(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
        };

        assert!(workflow.validate().is_ok());
    }

    #[test]
    fn test_invalid_workflow() {
        let workflow = Workflow::default();
        assert!(workflow.validate().is_err());
    }
}
```

### Integration Tests

Test adapter implementations:

```rust
// tests/tui_integration_tests.rs
#[tokio::test]
async fn test_file_repository() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = FileWorkflowRepository::new(temp_dir.path());

    // Create test workflow
    let workflow = WorkflowFile::new(
        temp_dir.path().join("test.yaml")
    );

    // Save
    repo.save(&workflow).await.unwrap();

    // Load
    let loaded = repo.find_by_id(workflow.id).await.unwrap();
    assert_eq!(loaded, Some(workflow));
}
```

### UI Tests

Test view behavior:

```rust
#[test]
fn test_workflow_browser_navigation() {
    let mut browser = WorkflowBrowser::new();

    // Add test data
    browser.load_workflows(vec![
        WorkflowFile::new("test1.yaml".into()),
        WorkflowFile::new("test2.yaml".into()),
    ]);

    // Test navigation
    let result = browser.handle_event(Event::Key(
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)
    ));

    assert_eq!(result, EventResult::Handled);
    assert_eq!(browser.selected_index(), 1);
}
```

### Mock Testing

Use mocks for testing without dependencies:

```rust
// tests/mocks.rs
pub struct MockWorkflowRepository {
    workflows: Arc<Mutex<HashMap<WorkflowId, WorkflowFile>>>,
}

#[async_trait]
impl WorkflowRepository for MockWorkflowRepository {
    async fn find_all(&self, _dir: &Path) -> Result<Vec<WorkflowFile>> {
        Ok(self.workflows.lock().unwrap().values().cloned().collect())
    }

    async fn save(&self, workflow: &WorkflowFile) -> Result<()> {
        self.workflows.lock().unwrap().insert(workflow.id, workflow.clone());
        Ok(())
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test --features tui

# Run specific test
cargo test test_workflow_browser --features tui

# Run with output
cargo test --features tui -- --nocapture

# Run integration tests only
cargo test --test tui_integration_tests --features tui

# Run with coverage
cargo tarpaulin --features tui --out Html
```

## Debugging

### Debug Mode

Enable debug logging:

```bash
# Run with debug flag
cargo run --bin periplon-tui --features tui -- --debug

# Check debug log
tail -f ~/.claude-sdk/tui-debug.log
```

### Debug Logging in Code

```rust
use tracing::{debug, info, warn, error};

pub async fn load_workflow(&self, path: &Path) -> Result<Workflow> {
    debug!("Loading workflow from {:?}", path);

    let content = tokio::fs::read_to_string(path).await?;
    debug!("Read {} bytes", content.len());

    let workflow = serde_yaml::from_str(&content)?;
    info!("Loaded workflow: {}", workflow.name);

    Ok(workflow)
}
```

### Debugging UI Issues

1. **Enable frame debugging:**

```rust
// app.rs - in render loop
if self.config.debug {
    debug!("Rendering view: {:?}", self.current_view);
    debug!("Frame size: {:?}", frame.size());
}
```

2. **Check terminal state:**

```bash
# Reset terminal if corrupted
reset

# Check terminal size
tput cols
tput lines
```

3. **Use alternate screen buffer:**

```rust
// Prevents output corruption
terminal::enable_raw_mode()?;
execute!(stdout, EnterAlternateScreen)?;

// Clean up on exit
execute!(stdout, LeaveAlternateScreen)?;
terminal::disable_raw_mode()?;
```

### Panic Handling

Ensure clean terminal restoration on panic:

```rust
// main.rs
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |panic_info| {
    // Restore terminal
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen
    );

    // Call original hook
    original_hook(panic_info);
}));
```

## Performance Optimization

### Rendering Performance

1. **Minimize redraws:**

```rust
// Only redraw when state changes
if self.state_changed {
    self.render(frame, area);
    self.state_changed = false;
}
```

2. **Use incremental rendering:**

```rust
// Only redraw changed regions
if self.list_changed {
    frame.render_widget(self.list_widget(), list_area);
    self.list_changed = false;
}
```

3. **Cache expensive calculations:**

```rust
// Cache layout calculations
if self.layout_cache.is_none() || area != self.last_area {
    self.layout_cache = Some(Layout::default()
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area));
    self.last_area = area;
}
```

### Memory Optimization

1. **Limit log retention:**

```rust
// Keep only last N log entries
const MAX_LOG_ENTRIES: usize = 1000;

if self.logs.len() > MAX_LOG_ENTRIES {
    self.logs.drain(0..self.logs.len() - MAX_LOG_ENTRIES);
}
```

2. **Stream large files:**

```rust
// Don't load entire file into memory
use tokio::io::{AsyncBufReadExt, BufReader};

let file = File::open(path).await?;
let reader = BufReader::new(file);
let mut lines = reader.lines();

while let Some(line) = lines.next_line().await? {
    process_line(line);
}
```

### Async Optimization

1. **Use buffered channels:**

```rust
// Prevent backpressure
let (tx, rx) = tokio::sync::mpsc::channel(100);
```

2. **Batch operations:**

```rust
// Batch file operations
let mut handles = Vec::new();
for path in paths {
    handles.push(tokio::spawn(async move {
        load_workflow(&path).await
    }));
}

let results = futures::future::join_all(handles).await;
```

### Profiling

```bash
# Generate flamegraph
cargo flamegraph --bin periplon-tui --features tui

# Profile with perf
perf record -g ./target/release/periplon-tui
perf report
```

## Contributing

### Pull Request Process

1. **Fork and create branch:**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Follow architecture:**
   - Domain first
   - Ports second
   - Adapters third
   - UI last

3. **Add tests:**
   - Unit tests for domain
   - Integration tests for adapters
   - UI tests for views

4. **Document changes:**
   - Update relevant docs
   - Add inline comments
   - Update CHANGELOG

5. **Submit PR:**
   - Clear description
   - Link related issues
   - Include test results

### Code Review Checklist

- [ ] Follows hexagonal architecture
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Formatted with rustfmt
- [ ] Documentation updated
- [ ] Performance considered
- [ ] Error handling proper
- [ ] No unwrap() in production code

### Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG
3. Tag release: `git tag -a v1.0.0`
4. Build release: `cargo build --release --features tui`
5. Test release binary
6. Push tag: `git push --tags`

## Resources

- [Ratatui Documentation](https://ratatui.rs/)
- [Crossterm Guide](https://docs.rs/crossterm/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)

---

**Version**: 1.0.0
**Last Updated**: 2025-10-21

# TUI REPL Implementation Roadmap

**Project:** DSL TUI REPL Development
**Version:** 1.0.0
**Date:** 2025-10-21
**Purpose:** Detailed implementation roadmap with milestones and acceptance criteria
**Based On:** architecture_analysis.md, architecture_design.md

---

## Overview

This roadmap breaks down the TUI REPL implementation into concrete phases with clear deliverables, dependencies, and acceptance criteria. Each phase builds upon the previous, following the hexagonal architecture principles established in the design documents.

---

## Phase 1: Foundation & Runtime (Week 1-2)

### Goals
- Establish basic TUI infrastructure
- Implement core runtime and event loop
- Set up state management foundation

### Tasks

#### 1.1 Project Setup
**Files:**
- `src/adapters/primary/tui/mod.rs`
- `src/adapters/primary/tui/lib.rs`
- `Cargo.toml` (verify TUI feature dependencies)

**Implementation:**
```rust
// src/adapters/primary/tui/mod.rs
pub mod runtime;
pub mod state;
pub mod ui;
pub mod input;
pub mod domain;
pub mod channels;
pub mod utils;

pub use runtime::TuiApp;
pub use state::AppState;

// Feature-gated re-export
#[cfg(feature = "tui")]
pub async fn run_tui(options: TuiOptions) -> crate::Result<()> {
    let app = TuiApp::new(options)?;
    app.run().await
}
```

**Acceptance Criteria:**
- ✅ TUI module compiles with `--features tui`
- ✅ Dependencies (ratatui, crossterm) properly configured
- ✅ Public API exports defined

#### 1.2 State Management Foundation
**Files:**
- `src/adapters/primary/tui/state/mod.rs`
- `src/adapters/primary/tui/state/manager.rs`
- `src/adapters/primary/tui/state/session.rs`
- `src/adapters/primary/tui/state/view.rs`
- `src/adapters/primary/tui/state/history.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/state/manager.rs
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub session: Arc<RwLock<SessionState>>,
    pub view: Arc<RwLock<ViewState>>,
    pub history: Arc<RwLock<HistoryState>>,
    pub config: Arc<RwLock<TuiConfig>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            session: Arc::new(RwLock::new(SessionState::default())),
            view: Arc::new(RwLock::new(ViewState::default())),
            history: Arc::new(RwLock::new(HistoryState::default())),
            config: Arc::new(RwLock::new(TuiConfig::default())),
        }
    }

    // Helper methods for common state updates
    pub async fn add_message(&self, session_id: SessionId, msg: Message) {
        let mut history = self.history.write().await;
        history.add_message(session_id, msg);
    }

    pub async fn set_active_panel(&self, panel: PanelId) {
        let mut view = self.view.write().await;
        view.focus.active_panel = Some(panel);
    }
}
```

**Acceptance Criteria:**
- ✅ State structs compile and follow Arc<RwLock> pattern
- ✅ Helper methods for common state updates
- ✅ Unit tests for state management (>80% coverage)

#### 1.3 Event Channel System
**Files:**
- `src/adapters/primary/tui/channels/mod.rs`
- `src/adapters/primary/tui/channels/types.rs`
- `src/adapters/primary/tui/channels/message_channel.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/channels/types.rs
use crate::domain::Message;
use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
pub enum TuiEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    DomainMessage { session_id: SessionId, message: Message },
    WorkflowEvent { workflow_id: WorkflowId, event: WorkflowEventType },
    CommandResult { command: String, result: Result<(), String> },
    Tick,
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum WorkflowEventType {
    Started,
    TaskStarted(String),
    TaskCompleted(String),
    TaskFailed { task: String, error: String },
    ProgressUpdate { completed: usize, total: usize },
    Completed,
    Failed(String),
}
```

**Acceptance Criteria:**
- ✅ Event types cover all necessary communication
- ✅ Channel creation and message passing tested
- ✅ No circular dependencies

#### 1.4 Basic Event Loop
**Files:**
- `src/adapters/primary/tui/runtime/mod.rs`
- `src/adapters/primary/tui/runtime/event_loop.rs`
- `src/adapters/primary/tui/runtime/app.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/runtime/event_loop.rs
use tokio::time::{interval, Duration};
use crossterm::event::{EventStream, Event};
use futures::StreamExt;

pub struct EventLoop {
    event_rx: mpsc::UnboundedReceiver<TuiEvent>,
    state: Arc<AppState>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl EventLoop {
    pub async fn run(mut self) -> Result<()> {
        let mut crossterm_events = EventStream::new();
        let mut tick_interval = interval(Duration::from_millis(250));

        loop {
            tokio::select! {
                Some(Ok(event)) = crossterm_events.next() => {
                    match event {
                        Event::Key(key) => self.handle_key(key).await?,
                        Event::Resize(w, h) => self.handle_resize(w, h).await?,
                        _ => {}
                    }
                }

                Some(tui_event) = self.event_rx.recv() => {
                    if matches!(tui_event, TuiEvent::Shutdown) {
                        break;
                    }
                    self.handle_tui_event(tui_event).await?;
                }

                _ = tick_interval.tick() => {
                    self.render().await?;
                }
            }
        }

        Ok(())
    }
}
```

**Acceptance Criteria:**
- ✅ Event loop runs without panicking
- ✅ Handles shutdown gracefully
- ✅ Terminal cleanup on exit
- ✅ Integration test verifies event flow

#### 1.5 Terminal Utilities
**Files:**
- `src/adapters/primary/tui/utils/mod.rs`
- `src/adapters/primary/tui/utils/terminal.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/utils/terminal.rs
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(Into::into)
}

pub fn cleanup_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
```

**Acceptance Criteria:**
- ✅ Terminal setup/cleanup works without errors
- ✅ Panic handler restores terminal state
- ✅ Works on macOS, Linux, Windows

### Phase 1 Deliverables
- [ ] Basic TUI application that starts and shuts down cleanly
- [ ] Event loop processing keyboard input
- [ ] State management infrastructure in place
- [ ] Terminal utilities functional
- [ ] All unit tests passing (>80% coverage)

---

## Phase 2: Basic UI & Panel System (Week 3-4)

### Goals
- Implement panel trait system
- Create basic chat panel
- Implement status bar
- Basic rendering working

### Tasks

#### 2.1 Panel Trait System
**Files:**
- `src/adapters/primary/tui/ui/panels/mod.rs`
- `src/adapters/primary/tui/ui/panels/traits.rs`
- `src/adapters/primary/tui/ui/panels/registry.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/ui/panels/traits.rs
use ratatui::{Frame, layout::Rect};
use crossterm::event::KeyEvent;

pub trait Panel: Send + Sync {
    fn id(&self) -> PanelId;
    fn name(&self) -> &str;
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState);
    fn handle_input(&mut self, key: KeyEvent, state: &AppState) -> Result<InputResult>;
    fn update(&mut self, state: &AppState) -> Result<()>;
    fn can_focus(&self) -> bool { true }
    fn min_size(&self) -> (u16, u16) { (20, 5) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    Chat,
    Workflow,
    Tasks,
    Thinking,
    Logs,
    Help,
    StatusBar,
}

pub enum InputResult {
    Handled,
    NotHandled,
    SwitchPanel(PanelId),
    ExecuteCommand(String),
}
```

**Acceptance Criteria:**
- ✅ Panel trait compiles and is object-safe
- ✅ PanelRegistry can store and retrieve panels
- ✅ Focus management works between panels

#### 2.2 Chat Panel (MVP)
**Files:**
- `src/adapters/primary/tui/ui/panels/chat.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/ui/panels/chat.rs
pub struct ChatPanel {
    id: PanelId,
    scroll_offset: usize,
    auto_scroll: bool,
}

impl Panel for ChatPanel {
    fn id(&self) -> PanelId { PanelId::Chat }
    fn name(&self) -> &str { "Chat" }

    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        let history = state.history.blocking_read();
        let session = state.session.blocking_read();

        if let Some(session_id) = session.active_session {
            if let Some(messages) = history.messages.get(&session_id) {
                let items: Vec<ListItem> = messages
                    .messages
                    .iter()
                    .skip(self.scroll_offset)
                    .map(|msg| self.render_message(msg))
                    .collect();

                let list = List::new(items)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title("Chat"));

                frame.render_widget(list, area);
            }
        }
    }

    fn handle_input(&mut self, key: KeyEvent, _state: &AppState) -> Result<InputResult> {
        match key.code {
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                Ok(InputResult::Handled)
            }
            KeyCode::Down => {
                self.scroll_offset += 1;
                Ok(InputResult::Handled)
            }
            _ => Ok(InputResult::NotHandled)
        }
    }
}
```

**Acceptance Criteria:**
- ✅ Displays messages from history
- ✅ Scrolling works (up/down/page up/page down)
- ✅ Auto-scroll to bottom on new messages
- ✅ Handles empty history gracefully

#### 2.3 Status Bar
**Files:**
- `src/adapters/primary/tui/ui/panels/status_bar.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/ui/panels/status_bar.rs
pub struct StatusBar {
    id: PanelId,
}

impl Panel for StatusBar {
    fn id(&self) -> PanelId { PanelId::StatusBar }
    fn name(&self) -> &str { "Status" }
    fn can_focus(&self) -> bool { false }

    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        let session = state.session.blocking_read();
        let view = state.view.blocking_read();

        let status_text = format!(
            " Session: {} | Panel: {:?} | Press F1 for help | q to quit ",
            session.active_session
                .map(|id| id.to_string())
                .unwrap_or_else(|| "None".to_string()),
            view.focus.active_panel
        );

        let paragraph = Paragraph::new(status_text)
            .style(Style::default().bg(Color::Blue).fg(Color::White));

        frame.render_widget(paragraph, area);
    }
}
```

**Acceptance Criteria:**
- ✅ Shows active session
- ✅ Shows focused panel
- ✅ Updates in real-time
- ✅ Displays keyboard shortcuts

#### 2.4 Layout Manager
**Files:**
- `src/adapters/primary/tui/ui/layout/mod.rs`
- `src/adapters/primary/tui/ui/layout/manager.rs`
- `src/adapters/primary/tui/ui/layout/presets.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/ui/layout/manager.rs
pub struct LayoutManager {
    current_layout: LayoutPreset,
}

impl LayoutManager {
    pub fn compute_layout(&self, terminal_size: Rect) -> HashMap<PanelId, Rect> {
        match self.current_layout {
            LayoutPreset::Default => self.layout_default(terminal_size),
            LayoutPreset::Minimal => self.layout_minimal(terminal_size),
            _ => HashMap::new(),
        }
    }

    fn layout_default(&self, area: Rect) -> HashMap<PanelId, Rect> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),    // Chat
                Constraint::Length(1),  // Status bar
            ])
            .split(area);

        let mut map = HashMap::new();
        map.insert(PanelId::Chat, chunks[0]);
        map.insert(PanelId::StatusBar, chunks[1]);
        map
    }
}
```

**Acceptance Criteria:**
- ✅ Default layout renders chat + status bar
- ✅ Layout adapts to terminal resize
- ✅ Minimum size constraints respected

#### 2.5 UI Renderer
**Files:**
- `src/adapters/primary/tui/ui/mod.rs`
- `src/adapters/primary/tui/ui/renderer.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/ui/renderer.rs
pub struct Renderer {
    layout_manager: LayoutManager,
    panels: HashMap<PanelId, Box<dyn Panel>>,
}

impl Renderer {
    pub fn render(&mut self, frame: &mut Frame, state: &AppState) -> Result<()> {
        let terminal_size = frame.size();
        let layout = self.layout_manager.compute_layout(terminal_size);

        // Render each panel in its area
        for (panel_id, area) in layout {
            if let Some(panel) = self.panels.get_mut(&panel_id) {
                panel.render(frame, area, state);
            }
        }

        Ok(())
    }
}
```

**Acceptance Criteria:**
- ✅ Renders all visible panels
- ✅ No visual artifacts or flickering
- ✅ Performance: renders in <16ms

### Phase 2 Deliverables
- [ ] Working chat panel displaying messages
- [ ] Status bar showing session info
- [ ] Panel system with trait-based design
- [ ] Layout manager handling terminal resize
- [ ] Basic rendering pipeline functional

---

## Phase 3: PeriplonSDKClient Integration (Week 5-6)

### Goals
- Integrate with PeriplonSDKClient
- Handle message streaming
- Display real-time responses
- Implement input handling

### Tasks

#### 3.1 Domain Adapters
**Files:**
- `src/adapters/primary/tui/domain/mod.rs`
- `src/adapters/primary/tui/domain/client_adapter.rs`
- `src/adapters/primary/tui/domain/message_adapter.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/domain/client_adapter.rs
use crate::adapters::primary::PeriplonSDKClient;
use crate::domain::Message;

pub struct ClientAdapter {
    client: PeriplonSDKClient,
    session_id: SessionId,
    event_tx: mpsc::UnboundedSender<TuiEvent>,
}

impl ClientAdapter {
    pub async fn new(
        options: AgentOptions,
        session_id: SessionId,
        event_tx: mpsc::UnboundedSender<TuiEvent>,
    ) -> Result<Self> {
        let mut client = PeriplonSDKClient::new(options);
        client.connect(None).await?;

        Ok(Self {
            client,
            session_id,
            event_tx,
        })
    }

    pub async fn send_query(&mut self, prompt: String) -> Result<()> {
        self.client.query(prompt).await?;

        // Spawn task to stream messages
        let session_id = self.session_id;
        let event_tx = self.event_tx.clone();
        let stream = self.client.receive_response()?;

        tokio::spawn(async move {
            futures::pin_mut!(stream);
            while let Some(msg) = stream.next().await {
                let _ = event_tx.send(TuiEvent::DomainMessage {
                    session_id,
                    message: msg,
                });
            }
        });

        Ok(())
    }
}
```

**Acceptance Criteria:**
- ✅ ClientAdapter wraps PeriplonSDKClient
- ✅ Messages stream to TUI via channels
- ✅ Error handling for connection failures
- ✅ Integration test with MockTransport

#### 3.2 Message Rendering
**Files:**
- `src/adapters/primary/tui/ui/widgets/mod.rs`
- `src/adapters/primary/tui/ui/widgets/message.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/ui/widgets/message.rs
pub struct MessageWidget<'a> {
    message: &'a DisplayMessage,
    show_timestamp: bool,
    show_thinking: bool,
}

impl<'a> MessageWidget<'a> {
    pub fn render(&self) -> ListItem<'a> {
        let spans = match &self.message.message {
            Message::User(user_msg) => {
                self.render_user_message(user_msg)
            }
            Message::Assistant(assistant_msg) => {
                self.render_assistant_message(assistant_msg)
            }
            Message::System(system_msg) => {
                self.render_system_message(system_msg)
            }
            Message::Result(result_msg) => {
                self.render_result_message(result_msg)
            }
            _ => vec![],
        };

        ListItem::new(Line::from(spans))
    }

    fn render_assistant_message(&self, msg: &AssistantMessage) -> Vec<Span<'a>> {
        let mut spans = vec![
            Span::styled("Assistant: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        ];

        for block in &msg.message.content {
            match block {
                ContentBlock::Text { text } => {
                    spans.push(Span::raw(text.clone()));
                }
                ContentBlock::Thinking { thinking, .. } if self.show_thinking => {
                    spans.push(Span::styled(
                        format!("[Thinking: {}]", thinking),
                        Style::default().fg(Color::Cyan).italic()
                    ));
                }
                ContentBlock::ToolUse { name, .. } => {
                    spans.push(Span::styled(
                        format!("[Using tool: {}]", name),
                        Style::default().fg(Color::Yellow)
                    ));
                }
                _ => {}
            }
        }

        spans
    }
}
```

**Acceptance Criteria:**
- ✅ Renders all message types correctly
- ✅ Applies appropriate styling (colors, bold)
- ✅ Handles tool use blocks
- ✅ Shows/hides thinking based on config

#### 3.3 Input Handler
**Files:**
- `src/adapters/primary/tui/input/mod.rs`
- `src/adapters/primary/tui/input/handler.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/input/handler.rs
pub struct InputHandler {
    input_buffer: String,
    cursor_position: usize,
    command_mode: bool,
}

impl InputHandler {
    pub async fn handle_key(
        &mut self,
        key: KeyEvent,
        state: &AppState,
        event_tx: &mpsc::UnboundedSender<TuiEvent>,
    ) -> Result<()> {
        match key.code {
            KeyCode::Char(c) if !self.command_mode => {
                self.input_buffer.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            KeyCode::Char('/') if self.input_buffer.is_empty() => {
                self.command_mode = true;
                self.input_buffer.push('/');
                self.cursor_position = 1;
            }
            KeyCode::Enter => {
                let input = self.input_buffer.drain(..).collect::<String>();
                self.cursor_position = 0;
                self.command_mode = false;

                if input.starts_with('/') {
                    // Handle command
                    let result = self.execute_command(&input, state).await;
                    event_tx.send(TuiEvent::CommandResult {
                        command: input,
                        result,
                    })?;
                } else {
                    // Send query to active session
                    self.send_query(input, state).await?;
                }
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input_buffer.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn send_query(&self, prompt: String, state: &AppState) -> Result<()> {
        let session = state.session.read().await;
        if let Some(session_id) = session.active_session {
            if let Some(client_ctx) = session.clients.get(&session_id) {
                // Send query via ClientAdapter
                // (This would be stored in session state)
            }
        }
        Ok(())
    }
}
```

**Acceptance Criteria:**
- ✅ Text input works with cursor movement
- ✅ Enter sends query to active session
- ✅ Backspace/delete work correctly
- ✅ Command mode activated with '/'

### Phase 3 Deliverables
- [ ] Full integration with PeriplonSDKClient
- [ ] Messages stream and display in real-time
- [ ] User can send queries via input field
- [ ] Message rendering supports all content types
- [ ] Integration tests with real CLI subprocess

---

## Phase 4: REPL Commands (Week 7-8)

### Goals
- Implement command parser
- Add session management commands
- Add workflow commands
- Implement help system

### Tasks

#### 4.1 Command Parser
**Files:**
- `src/adapters/primary/tui/input/commands/mod.rs`
- `src/adapters/primary/tui/input/commands/parser.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/input/commands/parser.rs
#[derive(Debug, Clone)]
pub enum ReplCommand {
    // Session commands
    NewSession,
    SwitchSession(SessionId),
    CloseSession(SessionId),
    ListSessions,

    // Workflow commands
    LoadWorkflow(PathBuf),
    ValidateWorkflow(Option<PathBuf>),
    RunWorkflow(Option<PathBuf>),
    StopWorkflow,

    // View commands
    SwitchLayout(LayoutPreset),
    TogglePanel(PanelId),
    ToggleThinking,

    // History commands
    ClearHistory,
    SaveHistory(PathBuf),

    // System commands
    Help(Option<String>),
    Quit,
}

pub fn parse_command(input: &str) -> Result<ReplCommand> {
    let input = input.trim_start_matches('/');
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return Err(Error::InvalidCommand("Empty command".to_string()));
    }

    match parts[0] {
        "new" => Ok(ReplCommand::NewSession),
        "switch" if parts.len() > 1 => {
            let id = parts[1].parse()?;
            Ok(ReplCommand::SwitchSession(id))
        }
        "load" if parts.len() > 1 => {
            Ok(ReplCommand::LoadWorkflow(PathBuf::from(parts[1])))
        }
        "run" => {
            let path = parts.get(1).map(PathBuf::from);
            Ok(ReplCommand::RunWorkflow(path))
        }
        "help" => {
            let topic = parts.get(1).map(|s| s.to_string());
            Ok(ReplCommand::Help(topic))
        }
        "quit" | "exit" => Ok(ReplCommand::Quit),
        _ => Err(Error::InvalidCommand(format!("Unknown command: {}", parts[0]))),
    }
}
```

**Acceptance Criteria:**
- ✅ Parses all command types
- ✅ Handles arguments correctly
- ✅ Provides helpful error messages
- ✅ Unit tests for all commands

#### 4.2 Command Registry
**Files:**
- `src/adapters/primary/tui/input/commands/registry.rs`
- `src/adapters/primary/tui/input/commands/builtin.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/input/commands/registry.rs
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn CommandHandler>>,
}

pub trait CommandHandler: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str;

    fn execute(
        &self,
        args: Vec<String>,
        state: &AppState,
    ) -> Pin<Box<dyn Future<Output = Result<CommandResult>> + Send + '_>>;
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };

        // Register built-in commands
        registry.register(Box::new(NewSessionCommand));
        registry.register(Box::new(QuitCommand));
        registry.register(Box::new(HelpCommand));
        // ... more commands

        registry
    }

    pub fn register(&mut self, handler: Box<dyn CommandHandler>) {
        self.commands.insert(handler.name().to_string(), handler);
    }

    pub async fn execute(
        &self,
        command: &str,
        args: Vec<String>,
        state: &AppState,
    ) -> Result<CommandResult> {
        if let Some(handler) = self.commands.get(command) {
            handler.execute(args, state).await
        } else {
            Err(Error::InvalidCommand(format!("Unknown command: {}", command)))
        }
    }
}
```

**Acceptance Criteria:**
- ✅ Command registration works
- ✅ Command execution delegates correctly
- ✅ Can list all available commands
- ✅ Extensible for custom commands

#### 4.3 Built-in Commands
**Files:**
- `src/adapters/primary/tui/input/commands/builtin.rs`

**Implementation:**
```rust
// NewSessionCommand
pub struct NewSessionCommand;

impl CommandHandler for NewSessionCommand {
    fn name(&self) -> &str { "new" }
    fn description(&self) -> &str { "Create a new chat session" }
    fn usage(&self) -> &str { "/new" }

    fn execute(
        &self,
        _args: Vec<String>,
        state: &AppState,
    ) -> Pin<Box<dyn Future<Output = Result<CommandResult>> + Send + '_>> {
        Box::pin(async move {
            let session_id = SessionId::new();

            // Create new ClientAdapter
            let options = AgentOptions::default();
            let event_tx = /* get from state */;
            let adapter = ClientAdapter::new(options, session_id, event_tx).await?;

            // Add to session state
            let mut session = state.session.write().await;
            session.clients.insert(session_id, ClientContext {
                id: session_id,
                adapter,
                status: SessionStatus::Active,
                created_at: Instant::now(),
            });
            session.active_session = Some(session_id);

            Ok(CommandResult {
                success: true,
                message: format!("Created new session: {}", session_id),
                data: None,
            })
        })
    }
}

// QuitCommand
pub struct QuitCommand;

impl CommandHandler for QuitCommand {
    fn name(&self) -> &str { "quit" }
    fn description(&self) -> &str { "Exit the TUI REPL" }
    fn usage(&self) -> &str { "/quit or /exit" }

    fn execute(
        &self,
        _args: Vec<String>,
        _state: &AppState,
    ) -> Pin<Box<dyn Future<Output = Result<CommandResult>> + Send + '_>> {
        Box::pin(async move {
            // Send shutdown event
            // (event_tx would be passed in via state)
            Ok(CommandResult {
                success: true,
                message: "Shutting down...".to_string(),
                data: None,
            })
        })
    }
}
```

**Acceptance Criteria:**
- ✅ All built-in commands implemented
- ✅ Commands modify state correctly
- ✅ Error handling for edge cases
- ✅ Integration tests for each command

#### 4.4 Help System
**Files:**
- `src/adapters/primary/tui/ui/panels/help.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/ui/panels/help.rs
pub struct HelpPanel {
    id: PanelId,
    current_topic: Option<String>,
    scroll_offset: usize,
}

impl Panel for HelpPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        let help_text = if let Some(topic) = &self.current_topic {
            self.get_topic_help(topic)
        } else {
            self.get_general_help()
        };

        let paragraph = Paragraph::new(help_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Help"))
            .scroll((self.scroll_offset as u16, 0));

        frame.render_widget(paragraph, area);
    }

    fn get_general_help(&self) -> String {
        r#"
TUI REPL Commands
=================

Session Management:
  /new              - Create a new chat session
  /switch <id>      - Switch to session <id>
  /close <id>       - Close session <id>
  /list             - List all sessions

Workflow Management:
  /load <file>      - Load workflow from file
  /validate [file]  - Validate workflow
  /run [file]       - Run workflow
  /stop             - Stop running workflow

View Control:
  /layout <preset>  - Switch layout (default, workflow, minimal)
  /toggle <panel>   - Toggle panel visibility
  /thinking         - Toggle extended thinking display

Keyboard Shortcuts:
  F1                - Toggle help
  Ctrl+C            - Interrupt current query
  Ctrl+D            - Quit
  Tab               - Switch panel focus

Type any text and press Enter to send a query to the active session.
        "#.to_string()
    }
}
```

**Acceptance Criteria:**
- ✅ Help panel displays command reference
- ✅ Context-sensitive help for topics
- ✅ Scrollable content
- ✅ Accessible via /help and F1

### Phase 4 Deliverables
- [ ] Command parser handling all command types
- [ ] Command registry with extensibility
- [ ] All built-in commands implemented
- [ ] Help system with comprehensive documentation
- [ ] Commands tested with integration tests

---

## Phase 5: Workflow Visualization (Week 9-10)

### Goals
- Integrate DSLExecutor
- Display task graph
- Show real-time progress
- Implement task panel

### Tasks

#### 5.1 DSLExecutor Integration
**Files:**
- `src/adapters/primary/tui/domain/executor_adapter.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/domain/executor_adapter.rs
pub struct ExecutorAdapter {
    executor: DSLExecutor,
    workflow: DSLWorkflow,
    workflow_id: WorkflowId,
    event_tx: mpsc::UnboundedSender<TuiEvent>,
}

impl ExecutorAdapter {
    pub async fn new(
        workflow: DSLWorkflow,
        workflow_id: WorkflowId,
        event_tx: mpsc::UnboundedSender<TuiEvent>,
    ) -> Result<Self> {
        let executor = DSLExecutor::new(workflow.clone())?;

        Ok(Self {
            executor,
            workflow,
            workflow_id,
            event_tx,
        })
    }

    pub async fn execute(&mut self) -> Result<()> {
        let workflow_id = self.workflow_id;
        let event_tx = self.event_tx.clone();

        // Send start event
        event_tx.send(TuiEvent::WorkflowEvent {
            workflow_id,
            event: WorkflowEventType::Started,
        })?;

        // Execute workflow with progress callbacks
        // (This would need to be added to DSLExecutor)
        let result = self.executor.execute().await;

        // Send completion event
        match result {
            Ok(_) => {
                event_tx.send(TuiEvent::WorkflowEvent {
                    workflow_id,
                    event: WorkflowEventType::Completed,
                })?;
            }
            Err(e) => {
                event_tx.send(TuiEvent::WorkflowEvent {
                    workflow_id,
                    event: WorkflowEventType::Failed(e.to_string()),
                })?;
            }
        }

        Ok(())
    }
}
```

**Acceptance Criteria:**
- ✅ DSLExecutor wrapped and integrated
- ✅ Workflow events stream to TUI
- ✅ Progress updates in real-time
- ✅ Error handling for workflow failures

#### 5.2 Workflow Panel
**Files:**
- `src/adapters/primary/tui/ui/panels/workflow.rs`
- `src/adapters/primary/tui/ui/widgets/tree.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/ui/panels/workflow.rs
pub struct WorkflowPanel {
    id: PanelId,
    selected_task: Option<String>,
}

impl Panel for WorkflowPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        let session = state.session.blocking_read();

        if let Some(workflow_ctx) = session.workflows.values().next() {
            // Split area for tree and progress
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(5),      // Task tree
                    Constraint::Length(3),   // Progress bar
                ])
                .split(area);

            // Render task tree
            let tree_items = self.build_task_tree(&workflow_ctx.executor);
            let tree = Tree::new(tree_items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Workflow: {}", workflow_ctx.workflow.name)))
                .highlight_style(Style::default().fg(Color::Yellow));

            frame.render_widget(tree, chunks[0]);

            // Render progress bar
            let progress = workflow_ctx.progress.completed_tasks as f64
                / workflow_ctx.progress.total_tasks as f64;

            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green))
                .percent((progress * 100.0) as u16)
                .label(format!(
                    "{}/{} tasks completed",
                    workflow_ctx.progress.completed_tasks,
                    workflow_ctx.progress.total_tasks
                ));

            frame.render_widget(gauge, chunks[1]);
        }
    }

    fn build_task_tree(&self, executor: &DSLExecutor) -> Vec<TreeItem> {
        // Build tree structure from task graph
        // This would need access to task graph state
        vec![]
    }
}
```

**Acceptance Criteria:**
- ✅ Displays task hierarchy as tree
- ✅ Shows task status (pending, running, completed, failed)
- ✅ Real-time progress bar
- ✅ Highlights current tasks

#### 5.3 Task Panel
**Files:**
- `src/adapters/primary/tui/ui/panels/tasks.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/ui/panels/tasks.rs
pub struct TaskPanel {
    id: PanelId,
    selected_index: usize,
}

impl Panel for TaskPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        let session = state.session.blocking_read();

        if let Some(workflow_ctx) = session.workflows.values().next() {
            let mut rows = vec![];

            // Current tasks
            for task_id in &workflow_ctx.progress.current_tasks {
                rows.push(Row::new(vec![
                    Cell::from(task_id.clone()),
                    Cell::from("Running").style(Style::default().fg(Color::Yellow)),
                ]));
            }

            // Failed tasks
            for task_id in &workflow_ctx.progress.failed_tasks {
                rows.push(Row::new(vec![
                    Cell::from(task_id.clone()),
                    Cell::from("Failed").style(Style::default().fg(Color::Red)),
                ]));
            }

            let table = Table::new(rows)
                .header(Row::new(vec!["Task", "Status"])
                    .style(Style::default().add_modifier(Modifier::BOLD)))
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Task Status"))
                .widths(&[
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ]);

            frame.render_widget(table, area);
        }
    }
}
```

**Acceptance Criteria:**
- ✅ Shows currently running tasks
- ✅ Shows failed tasks with errors
- ✅ Updates in real-time
- ✅ Supports task selection for details

### Phase 5 Deliverables
- [ ] DSLExecutor fully integrated
- [ ] Workflow panel displaying task graph
- [ ] Real-time progress tracking
- [ ] Task panel showing current/failed tasks
- [ ] Workflow layout preset working

---

## Phase 6: Advanced Features (Week 11-12)

### Goals
- Implement theme system
- Add configuration persistence
- Implement autocomplete
- Add extended thinking panel

### Tasks

#### 6.1 Theme System
**Files:**
- `src/adapters/primary/tui/ui/theme/mod.rs`
- `src/adapters/primary/tui/ui/theme/engine.rs`
- `src/adapters/primary/tui/ui/theme/default.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/ui/theme/engine.rs
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ColorScheme,
    pub styles: StyleMap,
}

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
}

pub struct ThemeEngine {
    themes: HashMap<String, Theme>,
    active_theme: String,
}

impl ThemeEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            themes: HashMap::new(),
            active_theme: "default".to_string(),
        };

        engine.register_theme(default_theme());
        engine
    }

    pub fn get_style(&self, element: &str) -> Style {
        if let Some(theme) = self.themes.get(&self.active_theme) {
            theme.styles.get(element).cloned().unwrap_or_default()
        } else {
            Style::default()
        }
    }
}
```

**Acceptance Criteria:**
- ✅ Theme system loads and applies themes
- ✅ Default theme provided
- ✅ Can switch themes at runtime
- ✅ Custom themes can be defined

#### 6.2 Configuration Persistence
**Files:**
- `src/adapters/primary/tui/state/config.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/state/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    pub default_layout: LayoutPreset,
    pub theme: String,
    pub keybindings: HashMap<String, KeyBinding>,
    pub history: HistoryConfig,
    pub ui: UiConfig,
    pub extensions: HashMap<String, serde_json::Value>,
}

impl TuiConfig {
    pub fn load() -> Self {
        let path = Self::config_path();
        Self::from_file(&path).unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml = toml::to_string_pretty(self)?;
        fs::write(&path, toml)?;

        Ok(())
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tui-repl")
            .join("config.toml")
    }
}
```

**Acceptance Criteria:**
- ✅ Config loads from ~/.config/tui-repl/config.toml
- ✅ Config saves on exit
- ✅ Handles missing config gracefully
- ✅ Migration support for config changes

#### 6.3 Autocomplete
**Files:**
- `src/adapters/primary/tui/input/autocomplete.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/input/autocomplete.rs
pub struct Autocomplete {
    registry: Arc<CommandRegistry>,
}

impl Autocomplete {
    pub fn suggest(&self, partial: &str) -> Vec<String> {
        if !partial.starts_with('/') {
            return vec![];
        }

        let command_part = &partial[1..];

        self.registry
            .commands()
            .filter(|name| name.starts_with(command_part))
            .map(|name| format!("/{}", name))
            .collect()
    }

    pub fn complete(&self, partial: &str) -> Option<String> {
        let suggestions = self.suggest(partial);

        if suggestions.len() == 1 {
            Some(suggestions[0].clone())
        } else if !suggestions.is_empty() {
            Some(self.find_common_prefix(&suggestions))
        } else {
            None
        }
    }

    fn find_common_prefix(&self, strings: &[String]) -> String {
        if strings.is_empty() {
            return String::new();
        }

        let mut prefix = strings[0].clone();

        for s in &strings[1..] {
            while !s.starts_with(&prefix) {
                prefix.pop();
                if prefix.is_empty() {
                    break;
                }
            }
        }

        prefix
    }
}
```

**Acceptance Criteria:**
- ✅ Tab completion for commands
- ✅ Shows suggestions inline
- ✅ Handles multiple matches
- ✅ Works with partial input

#### 6.4 Thinking Panel
**Files:**
- `src/adapters/primary/tui/ui/panels/thinking.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/ui/panels/thinking.rs
pub struct ThinkingPanel {
    id: PanelId,
    scroll_offset: usize,
}

impl Panel for ThinkingPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        let history = state.history.blocking_read();
        let session = state.session.blocking_read();

        if let Some(session_id) = session.active_session {
            if let Some(messages) = history.messages.get(&session_id) {
                // Extract thinking blocks
                let thinking_blocks: Vec<_> = messages
                    .messages
                    .iter()
                    .rev()
                    .filter_map(|msg| {
                        if let Message::Assistant(assistant) = &msg.message {
                            assistant.message.content
                                .iter()
                                .find_map(|block| {
                                    if let ContentBlock::Thinking { thinking, signature } = block {
                                        Some((thinking.clone(), signature.clone()))
                                    } else {
                                        None
                                    }
                                })
                        } else {
                            None
                        }
                    })
                    .collect();

                if let Some((thinking, signature)) = thinking_blocks.first() {
                    let text = format!("{}\n\nSignature: {}", thinking, signature);

                    let paragraph = Paragraph::new(text)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .title("Extended Thinking"))
                        .wrap(Wrap { trim: true })
                        .scroll((self.scroll_offset as u16, 0))
                        .style(Style::default().fg(Color::Cyan));

                    frame.render_widget(paragraph, area);
                }
            }
        }
    }
}
```

**Acceptance Criteria:**
- ✅ Displays extended thinking from latest response
- ✅ Scrollable content
- ✅ Shows signature
- ✅ Toggleable visibility

### Phase 6 Deliverables
- [ ] Theme system with multiple themes
- [ ] Configuration persistence working
- [ ] Tab autocomplete for commands
- [ ] Extended thinking panel
- [ ] All advanced features tested

---

## Phase 7: Extension System (Week 13-14)

### Goals
- Implement plugin architecture
- Create extension loader
- Add sample extensions
- Document extension API

### Tasks

#### 7.1 Extension Traits
**Files:**
- `src/adapters/primary/tui/extensions/mod.rs`
- `src/adapters/primary/tui/extensions/traits.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/extensions/traits.rs
pub trait PanelExtension: Panel {
    fn metadata(&self) -> ExtensionMetadata;
    fn initialize(&mut self, state: &AppState) -> Result<()>;
    fn cleanup(&mut self) -> Result<()>;
    fn configure(&mut self, config: serde_json::Value) -> Result<()>;
}

pub trait CommandExtension: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str;

    fn execute(
        &self,
        args: Vec<String>,
        state: &AppState,
    ) -> Pin<Box<dyn Future<Output = Result<CommandResult>> + Send>>;

    fn autocomplete(&self, partial: &str) -> Vec<String> {
        vec![]
    }
}

pub struct ExtensionMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub dependencies: Vec<String>,
}
```

**Acceptance Criteria:**
- ✅ Extension traits are object-safe
- ✅ Support panel and command extensions
- ✅ Metadata system in place
- ✅ Configuration support

#### 7.2 Extension Registry
**Files:**
- `src/adapters/primary/tui/extensions/registry.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/extensions/registry.rs
pub struct ExtensionRegistry {
    panels: HashMap<String, Box<dyn PanelExtension>>,
    commands: HashMap<String, Box<dyn CommandExtension>>,
    themes: HashMap<String, Theme>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
            commands: HashMap::new(),
            themes: HashMap::new(),
        }
    }

    pub fn register_panel(&mut self, panel: Box<dyn PanelExtension>) -> Result<()> {
        let name = panel.metadata().name.clone();

        // Check dependencies
        for dep in &panel.metadata().dependencies {
            if !self.has_extension(dep) {
                return Err(Error::MissingDependency(dep.clone()));
            }
        }

        self.panels.insert(name, panel);
        Ok(())
    }

    pub fn register_command(&mut self, command: Box<dyn CommandExtension>) -> Result<()> {
        let name = command.name().to_string();
        self.commands.insert(name, command);
        Ok(())
    }

    pub fn get_panel(&self, name: &str) -> Option<&dyn PanelExtension> {
        self.panels.get(name).map(|p| p.as_ref())
    }
}
```

**Acceptance Criteria:**
- ✅ Registry stores extensions
- ✅ Dependency checking works
- ✅ Can retrieve extensions by name
- ✅ Prevents duplicate registration

#### 7.3 Extension Loader
**Files:**
- `src/adapters/primary/tui/extensions/loader.rs`

**Implementation:**
```rust
// src/adapters/primary/tui/extensions/loader.rs
pub async fn load_extensions(
    path: &Path,
    registry: &mut ExtensionRegistry,
) -> Result<usize> {
    let mut loaded = 0;

    if !path.exists() {
        return Ok(0);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        // For now, support Rust-based extensions compiled as dynamic libraries
        if path.extension().and_then(|s| s.to_str()) == Some("so")
            || path.extension().and_then(|s| s.to_str()) == Some("dylib")
            || path.extension().and_then(|s| s.to_str()) == Some("dll")
        {
            // Load native extension
            // (This would use libloading crate)
            // Skipped for MVP - requires unsafe code
        }
    }

    Ok(loaded)
}
```

**Acceptance Criteria:**
- ✅ Loads extensions from directory
- ✅ Handles missing directory gracefully
- ✅ Error handling for invalid extensions
- ✅ Reports number of loaded extensions

### Phase 7 Deliverables
- [ ] Extension trait system complete
- [ ] Extension registry functional
- [ ] Extension loader working
- [ ] Sample extension created
- [ ] Extension API documented

---

## Testing & Quality Assurance

### Unit Testing (Ongoing)
**Target:** >80% code coverage

**Key Test Areas:**
- State management (all state updates)
- Command parsing (all command types)
- Panel logic (rendering, input handling)
- Event handling (all event types)
- Message rendering (all content blocks)

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_state_add_client() {
        let state = AppState::new();
        let session_id = SessionId::new();

        // Add session
        {
            let mut session = state.session.write().await;
            session.active_session = Some(session_id);
        }

        // Verify
        let session = state.session.read().await;
        assert_eq!(session.active_session, Some(session_id));
    }
}
```

### Integration Testing
**Target:** All critical paths covered

**Test Scenarios:**
1. Full query flow (user input → CLI → display)
2. Workflow execution (load → validate → run → complete)
3. Session lifecycle (create → use → close)
4. Command execution (all built-in commands)
5. Panel switching and layout changes

**Example:**
```rust
#[tokio::test]
async fn test_query_flow_integration() {
    let app = TuiApp::new(TuiOptions::default()).unwrap();

    // Start app in background
    let app_handle = tokio::spawn(async move {
        app.run().await
    });

    // Simulate user input
    // Send query
    // Wait for response
    // Verify state

    app_handle.await.unwrap().unwrap();
}
```

### Manual Testing Checklist
- [ ] Terminal resize handling
- [ ] Keyboard shortcuts all work
- [ ] Messages display correctly
- [ ] Scrolling works in all panels
- [ ] Color themes apply correctly
- [ ] Config loads and saves
- [ ] Autocomplete works
- [ ] Help system accurate
- [ ] Workflow visualization correct
- [ ] Performance acceptable (no lag)

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Startup time | <500ms | Time to first render |
| Frame rate | 60 FPS | <16ms per render |
| Memory usage | <50MB | RSS for typical session |
| Message latency | <100ms | Input to display |
| Scroll performance | Smooth | No dropped frames |

---

## Documentation Requirements

### Code Documentation
- [ ] All public APIs documented with rustdoc
- [ ] Module-level documentation
- [ ] Examples for key functions
- [ ] Architecture decisions recorded

### User Documentation
- [ ] README with quick start
- [ ] Command reference
- [ ] Configuration guide
- [ ] Extension development guide
- [ ] Troubleshooting guide

---

## Definition of Done

A phase is considered complete when:

1. ✅ All tasks implemented and committed
2. ✅ Unit tests written and passing (>80% coverage)
3. ✅ Integration tests passing
4. ✅ Manual testing checklist completed
5. ✅ Code reviewed
6. ✅ Documentation updated
7. ✅ No critical bugs
8. ✅ Performance targets met

---

## Risk Mitigation

### Technical Risks

**Risk:** Terminal compatibility issues
**Mitigation:** Test on macOS, Linux, Windows early. Use crossterm's cross-platform abstractions.

**Risk:** Performance degradation with large message history
**Mitigation:** Implement message pagination and bounded history from Phase 2.

**Risk:** Deadlocks in async state management
**Mitigation:** Use timeout on all lock acquisitions, careful lock ordering.

**Risk:** CLI subprocess crashes
**Mitigation:** Implement supervisor pattern, auto-restart on failure.

### Schedule Risks

**Risk:** Underestimated complexity
**Mitigation:** MVP approach - defer advanced features if needed.

**Risk:** Blocked on external dependencies
**Mitigation:** Use mocks and continue parallel development.

---

## Next Steps

### Immediate Actions (Phase 1)
1. Set up project structure
2. Implement AppState with RwLock pattern
3. Create basic event loop
4. Test terminal setup/cleanup

### Success Metrics
- Working TUI that starts and exits cleanly
- State management tested and verified
- Event loop processing input
- Foundation ready for Panel system

---

**End of Implementation Roadmap**

//! Core TUI application structure
//!
//! Implements the main TUI app with event loop, state management,
//! view routing, and modal system following hexagonal architecture.

use super::events::{AppEvent, EventHandler, ExecutionUpdate, KeyEvent};
use super::state::{AppState, ConfirmAction, InputAction, Modal, ViewMode};
use super::theme::Theme;
use super::ui::WorkflowListView;
use crate::dsl::{parse_workflow_file, DSLExecutor};
use crate::error::Result;
use crossterm::event::KeyCode;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

/// Configuration for TUI application
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Workflow directory to browse and manage
    pub workflow_dir: PathBuf,

    /// Specific workflow file to open
    pub workflow: Option<PathBuf>,

    /// Launch in readonly mode (no edits or execution)
    pub readonly: bool,

    /// Color theme name
    pub theme: String,

    /// State directory for workflow persistence
    pub state_dir: Option<PathBuf>,

    /// Enable debug logging
    pub debug: bool,

    /// Tick rate in milliseconds
    pub tick_rate: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            workflow_dir: PathBuf::from("."),
            workflow: None,
            readonly: false,
            theme: "dark".to_string(),
            state_dir: None,
            debug: false,
            tick_rate: 250,
        }
    }
}

/// Main TUI application
pub struct App {
    /// Application configuration
    #[allow(dead_code)]
    config: AppConfig,

    /// Application state
    state: AppState,

    /// Terminal backend
    terminal: Terminal<CrosstermBackend<io::Stdout>>,

    /// Event handler
    event_handler: EventHandler,

    /// Theme
    theme: Theme,

    /// Workflow executor (for running workflows)
    #[allow(dead_code)]
    executor: Option<DSLExecutor>,

    /// Execution update channel
    #[allow(dead_code)]
    execution_tx: Option<mpsc::UnboundedSender<AppEvent>>,
}

impl App {
    /// Create new TUI application with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(AppConfig::default())
    }

    /// Create new TUI application with custom configuration
    pub fn with_config(config: AppConfig) -> Result<Self> {
        // Initialize terminal
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Create event handler with configured tick rate
        let event_handler = EventHandler::new(Duration::from_millis(config.tick_rate));

        // Select theme based on config
        let theme = match config.theme.as_str() {
            "light" => Theme::light(),
            "monokai" => Theme::monokai(),
            "solarized" => Theme::solarized(),
            _ => Theme::default(), // dark
        };

        Ok(Self {
            config,
            state: AppState::new(),
            terminal,
            event_handler,
            theme,
            executor: None,
            execution_tx: None,
        })
    }

    /// Run the application event loop
    pub async fn run(&mut self) -> Result<()> {
        // Start event polling
        self.event_handler.start_polling().await;

        // Load workflows from default directory
        self.load_workflows().await?;

        // Main event loop
        while self.state.running {
            // Render current view
            self.render()?;

            // Handle next event
            if let Some(event) = self.event_handler.next().await {
                self.handle_event(event).await?;
            }
        }

        Ok(())
    }

    /// Render current view
    fn render(&mut self) -> Result<()> {
        let view_mode = self.state.view_mode;
        let modal = self.state.modal.clone();

        self.terminal.draw(|frame| {
            let area = frame.area();

            // Route to appropriate view based on state
            match view_mode {
                ViewMode::WorkflowList => {
                    WorkflowListView::render(frame, area, &self.state.workflows, self.state.selected_workflow, &self.theme);
                }
                ViewMode::Viewer => {
                    Self::render_viewer_static(frame, area, &self.state, &self.theme);
                }
                ViewMode::Editor => {
                    Self::render_editor_static(frame, area, &self.state.editor_state, &self.theme);
                }
                ViewMode::Generator => {
                    Self::render_generator_static(frame, area, &self.state.generator_state, &self.theme);
                }
                ViewMode::ExecutionMonitor => {
                    Self::render_execution_monitor_static(frame, area);
                }
                ViewMode::StateBrowser => {
                    Self::render_state_browser_static(frame, area, &mut self.state.state_browser, &self.theme);
                }
                ViewMode::Help => {
                    Self::render_help_static(frame, area, &mut self.state.help_state, &self.theme);
                }
            }

            // Render modal if active
            if let Some(ref modal) = modal {
                Self::render_modal_static(frame, area, modal, &self.state.input_buffer, &self.theme);
            }
        })?;

        Ok(())
    }

    /// Handle application event
    async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Key(key) => {
                // If modal is active, route to modal handler
                if self.state.has_modal() {
                    self.handle_modal_key(key).await?;
                } else {
                    // Otherwise route to current view
                    match self.state.view_mode {
                        ViewMode::WorkflowList => self.handle_workflow_list_key(key).await?,
                        ViewMode::Viewer => self.handle_viewer_key(key).await?,
                        ViewMode::Editor => self.handle_editor_key(key).await?,
                        ViewMode::Generator => self.handle_generator_key(key).await?,
                        ViewMode::ExecutionMonitor => {
                            self.handle_execution_monitor_key(key).await?
                        }
                        ViewMode::StateBrowser => self.handle_state_browser_key(key).await?,
                        ViewMode::Help => self.handle_help_key(key).await?,
                    }
                }
            }

            AppEvent::Resize(_, _) => {
                // Terminal will automatically handle resize
            }

            AppEvent::Tick => {
                // Periodic updates (e.g., for animations, async state updates)
                self.handle_tick().await?;
            }

            AppEvent::ExecutionUpdate(update) => {
                self.handle_execution_update(update).await?;
            }

            AppEvent::Error(msg) => {
                self.show_error("Error", &msg);
            }

            AppEvent::Quit => {
                self.state.running = false;
            }
        }

        Ok(())
    }

    /// Handle modal key events
    async fn handle_modal_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.close_modal();
                self.state.input_buffer.clear(); // Clear input buffer when canceling
            }

            KeyCode::Enter => {
                if let Some(modal) = self.state.modal.take() {
                    self.handle_modal_action(modal).await?;
                }
            }

            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(Modal::Confirm { action, .. }) = self.state.modal.take() {
                    self.handle_confirm_action(action).await?;
                }
            }

            KeyCode::Char('n') | KeyCode::Char('N') => {
                if matches!(self.state.modal, Some(Modal::Confirm { .. })) {
                    self.state.close_modal();
                }
            }

            _ => {
                // Handle input for Input modals
                if matches!(self.state.modal, Some(Modal::Input { .. })) {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.state.input_buffer.push(c);
                        }
                        KeyCode::Backspace => {
                            self.state.input_buffer.pop();
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle modal action when confirmed
    async fn handle_modal_action(&mut self, modal: Modal) -> Result<()> {
        match modal {
            Modal::Confirm { action, .. } => {
                self.handle_confirm_action(action).await?;
            }
            Modal::Input { action, .. } => {
                let value = self.state.input_buffer.clone();
                self.state.input_buffer.clear();
                self.handle_input_action(action, value).await?;
            }
            Modal::Error { .. } | Modal::Info { .. } | Modal::Success { .. } => {
                // Just close
            }
        }

        Ok(())
    }

    /// Handle confirmation actions
    async fn handle_confirm_action(&mut self, action: ConfirmAction) -> Result<()> {
        match action {
            ConfirmAction::DeleteWorkflow(_path) => {
                self.delete_selected_workflow().await?;
            }
            ConfirmAction::Exit => {
                self.state.running = false;
            }
            ConfirmAction::StopExecution => {
                self.stop_execution().await?;
            }
            ConfirmAction::DiscardChanges => {
                self.discard_changes().await?;
            }
            ConfirmAction::ExecuteWorkflow(_path) => {
                self.execute_selected_workflow().await?;
            }
        }

        Ok(())
    }

    /// Handle input actions
    async fn handle_input_action(&mut self, action: InputAction, value: String) -> Result<()> {
        match action {
            InputAction::CreateWorkflow => {
                self.create_workflow(value).await?;
            }
            InputAction::RenameWorkflow(_path) => {
                self.rename_workflow(value).await?;
            }
            InputAction::SetWorkflowDescription => {
                self.set_workflow_description(value).await?;
            }
            InputAction::SaveWorkflowAs => {
                self.save_workflow_as(value).await?;
            }
            InputAction::GenerateWorkflow => {
                self.generate_workflow(value).await?;
            }
        }

        Ok(())
    }

    /// Handle workflow list key events
    async fn handle_workflow_list_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                self.confirm_quit();
            }

            KeyCode::Char('n') => {
                self.prompt_create_workflow();
            }

            KeyCode::Char('?') => {
                self.state.view_mode = ViewMode::Help;
            }

            KeyCode::Up | KeyCode::Char('k') => {
                if !self.state.workflows.is_empty() && self.state.selected_workflow > 0 {
                    self.state.selected_workflow -= 1;
                }
            }

            KeyCode::Down | KeyCode::Char('j') => {
                if !self.state.workflows.is_empty() {
                    let max = self.state.workflows.len().saturating_sub(1);
                    if self.state.selected_workflow < max {
                        self.state.selected_workflow += 1;
                    }
                }
            }

            KeyCode::Enter => {
                self.open_selected_workflow().await?;
            }

            KeyCode::Char('d') if key.is_ctrl() => {
                self.confirm_delete_workflow();
            }

            KeyCode::Char('/') => {
                // Start search mode
                self.state.input_buffer = self.state.search_query.clone();
                self.state.modal = Some(Modal::Input {
                    title: "Search Workflows".to_string(),
                    prompt: "Enter search query:".to_string(),
                    default: self.state.search_query.clone(),
                    action: InputAction::CreateWorkflow, // Reuse for now
                });
            }

            KeyCode::Char('s') => {
                // Open state browser
                let _ = self.state.state_browser.load_states();
                self.state.view_mode = ViewMode::StateBrowser;
            }

            KeyCode::Char('e') => {
                // Open editor for selected workflow
                self.open_workflow_in_editor().await?;
            }

            KeyCode::Char('g') => {
                // Switch to generator view
                self.state.view_mode = ViewMode::Generator;
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle viewer key events
    async fn handle_viewer_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.view_mode = ViewMode::WorkflowList;
                self.state.viewer_state.reset();
            }

            KeyCode::Tab => {
                self.state.viewer_state.toggle_view_mode();
            }

            KeyCode::Up | KeyCode::Char('k') => {
                self.state.viewer_state.scroll_up();
            }

            KeyCode::Down | KeyCode::Char('j') => {
                // Calculate max lines based on current workflow
                let max_lines = 100; // TODO: Calculate actual max lines
                self.state.viewer_state.scroll_down(max_lines);
            }

            KeyCode::PageUp => {
                self.state.viewer_state.page_up();
            }

            KeyCode::PageDown => {
                // Calculate max lines based on current workflow
                let max_lines = 100; // TODO: Calculate actual max lines
                self.state.viewer_state.page_down(max_lines);
            }

            KeyCode::Home => {
                self.state.viewer_state.scroll_to_top();
            }

            KeyCode::End => {
                // Calculate max lines based on current workflow
                let max_lines = 100; // TODO: Calculate actual max lines
                self.state.viewer_state.scroll_to_bottom(max_lines);
            }

            KeyCode::Char('e') => {
                // Switch to editor mode - load current workflow into editor
                if self.state.current_workflow.is_some() {
                    if let Some(ref path) = self.state.current_workflow_path {
                        // Read file content
                        if let Ok(content) = std::fs::read_to_string(path) {
                            self.state.editor_state.reset();
                            self.state.editor_state.content = content;
                            self.state.editor_state.file_path = Some(path.clone());
                            self.state.editor_state.modified = false;
                            self.state.editor_state.validate_cursor();
                            self.state.view_mode = ViewMode::Editor;
                        }
                    }
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle editor key events
    async fn handle_editor_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                // If validation is expanded, close it first
                if self.state.editor_state.validation_expanded {
                    self.state.editor_state.validation_expanded = false;
                } else if self.state.editor_state.modified {
                    // Check for unsaved changes
                    self.state.modal = Some(Modal::Confirm {
                        title: "Unsaved Changes".to_string(),
                        message: "You have unsaved changes. Discard them?".to_string(),
                        action: ConfirmAction::DiscardChanges,
                    });
                } else {
                    self.state.view_mode = ViewMode::WorkflowList;
                }
            }

            KeyCode::Char('v') if !key.is_ctrl() => {
                // Toggle validation expanded view
                self.state.editor_state.validation_expanded = !self.state.editor_state.validation_expanded;
            }

            KeyCode::Char('s') if key.is_ctrl() => {
                self.save_current_workflow().await?;
            }

            KeyCode::Char('r') if key.is_ctrl() => {
                self.run_current_workflow().await?;
            }

            // Text editing
            KeyCode::Char(c) if !key.is_ctrl() => {
                let (mut line, mut col) = self.state.editor_state.cursor;
                let mut lines: Vec<String> = self.state.editor_state.content.lines().map(String::from).collect();

                if lines.is_empty() {
                    lines.push(String::new());
                }

                // Ensure line index is valid
                if line >= lines.len() {
                    line = lines.len().saturating_sub(1).max(0);
                    self.state.editor_state.cursor.0 = line;
                }

                // Ensure column index is valid
                let line_len = lines[line].len();
                if col > line_len {
                    col = line_len;
                    self.state.editor_state.cursor.1 = col;
                }

                lines[line].insert(col, c);
                self.state.editor_state.content = lines.join("\n");
                self.state.editor_state.cursor.1 += 1;
                self.state.editor_state.modified = true;
            }

            KeyCode::Enter => {
                let (line, col) = self.state.editor_state.cursor;
                let mut lines: Vec<String> = self.state.editor_state.content.lines().map(String::from).collect();

                if lines.is_empty() {
                    lines.push(String::new());
                }

                if line < lines.len() {
                    let current_line = lines[line].clone();
                    // Safely split at column position
                    let col = col.min(current_line.len());
                    let (before, after) = current_line.split_at(col);
                    lines[line] = before.to_string();
                    lines.insert(line + 1, after.to_string());

                    self.state.editor_state.content = lines.join("\n");
                    self.state.editor_state.cursor = (line + 1, 0);
                    self.state.editor_state.modified = true;
                }
            }

            KeyCode::Backspace => {
                let (line, col) = self.state.editor_state.cursor;
                let mut lines: Vec<String> = self.state.editor_state.content.lines().map(String::from).collect();

                if col > 0 && line < lines.len() {
                    // Delete character before cursor
                    lines[line].remove(col - 1);
                    self.state.editor_state.content = lines.join("\n");
                    self.state.editor_state.cursor.1 -= 1;
                    self.state.editor_state.modified = true;
                } else if col == 0 && line > 0 {
                    // Merge with previous line
                    let current = lines.remove(line);
                    let prev_len = lines[line - 1].len();
                    lines[line - 1].push_str(&current);
                    self.state.editor_state.content = lines.join("\n");
                    self.state.editor_state.cursor = (line - 1, prev_len);
                    self.state.editor_state.modified = true;
                }
            }

            KeyCode::Delete => {
                let (line, col) = self.state.editor_state.cursor;
                let mut lines: Vec<String> = self.state.editor_state.content.lines().map(String::from).collect();

                if lines.is_empty() {
                    return Ok(());
                }

                if line < lines.len() {
                    if col < lines[line].len() {
                        // Delete character at cursor
                        lines[line].remove(col);
                        self.state.editor_state.content = lines.join("\n");
                        self.state.editor_state.modified = true;
                    } else if line < lines.len() - 1 {
                        // Merge with next line (cursor is at end of line)
                        let next = lines.remove(line + 1);
                        lines[line].push_str(&next);
                        self.state.editor_state.content = lines.join("\n");
                        self.state.editor_state.modified = true;
                    }
                }
            }

            // Cursor movement
            KeyCode::Left => {
                let (line, col) = self.state.editor_state.cursor;
                if col > 0 {
                    self.state.editor_state.cursor.1 -= 1;
                } else if line > 0 {
                    // Move to end of previous line
                    let lines: Vec<&str> = self.state.editor_state.content.lines().collect();
                    self.state.editor_state.cursor.0 -= 1;
                    self.state.editor_state.cursor.1 = lines[line - 1].len();
                }
            }

            KeyCode::Right => {
                let (line, col) = self.state.editor_state.cursor;
                let lines: Vec<&str> = self.state.editor_state.content.lines().collect();
                if line < lines.len() {
                    if col < lines[line].len() {
                        self.state.editor_state.cursor.1 += 1;
                    } else if line < lines.len() - 1 {
                        // Move to start of next line
                        self.state.editor_state.cursor.0 += 1;
                        self.state.editor_state.cursor.1 = 0;
                    }
                }
            }

            KeyCode::Up => {
                if self.state.editor_state.cursor.0 > 0 {
                    self.state.editor_state.cursor.0 -= 1;
                    // Adjust column if new line is shorter
                    let lines: Vec<&str> = self.state.editor_state.content.lines().collect();
                    let new_line = self.state.editor_state.cursor.0;
                    if new_line < lines.len() {
                        let line_len = lines[new_line].len();
                        if self.state.editor_state.cursor.1 > line_len {
                            self.state.editor_state.cursor.1 = line_len;
                        }
                    }
                }
            }

            KeyCode::Down => {
                let lines: Vec<&str> = self.state.editor_state.content.lines().collect();
                if self.state.editor_state.cursor.0 < lines.len().saturating_sub(1) {
                    self.state.editor_state.cursor.0 += 1;
                    // Adjust column if new line is shorter
                    let new_line = self.state.editor_state.cursor.0;
                    if new_line < lines.len() {
                        let line_len = lines[new_line].len();
                        if self.state.editor_state.cursor.1 > line_len {
                            self.state.editor_state.cursor.1 = line_len;
                        }
                    }
                }
            }

            KeyCode::Home => {
                self.state.editor_state.cursor.1 = 0;
            }

            KeyCode::End => {
                let line = self.state.editor_state.cursor.0;
                let lines: Vec<&str> = self.state.editor_state.content.lines().collect();
                if line < lines.len() {
                    self.state.editor_state.cursor.1 = lines[line].len();
                }
            }

            KeyCode::PageUp => {
                if self.state.editor_state.scroll.0 > 10 {
                    self.state.editor_state.scroll.0 -= 10;
                } else {
                    self.state.editor_state.scroll.0 = 0;
                }
            }

            KeyCode::PageDown => {
                self.state.editor_state.scroll.0 += 10;
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle generator key events
    async fn handle_generator_key(&mut self, key: KeyEvent) -> Result<()> {
        // When Input panel is focused, handle text input and formatting
        if self.state.generator_state.focus == crate::tui::views::generator::FocusPanel::Input {
            match key.code {
                // All regular characters (not modified by Ctrl or Alt)
                KeyCode::Char(c) if !key.is_ctrl() && !key.is_alt() => {
                    self.state.generator_state.nl_input.insert(
                        self.state.generator_state.input_cursor,
                        c,
                    );
                    self.state.generator_state.input_cursor += 1;
                    return Ok(());
                }

                // Markdown formatting shortcuts (Ctrl+letter)
                KeyCode::Char('b') if key.is_ctrl() => {
                    self.state.generator_state.nl_input.insert_str(
                        self.state.generator_state.input_cursor,
                        "****",
                    );
                    self.state.generator_state.input_cursor += 2;
                    return Ok(());
                }

                KeyCode::Char('i') if key.is_ctrl() => {
                    self.state.generator_state.nl_input.insert_str(
                        self.state.generator_state.input_cursor,
                        "**",
                    );
                    self.state.generator_state.input_cursor += 1;
                    return Ok(());
                }

                KeyCode::Char('k') if key.is_ctrl() => {
                    self.state.generator_state.nl_input.insert_str(
                        self.state.generator_state.input_cursor,
                        "``",
                    );
                    self.state.generator_state.input_cursor += 1;
                    return Ok(());
                }

                KeyCode::Char('h') if key.is_ctrl() => {
                    let before = &self.state.generator_state.nl_input[..self.state.generator_state.input_cursor];
                    let line_start = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
                    self.state.generator_state.nl_input.insert_str(line_start, "# ");
                    self.state.generator_state.input_cursor += 2;
                    return Ok(());
                }

                _ => {
                    // Fall through to common handlers below
                }
            }
        }

        // Common handlers for both panels
        match key.code {
            KeyCode::Esc => {
                self.state.view_mode = ViewMode::WorkflowList;
            }

            KeyCode::Tab => {
                // Switch focus between input and preview panels
                self.state.generator_state.focus = match self.state.generator_state.focus {
                    crate::tui::views::generator::FocusPanel::Input => {
                        crate::tui::views::generator::FocusPanel::Preview
                    }
                    crate::tui::views::generator::FocusPanel::Preview => {
                        crate::tui::views::generator::FocusPanel::Input
                    }
                };
            }

            KeyCode::Char('g') if key.is_ctrl() => {
                // Start workflow generation
                if self.state.generator_state.can_generate() {
                    self.generate_workflow_from_nl().await?;
                }
            }

            KeyCode::Char('a') if key.is_ctrl() => {
                // Accept generated workflow
                if self.state.generator_state.can_accept() {
                    self.accept_generated_workflow().await?;
                }
            }

            KeyCode::Char('r') if key.is_ctrl() => {
                // Retry generation
                if self.state.generator_state.can_generate() {
                    self.generate_workflow_from_nl().await?;
                }
            }

            KeyCode::Char('d') if key.is_ctrl() => {
                // Toggle diff view
                self.state.generator_state.show_diff = !self.state.generator_state.show_diff;
            }

            // Input/Preview panel specific handlers
            _ => {
                if self.state.generator_state.focus == crate::tui::views::generator::FocusPanel::Input {
                    match key.code {
                        KeyCode::Backspace => {
                            if self.state.generator_state.input_cursor > 0 {
                                self.state.generator_state.input_cursor -= 1;
                                self.state.generator_state.nl_input.remove(
                                    self.state.generator_state.input_cursor
                                );
                            }
                        }

                        KeyCode::Delete => {
                            if self.state.generator_state.input_cursor < self.state.generator_state.nl_input.len() {
                                self.state.generator_state.nl_input.remove(
                                    self.state.generator_state.input_cursor
                                );
                            }
                        }

                        KeyCode::Left => {
                            if self.state.generator_state.input_cursor > 0 {
                                self.state.generator_state.input_cursor -= 1;
                            }
                        }

                        KeyCode::Right => {
                            if self.state.generator_state.input_cursor < self.state.generator_state.nl_input.len() {
                                self.state.generator_state.input_cursor += 1;
                            }
                        }

                        KeyCode::Home => {
                            self.state.generator_state.input_cursor = 0;
                        }

                        KeyCode::End => {
                            self.state.generator_state.input_cursor = self.state.generator_state.nl_input.len();
                        }

                        KeyCode::Enter => {
                            // Insert newline
                            self.state.generator_state.nl_input.insert(
                                self.state.generator_state.input_cursor,
                                '\n',
                            );
                            self.state.generator_state.input_cursor += 1;
                        }

                        _ => {}
                    }
                } else {
                    // Preview panel focused - handle scrolling
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.state.generator_state.preview_scroll =
                                self.state.generator_state.preview_scroll.saturating_sub(1);
                        }

                        KeyCode::Down | KeyCode::Char('j') => {
                            self.state.generator_state.preview_scroll += 1;
                        }

                        KeyCode::PageUp => {
                            self.state.generator_state.preview_scroll =
                                self.state.generator_state.preview_scroll.saturating_sub(10);
                        }

                        KeyCode::PageDown => {
                            self.state.generator_state.preview_scroll += 10;
                        }

                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle execution monitor key events
    async fn handle_execution_monitor_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.view_mode = ViewMode::WorkflowList;
            }

            KeyCode::Char('s') if key.is_ctrl() => {
                self.confirm_stop_execution();
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle help key events
    async fn handle_state_browser_key(&mut self, key: KeyEvent) -> Result<()> {
        use super::views::state_browser::StateBrowserViewMode;

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                if self.state.state_browser.view_mode == StateBrowserViewMode::Details {
                    self.state.state_browser.back_to_list();
                } else {
                    self.state.view_mode = ViewMode::WorkflowList;
                }
            }
            KeyCode::Up => {
                if self.state.state_browser.view_mode == StateBrowserViewMode::List {
                    self.state.state_browser.select_previous();
                } else {
                    self.state.state_browser.scroll_details_up();
                }
            }
            KeyCode::Down => {
                if self.state.state_browser.view_mode == StateBrowserViewMode::List {
                    self.state.state_browser.select_next();
                } else {
                    let max_lines = 100; // Will be calculated properly in render
                    self.state.state_browser.scroll_details_down(max_lines);
                }
            }
            KeyCode::PageUp => {
                if self.state.state_browser.view_mode == StateBrowserViewMode::Details {
                    self.state.state_browser.page_details_up();
                }
            }
            KeyCode::PageDown => {
                if self.state.state_browser.view_mode == StateBrowserViewMode::Details {
                    let max_lines = 100; // Will be calculated properly in render
                    self.state.state_browser.page_details_down(max_lines);
                }
            }
            KeyCode::Enter => {
                if self.state.state_browser.view_mode == StateBrowserViewMode::List {
                    let _ = self.state.state_browser.load_details();
                }
            }
            KeyCode::Char('r') => {
                // Resume selected workflow
                if self.state.state_browser.can_resume_selected() {
                    if let Some(entry) = self.state.state_browser.selected_state() {
                        // TODO: Implement resume workflow logic
                        println!("Resuming workflow: {}", entry.workflow_name);
                    }
                }
            }
            KeyCode::Char('d') => {
                // Delete selected state
                self.state.modal = Some(Modal::Confirm {
                    title: "Delete State".to_string(),
                    message: "Are you sure you want to delete this workflow state?".to_string(),
                    action: ConfirmAction::DeleteWorkflow(PathBuf::new()),
                });
            }
            KeyCode::Char('s') => {
                // Cycle sort mode
                self.state.state_browser.next_sort_mode();
            }
            KeyCode::Char('/') => {
                // Start filter mode (TODO: implement input modal)
            }
            KeyCode::Char('?') => {
                self.state.view_mode = ViewMode::Help;
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_help_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                // If viewing a topic, go back to browse; otherwise exit help
                if self.state.help_state.is_viewing_topic() {
                    self.state.help_state.back_to_browse();
                } else {
                    self.state.view_mode = ViewMode::WorkflowList;
                }
            }

            KeyCode::Enter => {
                // Select and view current topic
                self.state.help_state.select();
            }

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.help_state.scroll_up();
            }

            KeyCode::Down | KeyCode::Char('j') => {
                self.state.help_state.scroll_down();
            }

            KeyCode::PageUp => {
                self.state.help_state.page_up();
            }

            KeyCode::PageDown => {
                self.state.help_state.page_down();
            }

            KeyCode::Tab | KeyCode::Char('n') => {
                self.state.help_state.next_topic();
            }

            KeyCode::BackTab | KeyCode::Char('p') => {
                self.state.help_state.prev_topic();
            }

            KeyCode::Right | KeyCode::Char('l') => {
                self.state.help_state.next_category();
            }

            KeyCode::Left | KeyCode::Char('h') => {
                self.state.help_state.prev_category();
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle periodic tick events
    async fn handle_tick(&mut self) -> Result<()> {
        // Update any time-based UI elements
        // Check for execution updates
        Ok(())
    }

    /// Handle execution update events
    async fn handle_execution_update(&mut self, update: ExecutionUpdate) -> Result<()> {
        if let Some(exec_state) = &mut self.state.execution_state {
            match update {
                ExecutionUpdate::TaskStarted(task) => {
                    exec_state.current_task = Some(task);
                }
                ExecutionUpdate::TaskCompleted(task) => {
                    exec_state.completed_tasks.push(task);
                    exec_state.current_task = None;
                }
                ExecutionUpdate::TaskFailed { task, error } => {
                    exec_state.failed_tasks.push(task);
                    exec_state.current_task = None;
                    self.show_error("Task Failed", &error);
                }
                ExecutionUpdate::LogMessage { level, message } => {
                    // Add to logs (implementation in state)
                    let _ = (level, message); // Suppress warning for now
                }
                ExecutionUpdate::StatusChanged(status) => {
                    let _ = status; // Suppress warning for now
                }
            }
        }

        Ok(())
    }

    /// Load workflows from default directory
    async fn load_workflows(&mut self) -> Result<()> {
        use std::fs;
        use crate::tui::state::WorkflowEntry;

        self.state.workflows.clear();

        // Read directory entries
        let entries = match fs::read_dir(&self.config.workflow_dir) {
            Ok(entries) => entries,
            Err(e) => {
                log::error!("Failed to read workflow directory: {}", e);
                return Ok(());
            }
        };

        // Find all YAML workflow files
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                // Check if it's a YAML file
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "yaml" || ext == "yml" {
                            // Try to get the filename
                            if let Some(name) = path.file_stem() {
                                self.state.workflows.push(WorkflowEntry {
                                    name: name.to_string_lossy().to_string(),
                                    path: path.clone(),
                                    description: None,
                                    version: None,
                                    valid: true,
                                    errors: Vec::new(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Sort by name
        self.state.workflows.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(())
    }

    /// Open selected workflow in viewer
    async fn open_selected_workflow(&mut self) -> Result<()> {
        if let Some(entry) = self.state.workflows.get(self.state.selected_workflow) {
            let workflow = parse_workflow_file(&entry.path)?;
            self.state.current_workflow = Some(workflow);
            self.state.current_workflow_path = Some(entry.path.clone());
            self.state.viewer_state.reset();
            self.state.view_mode = ViewMode::Viewer;
        }
        Ok(())
    }

    /// Open selected workflow in editor mode
    async fn open_workflow_in_editor(&mut self) -> Result<()> {
        if let Some(entry) = self.state.workflows.get(self.state.selected_workflow) {
            // Read file content as string
            let content = std::fs::read_to_string(&entry.path)?;

            // Parse workflow to validate and store
            let workflow = parse_workflow_file(&entry.path)?;
            self.state.current_workflow = Some(workflow);
            self.state.current_workflow_path = Some(entry.path.clone());

            // Reset editor state first to clear any previous state
            self.state.editor_state.reset();

            // Set editor state with new content
            self.state.editor_state.content = content;
            self.state.editor_state.file_path = Some(entry.path.clone());
            self.state.editor_state.modified = false;

            // Validate cursor position
            self.state.editor_state.validate_cursor();

            // Switch to editor view
            self.state.view_mode = ViewMode::Editor;
        }
        Ok(())
    }

    /// Delete selected workflow
    async fn delete_selected_workflow(&mut self) -> Result<()> {
        if self.state.selected_workflow < self.state.workflows.len() {
            let entry = self.state.workflows.remove(self.state.selected_workflow);
            std::fs::remove_file(&entry.path)?;

            // Adjust selection
            if self.state.selected_workflow >= self.state.workflows.len()
                && self.state.selected_workflow > 0
            {
                self.state.selected_workflow -= 1;
            }
        }
        Ok(())
    }

    /// Save current workflow
    async fn save_current_workflow(&mut self) -> Result<()> {
        // Save from editor content if we're in editor mode
        if self.state.view_mode == ViewMode::Editor {
            if let Some(path) = &self.state.editor_state.file_path {
                tokio::fs::write(path, &self.state.editor_state.content).await?;
                self.state.editor_state.modified = false; // Clear modified flag
                self.show_info("Success", "Workflow saved successfully");
                return Ok(());
            }
        }

        // Otherwise save from parsed workflow
        if let (Some(workflow), Some(path)) = (
            &self.state.current_workflow,
            &self.state.current_workflow_path,
        ) {
            let yaml = serde_yaml::to_string(workflow)?;
            tokio::fs::write(path, yaml).await?;
            self.state.editor_state.modified = false; // Clear modified flag
            self.show_info("Success", "Workflow saved successfully");
        }
        Ok(())
    }

    /// Run current workflow
    async fn run_current_workflow(&mut self) -> Result<()> {
        if let Some(workflow) = &self.state.current_workflow {
            self.state.view_mode = ViewMode::ExecutionMonitor;
            // TODO: Start execution
            let _ = workflow; // Suppress warning
        }
        Ok(())
    }

    /// Stop workflow execution
    async fn stop_execution(&mut self) -> Result<()> {
        // TODO: Implement execution stopping
        self.state.execution_state = None;
        self.state.view_mode = ViewMode::WorkflowList;
        Ok(())
    }

    /// Discard changes and return to list
    async fn discard_changes(&mut self) -> Result<()> {
        self.state.current_workflow = None;
        self.state.current_workflow_path = None;
        self.state.editor_state.reset(); // Reset editor state
        self.state.view_mode = ViewMode::WorkflowList;
        Ok(())
    }

    /// Create new workflow
    async fn create_workflow(&mut self, name: String) -> Result<()> {
        use std::fs;

        if name.is_empty() {
            self.show_error("Invalid Name", "Workflow name cannot be empty");
            return Ok(());
        }

        // Sanitize filename
        let safe_name = name.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-");
        let filename = format!("{}.yaml", safe_name);
        let filepath = self.config.workflow_dir.join(&filename);

        // Check if file already exists
        if filepath.exists() {
            self.show_error("File Exists", &format!("Workflow '{}' already exists", filename));
            return Ok(());
        }

        // Generate basic workflow YAML
        let template = format!(
r#"# Workflow: {}
name: "{}"
version: "1.0.0"

# Define your agents
agents:
  researcher:
    description: "Research and gather information"
    model: "claude-sonnet-4-5"
    tools:
      - Read
      - WebSearch
    permissions:
      mode: "default"

# Define your tasks
tasks:
  research_task:
    description: "Perform research on the topic"
    agent: "researcher"
"#, name, name);

        // Write to file
        match fs::write(&filepath, template) {
            Ok(_) => {
                self.show_info("Success", &format!("Created workflow: {}", filename));
                // Refresh workflow list
                self.load_workflows().await?;
            }
            Err(e) => {
                self.show_error("Creation Failed", &format!("Failed to create workflow: {}", e));
            }
        }

        Ok(())
    }

    /// Rename workflow
    async fn rename_workflow(&mut self, name: String) -> Result<()> {
        // TODO: Implement workflow renaming
        let _ = name; // Suppress warning
        Ok(())
    }

    /// Set workflow description
    async fn set_workflow_description(&mut self, _description: String) -> Result<()> {
        // Note: DSLWorkflow doesn't have a description field
        // Description could be stored in metadata or as a comment
        // For now, this is a no-op
        Ok(())
    }

    /// Save workflow with new name
    async fn save_workflow_as(&mut self, name: String) -> Result<()> {
        // TODO: Implement save as
        let _ = name; // Suppress warning
        Ok(())
    }

    /// Execute selected workflow
    async fn execute_selected_workflow(&mut self) -> Result<()> {
        // TODO: Implement workflow execution
        Ok(())
    }

    /// Generate workflow from description
    async fn generate_workflow(&mut self, description: String) -> Result<()> {
        // TODO: Implement workflow generation
        let _ = description; // Suppress warning
        Ok(())
    }

    /// Generate workflow from natural language input in generator
    async fn generate_workflow_from_nl(&mut self) -> Result<()> {
        // TODO: Implement NL workflow generation using the generator state
        // This will call the nl_generator module with the nl_input
        self.state.generator_state.status = crate::tui::views::generator::GenerationStatus::InProgress {
            progress: "Generating workflow...".to_string(),
        };

        // For now, just show a placeholder
        self.show_info("Generation", "Workflow generation not yet implemented");
        self.state.generator_state.status = crate::tui::views::generator::GenerationStatus::Idle;

        Ok(())
    }

    /// Accept the generated workflow and save it
    async fn accept_generated_workflow(&mut self) -> Result<()> {
        if let Some(ref yaml) = self.state.generator_state.generated_yaml {
            // TODO: Save the generated workflow to a file
            let _ = yaml; // Suppress warning
            self.show_success("Success", "Workflow accepted (save not yet implemented)");
            self.state.view_mode = ViewMode::WorkflowList;
        }
        Ok(())
    }

    /// Show confirmation dialog for quitting
    fn confirm_quit(&mut self) {
        self.state.modal = Some(Modal::Confirm {
            title: "Quit Application".to_string(),
            message: "Are you sure you want to quit?".to_string(),
            action: ConfirmAction::Exit,
        });
    }

    /// Show confirmation dialog for deleting workflow
    fn confirm_delete_workflow(&mut self) {
        if let Some(entry) = self.state.workflows.get(self.state.selected_workflow) {
            let path = entry.path.clone();
            self.state.modal = Some(Modal::Confirm {
                title: "Delete Workflow".to_string(),
                message: format!("Delete workflow '{}'?", entry.name),
                action: ConfirmAction::DeleteWorkflow(path),
            });
        }
    }

    /// Show confirmation dialog for stopping execution
    fn confirm_stop_execution(&mut self) {
        self.state.modal = Some(Modal::Confirm {
            title: "Stop Execution".to_string(),
            message: "Stop the running workflow?".to_string(),
            action: ConfirmAction::StopExecution,
        });
    }

    /// Prompt for new workflow creation
    fn prompt_create_workflow(&mut self) {
        self.state.input_buffer.clear();
        self.state.modal = Some(Modal::Input {
            title: "Create Workflow".to_string(),
            prompt: "Enter workflow name:".to_string(),
            default: String::new(),
            action: InputAction::CreateWorkflow,
        });
    }

    /// Show error modal
    fn show_error(&mut self, title: &str, message: &str) {
        self.state.modal = Some(Modal::Error {
            title: title.to_string(),
            message: message.to_string(),
        });
    }

    /// Show info modal
    fn show_info(&mut self, title: &str, message: &str) {
        self.state.modal = Some(Modal::Info {
            title: title.to_string(),
            message: message.to_string(),
        });
    }

    /// Show success modal
    fn show_success(&mut self, title: &str, message: &str) {
        self.state.modal = Some(Modal::Success {
            title: title.to_string(),
            message: message.to_string(),
        });
    }

    // View rendering methods (delegated to ui module components)

    fn render_viewer_static(
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        state: &AppState,
        theme: &Theme,
    ) {
        use super::ui::viewer;

        if let Some(ref workflow) = state.current_workflow {
            viewer::render(frame, area, workflow, &state.viewer_state, theme);
        }
    }

    fn render_editor_static(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &crate::tui::state::EditorState, theme: &Theme) {
        use crate::tui::views::editor;

        // Validate content and get feedback
        let feedback = editor::validate_and_get_feedback(&state.content);
        editor::render(frame, area, state, &feedback, theme);
    }

    fn render_generator_static(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, state: &crate::tui::views::generator::GeneratorState, theme: &Theme) {
        use crate::tui::views::generator;
        generator::render(frame, area, state, theme);
    }

    fn render_execution_monitor_static(
        _frame: &mut ratatui::Frame,
        _area: ratatui::layout::Rect,
    ) {
        // TODO: Implement in ui module
    }

    fn render_help_static(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, help_state: &mut crate::tui::help::HelpViewState, theme: &Theme) {
        use crate::tui::help::HelpView;
        let help_view = HelpView::new();
        help_view.render(frame, area, help_state, theme);
    }

    fn render_state_browser_static(
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        state: &mut super::views::state_browser::StateBrowserState,
        theme: &Theme,
    ) {
        use super::views::state_browser;
        state_browser::render_state_browser(frame, area, state, theme);
    }

    fn render_modal_static(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, modal: &Modal, input_buffer: &str, theme: &Theme) {
        use super::ui::ModalView;
        ModalView::render(frame, area, modal, input_buffer, theme);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Cleanup terminal on drop
        let _ = disable_raw_mode();
        let _ = self.terminal.backend_mut().execute(LeaveAlternateScreen);
    }
}

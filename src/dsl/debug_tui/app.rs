//! Main TUI application
use super::events::{Event, EventHandler};
use super::layout::Pane;
use super::ui::{render, AppUI};
use crate::dsl::debugger::{DebuggerState, Inspector};
use crate::dsl::repl::parse_command;
use crate::dsl::DSLExecutor;
use crate::error::Result;

use crossterm::{
    event::{KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Main Debug TUI application
pub struct DebugTUI {
    /// Terminal backend
    terminal: Terminal<CrosstermBackend<io::Stdout>>,

    /// Event handler
    events: EventHandler,

    /// UI state
    ui: AppUI,

    /// DSL Executor (optional)
    executor: Option<Arc<Mutex<DSLExecutor>>>,

    /// Debugger state
    debugger: Option<Arc<Mutex<DebuggerState>>>,

    /// Inspector
    inspector: Option<Arc<Inspector>>,

    /// Running flag
    running: bool,
}

impl DebugTUI {
    /// Create a new debug TUI
    pub fn new() -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            events: EventHandler::new(),
            ui: AppUI::new(),
            executor: None,
            debugger: None,
            inspector: None,
            running: false,
        })
    }

    /// Create TUI with executor
    pub fn with_executor(mut self, executor: DSLExecutor) -> Result<Self> {
        let debugger = executor.debugger().cloned();
        let inspector = executor.inspector().cloned();

        self.executor = Some(Arc::new(Mutex::new(executor)));
        self.debugger = debugger;
        self.inspector = inspector;

        Ok(self)
    }

    /// Run the TUI
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;

        // Start event loop
        self.events.start();

        // Main loop
        while self.running {
            // Render
            self.terminal.draw(|frame| {
                let ui = &self.ui;
                let debugger = self.debugger.as_ref();
                let inspector = self.inspector.as_ref();

                // Render UI (blocking)
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        render(frame, ui, debugger, inspector).await;
                    })
                });
            })?;

            // Handle events
            if let Some(event) = self.events.next().await {
                self.handle_event(event).await?;
            }
        }

        Ok(())
    }

    /// Handle an event
    async fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) => self.handle_key_event(key).await?,
            Event::Resize(_, _) => {
                // Terminal will automatically redraw on next iteration
            }
            Event::Tick => {
                // Periodic update - could refresh state here
            }
            Event::Quit => {
                self.running = false;
            }
        }

        Ok(())
    }

    /// Handle keyboard event
    async fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        // Global shortcuts (work in all panes)
        match key.code {
            // Quit
            KeyCode::Char('q') if !matches!(self.ui.focused_pane, Pane::Repl) => {
                self.running = false;
                return Ok(());
            }

            // Help
            KeyCode::Char('?') | KeyCode::F(1) => {
                self.ui.toggle_help();
                return Ok(());
            }

            // Tab navigation
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.ui.prev_pane();
                } else {
                    self.ui.next_pane();
                }
                return Ok(());
            }

            // Function keys for execution control
            KeyCode::F(5) => {
                // F5: Continue
                self.execute_repl_command("continue").await?;
                return Ok(());
            }

            KeyCode::F(9) => {
                // F9: Toggle breakpoint (on current task)
                // TODO: Implement based on selected task in workflow tree
                return Ok(());
            }

            KeyCode::F(10) => {
                // F10: Step over
                self.execute_repl_command("next").await?;
                return Ok(());
            }

            KeyCode::F(11) => {
                // F11: Step into/out
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.execute_repl_command("finish").await?;
                } else {
                    self.execute_repl_command("stepi").await?;
                }
                return Ok(());
            }

            _ => {}
        }

        // Close help with Esc
        if self.ui.show_help && key.code == KeyCode::Esc {
            self.ui.toggle_help();
            return Ok(());
        }

        // Pane-specific handling
        match self.ui.focused_pane {
            Pane::Repl => self.handle_repl_key(key).await?,
            Pane::WorkflowTree => self.handle_workflow_tree_key(key).await?,
            Pane::Variables => self.handle_variables_key(key).await?,
            Pane::Timeline => self.handle_timeline_key(key).await?,
            Pane::Help => {}
        }

        Ok(())
    }

    /// Handle key in REPL pane
    async fn handle_repl_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char(c) => {
                self.ui.repl_add_char(c);
            }
            KeyCode::Backspace => {
                self.ui.repl_backspace();
            }
            KeyCode::Enter => {
                let command = self.ui.repl_take_input();
                if !command.is_empty() {
                    self.execute_repl_command(&command).await?;
                }
            }
            KeyCode::Esc => {
                self.ui.repl_clear();
            }
            _ => {}
        }

        Ok(())
    }

    /// Execute a REPL command
    async fn execute_repl_command(&mut self, command_str: &str) -> Result<()> {
        // Parse command
        let command = match parse_command(command_str) {
            Ok(cmd) => cmd,
            Err(e) => {
                // TODO: Show error in status bar or message area
                eprintln!("Parse error: {}", e);
                return Ok(());
            }
        };

        // Execute command
        // TODO: Integrate with actual REPL command execution
        // For now, just handle basic commands
        if matches!(command, crate::dsl::repl::ReplCommand::Quit) {
            self.running = false;
        }

        Ok(())
    }

    /// Handle key in workflow tree pane
    async fn handle_workflow_tree_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                // TODO: Navigate workflow tree
            }
            KeyCode::Enter => {
                // TODO: Expand/collapse node
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle key in variables pane
    async fn handle_variables_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Down => {
                // TODO: Scroll variables
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle key in timeline pane
    async fn handle_timeline_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Down => {
                // TODO: Scroll timeline
            }
            _ => {}
        }

        Ok(())
    }
}

impl Drop for DebugTUI {
    fn drop(&mut self) {
        // Cleanup terminal
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

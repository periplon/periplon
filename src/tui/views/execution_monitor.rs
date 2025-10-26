//! Real-time Workflow Execution Monitor
//!
//! Provides a comprehensive view for monitoring workflow execution in real-time.
//!
//! Features:
//! - Overall workflow progress bar
//! - Task status list with live updates
//! - Real-time log output stream
//! - Task dependency visualization
//! - Execution statistics (timing, resource usage)
//! - Pause/Resume/Cancel controls
//! - Auto-scrolling log viewer
//!
//! Layout:
//! ```text
//! ┌─ Execution Monitor ──────────────────────────────────┐
//! │ Workflow: Test Workflow v1.0.0                       │
//! │ Status: Running | Elapsed: 00:45 | Progress: 60%    │
//! │ ██████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░       │
//! ├──────────────────────────────────────────────────────┤
//! │ Task Status                    │ Output Stream       │
//! │ ✓ Task A (5s)                  │ [12:34:56] INFO    │
//! │ ◐ Task B (running, 3s)         │ Starting task B... │
//! │ ○ Task C (pending)             │ [12:34:58] DEBUG   │
//! │ ✗ Task D (failed)              │ Error in task D    │
//! │                                │ [12:35:00] INFO    │
//! │                                │ Task A completed   │
//! │                                │                     │
//! ├──────────────────────────────────────────────────────┤
//! │ Statistics                                           │
//! │ Tasks: 2/5 completed | 1 running | 1 failed         │
//! │ Avg. Task Time: 4.5s | Total Cost: $0.12           │
//! ├──────────────────────────────────────────────────────┤
//! │ Ctrl+P: Pause | Ctrl+C: Cancel | Esc: Back          │
//! └──────────────────────────────────────────────────────┘
//! ```

use crate::dsl::schema::DSLWorkflow;
use crate::dsl::state::{WorkflowState, WorkflowStatus as DslWorkflowStatus};
use crate::dsl::task_graph::TaskStatus as DslTaskStatus;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Gauge, List, ListItem, Paragraph, Wrap,
    },
    Frame,
};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Execution monitor state
#[derive(Debug, Clone)]
pub struct ExecutionMonitorState {
    /// Workflow being executed
    pub workflow: DSLWorkflow,

    /// Overall execution status
    pub status: ExecutionStatus,

    /// Task execution states
    pub tasks: HashMap<String, TaskExecutionState>,

    /// Task execution order (for display)
    pub task_order: Vec<String>,

    /// Log entries
    pub logs: Vec<LogEntry>,

    /// Log scroll offset
    pub log_scroll: usize,

    /// Task list scroll offset
    pub task_scroll: usize,

    /// Currently selected task (for details view)
    pub selected_task: Option<String>,

    /// Start time
    pub start_time: SystemTime,

    /// End time (when completed)
    pub end_time: Option<SystemTime>,

    /// Pause time (when paused)
    pub pause_time: Option<SystemTime>,

    /// Total paused duration
    pub total_paused_duration: Duration,

    /// Auto-scroll logs
    pub auto_scroll_logs: bool,

    /// Show task details panel
    pub show_task_details: bool,

    /// Execution statistics
    pub stats: ExecutionStatistics,

    /// Active panel focus
    pub focus: MonitorPanel,
}

/// Monitor panel focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorPanel {
    /// Task list panel
    TaskList,
    /// Log output panel
    LogOutput,
    /// Task details panel
    TaskDetails,
}

/// Overall execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Execution is running
    Running,
    /// Execution is paused
    Paused,
    /// Execution completed successfully
    Completed,
    /// Execution failed
    Failed,
    /// Execution was cancelled
    Cancelled,
}

/// Task execution state
#[derive(Debug, Clone)]
pub struct TaskExecutionState {
    /// Task ID
    pub task_id: String,

    /// Task description
    pub description: String,

    /// Current status
    pub status: TaskStatus,

    /// Agent assigned to this task
    pub agent: Option<String>,

    /// Start time
    pub start_time: Option<SystemTime>,

    /// End time
    pub end_time: Option<SystemTime>,

    /// Dependencies
    pub dependencies: Vec<String>,

    /// Progress (0-100 for running tasks)
    pub progress: u8,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Output file path (if applicable)
    pub output_path: Option<String>,

    /// Cost incurred by this task
    pub cost: Option<f64>,

    /// Token usage
    pub tokens: Option<TokenUsage>,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Pending execution
    Pending,
    /// Currently running
    Running,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// Skipped (dependencies failed)
    Skipped,
}

/// Token usage statistics
#[derive(Debug, Clone, Copy)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: SystemTime,

    /// Log level
    pub level: LogLevel,

    /// Message
    pub message: String,

    /// Associated task (if any)
    pub task_id: Option<String>,

    /// Associated agent (if any)
    pub agent_id: Option<String>,
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// Execution statistics
#[derive(Debug, Clone, Default)]
pub struct ExecutionStatistics {
    /// Total tasks
    pub total_tasks: usize,

    /// Completed tasks
    pub completed_tasks: usize,

    /// Failed tasks
    pub failed_tasks: usize,

    /// Running tasks
    pub running_tasks: usize,

    /// Pending tasks
    pub pending_tasks: usize,

    /// Skipped tasks
    pub skipped_tasks: usize,

    /// Total cost
    pub total_cost: f64,

    /// Total input tokens
    pub total_input_tokens: u64,

    /// Total output tokens
    pub total_output_tokens: u64,

    /// Average task duration
    pub avg_task_duration: Option<Duration>,

    /// Estimated time remaining
    pub estimated_time_remaining: Option<Duration>,
}

impl ExecutionMonitorState {
    /// Create new execution monitor state
    pub fn new(workflow: DSLWorkflow) -> Self {
        let task_order: Vec<String> = workflow.tasks.keys().cloned().collect();
        let task_count = task_order.len();

        Self {
            workflow,
            status: ExecutionStatus::Running,
            tasks: HashMap::new(),
            task_order,
            logs: Vec::new(),
            log_scroll: 0,
            task_scroll: 0,
            selected_task: None,
            start_time: SystemTime::now(),
            end_time: None,
            pause_time: None,
            total_paused_duration: Duration::ZERO,
            auto_scroll_logs: true,
            show_task_details: false,
            stats: ExecutionStatistics {
                total_tasks: task_count,
                ..Default::default()
            },
            focus: MonitorPanel::TaskList,
        }
    }

    /// Create execution monitor state from WorkflowState
    ///
    /// This allows resuming or monitoring workflows that have existing state.
    pub fn from_workflow_state(workflow: DSLWorkflow, state: &WorkflowState) -> Self {
        let task_order: Vec<String> = workflow.tasks.keys().cloned().collect();
        let task_count = task_order.len();

        // Convert DSL task statuses to monitor task execution states
        let mut tasks = HashMap::new();
        for (task_id, task_status) in &state.task_statuses {
            let monitor_status = match task_status {
                DslTaskStatus::Pending => TaskStatus::Pending,
                DslTaskStatus::Ready => TaskStatus::Pending,
                DslTaskStatus::Running => TaskStatus::Running,
                DslTaskStatus::Completed => TaskStatus::Completed,
                DslTaskStatus::Failed => TaskStatus::Failed,
                DslTaskStatus::Skipped => TaskStatus::Skipped,
            };

            let task_exec_state = TaskExecutionState {
                task_id: task_id.clone(),
                description: workflow
                    .tasks
                    .get(task_id)
                    .map(|t| t.description.clone())
                    .unwrap_or_else(|| task_id.clone()),
                status: monitor_status,
                agent: workflow.tasks.get(task_id).and_then(|t| t.agent.clone()),
                start_time: state.task_start_times.get(task_id).copied(),
                end_time: state.task_end_times.get(task_id).copied(),
                dependencies: workflow
                    .tasks
                    .get(task_id)
                    .map(|t| t.depends_on.clone())
                    .unwrap_or_default(),
                progress: if monitor_status == TaskStatus::Running {
                    50
                } else if monitor_status == TaskStatus::Completed {
                    100
                } else {
                    0
                },
                error: state.task_errors.get(task_id).cloned(),
                output_path: workflow.tasks.get(task_id).and_then(|t| t.output.clone()),
                cost: None,
                tokens: None,
            };

            tasks.insert(task_id.clone(), task_exec_state);
        }

        // Convert DSL workflow status to monitor execution status
        let monitor_status = match state.status {
            DslWorkflowStatus::Running => ExecutionStatus::Running,
            DslWorkflowStatus::Completed => ExecutionStatus::Completed,
            DslWorkflowStatus::Failed => ExecutionStatus::Failed,
            DslWorkflowStatus::Paused => ExecutionStatus::Paused,
        };

        // Calculate statistics from state
        let stats = ExecutionStatistics {
            total_tasks: task_count,
            completed_tasks: state.get_completed_tasks().len(),
            failed_tasks: state.get_failed_tasks().len(),
            running_tasks: tasks.values().filter(|t| t.status == TaskStatus::Running).count(),
            pending_tasks: state.get_pending_tasks().len(),
            skipped_tasks: tasks.values().filter(|t| t.status == TaskStatus::Skipped).count(),
            total_cost: 0.0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            avg_task_duration: None,
            estimated_time_remaining: None,
        };

        Self {
            workflow,
            status: monitor_status,
            tasks,
            task_order,
            logs: Vec::new(),
            log_scroll: 0,
            task_scroll: 0,
            selected_task: None,
            start_time: state.started_at,
            end_time: state.ended_at,
            pause_time: None,
            total_paused_duration: Duration::ZERO,
            auto_scroll_logs: true,
            show_task_details: false,
            stats,
            focus: MonitorPanel::TaskList,
        }
    }

    /// Sync monitor state with WorkflowState
    ///
    /// Updates the monitor's task states from the current WorkflowState.
    /// Useful for real-time updates during execution.
    pub fn sync_with_workflow_state(&mut self, state: &WorkflowState) {
        // Update task statuses
        for (task_id, task_status) in &state.task_statuses {
            if let Some(task) = self.tasks.get_mut(task_id) {
                let new_status = match task_status {
                    DslTaskStatus::Pending => TaskStatus::Pending,
                    DslTaskStatus::Ready => TaskStatus::Pending,
                    DslTaskStatus::Running => TaskStatus::Running,
                    DslTaskStatus::Completed => TaskStatus::Completed,
                    DslTaskStatus::Failed => TaskStatus::Failed,
                    DslTaskStatus::Skipped => TaskStatus::Skipped,
                };

                task.status = new_status;
                task.start_time = state.task_start_times.get(task_id).copied();
                task.end_time = state.task_end_times.get(task_id).copied();
                task.error = state.task_errors.get(task_id).cloned();
            }
        }

        // Update overall status
        self.status = match state.status {
            DslWorkflowStatus::Running => ExecutionStatus::Running,
            DslWorkflowStatus::Completed => ExecutionStatus::Completed,
            DslWorkflowStatus::Failed => ExecutionStatus::Failed,
            DslWorkflowStatus::Paused => ExecutionStatus::Paused,
        };

        self.end_time = state.ended_at;

        // Recalculate statistics
        self.stats.completed_tasks = state.get_completed_tasks().len();
        self.stats.failed_tasks = state.get_failed_tasks().len();
        self.stats.pending_tasks = state.get_pending_tasks().len();
        self.stats.running_tasks = self.tasks.values().filter(|t| t.status == TaskStatus::Running).count();
        self.recalculate_avg_duration();
    }

    /// Add a log entry
    pub fn add_log(&mut self, level: LogLevel, message: String, task_id: Option<String>, agent_id: Option<String>) {
        self.logs.push(LogEntry {
            timestamp: SystemTime::now(),
            level,
            message,
            task_id,
            agent_id,
        });

        // Auto-scroll to bottom if enabled
        if self.auto_scroll_logs {
            self.log_scroll = self.logs.len().saturating_sub(1);
        }
    }

    /// Update task status
    pub fn update_task(&mut self, task_id: String, status: TaskStatus) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            let prev_status = task.status;
            task.status = status;

            // Update timestamps
            match status {
                TaskStatus::Running => {
                    if task.start_time.is_none() {
                        task.start_time = Some(SystemTime::now());
                    }
                }
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Skipped => {
                    if task.end_time.is_none() {
                        task.end_time = Some(SystemTime::now());
                    }
                }
                _ => {}
            }

            // Update statistics
            self.update_statistics(prev_status, status);
        }
    }

    /// Update execution statistics
    fn update_statistics(&mut self, prev_status: TaskStatus, new_status: TaskStatus) {
        // Decrement previous status count
        match prev_status {
            TaskStatus::Pending => self.stats.pending_tasks = self.stats.pending_tasks.saturating_sub(1),
            TaskStatus::Running => self.stats.running_tasks = self.stats.running_tasks.saturating_sub(1),
            TaskStatus::Completed => self.stats.completed_tasks = self.stats.completed_tasks.saturating_sub(1),
            TaskStatus::Failed => self.stats.failed_tasks = self.stats.failed_tasks.saturating_sub(1),
            TaskStatus::Skipped => self.stats.skipped_tasks = self.stats.skipped_tasks.saturating_sub(1),
        }

        // Increment new status count
        match new_status {
            TaskStatus::Pending => self.stats.pending_tasks += 1,
            TaskStatus::Running => self.stats.running_tasks += 1,
            TaskStatus::Completed => self.stats.completed_tasks += 1,
            TaskStatus::Failed => self.stats.failed_tasks += 1,
            TaskStatus::Skipped => self.stats.skipped_tasks += 1,
        }

        // Recalculate average task duration
        self.recalculate_avg_duration();
    }

    /// Recalculate average task duration
    fn recalculate_avg_duration(&mut self) {
        let completed_tasks: Vec<&TaskExecutionState> = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Completed)
            .collect();

        if completed_tasks.is_empty() {
            self.stats.avg_task_duration = None;
            return;
        }

        let total_duration: Duration = completed_tasks
            .iter()
            .filter_map(|t| {
                if let (Some(start), Some(end)) = (t.start_time, t.end_time) {
                    end.duration_since(start).ok()
                } else {
                    None
                }
            })
            .sum();

        let avg = total_duration / completed_tasks.len() as u32;
        self.stats.avg_task_duration = Some(avg);

        // Estimate time remaining
        if let Some(avg_duration) = self.stats.avg_task_duration {
            let remaining_tasks = self.stats.pending_tasks + self.stats.running_tasks;
            self.stats.estimated_time_remaining = Some(avg_duration * remaining_tasks as u32);
        }
    }

    /// Get overall progress (0-100)
    pub fn progress_percentage(&self) -> u16 {
        if self.stats.total_tasks == 0 {
            return 100;
        }

        let completed = self.stats.completed_tasks + self.stats.failed_tasks + self.stats.skipped_tasks;
        ((completed as f64 / self.stats.total_tasks as f64) * 100.0) as u16
    }

    /// Get elapsed time (accounting for pauses)
    pub fn elapsed_time(&self) -> Duration {
        let now = SystemTime::now();
        let end = self.end_time.unwrap_or(now);

        if let Ok(duration) = end.duration_since(self.start_time) {
            duration.saturating_sub(self.total_paused_duration)
        } else {
            Duration::ZERO
        }
    }

    /// Pause execution
    pub fn pause(&mut self) {
        if self.status == ExecutionStatus::Running {
            self.status = ExecutionStatus::Paused;
            self.pause_time = Some(SystemTime::now());
        }
    }

    /// Resume execution
    pub fn resume(&mut self) {
        if self.status == ExecutionStatus::Paused {
            self.status = ExecutionStatus::Running;
            if let Some(pause_time) = self.pause_time {
                if let Ok(paused_duration) = SystemTime::now().duration_since(pause_time) {
                    self.total_paused_duration += paused_duration;
                }
                self.pause_time = None;
            }
        }
    }

    /// Cancel execution
    pub fn cancel(&mut self) {
        self.status = ExecutionStatus::Cancelled;
        self.end_time = Some(SystemTime::now());
    }

    /// Complete execution
    pub fn complete(&mut self, success: bool) {
        self.status = if success {
            ExecutionStatus::Completed
        } else {
            ExecutionStatus::Failed
        };
        self.end_time = Some(SystemTime::now());
    }

    /// Toggle auto-scroll
    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll_logs = !self.auto_scroll_logs;
    }

    /// Scroll logs up
    pub fn scroll_logs_up(&mut self) {
        self.auto_scroll_logs = false;
        self.log_scroll = self.log_scroll.saturating_sub(1);
    }

    /// Scroll logs down
    pub fn scroll_logs_down(&mut self) {
        self.auto_scroll_logs = false;
        if self.log_scroll < self.logs.len().saturating_sub(1) {
            self.log_scroll += 1;
        }
    }

    /// Scroll task list up
    pub fn scroll_tasks_up(&mut self) {
        self.task_scroll = self.task_scroll.saturating_sub(1);
    }

    /// Scroll task list down
    pub fn scroll_tasks_down(&mut self) {
        if self.task_scroll < self.task_order.len().saturating_sub(1) {
            self.task_scroll += 1;
        }
    }

    /// Toggle task details view
    pub fn toggle_task_details(&mut self) {
        self.show_task_details = !self.show_task_details;
    }

    /// Select task for details
    pub fn select_task(&mut self, task_id: String) {
        self.selected_task = Some(task_id);
        self.show_task_details = true;
    }

    /// Switch focus to next panel
    pub fn next_panel(&mut self) {
        self.focus = match self.focus {
            MonitorPanel::TaskList => MonitorPanel::LogOutput,
            MonitorPanel::LogOutput => {
                if self.show_task_details {
                    MonitorPanel::TaskDetails
                } else {
                    MonitorPanel::TaskList
                }
            }
            MonitorPanel::TaskDetails => MonitorPanel::TaskList,
        };
    }
}

/// Render the execution monitor view
pub fn render(frame: &mut Frame, area: Rect, state: &ExecutionMonitorState, theme: &Theme) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header with progress
            Constraint::Min(0),     // Main content area
            Constraint::Length(4),  // Statistics panel
            Constraint::Length(1),  // Shortcuts bar
        ])
        .split(area);

    // Render header with progress
    render_header(frame, chunks[0], state, theme);

    // Split main content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if state.show_task_details {
            vec![
                Constraint::Percentage(30), // Task list
                Constraint::Percentage(35), // Log output
                Constraint::Percentage(35), // Task details
            ]
        } else {
            vec![
                Constraint::Percentage(40), // Task list
                Constraint::Percentage(60), // Log output
            ]
        })
        .split(chunks[1]);

    // Render task list
    render_task_list(frame, content_chunks[0], state, theme);

    // Render log output
    render_log_output(frame, content_chunks[1], state, theme);

    // Render task details if visible
    if state.show_task_details && content_chunks.len() > 2 {
        render_task_details(frame, content_chunks[2], state, theme);
    }

    // Render statistics
    render_statistics(frame, chunks[2], state, theme);

    // Render shortcuts bar
    render_shortcuts(frame, chunks[3], state, theme);
}

/// Render header with workflow info and progress
fn render_header(frame: &mut Frame, area: Rect, state: &ExecutionMonitorState, theme: &Theme) {
    // Create layout for header
    let header_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title and status
            Constraint::Length(1), // Progress bar
            Constraint::Length(1), // Border spacing
        ])
        .split(area);

    // Status icon and color
    let (status_icon, status_color) = match state.status {
        ExecutionStatus::Running => ("◐", theme.accent),
        ExecutionStatus::Paused => ("⏸", theme.warning),
        ExecutionStatus::Completed => ("✓", theme.success),
        ExecutionStatus::Failed => ("✗", theme.error),
        ExecutionStatus::Cancelled => ("⊗", theme.muted),
    };

    let status_text = format!("{:?}", state.status);
    let elapsed = format_duration(state.elapsed_time());
    let progress = state.progress_percentage();

    // Title line
    let title_line = Line::from(vec![
        Span::styled(
            "Execution Monitor",
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | Workflow: ", Style::default().fg(theme.muted)),
        Span::styled(&state.workflow.name, Style::default().fg(theme.accent)),
        Span::styled(
            format!(" v{}", state.workflow.version),
            Style::default().fg(theme.muted),
        ),
        Span::styled(" | Status: ", Style::default().fg(theme.muted)),
        Span::styled(status_icon, Style::default().fg(status_color)),
        Span::styled(
            format!(" {} ", status_text),
            Style::default().fg(status_color),
        ),
        Span::styled("| Elapsed: ", Style::default().fg(theme.muted)),
        Span::styled(elapsed, Style::default().fg(theme.fg)),
        Span::styled(
            format!(" | Progress: {}%", progress),
            Style::default().fg(theme.muted),
        ),
    ]);

    let title = Paragraph::new(title_line).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(theme.border)),
    );

    frame.render_widget(title, header_chunks[0]);

    // Progress bar
    let progress_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(theme.border)),
        )
        .gauge_style(Style::default().fg(theme.success).bg(theme.bg))
        .percent(progress);

    frame.render_widget(progress_gauge, header_chunks[1]);

    // Border spacing
    let spacing = Block::default()
        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(theme.border));

    frame.render_widget(spacing, header_chunks[2]);
}

/// Render task status list
fn render_task_list(frame: &mut Frame, area: Rect, state: &ExecutionMonitorState, theme: &Theme) {
    let is_focused = state.focus == MonitorPanel::TaskList;

    let border_style = if is_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.border)
    };

    let items: Vec<ListItem> = state
        .task_order
        .iter()
        .filter_map(|task_id| {
            state.tasks.get(task_id).map(|task| {
                let (icon, icon_color) = match task.status {
                    TaskStatus::Pending => ("○", theme.muted),
                    TaskStatus::Running => ("◐", theme.accent),
                    TaskStatus::Completed => ("✓", theme.success),
                    TaskStatus::Failed => ("✗", theme.error),
                    TaskStatus::Skipped => ("⊘", theme.warning),
                };

                let duration_str = if let (Some(start), end_time) = (task.start_time, task.end_time) {
                    if let Some(end) = end_time {
                        if let Ok(duration) = end.duration_since(start) {
                            format!(" ({})", format_duration(duration))
                        } else {
                            String::new()
                        }
                    } else if task.status == TaskStatus::Running {
                        if let Ok(duration) = SystemTime::now().duration_since(start) {
                            format!(" (running, {})", format_duration(duration))
                        } else {
                            String::from(" (running)")
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let task_name = if task.description.is_empty() {
                    task_id.clone()
                } else {
                    task.description.clone()
                };

                let line = Line::from(vec![
                    Span::styled(icon, Style::default().fg(icon_color)),
                    Span::raw(" "),
                    Span::styled(task_name, Style::default().fg(theme.fg)),
                    Span::styled(duration_str, Style::default().fg(theme.muted)),
                ]);

                ListItem::new(line)
            })
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Task Status "),
        )
        .highlight_style(
            Style::default()
                .bg(theme.bg)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(list, area);
}

/// Render log output stream
fn render_log_output(frame: &mut Frame, area: Rect, state: &ExecutionMonitorState, theme: &Theme) {
    let is_focused = state.focus == MonitorPanel::LogOutput;

    let border_style = if is_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.border)
    };

    let title = if state.auto_scroll_logs {
        " Output Stream [Auto-scroll: ON] "
    } else {
        " Output Stream [Auto-scroll: OFF] "
    };

    let log_lines: Vec<Line> = state
        .logs
        .iter()
        .map(|entry| {
            let timestamp = format_timestamp(entry.timestamp);
            let (level_str, level_color) = match entry.level {
                LogLevel::Debug => ("DEBUG", theme.muted),
                LogLevel::Info => ("INFO ", theme.accent),
                LogLevel::Warning => ("WARN ", theme.warning),
                LogLevel::Error => ("ERROR", theme.error),
            };

            let task_prefix = if let Some(ref task_id) = entry.task_id {
                format!("[{}] ", task_id)
            } else {
                String::new()
            };

            Line::from(vec![
                Span::styled(
                    format!("[{}] ", timestamp),
                    Style::default().fg(theme.muted),
                ),
                Span::styled(level_str, Style::default().fg(level_color)),
                Span::raw(" "),
                Span::styled(task_prefix, Style::default().fg(theme.muted)),
                Span::styled(&entry.message, Style::default().fg(theme.fg)),
            ])
        })
        .collect();

    let text = Text::from(log_lines);

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title),
        )
        .scroll((state.log_scroll as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render task details panel
fn render_task_details(frame: &mut Frame, area: Rect, state: &ExecutionMonitorState, theme: &Theme) {
    let is_focused = state.focus == MonitorPanel::TaskDetails;

    let border_style = if is_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.border)
    };

    let content = if let Some(ref task_id) = state.selected_task {
        if let Some(task) = state.tasks.get(task_id) {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Task ID: ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
                    Span::styled(&task.task_id, Style::default().fg(theme.fg)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
                    Span::styled(&task.description, Style::default().fg(theme.fg)),
                ]),
                Line::from(""),
                {
                    let status_text = format!("{:?}", task.status);
                    Line::from(vec![
                        Span::styled("Status: ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
                        Span::styled(status_text, Style::default().fg(theme.accent)),
                    ])
                },
            ];

            if let Some(ref agent) = task.agent {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Agent: ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
                    Span::styled(agent, Style::default().fg(theme.fg)),
                ]));
            }

            if let Some(start) = task.start_time {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Started: ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
                    Span::styled(format_timestamp(start), Style::default().fg(theme.fg)),
                ]));

                if let Some(end) = task.end_time {
                    if let Ok(duration) = end.duration_since(start) {
                        lines.push(Line::from(vec![
                            Span::styled("Duration: ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
                            Span::styled(format_duration(duration), Style::default().fg(theme.fg)),
                        ]));
                    }
                }
            }

            if let Some(ref error) = task.error {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(Span::styled(error, Style::default().fg(theme.error))));
            }

            if let Some(cost) = task.cost {
                lines.push(Line::from(""));
                let cost_text = format!("${:.4}", cost);
                lines.push(Line::from(vec![
                    Span::styled("Cost: ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
                    Span::styled(cost_text, Style::default().fg(theme.fg)),
                ]));
            }

            if let Some(tokens) = task.tokens {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Tokens: ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
                ]));
                let input_text = format!("{}", tokens.input_tokens);
                lines.push(Line::from(vec![
                    Span::raw("  Input: "),
                    Span::styled(input_text, Style::default().fg(theme.fg)),
                ]));
                let output_text = format!("{}", tokens.output_tokens);
                lines.push(Line::from(vec![
                    Span::raw("  Output: "),
                    Span::styled(output_text, Style::default().fg(theme.fg)),
                ]));
            }

            Text::from(lines)
        } else {
            Text::from(vec![
                Line::from(""),
                Line::from(Span::styled("Task not found", Style::default().fg(theme.error))),
            ])
        }
    } else {
        Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No task selected",
                Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Select a task from the list to view details",
                Style::default().fg(theme.muted),
            )),
        ])
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Task Details "),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render statistics panel
fn render_statistics(frame: &mut Frame, area: Rect, state: &ExecutionMonitorState, theme: &Theme) {
    let stats = &state.stats;

    let lines = vec![
        Line::from(vec![
            Span::styled("Tasks: ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{}/{} completed", stats.completed_tasks, stats.total_tasks),
                Style::default().fg(theme.success),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("{} running", stats.running_tasks),
                Style::default().fg(theme.accent),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("{} failed", stats.failed_tasks),
                Style::default().fg(theme.error),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("{} pending", stats.pending_tasks),
                Style::default().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            Span::styled("Avg. Task Time: ", Style::default().fg(theme.muted)),
            Span::styled(
                if let Some(avg) = stats.avg_task_duration {
                    format_duration(avg)
                } else {
                    "N/A".to_string()
                },
                Style::default().fg(theme.fg),
            ),
            Span::raw(" | "),
            Span::styled("Est. Remaining: ", Style::default().fg(theme.muted)),
            Span::styled(
                if let Some(remaining) = stats.estimated_time_remaining {
                    format_duration(remaining)
                } else {
                    "N/A".to_string()
                },
                Style::default().fg(theme.fg),
            ),
            Span::raw(" | "),
            Span::styled("Total Cost: ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("${:.4}", stats.total_cost),
                Style::default().fg(theme.fg),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Statistics "),
    );

    frame.render_widget(paragraph, area);
}

/// Render shortcuts bar
fn render_shortcuts(frame: &mut Frame, area: Rect, state: &ExecutionMonitorState, theme: &Theme) {
    let shortcuts = match state.status {
        ExecutionStatus::Running => "Ctrl+P: Pause | Ctrl+C: Cancel | Tab: Switch Panel | Esc: Back",
        ExecutionStatus::Paused => "Ctrl+P: Resume | Ctrl+C: Cancel | Tab: Switch Panel | Esc: Back",
        _ => "Tab: Switch Panel | Esc: Back",
    };

    let paragraph = Paragraph::new(shortcuts)
        .style(Style::default().fg(theme.muted).bg(theme.bg));

    frame.render_widget(paragraph, area);
}

/// Format duration as HH:MM:SS
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// Format timestamp as HH:MM:SS
fn format_timestamp(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
        let total_secs = duration.as_secs();
        let hours = (total_secs / 3600) % 24;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        "??:??:??".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_workflow() -> DSLWorkflow {
        DSLWorkflow {
            name: "Test Workflow".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            agents: HashMap::new(),
            tasks: {
                let mut tasks = HashMap::new();
                tasks.insert("task1".to_string(), Default::default());
                tasks.insert("task2".to_string(), Default::default());
                tasks
            },
            workflows: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        }
    }

    #[test]
    fn test_execution_monitor_state_creation() {
        let workflow = create_test_workflow();
        let state = ExecutionMonitorState::new(workflow);

        assert_eq!(state.status, ExecutionStatus::Running);
        assert_eq!(state.stats.total_tasks, 2);
        assert_eq!(state.focus, MonitorPanel::TaskList);
        assert!(state.auto_scroll_logs);
    }

    #[test]
    fn test_progress_calculation() {
        let workflow = create_test_workflow();
        let mut state = ExecutionMonitorState::new(workflow);

        state.stats.total_tasks = 4;
        state.stats.completed_tasks = 2;
        state.stats.failed_tasks = 1;

        assert_eq!(state.progress_percentage(), 75); // (2 + 1) / 4 * 100
    }

    #[test]
    fn test_pause_resume() {
        let workflow = create_test_workflow();
        let mut state = ExecutionMonitorState::new(workflow);

        assert_eq!(state.status, ExecutionStatus::Running);

        state.pause();
        assert_eq!(state.status, ExecutionStatus::Paused);
        assert!(state.pause_time.is_some());

        state.resume();
        assert_eq!(state.status, ExecutionStatus::Running);
        assert!(state.pause_time.is_none());
    }

    #[test]
    fn test_log_entry() {
        let workflow = create_test_workflow();
        let mut state = ExecutionMonitorState::new(workflow);

        state.add_log(
            LogLevel::Info,
            "Test message".to_string(),
            Some("task1".to_string()),
            None,
        );

        assert_eq!(state.logs.len(), 1);
        assert_eq!(state.logs[0].message, "Test message");
        assert_eq!(state.logs[0].level, LogLevel::Info);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(45)), "00:45");
        assert_eq!(format_duration(Duration::from_secs(125)), "02:05");
        assert_eq!(format_duration(Duration::from_secs(3665)), "01:01:05");
    }

    #[test]
    fn test_from_workflow_state() {
        let workflow = create_test_workflow();
        let mut state = WorkflowState::new("Test Workflow".to_string(), "1.0.0".to_string());

        // Add some task statuses
        state.update_task_status("task1", DslTaskStatus::Completed);
        state.update_task_status("task2", DslTaskStatus::Running);

        let monitor_state = ExecutionMonitorState::from_workflow_state(workflow, &state);

        assert_eq!(monitor_state.status, ExecutionStatus::Running);
        assert_eq!(monitor_state.stats.completed_tasks, 1);
        assert_eq!(monitor_state.tasks.get("task1").unwrap().status, TaskStatus::Completed);
        assert_eq!(monitor_state.tasks.get("task2").unwrap().status, TaskStatus::Running);
    }

    #[test]
    fn test_sync_with_workflow_state() {
        let workflow = create_test_workflow();
        let mut monitor_state = ExecutionMonitorState::new(workflow.clone());

        // Add initial tasks to monitor
        monitor_state.tasks.insert(
            "task1".to_string(),
            TaskExecutionState {
                task_id: "task1".to_string(),
                description: "Task 1".to_string(),
                status: TaskStatus::Pending,
                agent: None,
                start_time: None,
                end_time: None,
                dependencies: vec![],
                progress: 0,
                error: None,
                output_path: None,
                cost: None,
                tokens: None,
            },
        );

        // Create workflow state with updated status
        let mut state = WorkflowState::new("Test Workflow".to_string(), "1.0.0".to_string());
        state.update_task_status("task1", DslTaskStatus::Completed);

        // Sync monitor with workflow state
        monitor_state.sync_with_workflow_state(&state);

        assert_eq!(monitor_state.tasks.get("task1").unwrap().status, TaskStatus::Completed);
        assert_eq!(monitor_state.stats.completed_tasks, 1);
    }
}

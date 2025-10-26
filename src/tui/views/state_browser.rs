//! Workflow State Browser
//!
//! Provides an interactive UI for browsing, viewing, and resuming workflow states.
//! Features include state listing, filtering, detailed state inspection, and resume controls.

use crate::dsl::state::{StatePersistence, WorkflowState, WorkflowStatus};
use crate::error::Result;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
    Frame,
};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// State browser view state
#[derive(Debug, Clone)]
pub struct StateBrowserState {
    /// List of available workflow states
    pub states: Vec<StateEntry>,

    /// Currently selected state index
    pub selected_index: usize,

    /// List state for navigation
    pub list_state: ListState,

    /// Currently loaded detailed state (for details view)
    pub current_state: Option<WorkflowState>,

    /// Current view mode
    pub view_mode: StateBrowserViewMode,

    /// Search/filter query
    pub filter_query: String,

    /// Sort mode
    pub sort_mode: StateSortMode,

    /// Scroll offset for details view
    pub details_scroll: usize,

    /// Details page size
    pub details_page_size: usize,

    /// Scrollbar state for details
    pub scrollbar_state: ScrollbarState,

    /// State directory path
    pub state_dir: PathBuf,
}

impl StateBrowserState {
    /// Create new state browser state
    pub fn new(state_dir: PathBuf) -> Self {
        Self {
            states: Vec::new(),
            selected_index: 0,
            list_state: ListState::default(),
            current_state: None,
            view_mode: StateBrowserViewMode::List,
            filter_query: String::new(),
            sort_mode: StateSortMode::ModifiedDesc,
            details_scroll: 0,
            details_page_size: 20,
            scrollbar_state: ScrollbarState::default(),
            state_dir,
        }
    }

    /// Load available states from directory
    pub fn load_states(&mut self) -> Result<()> {
        let persistence = StatePersistence::new(&self.state_dir)?;
        let workflow_names = persistence.list_states()?;

        self.states.clear();
        for name in workflow_names {
            if let Ok(state) = persistence.load_state(&name) {
                let entry = StateEntry::from_state(state, &persistence);
                self.states.push(entry);
            }
        }

        self.apply_sort();
        self.selected_index = self.selected_index.min(self.states.len().saturating_sub(1));
        self.update_list_state();

        Ok(())
    }

    /// Get filtered states based on current filter query
    pub fn filtered_states(&self) -> Vec<&StateEntry> {
        if self.filter_query.is_empty() {
            self.states.iter().collect()
        } else {
            let query = self.filter_query.to_lowercase();
            self.states
                .iter()
                .filter(|s| {
                    s.workflow_name.to_lowercase().contains(&query)
                        || s.status_text().to_lowercase().contains(&query)
                })
                .collect()
        }
    }

    /// Apply current sort mode to states
    fn apply_sort(&mut self) {
        match self.sort_mode {
            StateSortMode::NameAsc => {
                self.states
                    .sort_by(|a, b| a.workflow_name.cmp(&b.workflow_name));
            }
            StateSortMode::NameDesc => {
                self.states
                    .sort_by(|a, b| b.workflow_name.cmp(&a.workflow_name));
            }
            StateSortMode::ModifiedAsc => {
                self.states.sort_by_key(|s| s.checkpoint_at);
            }
            StateSortMode::ModifiedDesc => {
                self.states
                    .sort_by(|a, b| b.checkpoint_at.cmp(&a.checkpoint_at));
            }
            StateSortMode::ProgressAsc => {
                self.states.sort_by(|a, b| {
                    a.progress
                        .partial_cmp(&b.progress)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            StateSortMode::ProgressDesc => {
                self.states.sort_by(|a, b| {
                    b.progress
                        .partial_cmp(&a.progress)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
    }

    /// Cycle to next sort mode
    pub fn next_sort_mode(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.apply_sort();
    }

    /// Select next state
    pub fn select_next(&mut self) {
        let filtered_count = self.filtered_states().len();
        if filtered_count > 0 {
            self.selected_index = (self.selected_index + 1).min(filtered_count - 1);
            self.update_list_state();
        }
    }

    /// Select previous state
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_list_state();
        }
    }

    /// Update internal list state for rendering
    fn update_list_state(&mut self) {
        self.list_state.select(Some(self.selected_index));
    }

    /// Load detailed state for selected workflow
    pub fn load_details(&mut self) -> Result<()> {
        let filtered = self.filtered_states();
        if self.selected_index < filtered.len() {
            let entry = filtered[self.selected_index];
            let persistence = StatePersistence::new(&self.state_dir)?;
            let state = persistence.load_state(&entry.workflow_name)?;
            self.current_state = Some(state);
            self.view_mode = StateBrowserViewMode::Details;
            self.details_scroll = 0;
        }
        Ok(())
    }

    /// Return to list view
    pub fn back_to_list(&mut self) {
        self.view_mode = StateBrowserViewMode::List;
        self.current_state = None;
        self.details_scroll = 0;
    }

    /// Scroll details view down
    pub fn scroll_details_down(&mut self, max_lines: usize) {
        if self.details_scroll < max_lines.saturating_sub(self.details_page_size) {
            self.details_scroll += 1;
            self.scrollbar_state = self.scrollbar_state.position(self.details_scroll);
        }
    }

    /// Scroll details view up
    pub fn scroll_details_up(&mut self) {
        self.details_scroll = self.details_scroll.saturating_sub(1);
        self.scrollbar_state = self.scrollbar_state.position(self.details_scroll);
    }

    /// Page down in details view
    pub fn page_details_down(&mut self, max_lines: usize) {
        let max_scroll = max_lines.saturating_sub(self.details_page_size);
        self.details_scroll = (self.details_scroll + self.details_page_size).min(max_scroll);
        self.scrollbar_state = self.scrollbar_state.position(self.details_scroll);
    }

    /// Page up in details view
    pub fn page_details_up(&mut self) {
        self.details_scroll = self.details_scroll.saturating_sub(self.details_page_size);
        self.scrollbar_state = self.scrollbar_state.position(self.details_scroll);
    }

    /// Check if selected state can be resumed
    pub fn can_resume_selected(&self) -> bool {
        if let Some(state) = &self.current_state {
            state.can_resume()
        } else {
            let filtered = self.filtered_states();
            if self.selected_index < filtered.len() {
                let entry = filtered[self.selected_index];
                matches!(
                    entry.status,
                    WorkflowStatus::Running | WorkflowStatus::Paused
                )
            } else {
                false
            }
        }
    }

    /// Get selected state entry
    pub fn selected_state(&self) -> Option<&StateEntry> {
        let filtered = self.filtered_states();
        filtered.get(self.selected_index).copied()
    }

    /// Delete selected state
    pub fn delete_selected(&mut self) -> Result<()> {
        let filtered = self.filtered_states();
        if let Some(entry) = filtered.get(self.selected_index) {
            let persistence = StatePersistence::new(&self.state_dir)?;
            persistence.delete_state(&entry.workflow_name)?;
            self.load_states()?;
        }
        Ok(())
    }

    /// Update page size based on terminal height
    pub fn update_page_size(&mut self, height: usize) {
        self.details_page_size = height.saturating_sub(5);
    }
}

impl Default for StateBrowserState {
    fn default() -> Self {
        Self::new(PathBuf::from(".workflow_states"))
    }
}

/// State browser view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateBrowserViewMode {
    /// List of available states
    List,
    /// Detailed view of selected state
    Details,
}

/// State sort mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateSortMode {
    NameAsc,
    NameDesc,
    ModifiedAsc,
    ModifiedDesc,
    ProgressAsc,
    ProgressDesc,
}

impl StateSortMode {
    /// Get next sort mode in cycle
    pub fn next(self) -> Self {
        match self {
            Self::NameAsc => Self::NameDesc,
            Self::NameDesc => Self::ModifiedAsc,
            Self::ModifiedAsc => Self::ModifiedDesc,
            Self::ModifiedDesc => Self::ProgressAsc,
            Self::ProgressAsc => Self::ProgressDesc,
            Self::ProgressDesc => Self::NameAsc,
        }
    }

    /// Get display name for sort mode
    pub fn display_name(self) -> &'static str {
        match self {
            Self::NameAsc => "Name ↑",
            Self::NameDesc => "Name ↓",
            Self::ModifiedAsc => "Modified ↑",
            Self::ModifiedDesc => "Modified ↓",
            Self::ProgressAsc => "Progress ↑",
            Self::ProgressDesc => "Progress ↓",
        }
    }
}

/// State entry for list display
#[derive(Debug, Clone)]
pub struct StateEntry {
    /// Workflow name
    pub workflow_name: String,

    /// Workflow version
    pub workflow_version: String,

    /// Workflow status
    pub status: WorkflowStatus,

    /// Progress (0.0 to 1.0)
    pub progress: f64,

    /// Checkpoint timestamp
    pub checkpoint_at: SystemTime,

    /// Started at timestamp
    pub started_at: SystemTime,

    /// Number of tasks
    pub total_tasks: usize,

    /// Completed tasks
    pub completed_tasks: usize,

    /// Failed tasks
    pub failed_tasks: usize,

    /// File path to state
    pub file_path: PathBuf,
}

impl StateEntry {
    /// Create state entry from workflow state
    pub fn from_state(state: WorkflowState, persistence: &StatePersistence) -> Self {
        let total_tasks = state.task_statuses.len();
        let completed_tasks = state.get_completed_tasks().len();
        let failed_tasks = state.get_failed_tasks().len();

        Self {
            workflow_name: state.workflow_name.clone(),
            workflow_version: state.workflow_version.clone(),
            status: state.status,
            progress: state.get_progress(),
            checkpoint_at: state.checkpoint_at,
            started_at: state.started_at,
            total_tasks,
            completed_tasks,
            failed_tasks,
            file_path: persistence
                .state_dir()
                .join(format!("{}.state.json", state.workflow_name)),
        }
    }

    /// Get status text representation
    pub fn status_text(&self) -> &'static str {
        match self.status {
            WorkflowStatus::Running => "Running",
            WorkflowStatus::Completed => "Completed",
            WorkflowStatus::Failed => "Failed",
            WorkflowStatus::Paused => "Paused",
        }
    }

    /// Get status color
    pub fn status_color(&self) -> Color {
        match self.status {
            WorkflowStatus::Running => Color::Yellow,
            WorkflowStatus::Completed => Color::Green,
            WorkflowStatus::Failed => Color::Red,
            WorkflowStatus::Paused => Color::Cyan,
        }
    }

    /// Format elapsed time since checkpoint
    pub fn time_since_checkpoint(&self) -> String {
        if let Ok(duration) = SystemTime::now().duration_since(self.checkpoint_at) {
            format_duration(duration)
        } else {
            "Unknown".to_string()
        }
    }

    /// Format execution duration
    pub fn execution_duration(&self) -> String {
        if let Ok(duration) = self.checkpoint_at.duration_since(self.started_at) {
            format_duration(duration)
        } else {
            "Unknown".to_string()
        }
    }
}

/// Render state browser
pub fn render_state_browser(
    frame: &mut Frame,
    area: Rect,
    state: &mut StateBrowserState,
    theme: &Theme,
) {
    match state.view_mode {
        StateBrowserViewMode::List => render_state_list(frame, area, state, theme),
        StateBrowserViewMode::Details => render_state_details(frame, area, state, theme),
    }
}

/// Render state list view
fn render_state_list(frame: &mut Frame, area: Rect, state: &mut StateBrowserState, theme: &Theme) {
    // Layout: Header | Search | List | Status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Search/filter
            Constraint::Min(10),   // List
            Constraint::Length(2), // Status bar
        ])
        .split(area);

    // Header
    let header = Paragraph::new("Workflow State Browser")
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Search/filter bar
    let filter_text = if state.filter_query.is_empty() {
        "Filter: (type to search)".to_string()
    } else {
        format!("Filter: {}", state.filter_query)
    };
    let filter = Paragraph::new(filter_text)
        .style(Style::default().fg(theme.fg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Sort: {} ", state.sort_mode.display_name())),
        );
    frame.render_widget(filter, chunks[1]);

    // State list
    let filtered_states = state.filtered_states();
    let items: Vec<ListItem> = filtered_states
        .iter()
        .map(|entry| {
            let status_span = Span::styled(
                format!("[{}]", entry.status_text()),
                Style::default()
                    .fg(entry.status_color())
                    .add_modifier(Modifier::BOLD),
            );

            let progress_bar = render_progress_bar(entry.progress, 20);

            let line = Line::from(vec![
                status_span,
                Span::raw(" "),
                Span::styled(
                    entry.workflow_name.clone(),
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" v{} ", entry.workflow_version)),
                progress_bar,
                Span::raw(format!(
                    " ({}/{} tasks) ",
                    entry.completed_tasks, entry.total_tasks
                )),
                Span::styled(
                    entry.time_since_checkpoint(),
                    Style::default().fg(theme.muted),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" States ({}) ", filtered_states.len())),
        )
        .highlight_style(
            Style::default()
                .bg(theme.highlight)
                .fg(theme.bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    frame.render_stateful_widget(list, chunks[2], &mut state.list_state);

    // Status bar
    let help_text = "↑/↓: Navigate | Enter: View Details | r: Resume | d: Delete | s: Sort | /: Filter | q: Back";
    let status_bar = Paragraph::new(help_text)
        .style(Style::default().fg(theme.muted))
        .alignment(Alignment::Center);
    frame.render_widget(status_bar, chunks[3]);
}

/// Render state details view
fn render_state_details(
    frame: &mut Frame,
    area: Rect,
    state: &mut StateBrowserState,
    theme: &Theme,
) {
    if let Some(workflow_state) = &state.current_state {
        // Layout: Header | Details | Controls
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Details
                Constraint::Length(2), // Controls
            ])
            .split(area);

        // Header
        let header = Paragraph::new(format!(
            "Workflow State: {} v{}",
            workflow_state.workflow_name, workflow_state.workflow_version
        ))
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);

        // Details content
        let details_text = build_details_text(workflow_state, theme);
        let total_lines = details_text.lines.len();

        // Update scrollbar
        state.scrollbar_state = state
            .scrollbar_state
            .content_length(total_lines)
            .viewport_content_length(state.details_page_size);

        let details = Paragraph::new(details_text)
            .block(Block::default().borders(Borders::ALL).title(" Details "))
            .wrap(Wrap { trim: false })
            .scroll((state.details_scroll as u16, 0));

        frame.render_widget(details, chunks[1]);

        // Render scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        frame.render_stateful_widget(
            scrollbar,
            chunks[1].inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut state.scrollbar_state,
        );

        // Controls
        let can_resume = workflow_state.can_resume();
        let resume_text = if can_resume { "r: Resume | " } else { "" };
        let help_text = format!(
            "{}d: Delete | Esc/q: Back | ↑/↓: Scroll | PgUp/PgDn: Page",
            resume_text
        );
        let controls = Paragraph::new(help_text)
            .style(Style::default().fg(theme.muted))
            .alignment(Alignment::Center);
        frame.render_widget(controls, chunks[2]);
    } else {
        let error = Paragraph::new("No state loaded")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        frame.render_widget(error, area);
    }
}

/// Build detailed text for state
fn build_details_text<'a>(state: &'a WorkflowState, theme: &'a Theme) -> Text<'a> {
    let mut lines: Vec<Line> = Vec::new();

    // Status overview
    lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{:?}", state.status),
            Style::default().fg(match state.status {
                WorkflowStatus::Running => Color::Yellow,
                WorkflowStatus::Completed => Color::Green,
                WorkflowStatus::Failed => Color::Red,
                WorkflowStatus::Paused => Color::Cyan,
            }),
        ),
    ]));

    // Progress
    let progress_pct = (state.get_progress() * 100.0) as u32;
    lines.push(Line::from(vec![
        Span::styled("Progress: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}%", progress_pct)),
        Span::raw("  "),
        render_progress_bar(state.get_progress(), 30),
    ]));

    // Timing
    if let Ok(duration) = state.checkpoint_at.duration_since(state.started_at) {
        lines.push(Line::from(vec![
            Span::styled("Duration: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format_duration(duration)),
        ]));
    }

    lines.push(Line::from(""));

    // Task breakdown
    lines.push(Line::from(vec![Span::styled(
        "Task Status:",
        Style::default().add_modifier(Modifier::BOLD),
    )]));

    let completed = state.get_completed_tasks();
    let failed = state.get_failed_tasks();
    let pending = state.get_pending_tasks();

    lines.push(Line::from(vec![
        Span::styled("  Completed: ", Style::default().fg(Color::Green)),
        Span::raw(format!("{}", completed.len())),
    ]));

    for task_id in &completed {
        lines.push(Line::from(vec![
            Span::raw("    ✓ "),
            Span::styled(task_id.clone(), Style::default().fg(Color::Green)),
        ]));
        if let Some(result) = state.task_results.get(task_id) {
            if !result.trim().is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(result.clone(), Style::default().fg(theme.muted)),
                ]));
            }
        }
    }

    if !failed.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Failed: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", failed.len())),
        ]));

        for task_id in &failed {
            lines.push(Line::from(vec![
                Span::raw("    ✗ "),
                Span::styled(task_id.clone(), Style::default().fg(Color::Red)),
            ]));
            if let Some(error) = state.task_errors.get(task_id) {
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(error.clone(), Style::default().fg(Color::Red)),
                ]));
            }
        }
    }

    if !pending.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Pending: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", pending.len())),
        ]));

        for task_id in &pending {
            lines.push(Line::from(vec![
                Span::raw("    ○ "),
                Span::styled(task_id.clone(), Style::default().fg(theme.muted)),
            ]));
        }
    }

    // Loop states if any
    if !state.loop_states.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Loop States:",
            Style::default().add_modifier(Modifier::BOLD),
        )]));

        for (task_id, loop_state) in &state.loop_states {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(task_id, Style::default().fg(theme.primary)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("    Iteration: "),
                Span::raw(format!("{}", loop_state.current_iteration)),
            ]));
            if let Some(total) = loop_state.total_iterations {
                lines.push(Line::from(vec![
                    Span::raw("    Total: "),
                    Span::raw(format!("{}", total)),
                ]));
                let loop_progress = loop_state.current_iteration as f64 / total as f64;
                lines.push(Line::from(vec![
                    Span::raw("    Progress: "),
                    render_progress_bar(loop_progress, 20),
                ]));
            }
        }
    }

    // Metadata if any
    if !state.metadata.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Metadata:",
            Style::default().add_modifier(Modifier::BOLD),
        )]));

        for (key, value) in &state.metadata {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(key, Style::default().fg(theme.primary)),
                Span::raw(": "),
                Span::raw(value.to_string()),
            ]));
        }
    }

    Text::from(lines)
}

/// Render progress bar as a span
fn render_progress_bar(progress: f64, width: usize) -> Span<'static> {
    let filled = ((progress * width as f64) as usize).min(width);
    let empty = width - filled;

    let mut bar = String::new();
    bar.push('[');
    for _ in 0..filled {
        bar.push('█');
    }
    for _ in 0..empty {
        bar.push('░');
    }
    bar.push(']');

    let color = if progress >= 1.0 {
        Color::Green
    } else if progress >= 0.5 {
        Color::Yellow
    } else {
        Color::Red
    };

    Span::styled(bar, Style::default().fg(color))
}

/// Format duration in human-readable form
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_browser_creation() {
        let state = StateBrowserState::new(PathBuf::from(".test_states"));
        assert_eq!(state.view_mode, StateBrowserViewMode::List);
        assert_eq!(state.selected_index, 0);
        assert!(state.states.is_empty());
    }

    #[test]
    fn test_state_entry_from_state() {
        let workflow_state = WorkflowState::new("test_workflow".to_string(), "1.0.0".to_string());
        let persistence = StatePersistence::new(".test_states").unwrap();

        let entry = StateEntry::from_state(workflow_state, &persistence);
        assert_eq!(entry.workflow_name, "test_workflow");
        assert_eq!(entry.workflow_version, "1.0.0");
        assert_eq!(entry.status, WorkflowStatus::Running);
    }

    #[test]
    fn test_sort_mode_cycle() {
        let mut mode = StateSortMode::NameAsc;
        mode = mode.next();
        assert_eq!(mode, StateSortMode::NameDesc);
        mode = mode.next();
        assert_eq!(mode, StateSortMode::ModifiedAsc);
    }

    #[test]
    fn test_progress_bar_rendering() {
        let bar = render_progress_bar(0.5, 10);
        assert!(bar.content.contains('['));
        assert!(bar.content.contains(']'));
    }

    #[test]
    fn test_duration_formatting() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3665)), "1h 1m");
        assert_eq!(format_duration(Duration::from_secs(90000)), "1d 1h");
    }

    #[test]
    fn test_state_filtering() {
        let mut state = StateBrowserState::new(PathBuf::from(".test_states"));

        // Add mock entries
        state.states.push(StateEntry {
            workflow_name: "test_workflow_1".to_string(),
            workflow_version: "1.0.0".to_string(),
            status: WorkflowStatus::Running,
            progress: 0.5,
            checkpoint_at: SystemTime::now(),
            started_at: SystemTime::now(),
            total_tasks: 10,
            completed_tasks: 5,
            failed_tasks: 0,
            file_path: PathBuf::from("test1.state.json"),
        });

        state.states.push(StateEntry {
            workflow_name: "other_workflow".to_string(),
            workflow_version: "1.0.0".to_string(),
            status: WorkflowStatus::Completed,
            progress: 1.0,
            checkpoint_at: SystemTime::now(),
            started_at: SystemTime::now(),
            total_tasks: 5,
            completed_tasks: 5,
            failed_tasks: 0,
            file_path: PathBuf::from("test2.state.json"),
        });

        // Test no filter
        assert_eq!(state.filtered_states().len(), 2);

        // Test with filter
        state.filter_query = "test".to_string();
        assert_eq!(state.filtered_states().len(), 1);
        assert_eq!(state.filtered_states()[0].workflow_name, "test_workflow_1");
    }
}

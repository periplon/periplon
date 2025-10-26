//! Workflow File Manager
//!
//! Provides an interactive UI for browsing, previewing, and managing DSL workflow files.
//! Features include directory tree navigation, file preview with syntax highlighting,
//! and file operations (open, delete, rename, copy).

use crate::dsl::parse_workflow_file;
use crate::dsl::schema::DSLWorkflow;
use crate::dsl::validator::validate_workflow;
use crate::error::{Error, Result};
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
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// File manager view state
#[derive(Debug, Clone)]
pub struct FileManagerState {
    /// Current directory being browsed
    pub current_dir: PathBuf,

    /// File tree entries (flattened hierarchy)
    pub entries: Vec<FileEntry>,

    /// Currently selected entry index
    pub selected_index: usize,

    /// List state for navigation
    pub list_state: ListState,

    /// Current view mode
    pub view_mode: FileManagerViewMode,

    /// File content preview (for preview mode)
    pub preview_content: Option<String>,

    /// Preview scroll offset
    pub preview_scroll: usize,

    /// Preview page size
    pub preview_page_size: usize,

    /// Scrollbar state for preview
    pub scrollbar_state: ScrollbarState,

    /// Search/filter query
    pub filter_query: String,

    /// Show hidden files
    pub show_hidden: bool,

    /// Current action mode (for rename/copy operations)
    pub action_mode: FileActionMode,

    /// Input buffer for rename/copy operations
    pub input_buffer: String,

    /// Expanded directories (for tree view)
    pub expanded_dirs: Vec<PathBuf>,

    /// Sort mode
    pub sort_mode: FileSortMode,

    /// Loaded workflow (for validation and inspection)
    pub loaded_workflow: Option<DSLWorkflow>,

    /// Workflow validation errors
    pub validation_errors: Vec<String>,
}

impl FileManagerState {
    /// Create new file manager state
    pub fn new(starting_dir: PathBuf) -> Result<Self> {
        let mut state = Self {
            current_dir: starting_dir,
            entries: Vec::new(),
            selected_index: 0,
            list_state: ListState::default(),
            view_mode: FileManagerViewMode::Tree,
            preview_content: None,
            preview_scroll: 0,
            preview_page_size: 20,
            scrollbar_state: ScrollbarState::default(),
            filter_query: String::new(),
            show_hidden: false,
            action_mode: FileActionMode::None,
            input_buffer: String::new(),
            expanded_dirs: Vec::new(),
            sort_mode: FileSortMode::NameAsc,
            loaded_workflow: None,
            validation_errors: Vec::new(),
        };

        // Load initial directory
        state.load_directory()?;
        state.update_list_state();

        Ok(state)
    }

    /// Load directory contents
    pub fn load_directory(&mut self) -> Result<()> {
        self.entries.clear();
        let current_dir = self.current_dir.clone();
        self.load_directory_recursive(&current_dir, 0)?;
        self.apply_sort();
        self.selected_index = self.selected_index.min(self.entries.len().saturating_sub(1));
        self.update_list_state();
        Ok(())
    }

    /// Load directory recursively for tree view
    fn load_directory_recursive(&mut self, dir: &Path, depth: usize) -> Result<()> {
        let read_dir = fs::read_dir(dir).map_err(|e| {
            Error::InvalidInput(format!("Failed to read directory {:?}: {}", dir, e))
        })?;

        let mut entries: Vec<(PathBuf, fs::Metadata)> = read_dir
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let metadata = entry.metadata().ok()?;

                // Skip hidden files unless show_hidden is true
                if !self.show_hidden {
                    if let Some(name) = path.file_name() {
                        if name.to_string_lossy().starts_with('.') {
                            return None;
                        }
                    }
                }

                Some((path, metadata))
            })
            .collect();

        // Sort entries: directories first, then files
        entries.sort_by(|a, b| {
            match (a.1.is_dir(), b.1.is_dir()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.0.cmp(&b.0),
            }
        });

        for (path, metadata) in entries {
            let is_dir = metadata.is_dir();
            let is_workflow = !is_dir && is_workflow_file(&path);
            let modified = metadata
                .modified()
                .unwrap_or_else(|_| SystemTime::now());
            let size = if is_dir { 0 } else { metadata.len() };

            let entry = FileEntry {
                path: path.clone(),
                name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                is_dir,
                is_workflow,
                size,
                modified,
                depth,
            };

            self.entries.push(entry);

            // If directory is expanded, load its contents
            if is_dir && self.expanded_dirs.contains(&path) {
                self.load_directory_recursive(&path, depth + 1)?;
            }
        }

        Ok(())
    }

    /// Get filtered entries based on current filter
    pub fn filtered_entries(&self) -> Vec<&FileEntry> {
        if self.filter_query.is_empty() {
            self.entries.iter().collect()
        } else {
            let query = self.filter_query.to_lowercase();
            self.entries
                .iter()
                .filter(|e| {
                    e.name.to_lowercase().contains(&query)
                        || (e.is_workflow && e.path.to_string_lossy().contains(&query))
                })
                .collect()
        }
    }

    /// Apply current sort mode
    fn apply_sort(&mut self) {
        match self.sort_mode {
            FileSortMode::NameAsc => {
                self.entries.sort_by(|a, b| {
                    match (a.is_dir, b.is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.name.cmp(&b.name),
                    }
                });
            }
            FileSortMode::NameDesc => {
                self.entries.sort_by(|a, b| {
                    match (a.is_dir, b.is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => b.name.cmp(&a.name),
                    }
                });
            }
            FileSortMode::ModifiedAsc => {
                self.entries.sort_by_key(|e| e.modified);
            }
            FileSortMode::ModifiedDesc => {
                self.entries.sort_by(|a, b| b.modified.cmp(&a.modified));
            }
            FileSortMode::SizeAsc => {
                self.entries.sort_by_key(|e| e.size);
            }
            FileSortMode::SizeDesc => {
                self.entries.sort_by(|a, b| b.size.cmp(&a.size));
            }
            FileSortMode::TypeAsc => {
                self.entries.sort_by(|a, b| {
                    let a_type = if a.is_dir {
                        "dir"
                    } else if a.is_workflow {
                        "workflow"
                    } else {
                        "file"
                    };
                    let b_type = if b.is_dir {
                        "dir"
                    } else if b.is_workflow {
                        "workflow"
                    } else {
                        "file"
                    };
                    a_type.cmp(b_type)
                });
            }
        }
    }

    /// Cycle to next sort mode
    pub fn next_sort_mode(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.apply_sort();
    }

    /// Select next entry
    pub fn select_next(&mut self) {
        let filtered_count = self.filtered_entries().len();
        if filtered_count > 0 {
            self.selected_index = (self.selected_index + 1).min(filtered_count - 1);
            self.update_list_state();
        }
    }

    /// Select previous entry
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_list_state();
        }
    }

    /// Update internal list state
    fn update_list_state(&mut self) {
        self.list_state.select(Some(self.selected_index));
    }

    /// Toggle directory expansion
    pub fn toggle_directory(&mut self) -> Result<()> {
        let filtered = self.filtered_entries();
        if self.selected_index < filtered.len() {
            let entry = filtered[self.selected_index];
            if entry.is_dir {
                let path = entry.path.clone();
                if let Some(pos) = self.expanded_dirs.iter().position(|p| p == &path) {
                    // Collapse directory
                    self.expanded_dirs.remove(pos);
                } else {
                    // Expand directory
                    self.expanded_dirs.push(path);
                }
                self.load_directory()?;
            }
        }
        Ok(())
    }

    /// Load preview for selected file
    pub fn load_preview(&mut self) -> Result<()> {
        let filtered = self.filtered_entries();
        if self.selected_index < filtered.len() {
            let entry = filtered[self.selected_index];
            if !entry.is_dir {
                let entry_path = entry.path.clone();
                let is_workflow = entry.is_workflow;

                match fs::read_to_string(&entry_path) {
                    Ok(content) => {
                        self.preview_content = Some(content);
                        self.view_mode = FileManagerViewMode::Preview;
                        self.preview_scroll = 0;

                        // If it's a workflow file, also try to load and validate it
                        if is_workflow {
                            self.load_workflow(&entry_path)?;
                        }
                    }
                    Err(e) => {
                        self.preview_content = Some(format!("Error reading file: {}", e));
                        self.view_mode = FileManagerViewMode::Preview;
                    }
                }
            }
        }
        Ok(())
    }

    /// Load and validate a workflow file
    pub fn load_workflow(&mut self, path: &Path) -> Result<()> {
        self.validation_errors.clear();

        match parse_workflow_file(path) {
            Ok(workflow) => {
                // Validate the workflow
                match validate_workflow(&workflow) {
                    Ok(_) => {
                        self.loaded_workflow = Some(workflow);
                    }
                    Err(e) => {
                        self.loaded_workflow = Some(workflow);
                        self.validation_errors.push(format!("Validation error: {}", e));
                    }
                }
            }
            Err(e) => {
                self.loaded_workflow = None;
                self.validation_errors.push(format!("Parse error: {}", e));
            }
        }

        Ok(())
    }

    /// Get loaded workflow reference
    pub fn get_loaded_workflow(&self) -> Option<&DSLWorkflow> {
        self.loaded_workflow.as_ref()
    }

    /// Check if loaded workflow has validation errors
    pub fn has_validation_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// Get validation errors
    pub fn get_validation_errors(&self) -> &[String] {
        &self.validation_errors
    }

    /// Return to tree view
    pub fn back_to_tree(&mut self) {
        self.view_mode = FileManagerViewMode::Tree;
        self.preview_content = None;
        self.preview_scroll = 0;
        self.loaded_workflow = None;
        self.validation_errors.clear();
    }

    /// Scroll preview down
    pub fn scroll_preview_down(&mut self, max_lines: usize) {
        if self.preview_scroll < max_lines.saturating_sub(self.preview_page_size) {
            self.preview_scroll += 1;
            self.scrollbar_state = self.scrollbar_state.position(self.preview_scroll);
        }
    }

    /// Scroll preview up
    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
        self.scrollbar_state = self.scrollbar_state.position(self.preview_scroll);
    }

    /// Page down in preview
    pub fn page_preview_down(&mut self, max_lines: usize) {
        let max_scroll = max_lines.saturating_sub(self.preview_page_size);
        self.preview_scroll = (self.preview_scroll + self.preview_page_size).min(max_scroll);
        self.scrollbar_state = self.scrollbar_state.position(self.preview_scroll);
    }

    /// Page up in preview
    pub fn page_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(self.preview_page_size);
        self.scrollbar_state = self.scrollbar_state.position(self.preview_scroll);
    }

    /// Get selected entry
    pub fn selected_entry(&self) -> Option<&FileEntry> {
        let filtered = self.filtered_entries();
        filtered.get(self.selected_index).copied()
    }

    /// Delete selected file
    pub fn delete_selected(&mut self) -> Result<()> {
        let filtered = self.filtered_entries();
        if let Some(entry) = filtered.get(self.selected_index) {
            if entry.is_dir {
                fs::remove_dir_all(&entry.path).map_err(|e| {
                    Error::InvalidInput(format!("Failed to delete directory: {}", e))
                })?;
            } else {
                fs::remove_file(&entry.path).map_err(|e| {
                    Error::InvalidInput(format!("Failed to delete file: {}", e))
                })?;
            }
            self.load_directory()?;
        }
        Ok(())
    }

    /// Start rename action
    pub fn start_rename(&mut self) {
        if let Some(entry) = self.selected_entry() {
            let name = entry.name.clone();
            self.action_mode = FileActionMode::Rename;
            self.input_buffer = name;
        }
    }

    /// Start copy action
    pub fn start_copy(&mut self) {
        if let Some(entry) = self.selected_entry() {
            let name = entry.name.clone();
            self.action_mode = FileActionMode::Copy;
            self.input_buffer = name;
        }
    }

    /// Complete current action
    pub fn complete_action(&mut self) -> Result<()> {
        match self.action_mode {
            FileActionMode::Rename => {
                if let Some(entry) = self.selected_entry() {
                    let new_path = entry.path.parent().unwrap().join(&self.input_buffer);
                    fs::rename(&entry.path, &new_path).map_err(|e| {
                        Error::InvalidInput(format!("Failed to rename file: {}", e))
                    })?;
                    self.load_directory()?;
                }
            }
            FileActionMode::Copy => {
                if let Some(entry) = self.selected_entry() {
                    let new_path = entry.path.parent().unwrap().join(&self.input_buffer);
                    fs::copy(&entry.path, &new_path).map_err(|e| {
                        Error::InvalidInput(format!("Failed to copy file: {}", e))
                    })?;
                    self.load_directory()?;
                }
            }
            FileActionMode::None => {}
        }
        self.action_mode = FileActionMode::None;
        self.input_buffer.clear();
        Ok(())
    }

    /// Cancel current action
    pub fn cancel_action(&mut self) {
        self.action_mode = FileActionMode::None;
        self.input_buffer.clear();
    }

    /// Navigate to parent directory
    pub fn go_to_parent(&mut self) -> Result<()> {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.load_directory()?;
        }
        Ok(())
    }

    /// Navigate into selected directory
    pub fn navigate_into(&mut self) -> Result<()> {
        let filtered = self.filtered_entries();
        if self.selected_index < filtered.len() {
            let entry = filtered[self.selected_index];
            if entry.is_dir {
                self.current_dir = entry.path.clone();
                self.expanded_dirs.clear();
                self.load_directory()?;
            }
        }
        Ok(())
    }

    /// Toggle hidden files visibility
    pub fn toggle_hidden(&mut self) -> Result<()> {
        self.show_hidden = !self.show_hidden;
        self.load_directory()
    }

    /// Update page size based on terminal height
    pub fn update_page_size(&mut self, height: usize) {
        self.preview_page_size = height.saturating_sub(5);
    }

    /// Add character to input buffer
    pub fn input_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    /// Remove last character from input buffer
    pub fn input_backspace(&mut self) {
        self.input_buffer.pop();
    }
}

/// File manager view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileManagerViewMode {
    /// Tree view with file listing
    Tree,
    /// Preview mode showing file contents
    Preview,
}

/// File action mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileActionMode {
    /// No action in progress
    None,
    /// Renaming file
    Rename,
    /// Copying file
    Copy,
}

/// File sort mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSortMode {
    NameAsc,
    NameDesc,
    ModifiedAsc,
    ModifiedDesc,
    SizeAsc,
    SizeDesc,
    TypeAsc,
}

impl FileSortMode {
    /// Get next sort mode in cycle
    pub fn next(self) -> Self {
        match self {
            Self::NameAsc => Self::NameDesc,
            Self::NameDesc => Self::ModifiedAsc,
            Self::ModifiedAsc => Self::ModifiedDesc,
            Self::ModifiedDesc => Self::SizeAsc,
            Self::SizeAsc => Self::SizeDesc,
            Self::SizeDesc => Self::TypeAsc,
            Self::TypeAsc => Self::NameAsc,
        }
    }

    /// Get display name
    pub fn display_name(self) -> &'static str {
        match self {
            Self::NameAsc => "Name ↑",
            Self::NameDesc => "Name ↓",
            Self::ModifiedAsc => "Modified ↑",
            Self::ModifiedDesc => "Modified ↓",
            Self::SizeAsc => "Size ↑",
            Self::SizeDesc => "Size ↓",
            Self::TypeAsc => "Type ↑",
        }
    }
}

/// File entry in the tree
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Full path to file/directory
    pub path: PathBuf,

    /// Display name
    pub name: String,

    /// Is this a directory
    pub is_dir: bool,

    /// Is this a workflow file
    pub is_workflow: bool,

    /// File size in bytes
    pub size: u64,

    /// Last modified time
    pub modified: SystemTime,

    /// Tree depth level
    pub depth: usize,
}

impl FileEntry {
    /// Get file type icon
    pub fn icon(&self) -> &'static str {
        if self.is_dir {
            "📁"
        } else if self.is_workflow {
            "⚙️"
        } else {
            "📄"
        }
    }

    /// Get file type color
    pub fn type_color(&self) -> Color {
        if self.is_dir {
            Color::Cyan
        } else if self.is_workflow {
            Color::Green
        } else {
            Color::White
        }
    }

    /// Format file size
    pub fn format_size(&self) -> String {
        if self.is_dir {
            "-".to_string()
        } else {
            format_bytes(self.size)
        }
    }

    /// Format modified time
    pub fn format_modified(&self) -> String {
        if let Ok(duration) = SystemTime::now().duration_since(self.modified) {
            format_duration_ago(duration)
        } else {
            "Unknown".to_string()
        }
    }
}

/// Render file manager
pub fn render_file_manager(
    frame: &mut Frame,
    area: Rect,
    state: &mut FileManagerState,
    theme: &Theme,
) {
    match state.view_mode {
        FileManagerViewMode::Tree => render_file_tree(frame, area, state, theme),
        FileManagerViewMode::Preview => render_file_preview(frame, area, state, theme),
    }
}

/// Render file tree view
fn render_file_tree(
    frame: &mut Frame,
    area: Rect,
    state: &mut FileManagerState,
    theme: &Theme,
) {
    // Layout: Header | Filter | Tree | Actions | Status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Filter
            Constraint::Min(10),   // Tree
            Constraint::Length(3), // Actions
            Constraint::Length(2), // Status
        ])
        .split(area);

    // Header
    let header_text = format!("Workflow File Manager - {}", state.current_dir.display());
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Filter bar
    let filter_text = if state.filter_query.is_empty() {
        "Filter: (type to search)".to_string()
    } else {
        format!("Filter: {}", state.filter_query)
    };
    let filter = Paragraph::new(filter_text)
        .style(Style::default().fg(theme.fg))
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Sort: {} | Hidden: {} ",
            state.sort_mode.display_name(),
            if state.show_hidden { "On" } else { "Off" }
        )));
    frame.render_widget(filter, chunks[1]);

    // File tree
    let filtered_entries = state.filtered_entries();
    let items: Vec<ListItem> = filtered_entries
        .iter()
        .map(|entry| {
            let indent = "  ".repeat(entry.depth);
            let expand_indicator = if entry.is_dir {
                if state.expanded_dirs.contains(&entry.path) {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };

            let line = Line::from(vec![
                Span::raw(indent),
                Span::raw(expand_indicator),
                Span::raw(format!("{} ", entry.icon())),
                Span::styled(
                    entry.name.clone(),
                    Style::default()
                        .fg(entry.type_color())
                        .add_modifier(if entry.is_workflow {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::raw("  "),
                Span::styled(
                    entry.format_size(),
                    Style::default().fg(theme.muted),
                ),
                Span::raw("  "),
                Span::styled(
                    entry.format_modified(),
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
                .title(format!(" Files ({}) ", filtered_entries.len())),
        )
        .highlight_style(
            Style::default()
                .bg(theme.highlight)
                .fg(theme.bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    frame.render_stateful_widget(list, chunks[2], &mut state.list_state);

    // Action input (if in action mode)
    match state.action_mode {
        FileActionMode::Rename => {
            let input = Paragraph::new(format!("Rename to: {}", state.input_buffer))
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Rename (Enter: Confirm, Esc: Cancel) "),
                );
            frame.render_widget(input, chunks[3]);
        }
        FileActionMode::Copy => {
            let input = Paragraph::new(format!("Copy to: {}", state.input_buffer))
                .style(Style::default().fg(Color::Cyan))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Copy (Enter: Confirm, Esc: Cancel) "),
                );
            frame.render_widget(input, chunks[3]);
        }
        FileActionMode::None => {
            let actions = Paragraph::new("o: Open | p: Preview | d: Delete | r: Rename | c: Copy")
                .style(Style::default().fg(theme.fg))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Actions "));
            frame.render_widget(actions, chunks[3]);
        }
    }

    // Status bar
    let help_text = "↑/↓: Navigate | Enter: Open/Expand | Backspace: Parent | s: Sort | h: Toggle Hidden | /: Filter | q: Back";
    let status = Paragraph::new(help_text)
        .style(Style::default().fg(theme.muted))
        .alignment(Alignment::Center);
    frame.render_widget(status, chunks[4]);
}

/// Render file preview
fn render_file_preview(
    frame: &mut Frame,
    area: Rect,
    state: &mut FileManagerState,
    theme: &Theme,
) {
    if let Some(content) = &state.preview_content {
        // Determine if we need a validation section
        let has_validation = state.loaded_workflow.is_some() || !state.validation_errors.is_empty();

        // Layout: Header | Validation (optional) | Preview | Controls
        let chunks = if has_validation {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Length(4), // Validation status
                    Constraint::Min(10),   // Preview
                    Constraint::Length(2), // Controls
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(10),   // Preview
                    Constraint::Length(2), // Controls
                ])
                .split(area)
        };

        // Header
        let header_text = if let Some(entry) = state.selected_entry() {
            format!("Preview: {}", entry.path.display())
        } else {
            "Preview".to_string()
        };
        let header = Paragraph::new(header_text)
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);

        // Validation status (if workflow was loaded)
        if has_validation {
            let validation_text = if state.has_validation_errors() {
                let errors = state.get_validation_errors();
                format!("❌ Validation Failed: {}", errors.join(", "))
            } else if state.loaded_workflow.is_some() {
                "✅ Valid Workflow".to_string()
            } else {
                "⚠️  Not a workflow file".to_string()
            };

            let validation_color = if state.has_validation_errors() {
                Color::Red
            } else if state.loaded_workflow.is_some() {
                Color::Green
            } else {
                Color::Yellow
            };

            let validation = Paragraph::new(validation_text)
                .style(Style::default().fg(validation_color).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Validation "));
            frame.render_widget(validation, chunks[1]);
        }

        // Preview content with syntax highlighting
        let highlighted_content = highlight_yaml(content, theme);
        let total_lines = highlighted_content.lines.len();

        // Determine preview and controls chunk indices based on layout
        let preview_chunk_idx = if has_validation { 2 } else { 1 };
        let controls_chunk_idx = if has_validation { 3 } else { 2 };

        // Update scrollbar
        state.scrollbar_state = state
            .scrollbar_state
            .content_length(total_lines)
            .viewport_content_length(state.preview_page_size);

        let preview = Paragraph::new(highlighted_content)
            .block(Block::default().borders(Borders::ALL).title(" Content "))
            .wrap(Wrap { trim: false })
            .scroll((state.preview_scroll as u16, 0));

        frame.render_widget(preview, chunks[preview_chunk_idx]);

        // Render scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        frame.render_stateful_widget(
            scrollbar,
            chunks[preview_chunk_idx].inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut state.scrollbar_state,
        );

        // Controls
        let help_text = "Esc/q: Back | ↑/↓: Scroll | PgUp/PgDn: Page";
        let controls = Paragraph::new(help_text)
            .style(Style::default().fg(theme.muted))
            .alignment(Alignment::Center);
        frame.render_widget(controls, chunks[controls_chunk_idx]);
    } else {
        let error = Paragraph::new("No preview available")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        frame.render_widget(error, area);
    }
}

/// Check if file is a workflow file
fn is_workflow_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        ext_str == "yaml" || ext_str == "yml"
    } else {
        false
    }
}

/// Format bytes in human-readable form
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration as "time ago"
fn format_duration_ago(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// Simple YAML syntax highlighting
fn highlight_yaml<'a>(content: &'a str, theme: &Theme) -> Text<'a> {
    let mut lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim_start();

        // Comment lines
        if trimmed.starts_with('#') {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(theme.muted),
            )));
        }
        // Key-value pairs
        else if let Some(colon_pos) = trimmed.find(':') {
            let (key, value) = trimmed.split_at(colon_pos);
            let indent = " ".repeat(line.len() - trimmed.len());

            lines.push(Line::from(vec![
                Span::raw(indent),
                Span::styled(
                    key.to_string(),
                    Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(":", Style::default().fg(theme.fg)),
                Span::styled(
                    value[1..].to_string(),
                    Style::default().fg(theme.fg),
                ),
            ]));
        }
        // List items
        else if let Some(stripped) = trimmed.strip_prefix('-') {
            let indent = " ".repeat(line.len() - trimmed.len());
            lines.push(Line::from(vec![
                Span::raw(indent),
                Span::styled("-", Style::default().fg(Color::Yellow)),
                Span::styled(
                    stripped.to_string(),
                    Style::default().fg(theme.fg),
                ),
            ]));
        }
        // Regular lines
        else {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(theme.fg),
            )));
        }
    }

    Text::from(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_file_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let state = FileManagerState::new(temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(state.view_mode, FileManagerViewMode::Tree);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_is_workflow_file() {
        assert!(is_workflow_file(Path::new("test.yaml")));
        assert!(is_workflow_file(Path::new("test.yml")));
        assert!(!is_workflow_file(Path::new("test.txt")));
        assert!(!is_workflow_file(Path::new("test")));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_sort_mode_cycle() {
        let mut mode = FileSortMode::NameAsc;
        mode = mode.next();
        assert_eq!(mode, FileSortMode::NameDesc);
        mode = mode.next();
        assert_eq!(mode, FileSortMode::ModifiedAsc);
    }

    #[test]
    fn test_file_entry_icon() {
        let dir_entry = FileEntry {
            path: PathBuf::from("/test"),
            name: "test".to_string(),
            is_dir: true,
            is_workflow: false,
            size: 0,
            modified: SystemTime::now(),
            depth: 0,
        };
        assert_eq!(dir_entry.icon(), "📁");

        let workflow_entry = FileEntry {
            path: PathBuf::from("/test.yaml"),
            name: "test.yaml".to_string(),
            is_dir: false,
            is_workflow: true,
            size: 100,
            modified: SystemTime::now(),
            depth: 0,
        };
        assert_eq!(workflow_entry.icon(), "⚙️");
    }

    #[test]
    fn test_load_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        File::create(temp_dir.path().join("test1.yaml"))
            .unwrap()
            .write_all(b"name: test1\nversion: 1.0.0")
            .unwrap();
        File::create(temp_dir.path().join("test2.yml"))
            .unwrap()
            .write_all(b"name: test2\nversion: 1.0.0")
            .unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let mut state = FileManagerState::new(temp_dir.path().to_path_buf()).unwrap();
        assert!(state.entries.len() >= 2);

        let workflow_files: Vec<_> = state
            .entries
            .iter()
            .filter(|e| e.is_workflow)
            .collect();
        assert_eq!(workflow_files.len(), 2);
    }

    #[test]
    fn test_filtering() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("workflow1.yaml")).unwrap();
        File::create(temp_dir.path().join("workflow2.yaml")).unwrap();
        File::create(temp_dir.path().join("other.txt")).unwrap();

        let mut state = FileManagerState::new(temp_dir.path().to_path_buf()).unwrap();

        // No filter
        assert!(state.filtered_entries().len() >= 3);

        // Filter by name
        state.filter_query = "workflow".to_string();
        let filtered = state.filtered_entries();
        assert_eq!(filtered.len(), 2);
    }
}

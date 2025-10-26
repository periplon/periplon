//! Help view component
//!
//! Interactive help view with:
//! - Topic browsing
//! - Search interface
//! - Context-aware suggestions
//! - Markdown rendering
//! - Navigation breadcrumbs

use super::{
    HelpContent, HelpContext, HelpSearchEngine, HelpTopic, MarkdownRenderer, SearchResult,
};
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

/// Help view display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpViewMode {
    /// Browse topics by category
    Browse,
    /// View a specific topic
    Topic,
    /// Search results
    Search,
}

/// Help view state
#[derive(Debug)]
pub struct HelpViewState {
    /// Current view mode
    mode: HelpViewMode,
    /// Current context for context-aware help
    context: HelpContext,
    /// Selected category index (in browse mode)
    selected_category: usize,
    /// Selected topic index within category
    selected_topic: usize,
    /// Currently displayed topic
    current_topic: Option<HelpTopic>,
    /// Search query
    search_query: String,
    /// Search results
    search_results: Vec<SearchResult>,
    /// Selected search result index
    selected_result: usize,
    /// Scroll offset for topic content
    scroll_offset: usize,
    /// Page size for scrolling
    page_size: usize,
    /// Search engine
    search_engine: HelpSearchEngine,
    /// Markdown renderer
    renderer: MarkdownRenderer,
    /// Navigation history (breadcrumbs)
    history: Vec<String>,
}

impl Clone for HelpViewState {
    fn clone(&self) -> Self {
        // Recreate search engine and renderer for clone
        let content = HelpContent::new();
        let search_engine = HelpSearchEngine::new(content);
        let renderer = MarkdownRenderer::new();

        Self {
            mode: self.mode,
            context: self.context,
            selected_category: self.selected_category,
            selected_topic: self.selected_topic,
            current_topic: self.current_topic.clone(),
            search_query: self.search_query.clone(),
            search_results: self.search_results.clone(),
            selected_result: self.selected_result,
            scroll_offset: self.scroll_offset,
            page_size: self.page_size,
            search_engine,
            renderer,
            history: self.history.clone(),
        }
    }
}

impl HelpViewState {
    /// Create new help view state
    pub fn new(context: HelpContext) -> Self {
        let content = HelpContent::new();
        let search_engine = HelpSearchEngine::new(content);

        Self {
            mode: HelpViewMode::Browse,
            context,
            selected_category: 0,
            selected_topic: 0,
            current_topic: None,
            search_query: String::new(),
            search_results: Vec::new(),
            selected_result: 0,
            scroll_offset: 0,
            page_size: 20,
            search_engine,
            renderer: MarkdownRenderer::new(),
            history: Vec::new(),
        }
    }

    /// Set context for context-aware help
    pub fn set_context(&mut self, context: HelpContext) {
        self.context = context;
        self.reset();
    }

    /// Get current context
    pub fn context(&self) -> HelpContext {
        self.context
    }

    /// Switch to browse mode
    pub fn enter_browse_mode(&mut self) {
        self.mode = HelpViewMode::Browse;
        self.current_topic = None;
        self.scroll_offset = 0;
    }

    /// Switch to search mode
    pub fn enter_search_mode(&mut self) {
        self.mode = HelpViewMode::Search;
        self.search_query.clear();
        self.search_results.clear();
        self.selected_result = 0;
    }

    /// View a specific topic
    pub fn view_topic(&mut self, topic: HelpTopic) {
        self.history.push(topic.id.clone());
        self.current_topic = Some(topic);
        self.mode = HelpViewMode::Topic;
        self.scroll_offset = 0;
    }

    /// Go back to previous view
    pub fn go_back(&mut self) {
        if self.mode == HelpViewMode::Topic {
            self.history.pop();
            if let Some(prev_id) = self.history.last() {
                if let Some(topic) = self.search_engine.content().get_topic(prev_id) {
                    self.current_topic = Some(topic.clone());
                } else {
                    self.enter_browse_mode();
                }
            } else {
                self.enter_browse_mode();
            }
        } else if self.mode == HelpViewMode::Search {
            self.enter_browse_mode();
        }
    }

    /// Navigate to next category
    pub fn next_category(&mut self) {
        let categories = self.search_engine.content().all_categories();
        if self.selected_category + 1 < categories.len() {
            self.selected_category += 1;
            self.selected_topic = 0;
        }
    }

    /// Navigate to previous category
    pub fn prev_category(&mut self) {
        if self.selected_category > 0 {
            self.selected_category -= 1;
            self.selected_topic = 0;
        }
    }

    /// Navigate to next topic
    pub fn next_topic(&mut self) {
        match self.mode {
            HelpViewMode::Browse => {
                let categories = self.search_engine.content().all_categories();
                if let Some((_, topics)) = categories.get(self.selected_category) {
                    if self.selected_topic + 1 < topics.len() {
                        self.selected_topic += 1;
                    }
                }
            }
            HelpViewMode::Search => {
                if self.selected_result + 1 < self.search_results.len() {
                    self.selected_result += 1;
                }
            }
            HelpViewMode::Topic => {
                // Scroll down
                self.scroll_down();
            }
        }
    }

    /// Navigate to previous topic
    pub fn prev_topic(&mut self) {
        match self.mode {
            HelpViewMode::Browse => {
                if self.selected_topic > 0 {
                    self.selected_topic -= 1;
                }
            }
            HelpViewMode::Search => {
                if self.selected_result > 0 {
                    self.selected_result -= 1;
                }
            }
            HelpViewMode::Topic => {
                // Scroll up
                self.scroll_up();
            }
        }
    }

    /// Select current item (open topic)
    pub fn select(&mut self) {
        match self.mode {
            HelpViewMode::Browse => {
                let categories = self.search_engine.content().all_categories();
                if let Some((_, topics)) = categories.get(self.selected_category) {
                    if let Some(&topic) = topics.get(self.selected_topic) {
                        self.view_topic(topic.clone());
                    }
                }
            }
            HelpViewMode::Search => {
                if let Some(result) = self.search_results.get(self.selected_result) {
                    self.view_topic(result.topic.clone());
                }
            }
            HelpViewMode::Topic => {
                // Follow link or go back
                self.go_back();
            }
        }
    }

    /// Update search query and perform search
    pub fn update_search(&mut self, query: String) {
        self.search_query = query;
        self.search_results = self.search_engine.search(&self.search_query);
        self.selected_result = 0;
    }

    /// Check if currently viewing a topic
    pub fn is_viewing_topic(&self) -> bool {
        self.mode == HelpViewMode::Topic
    }

    /// Go back to browse mode from topic view
    pub fn back_to_browse(&mut self) {
        self.mode = HelpViewMode::Browse;
        self.current_topic = None;
        self.scroll_offset = 0;
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        match self.mode {
            HelpViewMode::Browse => {
                // In browse mode, navigate topics
                self.prev_topic();
            }
            HelpViewMode::Topic => {
                // In topic mode, scroll content
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            HelpViewMode::Search => {
                // In search mode, navigate results
                if self.selected_result > 0 {
                    self.selected_result -= 1;
                }
            }
        }
    }

    /// Scroll down
    pub fn scroll_down(&mut self) {
        match self.mode {
            HelpViewMode::Browse => {
                // In browse mode, navigate topics
                self.next_topic();
            }
            HelpViewMode::Topic => {
                // In topic mode, scroll content
                if let Some(topic) = &self.current_topic {
                    let content_lines = topic.content.lines().count();
                    let max_scroll = content_lines.saturating_sub(self.page_size);
                    if self.scroll_offset < max_scroll {
                        self.scroll_offset += 1;
                    }
                }
            }
            HelpViewMode::Search => {
                // In search mode, navigate results
                if self.selected_result + 1 < self.search_results.len() {
                    self.selected_result += 1;
                }
            }
        }
    }

    /// Scroll page up
    pub fn page_up(&mut self) {
        if self.mode == HelpViewMode::Topic {
            self.scroll_offset = self.scroll_offset.saturating_sub(self.page_size);
        }
    }

    /// Scroll page down
    pub fn page_down(&mut self) {
        if self.mode == HelpViewMode::Topic {
            if let Some(topic) = &self.current_topic {
                let content_lines = topic.content.lines().count();
                let max_scroll = content_lines.saturating_sub(self.page_size);
                self.scroll_offset = (self.scroll_offset + self.page_size).min(max_scroll);
            }
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.mode = HelpViewMode::Browse;
        self.selected_category = 0;
        self.selected_topic = 0;
        self.current_topic = None;
        self.search_query.clear();
        self.search_results.clear();
        self.selected_result = 0;
        self.scroll_offset = 0;
        self.history.clear();
    }

    /// Update page size based on terminal height
    pub fn update_page_size(&mut self, height: usize) {
        self.page_size = height.saturating_sub(5);
    }
}

/// Help view component
pub struct HelpView;

impl HelpView {
    /// Create new help view
    pub fn new() -> Self {
        Self
    }

    /// Render the help view
    pub fn render(&self, frame: &mut Frame, area: Rect, state: &mut HelpViewState, theme: &Theme) {
        // Update page size
        state.update_page_size(area.height as usize);

        // Split area for content and status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),     // Main content
                Constraint::Length(1),  // Status bar
            ])
            .split(area);

        // Render main content based on mode
        match state.mode {
            HelpViewMode::Browse => self.render_browse(frame, chunks[0], state, theme),
            HelpViewMode::Topic => self.render_topic(frame, chunks[0], state, theme),
            HelpViewMode::Search => self.render_search(frame, chunks[0], state, theme),
        }

        // Render status bar
        self.render_status_bar(frame, chunks[1], state, theme);
    }

    /// Render status bar with keybindings
    fn render_status_bar(&self, frame: &mut Frame, area: Rect, state: &HelpViewState, theme: &Theme) {
        let status_text = match state.mode {
            HelpViewMode::Browse => "↑↓/j/k: Select Topic | ←→/h/l: Change Category | Enter: View Topic | Esc/q/?: Exit Help",
            HelpViewMode::Topic => "↑↓/j/k: Scroll | PgUp/PgDn: Page | Tab/n: Next Topic | Shift+Tab/p: Prev | Esc/q: Back to Browse",
            HelpViewMode::Search => "↑↓: Select Result | Enter: View Topic | Esc/q: Back to Browse",
        };

        let status = Paragraph::new(status_text)
            .style(Style::default().fg(theme.muted).bg(theme.bg));

        frame.render_widget(status, area);
    }

    /// Render browse mode (category and topic list)
    fn render_browse(&self, frame: &mut Frame, area: Rect, state: &HelpViewState, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // Render category list
        self.render_categories(frame, chunks[0], state, theme);

        // Render topic list for selected category
        self.render_topic_list(frame, chunks[1], state, theme);
    }

    /// Render category list
    fn render_categories(&self, frame: &mut Frame, area: Rect, state: &HelpViewState, theme: &Theme) {
        let categories = state.search_engine.content().all_categories();
        let items: Vec<ListItem> = categories
            .iter()
            .enumerate()
            .map(|(i, (cat, topics))| {
                let style = if i == state.selected_category {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg)
                };

                let content = format!("{} ({})", cat.name(), topics.len());
                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Categories")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            )
            .highlight_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            );

        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_category));

        frame.render_stateful_widget(list, area, &mut list_state);
    }

    /// Render topic list for selected category
    fn render_topic_list(&self, frame: &mut Frame, area: Rect, state: &HelpViewState, theme: &Theme) {
        let categories = state.search_engine.content().all_categories();
        let (category, topics) = &categories[state.selected_category];

        let items: Vec<ListItem> = topics
            .iter()
            .enumerate()
            .map(|(i, topic)| {
                let style = if i == state.selected_topic {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg)
                };

                ListItem::new(topic.title.clone()).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("{} Topics", category.name()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            )
            .highlight_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            );

        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_topic));

        frame.render_stateful_widget(list, area, &mut list_state);
    }

    /// Render topic view
    fn render_topic(&self, frame: &mut Frame, area: Rect, state: &HelpViewState, theme: &Theme) {
        if let Some(topic) = &state.current_topic {
            // Render breadcrumbs
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(0)])
                .split(area);

            let breadcrumbs = self.render_breadcrumbs(state, theme);
            frame.render_widget(breadcrumbs, chunks[0]);

            // Render topic content with markdown
            let rendered_content = state.renderer.render(&topic.content);

            // Apply scroll offset
            let total_lines = rendered_content.lines.len();
            let visible_lines: Vec<Line> = rendered_content
                .lines
                .into_iter()
                .skip(state.scroll_offset)
                .take(state.page_size)
                .collect();

            let paragraph = Paragraph::new(visible_lines)
                .block(
                    Block::default()
                        .title(topic.title.clone())
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border)),
                )
                .wrap(Wrap { trim: false });

            frame.render_widget(paragraph, chunks[1]);

            // Render scrollbar if needed
            if total_lines > state.page_size {
                let scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"));

                let mut scrollbar_state = ScrollbarState::default()
                    .content_length(total_lines)
                    .position(state.scroll_offset);

                frame.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);
            }
        }
    }

    /// Render breadcrumbs navigation
    fn render_breadcrumbs(&self, state: &HelpViewState, theme: &Theme) -> Paragraph<'static> {
        let mut spans = vec![
            Span::styled("Help", Style::default().fg(theme.fg)),
            Span::raw(" > "),
            Span::styled(state.context.title(), Style::default().fg(theme.accent)),
        ];

        if let Some(topic) = &state.current_topic {
            spans.push(Span::raw(" > "));
            spans.push(Span::styled(
                topic.title.clone(),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        Paragraph::new(Line::from(spans))
            .style(Style::default().fg(theme.fg))
    }

    /// Render search mode
    fn render_search(&self, frame: &mut Frame, area: Rect, state: &HelpViewState, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Render search input
        let search_input = Paragraph::new(state.search_query.clone())
            .block(
                Block::default()
                    .title("Search Help")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.accent)),
            )
            .style(Style::default().fg(theme.fg));

        frame.render_widget(search_input, chunks[0]);

        // Render search results
        if state.search_results.is_empty() {
            let message = if state.search_query.is_empty() {
                "Type to search..."
            } else {
                "No results found"
            };
            let no_results = Paragraph::new(message)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray));

            frame.render_widget(no_results, chunks[1]);
        } else {
            let items: Vec<ListItem> = state
                .search_results
                .iter()
                .enumerate()
                .map(|(i, result)| {
                    let style = if i == state.selected_result {
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.fg)
                    };

                    let score_pct = (result.score * 100.0) as u8;
                    let content = vec![
                        Line::from(vec![
                            Span::styled(
                                result.topic.title.clone(),
                                style.add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(format!(" ({}%)", score_pct)),
                        ]),
                        Line::from(Span::styled(
                            result.excerpt.clone(),
                            Style::default().fg(Color::Gray),
                        )),
                    ];

                    ListItem::new(content).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(format!("Results ({})", state.search_results.len()))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border)),
                )
                .highlight_style(
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                );

            let mut list_state = ListState::default();
            list_state.select(Some(state.selected_result));

            frame.render_stateful_widget(list, chunks[1], &mut list_state);
        }
    }
}

impl Default for HelpView {
    fn default() -> Self {
        Self::new()
    }
}

//! Comprehensive help and documentation system
//!
//! This module provides a searchable, context-aware help system with:
//! - Markdown rendering for rich documentation
//! - Full-text search across all help content
//! - Context-aware assistance based on current view
//! - Keyboard shortcut reference
//! - Interactive navigation

pub mod content;
pub mod markdown;
pub mod search;
pub mod view;

#[cfg(test)]
mod tests;

pub use content::{HelpContent, HelpSection, HelpTopic};
pub use markdown::MarkdownRenderer;
pub use search::{HelpSearchEngine, SearchResult};
pub use view::{HelpView, HelpViewState};

/// Help system context for context-aware assistance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpContext {
    /// Help for the workflow list view
    WorkflowList,
    /// Help for the workflow viewer
    Viewer,
    /// Help for the workflow editor
    Editor,
    /// Help for the execution monitor
    ExecutionMonitor,
    /// Help for the AI generator
    Generator,
    /// General help (no specific context)
    General,
}

impl HelpContext {
    /// Get context-specific help topics
    pub fn topics(&self) -> Vec<&'static str> {
        match self {
            HelpContext::WorkflowList => vec![
                "navigating_workflows",
                "creating_workflows",
                "keyboard_shortcuts_list",
            ],
            HelpContext::Viewer => vec![
                "viewing_workflows",
                "navigation",
                "keyboard_shortcuts_viewer",
            ],
            HelpContext::Editor => vec![
                "editing_workflows",
                "yaml_syntax",
                "validation",
                "keyboard_shortcuts_editor",
            ],
            HelpContext::ExecutionMonitor => vec![
                "monitoring_execution",
                "task_status",
                "keyboard_shortcuts_monitor",
            ],
            HelpContext::Generator => vec![
                "generating_workflows",
                "natural_language",
                "keyboard_shortcuts_generator",
            ],
            HelpContext::General => vec![
                "overview",
                "getting_started",
                "keyboard_shortcuts_global",
            ],
        }
    }

    /// Get context title for display
    pub fn title(&self) -> &'static str {
        match self {
            HelpContext::WorkflowList => "Workflow List Help",
            HelpContext::Viewer => "Workflow Viewer Help",
            HelpContext::Editor => "Workflow Editor Help",
            HelpContext::ExecutionMonitor => "Execution Monitor Help",
            HelpContext::Generator => "AI Generator Help",
            HelpContext::General => "General Help",
        }
    }
}

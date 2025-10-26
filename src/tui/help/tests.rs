//! Tests for the help system

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::*;

    #[test]
    fn test_help_content_initialization() {
        let content = HelpContent::new();
        let topics = content.all_topics();
        assert!(!topics.is_empty(), "Help content should have topics");
    }

    #[test]
    fn test_help_categories() {
        let content = HelpContent::new();
        let categories = content.all_categories();
        assert!(!categories.is_empty(), "Should have categories");

        // Check that each category has topics
        for (cat, topics) in categories {
            assert!(
                !topics.is_empty(),
                "Category {} should have topics",
                cat.name()
            );
        }
    }

    #[test]
    fn test_get_topic_by_id() {
        let content = HelpContent::new();
        let topic = content.get_topic("getting_started");
        assert!(topic.is_some(), "Should find getting_started topic");
        assert_eq!(topic.unwrap().id, "getting_started");
    }

    #[test]
    fn test_search_engine_basic() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let results = engine.search("workflow");
        assert!(!results.is_empty(), "Should find workflow-related topics");
    }

    #[test]
    fn test_search_case_insensitive() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let lower = engine.search("workflow");
        let upper = engine.search("WORKFLOW");
        assert_eq!(
            lower.len(),
            upper.len(),
            "Search should be case-insensitive"
        );
    }

    #[test]
    fn test_search_relevance_scoring() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let results = engine.search("keyboard shortcuts");
        assert!(!results.is_empty(), "Should find keyboard shortcut topics");

        // Results should be sorted by relevance
        for i in 1..results.len() {
            assert!(
                results[i - 1].score >= results[i].score,
                "Results should be sorted by descending relevance"
            );
        }
    }

    #[test]
    fn test_search_suggestions() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let suggestions = engine.suggest("work");
        assert!(
            !suggestions.is_empty(),
            "Should have suggestions for 'work'"
        );
    }

    #[test]
    fn test_markdown_renderer_headers() {
        let renderer = MarkdownRenderer::new();
        let text = renderer.render("# Header 1\n## Header 2\n### Header 3");
        assert_eq!(text.lines.len(), 3, "Should render 3 header lines");
    }

    #[test]
    fn test_markdown_renderer_lists() {
        let renderer = MarkdownRenderer::new();
        let text = renderer.render("- Item 1\n- Item 2\n- Item 3");
        assert_eq!(text.lines.len(), 3, "Should render 3 list items");
    }

    #[test]
    fn test_markdown_renderer_code_blocks() {
        let renderer = MarkdownRenderer::new();
        let text = renderer.render("```\ncode line 1\ncode line 2\n```");
        // 2 code lines + 1 empty line after block
        assert!(text.lines.len() >= 2, "Should render code block lines");
    }

    #[test]
    fn test_markdown_inline_styles() {
        let renderer = MarkdownRenderer::new();
        let text = renderer.render("**bold** and *italic* and `code`");
        assert_eq!(
            text.lines.len(),
            1,
            "Should render one line with inline styles"
        );
    }

    #[test]
    fn test_help_view_state_creation() {
        let state = HelpViewState::new(HelpContext::General);
        assert_eq!(state.context(), HelpContext::General);
    }

    #[test]
    fn test_help_view_state_context_switch() {
        let mut state = HelpViewState::new(HelpContext::General);
        state.set_context(HelpContext::Editor);
        assert_eq!(state.context(), HelpContext::Editor);
    }

    #[test]
    fn test_help_view_state_navigation() {
        let mut state = HelpViewState::new(HelpContext::General);

        // Test navigation - methods should work without errors
        state.next_category();
        state.prev_category();
        // Navigation completed successfully
    }

    #[test]
    fn test_help_view_state_search() {
        let mut state = HelpViewState::new(HelpContext::General);

        state.enter_search_mode();
        state.update_search("workflow".to_string());

        // Search functionality works without errors
    }

    #[test]
    fn test_help_view_state_clone() {
        let state1 = HelpViewState::new(HelpContext::General);
        let state2 = state1.clone();

        assert_eq!(state1.context(), state2.context());
        // Clone creates independent copy
    }

    #[test]
    fn test_help_context_topics() {
        let topics = HelpContext::WorkflowList.topics();
        assert!(
            !topics.is_empty(),
            "WorkflowList context should have topics"
        );

        let topics = HelpContext::Editor.topics();
        assert!(!topics.is_empty(), "Editor context should have topics");
    }

    #[test]
    fn test_help_context_title() {
        assert_eq!(HelpContext::General.title(), "General Help");
        assert_eq!(HelpContext::Editor.title(), "Workflow Editor Help");
    }

    #[test]
    fn test_category_names() {
        use super::super::content::HelpCategory;

        assert_eq!(HelpCategory::GettingStarted.name(), "Getting Started");
        assert_eq!(
            HelpCategory::WorkflowManagement.name(),
            "Workflow Management"
        );
        assert_eq!(HelpCategory::KeyboardShortcuts.name(), "Keyboard Shortcuts");
    }

    #[test]
    fn test_search_excerpt_extraction() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let results = engine.search("workflow");
        if let Some(result) = results.first() {
            assert!(!result.excerpt.is_empty(), "Excerpt should not be empty");
            assert!(
                result.excerpt.len() <= 200,
                "Excerpt should be reasonably sized"
            );
        }
    }

    #[test]
    fn test_search_matched_keywords() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let results = engine.search("workflow");
        if let Some(result) = results.first() {
            assert!(
                !result.matched_keywords.is_empty(),
                "Should have matched keywords"
            );
        }
    }

    #[test]
    fn test_empty_search_query() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let results = engine.search("");
        assert!(results.is_empty(), "Empty query should return no results");
    }

    #[test]
    fn test_search_no_results() {
        let content = HelpContent::new();
        let engine = HelpSearchEngine::new(content);

        let results = engine.search("xyznonexistentkeyword123");
        assert!(
            results.is_empty(),
            "Non-existent keyword should return no results"
        );
    }

    #[test]
    fn test_topic_related_topics() {
        let content = HelpContent::new();
        if let Some(topic) = content.get_topic("getting_started") {
            assert!(
                !topic.related.is_empty(),
                "Getting started should have related topics"
            );
        }
    }

    #[test]
    fn test_topic_keywords() {
        let content = HelpContent::new();
        if let Some(topic) = content.get_topic("getting_started") {
            assert!(!topic.keywords.is_empty(), "Should have keywords");
        }
    }

    #[test]
    fn test_markdown_table_rendering() {
        let renderer = MarkdownRenderer::new();
        let table = "| Col1 | Col2 |\n|------|------|\n| A | B |";
        let text = renderer.render(table);
        assert!(text.lines.len() >= 2, "Should render table rows");
    }

    #[test]
    fn test_markdown_blockquote() {
        let renderer = MarkdownRenderer::new();
        let text = renderer.render("> This is a quote");
        assert_eq!(text.lines.len(), 1, "Should render blockquote");
    }

    #[test]
    fn test_help_view_state_scroll() {
        let mut state = HelpViewState::new(HelpContext::General);

        state.scroll_down();
        // Should scroll down without error

        state.scroll_up();
        // Should scroll back up without error
    }

    #[test]
    fn test_help_view_state_page_size() {
        let mut state = HelpViewState::new(HelpContext::General);

        state.update_page_size(50);
        // Page size updated successfully
    }
}

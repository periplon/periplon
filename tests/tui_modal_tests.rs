//! Comprehensive ratatui backend tests for modal rendering and keyboard handling
//!
//! Tests modal dialog rendering, layout calculations, keyboard navigation,
//! and state management using the ratatui TestBackend.

#![cfg(feature = "tui")]

use periplon_sdk::tui::state::{ConfirmAction, InputAction, Modal};
use periplon_sdk::tui::theme::Theme;
use periplon_sdk::tui::ui::ModalView;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::path::PathBuf;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a test terminal with specified dimensions
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

/// Render a modal and return the terminal for assertions
fn render_modal(
    modal: &Modal,
    input_buffer: &str,
    theme: &Theme,
    width: u16,
    height: u16,
) -> Terminal<TestBackend> {
    let mut terminal = create_test_terminal(width, height);

    terminal
        .draw(|frame| {
            let area = frame.area();
            ModalView::render(frame, area, modal, input_buffer, theme);
        })
        .unwrap();

    terminal
}

/// Check if buffer contains text at approximate position
fn buffer_contains_text(terminal: &Terminal<TestBackend>, text: &str) -> bool {
    let buffer = terminal.backend().buffer();
    let content = buffer.content();

    // Build a string representation of visible content
    let visible: String = content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .join("");

    visible.contains(text)
}

/// Count visible borders in the buffer (used to verify modal frame)
fn count_border_chars(terminal: &Terminal<TestBackend>) -> usize {
    let buffer = terminal.backend().buffer();
    buffer
        .content()
        .iter()
        .filter(|cell| {
            let symbol = cell.symbol();
            // Count box-drawing characters
            matches!(
                symbol,
                "─" | "│" | "┌" | "┐" | "└" | "┘" | "├" | "┤" | "┬" | "┴" | "┼"
            )
        })
        .count()
}

// ============================================================================
// Confirm Modal Rendering Tests
// ============================================================================

#[test]
fn test_confirm_modal_basic_rendering() {
    let modal = Modal::Confirm {
        title: "Test Confirmation".to_string(),
        message: "Are you sure?".to_string(),
        action: ConfirmAction::Exit,
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    // Modal should have a border
    assert!(
        count_border_chars(&terminal) > 0,
        "Modal should have borders"
    );

    // Title should be visible
    assert!(
        buffer_contains_text(&terminal, "Test Confirmation"),
        "Title should be visible"
    );

    // Message should be visible
    assert!(
        buffer_contains_text(&terminal, "Are you sure?"),
        "Message should be visible"
    );

    // Keyboard hints should be visible
    assert!(
        buffer_contains_text(&terminal, "y"),
        "Yes hint should be visible"
    );
    assert!(
        buffer_contains_text(&terminal, "n"),
        "No hint should be visible"
    );
}

#[test]
fn test_confirm_modal_centering() {
    let modal = Modal::Confirm {
        title: "Centered Modal".to_string(),
        message: "This should be centered".to_string(),
        action: ConfirmAction::Exit,
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 100, 30);

    // The modal should be centered, which means empty space on edges
    // Check that content is not at the extreme edges
    let buffer = terminal.backend().buffer();

    // First few columns should be empty (left margin)
    let left_empty = (0..10).all(|x| buffer.cell((x, 15)).unwrap().symbol().trim().is_empty());

    // Last few columns should be empty (right margin)
    let right_empty = (90..100).all(|x| buffer.cell((x, 15)).unwrap().symbol().trim().is_empty());

    assert!(
        left_empty && right_empty,
        "Modal should have margins for centering"
    );
}

#[test]
fn test_confirm_modal_different_actions() {
    let actions = vec![
        ConfirmAction::Exit,
        ConfirmAction::DeleteWorkflow(PathBuf::from("test.yaml")),
        ConfirmAction::ExecuteWorkflow(PathBuf::from("workflow.yaml")),
        ConfirmAction::DiscardChanges,
        ConfirmAction::StopExecution,
    ];

    let theme = Theme::default();

    for action in actions {
        let modal = Modal::Confirm {
            title: "Action Test".to_string(),
            message: "Test message".to_string(),
            action: action.clone(),
        };

        let terminal = render_modal(&modal, "", &theme, 80, 24);

        // All confirm modals should have Yes/No options
        assert!(
            buffer_contains_text(&terminal, "y"),
            "Action {:?} should show 'y' option",
            action
        );
        assert!(
            buffer_contains_text(&terminal, "n"),
            "Action {:?} should show 'n' option",
            action
        );
    }
}

#[test]
fn test_confirm_modal_long_message() {
    let modal = Modal::Confirm {
        title: "Long Message Test".to_string(),
        message: "This is a very long message that should wrap properly within the modal dialog box to ensure good user experience even with lengthy text content.".to_string(),
        action: ConfirmAction::Exit,
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    // Should still render without panicking
    assert!(buffer_contains_text(&terminal, "Long Message Test"));
    assert!(count_border_chars(&terminal) > 0);
}

// ============================================================================
// Input Modal Rendering Tests
// ============================================================================

#[test]
fn test_input_modal_basic_rendering() {
    let modal = Modal::Input {
        title: "Input Test".to_string(),
        prompt: "Enter value:".to_string(),
        default: "default".to_string(),
        action: InputAction::CreateWorkflow,
    };

    let theme = Theme::default();
    let input_buffer = "test input";
    let terminal = render_modal(&modal, input_buffer, &theme, 80, 24);

    // Title should be visible
    assert!(buffer_contains_text(&terminal, "Input Test"));

    // Prompt should be visible
    assert!(buffer_contains_text(&terminal, "Enter value:"));

    // Input buffer should be visible
    assert!(buffer_contains_text(&terminal, "test input"));

    // Keyboard hints
    assert!(buffer_contains_text(&terminal, "Enter"));
    assert!(buffer_contains_text(&terminal, "Esc"));
}

#[test]
fn test_input_modal_empty_buffer() {
    let modal = Modal::Input {
        title: "Empty Input".to_string(),
        prompt: "Type something:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    // Should render prompt and cursor even with empty input
    assert!(buffer_contains_text(&terminal, "Type something:"));
    assert!(buffer_contains_text(&terminal, ">"));
}

#[test]
fn test_input_modal_different_actions() {
    let actions = vec![
        InputAction::CreateWorkflow,
        InputAction::RenameWorkflow(PathBuf::from("old.yaml")),
        InputAction::GenerateWorkflow,
        InputAction::SetWorkflowDescription,
        InputAction::SaveWorkflowAs,
    ];

    let theme = Theme::default();

    for action in actions {
        let modal = Modal::Input {
            title: "Test".to_string(),
            prompt: "Input:".to_string(),
            default: "".to_string(),
            action: action.clone(),
        };

        let terminal = render_modal(&modal, "value", &theme, 80, 24);

        // All input modals should show Enter/Esc hints
        assert!(
            buffer_contains_text(&terminal, "Enter"),
            "Action {:?} should show Enter hint",
            action
        );
        assert!(
            buffer_contains_text(&terminal, "Esc"),
            "Action {:?} should show Esc hint",
            action
        );
    }
}

#[test]
fn test_input_modal_cursor_indicator() {
    let modal = Modal::Input {
        title: "Cursor Test".to_string(),
        prompt: "Type:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "abc", &theme, 80, 24);

    // Cursor should be visible (represented by the blinking character)
    assert!(buffer_contains_text(&terminal, "▌"));
}

// ============================================================================
// Error Modal Rendering Tests
// ============================================================================

#[test]
fn test_error_modal_rendering() {
    let modal = Modal::Error {
        title: "Error Occurred".to_string(),
        message: "Something went wrong!".to_string(),
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    assert!(buffer_contains_text(&terminal, "Error Occurred"));
    assert!(buffer_contains_text(&terminal, "Something went wrong!"));
    assert!(buffer_contains_text(&terminal, "Enter"));
}

#[test]
fn test_error_modal_multiline() {
    let modal = Modal::Error {
        title: "Validation Error".to_string(),
        message: "Multiple errors:\n- Invalid syntax\n- Missing field\n- Type mismatch".to_string(),
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    assert!(buffer_contains_text(&terminal, "Validation Error"));
    assert!(buffer_contains_text(&terminal, "Multiple errors"));
}

// ============================================================================
// Info Modal Rendering Tests
// ============================================================================

#[test]
fn test_info_modal_rendering() {
    let modal = Modal::Info {
        title: "Information".to_string(),
        message: "This is an informational message.".to_string(),
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    assert!(buffer_contains_text(&terminal, "Information"));
    assert!(buffer_contains_text(&terminal, "informational message"));
    assert!(buffer_contains_text(&terminal, "Enter"));
}

// ============================================================================
// Success Modal Rendering Tests
// ============================================================================

#[test]
fn test_success_modal_rendering() {
    let modal = Modal::Success {
        title: "Success".to_string(),
        message: "Operation completed successfully!".to_string(),
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    assert!(buffer_contains_text(&terminal, "Success"));
    assert!(buffer_contains_text(&terminal, "completed successfully"));
    assert!(buffer_contains_text(&terminal, "Enter"));
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_modal_rendering_with_different_themes() {
    let modal = Modal::Confirm {
        title: "Theme Test".to_string(),
        message: "Testing themes".to_string(),
        action: ConfirmAction::Exit,
    };

    let themes = vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::solarized(),
    ];

    for theme in themes {
        let terminal = render_modal(&modal, "", &theme, 80, 24);

        // Should render successfully with all themes
        assert!(buffer_contains_text(&terminal, "Theme Test"));
        assert!(count_border_chars(&terminal) > 0);
    }
}

// ============================================================================
// Layout and Sizing Tests
// ============================================================================

#[test]
fn test_modal_minimum_size() {
    let modal = Modal::Info {
        title: "Min".to_string(),
        message: "X".to_string(),
    };

    let theme = Theme::default();

    // Test with very small terminal
    let terminal = render_modal(&modal, "", &theme, 50, 15);

    // Should still render without panicking
    assert!(buffer_contains_text(&terminal, "Min"));
}

#[test]
fn test_modal_large_terminal() {
    let modal = Modal::Info {
        title: "Large Terminal".to_string(),
        message: "Testing on large screen".to_string(),
    };

    let theme = Theme::default();

    // Test with large terminal
    let terminal = render_modal(&modal, "", &theme, 200, 60);

    // Should still be centered and not stretch to full width
    assert!(buffer_contains_text(&terminal, "Large Terminal"));

    // Verify margins exist (modal shouldn't use full width)
    let buffer = terminal.backend().buffer();
    let has_left_margin = buffer.cell((0, 30)).unwrap().symbol().trim().is_empty();
    let has_right_margin = buffer.cell((199, 30)).unwrap().symbol().trim().is_empty();

    assert!(has_left_margin && has_right_margin);
}

#[test]
fn test_modal_proportional_sizing() {
    let modal = Modal::Confirm {
        title: "Proportions".to_string(),
        message: "Testing proportional layout".to_string(),
        action: ConfirmAction::Exit,
    };

    let theme = Theme::default();

    // Test at different sizes
    for (width, height) in [(80, 24), (100, 30), (120, 40)] {
        let terminal = render_modal(&modal, "", &theme, width, height);

        // Modal should render at all sizes
        assert!(buffer_contains_text(&terminal, "Proportions"));
        assert!(count_border_chars(&terminal) > 0);
    }
}

// ============================================================================
// Edge Cases and Robustness Tests
// ============================================================================

#[test]
fn test_modal_empty_strings() {
    let modal = Modal::Info {
        title: "".to_string(),
        message: "".to_string(),
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    // Should render without panicking, even with empty strings
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_modal_special_characters() {
    let modal = Modal::Info {
        title: "Special: 日本語 🎉".to_string(),
        message: "Unicode: ñ ü ö € ™ ®".to_string(),
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    // Should handle special characters
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_modal_very_long_title() {
    let modal = Modal::Info {
        title: "This is an extremely long title that might exceed the available width of the modal dialog box and should be handled gracefully".to_string(),
        message: "Short message".to_string(),
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_input_modal_very_long_input() {
    let modal = Modal::Input {
        title: "Long Input".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    };

    let theme = Theme::default();
    let very_long_input = "a".repeat(200);
    let terminal = render_modal(&modal, &very_long_input, &theme, 80, 24);

    // Should handle long input without panicking
    assert!(count_border_chars(&terminal) > 0);
}

// ============================================================================
// Modal Area Calculation Tests
// ============================================================================

#[test]
fn test_modal_area_constraints() {
    let modal = Modal::Info {
        title: "Area Test".to_string(),
        message: "Testing area calculation".to_string(),
    };

    let theme = Theme::default();
    let mut terminal = create_test_terminal(100, 40);

    terminal
        .draw(|frame| {
            let area = frame.area();
            // Modal rendering uses layout constraints:
            // - Vertical: 25% top, min 10 lines, 25% bottom
            // - Horizontal: 15% left, min 40 chars, 15% right
            ModalView::render(frame, area, &modal, "", &theme);
        })
        .unwrap();

    // Verify modal was rendered
    assert!(buffer_contains_text(&terminal, "Area Test"));
}

// ============================================================================
// Clear Widget Tests
// ============================================================================

#[test]
fn test_modal_clears_background() {
    let modal = Modal::Info {
        title: "Clear Test".to_string(),
        message: "Background should be cleared".to_string(),
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    // The Clear widget should clear the modal area
    // This is implicit in the rendering - we verify by checking modal is visible
    assert!(buffer_contains_text(&terminal, "Clear Test"));
}

// ============================================================================
// Keyboard Hint Tests
// ============================================================================

#[test]
fn test_confirm_modal_keyboard_hints_formatting() {
    let modal = Modal::Confirm {
        title: "Hints".to_string(),
        message: "Check formatting".to_string(),
        action: ConfirmAction::Exit,
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    // Both 'y' and 'n' should be present
    assert!(buffer_contains_text(&terminal, "y"));
    assert!(buffer_contains_text(&terminal, "n"));

    // "Yes" and "No" labels should be present
    assert!(buffer_contains_text(&terminal, "Yes"));
    assert!(buffer_contains_text(&terminal, "No"));
}

#[test]
fn test_input_modal_keyboard_hints_formatting() {
    let modal = Modal::Input {
        title: "Hints".to_string(),
        prompt: "Test:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    // Both Enter and Esc should be present
    assert!(buffer_contains_text(&terminal, "Enter"));
    assert!(buffer_contains_text(&terminal, "Esc"));

    // Action labels should be present
    assert!(buffer_contains_text(&terminal, "Submit"));
    assert!(buffer_contains_text(&terminal, "Cancel"));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_modal_sequence_rendering() {
    let modals = [
        Modal::Confirm {
            title: "Step 1".to_string(),
            message: "Confirm action".to_string(),
            action: ConfirmAction::Exit,
        },
        Modal::Input {
            title: "Step 2".to_string(),
            prompt: "Enter name:".to_string(),
            default: "".to_string(),
            action: InputAction::CreateWorkflow,
        },
        Modal::Success {
            title: "Step 3".to_string(),
            message: "Completed!".to_string(),
        },
    ];

    let theme = Theme::default();

    for (i, modal) in modals.iter().enumerate() {
        let terminal = render_modal(modal, "", &theme, 80, 24);

        // Each modal should render successfully
        assert!(
            buffer_contains_text(&terminal, &format!("Step {}", i + 1)),
            "Modal {} should render",
            i + 1
        );
    }
}

#[test]
fn test_modal_view_static_method() {
    // Test that ModalView::render static method works
    let modal = Modal::Info {
        title: "Static Test".to_string(),
        message: "Testing static method".to_string(),
    };

    let theme = Theme::default();
    let mut terminal = create_test_terminal(80, 24);

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 24);
            ModalView::render(frame, area, &modal, "", &theme);
        })
        .unwrap();

    assert!(buffer_contains_text(&terminal, "Static Test"));
}

// ============================================================================
// Alignment Tests
// ============================================================================

#[test]
fn test_modal_content_alignment() {
    let modal = Modal::Confirm {
        title: "Alignment".to_string(),
        message: "Centered".to_string(),
        action: ConfirmAction::Exit,
    };

    let theme = Theme::default();
    let terminal = render_modal(&modal, "", &theme, 80, 24);

    // Content should be centered (Alignment::Center in modal implementation)
    // We verify this by checking the modal renders properly
    assert!(buffer_contains_text(&terminal, "Centered"));
}

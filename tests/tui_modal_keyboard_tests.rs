//! Comprehensive keyboard handling tests for modal dialogs
//!
//! Tests modal keyboard event handling, input processing, and state transitions.

#![cfg(feature = "tui")]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use periplon_sdk::tui::state::{AppState, ConfirmAction, InputAction, Modal};

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a KeyEvent helper for testing
fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

/// Create a KeyEvent with Ctrl modifier
#[allow(dead_code)]
fn ctrl_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

/// Simulate modal key handler behavior (from app.rs handle_modal_key logic)
async fn handle_modal_key_simulation(
    state: &mut AppState,
    key_event: KeyEvent,
) -> Option<ModalAction> {
    match key_event.code {
        KeyCode::Esc => {
            state.close_modal();
            Some(ModalAction::Closed)
        }

        KeyCode::Enter => {
            if let Some(modal) = state.modal.take() {
                match modal {
                    Modal::Confirm { action, .. } => Some(ModalAction::Confirmed(action)),
                    Modal::Input { action, .. } => {
                        let value = state.input_buffer.clone();
                        state.input_buffer.clear();
                        Some(ModalAction::InputSubmitted(action, value))
                    }
                    Modal::Error { .. } | Modal::Info { .. } | Modal::Success { .. } => {
                        Some(ModalAction::Closed)
                    }
                }
            } else {
                None
            }
        }

        KeyCode::Char('y') | KeyCode::Char('Y') => {
            // y/Y only work for Confirm modals
            // We need to check BEFORE taking to avoid removing non-Confirm modals
            if matches!(state.modal, Some(Modal::Confirm { .. })) {
                if let Some(Modal::Confirm { action, .. }) = state.modal.take() {
                    Some(ModalAction::Confirmed(action))
                } else {
                    unreachable!()
                }
            } else {
                // For Input modals, y/Y don't get added as they're consumed here
                None
            }
        }

        KeyCode::Char('n') | KeyCode::Char('N') => {
            // n/N only work for Confirm modals, ignored otherwise
            if matches!(state.modal, Some(Modal::Confirm { .. })) {
                state.close_modal();
                Some(ModalAction::Closed)
            } else {
                // For Input modals, n/N don't get added as they're consumed here
                None
            }
        }

        _ => {
            // Handle input for Input modals
            if matches!(state.modal, Some(Modal::Input { .. })) {
                match key_event.code {
                    KeyCode::Char(c) => {
                        // Note: y/Y/n/N are already handled above, so won't reach here
                        state.input_buffer.push(c);
                        Some(ModalAction::InputChanged)
                    }
                    KeyCode::Backspace => {
                        state.input_buffer.pop();
                        Some(ModalAction::InputChanged)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

/// Actions that can result from modal key handling
#[derive(Debug, Clone, PartialEq)]
enum ModalAction {
    Closed,
    Confirmed(ConfirmAction),
    InputSubmitted(InputAction, String),
    InputChanged,
}

// ============================================================================
// Confirm Modal Keyboard Tests
// ============================================================================

#[tokio::test]
async fn test_confirm_modal_yes_key() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Confirm {
        title: "Test".to_string(),
        message: "Confirm?".to_string(),
        action: ConfirmAction::Exit,
    });

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Char('y')))
        .await
        .unwrap();

    assert_eq!(action, ModalAction::Confirmed(ConfirmAction::Exit));
    assert!(!state.has_modal(), "Modal should be closed after 'y'");
}

#[tokio::test]
async fn test_confirm_modal_yes_uppercase() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Confirm {
        title: "Test".to_string(),
        message: "Confirm?".to_string(),
        action: ConfirmAction::Exit,
    });

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Char('Y')))
        .await
        .unwrap();

    assert_eq!(action, ModalAction::Confirmed(ConfirmAction::Exit));
    assert!(!state.has_modal());
}

#[tokio::test]
async fn test_confirm_modal_no_key() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Confirm {
        title: "Test".to_string(),
        message: "Confirm?".to_string(),
        action: ConfirmAction::Exit,
    });

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Char('n')))
        .await
        .unwrap();

    assert_eq!(action, ModalAction::Closed);
    assert!(!state.has_modal(), "Modal should be closed after 'n'");
}

#[tokio::test]
async fn test_confirm_modal_no_uppercase() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Confirm {
        title: "Test".to_string(),
        message: "Confirm?".to_string(),
        action: ConfirmAction::Exit,
    });

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Char('N')))
        .await
        .unwrap();

    assert_eq!(action, ModalAction::Closed);
    assert!(!state.has_modal());
}

#[tokio::test]
async fn test_confirm_modal_enter_key() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Confirm {
        title: "Test".to_string(),
        message: "Confirm?".to_string(),
        action: ConfirmAction::StopExecution,
    });

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Enter))
        .await
        .unwrap();

    assert_eq!(action, ModalAction::Confirmed(ConfirmAction::StopExecution));
    assert!(!state.has_modal());
}

#[tokio::test]
async fn test_confirm_modal_escape_key() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Confirm {
        title: "Test".to_string(),
        message: "Confirm?".to_string(),
        action: ConfirmAction::Exit,
    });

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Esc))
        .await
        .unwrap();

    assert_eq!(action, ModalAction::Closed);
    assert!(!state.has_modal(), "Modal should be closed by Escape");
}

#[tokio::test]
async fn test_confirm_modal_different_actions() {
    let actions = vec![
        ConfirmAction::Exit,
        ConfirmAction::DeleteWorkflow(std::path::PathBuf::from("test.yaml")),
        ConfirmAction::ExecuteWorkflow(std::path::PathBuf::from("workflow.yaml")),
        ConfirmAction::DiscardChanges,
        ConfirmAction::StopExecution,
    ];

    for confirm_action in actions {
        let mut state = AppState::new();
        state.modal = Some(Modal::Confirm {
            title: "Test".to_string(),
            message: "Confirm?".to_string(),
            action: confirm_action.clone(),
        });

        let result = handle_modal_key_simulation(&mut state, key(KeyCode::Char('y')))
            .await
            .unwrap();

        assert_eq!(result, ModalAction::Confirmed(confirm_action));
        assert!(!state.has_modal());
    }
}

// ============================================================================
// Input Modal Keyboard Tests
// ============================================================================

#[tokio::test]
async fn test_input_modal_text_entry() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type "test"
    for c in ['t', 'e', 's', 't'] {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }

    assert_eq!(state.input_buffer, "test");
    assert!(state.has_modal());
}

#[tokio::test]
async fn test_input_modal_backspace() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type "hello"
    for c in ['h', 'e', 'l', 'l', 'o'] {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }
    assert_eq!(state.input_buffer, "hello");

    // Backspace twice
    handle_modal_key_simulation(&mut state, key(KeyCode::Backspace)).await;
    handle_modal_key_simulation(&mut state, key(KeyCode::Backspace)).await;

    assert_eq!(state.input_buffer, "hel");
}

#[tokio::test]
async fn test_input_modal_backspace_empty() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Backspace on empty buffer should not panic
    handle_modal_key_simulation(&mut state, key(KeyCode::Backspace)).await;

    assert_eq!(state.input_buffer, "");
}

#[tokio::test]
async fn test_input_modal_enter_submit() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type some text
    for c in ['t', 'e', 's', 't'] {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Enter))
        .await
        .unwrap();

    assert_eq!(
        action,
        ModalAction::InputSubmitted(InputAction::CreateWorkflow, "test".to_string())
    );
    assert!(!state.has_modal());
    assert_eq!(
        state.input_buffer, "",
        "Buffer should be cleared after submit"
    );
}

#[tokio::test]
async fn test_input_modal_escape_cancel() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type some text
    for c in ['t', 'e', 's', 't'] {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Esc))
        .await
        .unwrap();

    assert_eq!(action, ModalAction::Closed);
    assert!(!state.has_modal());
    // Note: buffer may still contain text after escape
}

#[tokio::test]
async fn test_input_modal_special_characters() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type special characters
    for c in ['!', '@', '#', '$', '%', '^', '&', '*'] {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }

    assert_eq!(state.input_buffer, "!@#$%^&*");
}

#[tokio::test]
async fn test_input_modal_spaces() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type with spaces
    for c in "hello world".chars() {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }

    assert_eq!(state.input_buffer, "hello world");
}

#[tokio::test]
async fn test_input_modal_numbers() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type numbers
    for c in "12345".chars() {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }

    assert_eq!(state.input_buffer, "12345");
}

#[tokio::test]
async fn test_input_modal_different_actions() {
    let actions = vec![
        InputAction::CreateWorkflow,
        InputAction::RenameWorkflow(std::path::PathBuf::from("old.yaml")),
        InputAction::GenerateWorkflow,
        InputAction::SetWorkflowDescription,
        InputAction::SaveWorkflowAs,
    ];

    for input_action in actions {
        let mut state = AppState::new();
        state.modal = Some(Modal::Input {
            title: "Test".to_string(),
            prompt: "Enter:".to_string(),
            default: "".to_string(),
            action: input_action.clone(),
        });

        // Type and submit
        for c in "value".chars() {
            handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
        }

        let result = handle_modal_key_simulation(&mut state, key(KeyCode::Enter))
            .await
            .unwrap();

        assert_eq!(
            result,
            ModalAction::InputSubmitted(input_action, "value".to_string())
        );
        assert!(!state.has_modal());
    }
}

// ============================================================================
// Info/Error/Success Modal Keyboard Tests
// ============================================================================

#[tokio::test]
async fn test_info_modal_enter_close() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Info {
        title: "Info".to_string(),
        message: "Message".to_string(),
    });

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Enter))
        .await
        .unwrap();

    assert_eq!(action, ModalAction::Closed);
    assert!(!state.has_modal());
}

#[tokio::test]
async fn test_error_modal_enter_close() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Error {
        title: "Error".to_string(),
        message: "Error message".to_string(),
    });

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Enter))
        .await
        .unwrap();

    assert_eq!(action, ModalAction::Closed);
    assert!(!state.has_modal());
}

#[tokio::test]
async fn test_success_modal_enter_close() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Success {
        title: "Success".to_string(),
        message: "Success message".to_string(),
    });

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Enter))
        .await
        .unwrap();

    assert_eq!(action, ModalAction::Closed);
    assert!(!state.has_modal());
}

#[tokio::test]
async fn test_info_modal_escape_close() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Info {
        title: "Info".to_string(),
        message: "Message".to_string(),
    });

    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Esc))
        .await
        .unwrap();

    assert_eq!(action, ModalAction::Closed);
    assert!(!state.has_modal());
}

// ============================================================================
// Edge Cases and Robustness Tests
// ============================================================================

#[tokio::test]
async fn test_no_modal_key_handling() {
    let mut state = AppState::new();
    // No modal active

    let result = handle_modal_key_simulation(&mut state, key(KeyCode::Enter)).await;

    assert!(result.is_none(), "Should return None when no modal active");
}

#[tokio::test]
async fn test_input_modal_y_n_keys_ignored() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Y and N are consumed by confirm modal handling, not added to input buffer
    // This matches the actual implementation in app.rs where y/Y/n/N are checked
    // before the fallback input handler
    handle_modal_key_simulation(&mut state, key(KeyCode::Char('y'))).await;
    handle_modal_key_simulation(&mut state, key(KeyCode::Char('n'))).await;

    assert_eq!(state.input_buffer, "");
    assert!(state.has_modal(), "Modal should still be open");
}

#[tokio::test]
async fn test_confirm_modal_ignores_regular_chars() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Confirm {
        title: "Test".to_string(),
        message: "Confirm?".to_string(),
        action: ConfirmAction::Exit,
    });

    // Regular characters should be ignored (not y/n)
    let result = handle_modal_key_simulation(&mut state, key(KeyCode::Char('a'))).await;

    assert!(result.is_none(), "Regular chars should be ignored");
    assert!(state.has_modal(), "Modal should still be active");
}

#[tokio::test]
async fn test_ctrl_keys_in_input_modal() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Note: The actual app.rs implementation doesn't check key modifiers,
    // so Ctrl+A with Char('a') code would still be treated as 'a'
    // This test documents that behavior - modifiers aren't filtered
    let result = handle_modal_key_simulation(&mut state, key(KeyCode::Char('a'))).await;

    // Regular 'a' gets added
    assert_eq!(result, Some(ModalAction::InputChanged));
    assert_eq!(state.input_buffer, "a");
}

#[tokio::test]
async fn test_input_modal_very_long_input() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type a very long string
    for c in "a".repeat(1000).chars() {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }

    assert_eq!(state.input_buffer.len(), 1000);
}

// ============================================================================
// State Transition Tests
// ============================================================================

#[tokio::test]
async fn test_confirm_to_no_modal_transition() {
    let mut state = AppState::new();

    // Start with confirm modal
    state.modal = Some(Modal::Confirm {
        title: "Quit".to_string(),
        message: "Are you sure?".to_string(),
        action: ConfirmAction::Exit,
    });
    assert!(state.has_modal());

    // Press 'y' to confirm
    handle_modal_key_simulation(&mut state, key(KeyCode::Char('y'))).await;

    // Should transition to no modal
    assert!(!state.has_modal());
}

#[tokio::test]
async fn test_input_to_no_modal_transition() {
    let mut state = AppState::new();

    // Start with input modal
    state.modal = Some(Modal::Input {
        title: "Create".to_string(),
        prompt: "Name:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });
    assert!(state.has_modal());

    // Type and submit
    for c in "test".chars() {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }
    handle_modal_key_simulation(&mut state, key(KeyCode::Enter)).await;

    // Should transition to no modal
    assert!(!state.has_modal());
}

#[tokio::test]
async fn test_input_buffer_cleared_after_submit() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type text - note 'n' is consumed by confirm handling
    // So we use characters that aren't y/Y/n/N
    for c in "test123".chars() {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }
    assert_eq!(state.input_buffer, "test123");

    // Submit
    handle_modal_key_simulation(&mut state, key(KeyCode::Enter)).await;

    // Buffer should be cleared
    assert_eq!(state.input_buffer, "");
}

#[tokio::test]
async fn test_multiple_modal_escape_sequence() {
    let mut state = AppState::new();

    // Modal 1
    state.modal = Some(Modal::Info {
        title: "Step 1".to_string(),
        message: "First".to_string(),
    });
    assert!(state.has_modal());

    handle_modal_key_simulation(&mut state, key(KeyCode::Esc)).await;
    assert!(!state.has_modal());

    // Modal 2
    state.modal = Some(Modal::Confirm {
        title: "Step 2".to_string(),
        message: "Second".to_string(),
        action: ConfirmAction::Exit,
    });
    assert!(state.has_modal());

    handle_modal_key_simulation(&mut state, key(KeyCode::Esc)).await;
    assert!(!state.has_modal());
}

// ============================================================================
// Unicode and International Input Tests
// ============================================================================

#[tokio::test]
async fn test_input_modal_unicode_characters() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type unicode characters
    for c in "日本語".chars() {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }

    assert_eq!(state.input_buffer, "日本語");
}

#[tokio::test]
async fn test_input_modal_emoji() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type emoji (multi-byte characters)
    for c in "🎉🚀💻".chars() {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }

    assert_eq!(state.input_buffer, "🎉🚀💻");
}

#[tokio::test]
async fn test_input_modal_mixed_unicode() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Mix ASCII and Unicode
    for c in "Hello 世界 🌍".chars() {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }

    assert_eq!(state.input_buffer, "Hello 世界 🌍");
}

// ============================================================================
// Input Buffer Edge Cases
// ============================================================================

#[tokio::test]
async fn test_input_modal_backspace_unicode() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Type unicode
    for c in "日本".chars() {
        handle_modal_key_simulation(&mut state, key(KeyCode::Char(c))).await;
    }
    assert_eq!(state.input_buffer, "日本");

    // Backspace should remove one character
    handle_modal_key_simulation(&mut state, key(KeyCode::Backspace)).await;

    assert_eq!(state.input_buffer, "日");
}

#[tokio::test]
async fn test_input_modal_empty_submit() {
    let mut state = AppState::new();
    state.modal = Some(Modal::Input {
        title: "Test".to_string(),
        prompt: "Enter:".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // Submit without typing anything
    let action = handle_modal_key_simulation(&mut state, key(KeyCode::Enter))
        .await
        .unwrap();

    assert_eq!(
        action,
        ModalAction::InputSubmitted(InputAction::CreateWorkflow, "".to_string())
    );
    assert!(!state.has_modal());
}

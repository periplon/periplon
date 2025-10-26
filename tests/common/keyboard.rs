//! Keyboard event utilities for TUI testing
//!
//! Provides helper functions for creating keyboard events with various
//! modifiers, simplifying keyboard interaction testing.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Create a simple KeyEvent without modifiers
///
/// # Examples
///
/// ```ignore
/// let event = key(KeyCode::Enter);
/// let event = key(KeyCode::Char('a'));
/// ```
pub fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Create a KeyEvent with Ctrl modifier
///
/// # Examples
///
/// ```ignore
/// let event = ctrl_key(KeyCode::Char('c'));  // Ctrl+C
/// let event = ctrl_key(KeyCode::Char('d'));  // Ctrl+D
/// ```
pub fn ctrl_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

/// Create a KeyEvent with Alt modifier
///
/// # Examples
///
/// ```ignore
/// let event = alt_key(KeyCode::Char('x'));  // Alt+X
/// ```
pub fn alt_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::ALT)
}

/// Create a KeyEvent with Shift modifier
///
/// # Examples
///
/// ```ignore
/// let event = shift_key(KeyCode::Tab);  // Shift+Tab (BackTab)
/// ```
pub fn shift_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::SHIFT)
}

/// Create a KeyEvent with custom modifiers
///
/// # Examples
///
/// ```ignore
/// let event = key_with_modifiers(
///     KeyCode::Char('c'),
///     KeyModifiers::CONTROL | KeyModifiers::SHIFT
/// );  // Ctrl+Shift+C
/// ```
pub fn key_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

/// Helper to create Ctrl+Shift combination
pub fn ctrl_shift_key(code: KeyCode) -> KeyEvent {
    key_with_modifiers(code, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
}

/// Helper to create Alt+Shift combination
pub fn alt_shift_key(code: KeyCode) -> KeyEvent {
    key_with_modifiers(code, KeyModifiers::ALT | KeyModifiers::SHIFT)
}

/// Check if a KeyEvent has Ctrl modifier
pub fn is_ctrl(event: &KeyEvent) -> bool {
    event.modifiers.contains(KeyModifiers::CONTROL)
}

/// Check if a KeyEvent has Alt modifier
pub fn is_alt(event: &KeyEvent) -> bool {
    event.modifiers.contains(KeyModifiers::ALT)
}

/// Check if a KeyEvent has Shift modifier
pub fn is_shift(event: &KeyEvent) -> bool {
    event.modifiers.contains(KeyModifiers::SHIFT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_simple() {
        let event = key(KeyCode::Enter);
        assert_eq!(event.code, KeyCode::Enter);
        assert_eq!(event.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_key_char() {
        let event = key(KeyCode::Char('a'));
        assert_eq!(event.code, KeyCode::Char('a'));
        assert_eq!(event.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_ctrl_key() {
        let event = ctrl_key(KeyCode::Char('c'));
        assert_eq!(event.code, KeyCode::Char('c'));
        assert!(is_ctrl(&event));
        assert!(!is_alt(&event));
        assert!(!is_shift(&event));
    }

    #[test]
    fn test_alt_key() {
        let event = alt_key(KeyCode::Char('x'));
        assert_eq!(event.code, KeyCode::Char('x'));
        assert!(!is_ctrl(&event));
        assert!(is_alt(&event));
        assert!(!is_shift(&event));
    }

    #[test]
    fn test_shift_key() {
        let event = shift_key(KeyCode::Tab);
        assert_eq!(event.code, KeyCode::Tab);
        assert!(!is_ctrl(&event));
        assert!(!is_alt(&event));
        assert!(is_shift(&event));
    }

    #[test]
    fn test_ctrl_shift_key() {
        let event = ctrl_shift_key(KeyCode::Char('z'));
        assert!(is_ctrl(&event));
        assert!(is_shift(&event));
        assert!(!is_alt(&event));
    }

    #[test]
    fn test_alt_shift_key() {
        let event = alt_shift_key(KeyCode::Char('f'));
        assert!(!is_ctrl(&event));
        assert!(is_shift(&event));
        assert!(is_alt(&event));
    }

    #[test]
    fn test_key_with_modifiers() {
        let event = key_with_modifiers(
            KeyCode::Char('a'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );
        assert!(is_ctrl(&event));
        assert!(is_alt(&event));
        assert!(!is_shift(&event));
    }
}

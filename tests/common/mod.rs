//! Shared Test Utilities for TUI Tests
//!
//! This module provides common utilities, helpers, and mock backends
//! for TUI component testing. It reduces code duplication and provides
//! consistent testing patterns across all TUI tests.
//!
//! # Modules
//!
//! - `terminal`: Terminal creation and rendering utilities
//! - `keyboard`: Keyboard event creation and simulation
//! - `assertions`: Common test assertions
//! - `fixtures`: Test data fixtures and builders
//! - `mocks`: Mock backends and test doubles

#![cfg(feature = "tui")]

pub mod terminal;
pub mod keyboard;
pub mod assertions;
pub mod fixtures;
pub mod mocks;

// Re-export commonly used items for convenience
pub use terminal::{create_terminal, buffer_contains, buffer_content, render_with_terminal};
pub use keyboard::{key, ctrl_key, alt_key, shift_key, key_with_modifiers};
pub use assertions::{assert_buffer_contains, assert_buffer_not_contains, assert_terminal_size};
pub use fixtures::{create_test_theme, create_all_themes};

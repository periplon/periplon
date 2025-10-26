//! Output Truncation Strategies
//!
//! This module provides truncation strategies for managing unbounded stdout/stderr
//! from script tasks and external commands.

use crate::dsl::schema::TruncationStrategy;
use crate::dsl::state::{OutputType, TaskOutput};

/// Truncate output based on the specified strategy
///
/// # Arguments
///
/// * `content` - The content to truncate
/// * `max_bytes` - Maximum allowed bytes
/// * `strategy` - Truncation strategy to use
///
/// # Returns
///
/// Tuple of (truncated_content, was_truncated)
pub fn truncate_output(
    content: &str,
    max_bytes: usize,
    strategy: &TruncationStrategy,
) -> (String, bool) {
    if content.len() <= max_bytes {
        return (content.to_string(), false);
    }

    let truncated = match strategy {
        TruncationStrategy::Head => truncate_head(content, max_bytes),
        TruncationStrategy::Tail => truncate_tail(content, max_bytes),
        TruncationStrategy::Both => truncate_both(content, max_bytes),
        TruncationStrategy::Summary => {
            // For now, use tail truncation as summary requires AI
            // TODO: Implement AI-based summarization
            truncate_tail(content, max_bytes)
        }
    };

    (truncated, true)
}

/// Keep first N bytes with truncation notice
fn truncate_head(content: &str, max_bytes: usize) -> String {
    let safe_len = max_bytes.min(content.len());
    let truncated_bytes = content.len() - safe_len;

    format!(
        "--- Output (showing first {} bytes of {}) ---\n{}\n--- [{} bytes truncated] ---",
        safe_len,
        content.len(),
        &content[..safe_len],
        truncated_bytes
    )
}

/// Keep last N bytes with truncation notice
fn truncate_tail(content: &str, max_bytes: usize) -> String {
    let safe_len = max_bytes.min(content.len());
    let start = content.len() - safe_len;

    format!(
        "--- [{} bytes truncated] ---\n{}\n--- Output (showing last {} bytes of {}) ---",
        start,
        &content[start..],
        safe_len,
        content.len()
    )
}

/// Keep first N/2 and last N/2 bytes with truncation notice
fn truncate_both(content: &str, max_bytes: usize) -> String {
    let half = max_bytes / 2;
    let safe_half = half.min(content.len());
    let end_start = content.len().saturating_sub(safe_half);
    let truncated_bytes = content.len().saturating_sub(max_bytes);

    format!(
        "--- Output (showing first/last {} bytes of {}) ---\n{}\n--- [{} bytes truncated] ---\n{}",
        safe_half,
        content.len(),
        &content[..safe_half],
        truncated_bytes,
        &content[end_start..]
    )
}

/// Create a TaskOutput from raw content with truncation
pub fn create_task_output(
    task_id: String,
    output_type: OutputType,
    content: String,
    max_bytes: usize,
    strategy: &TruncationStrategy,
) -> TaskOutput {
    let original_size = content.len();
    let (truncated_content, was_truncated) = truncate_output(&content, max_bytes, strategy);

    TaskOutput::new(
        task_id,
        output_type,
        truncated_content,
        original_size,
        was_truncated,
        strategy.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_output_no_truncation_needed() {
        let content = "short content";
        let (result, truncated) = truncate_output(content, 100, &TruncationStrategy::Tail);
        assert_eq!(result, content);
        assert!(!truncated);
    }

    #[test]
    fn test_truncate_head() {
        let content = "0123456789".repeat(10); // 100 bytes
        let (result, truncated) = truncate_output(&content, 50, &TruncationStrategy::Head);
        assert!(truncated);
        assert!(result.contains("showing first"));
        assert!(result.contains("50 bytes truncated"));
    }

    #[test]
    fn test_truncate_tail() {
        let content = "0123456789".repeat(10); // 100 bytes
        let (result, truncated) = truncate_output(&content, 50, &TruncationStrategy::Tail);
        assert!(truncated);
        assert!(result.contains("showing last"));
        assert!(result.contains("50 bytes truncated"));
    }

    #[test]
    fn test_truncate_both() {
        let content = "0123456789".repeat(10); // 100 bytes
        let (result, truncated) = truncate_output(&content, 50, &TruncationStrategy::Both);
        assert!(truncated);
        assert!(result.contains("showing first/last"));
        assert!(result.contains("bytes truncated"));
    }

    #[test]
    fn test_create_task_output_with_truncation() {
        let content = "x".repeat(1000);
        let output = create_task_output(
            "task1".to_string(),
            OutputType::Stdout,
            content.clone(),
            500,
            &TruncationStrategy::Tail,
        );

        assert_eq!(output.task_id, "task1");
        assert_eq!(output.output_type, OutputType::Stdout);
        assert_eq!(output.original_size, 1000);
        assert!(output.truncated);
        assert_eq!(output.strategy, TruncationStrategy::Tail);
    }

    #[test]
    fn test_create_task_output_no_truncation() {
        let content = "small output";
        let output = create_task_output(
            "task1".to_string(),
            OutputType::Stdout,
            content.to_string(),
            1000,
            &TruncationStrategy::Tail,
        );

        assert!(!output.truncated);
        assert_eq!(output.content, content);
        assert_eq!(output.original_size, content.len());
    }

    #[test]
    fn test_truncate_empty_content() {
        let content = "";
        let (result, truncated) = truncate_output(content, 100, &TruncationStrategy::Tail);
        assert_eq!(result, "");
        assert!(!truncated);
    }

    #[test]
    fn test_truncate_exactly_max_bytes() {
        let content = "x".repeat(100);
        let (result, truncated) = truncate_output(&content, 100, &TruncationStrategy::Tail);
        assert_eq!(result, content);
        assert!(!truncated);
    }
}

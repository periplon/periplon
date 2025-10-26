//! Message Formatter
//!
//! Provides formatting utilities for displaying messages in interactive and JSON modes.
//! Interactive mode provides colored, condensed output limited to 2 lines with JSON parsing.
//! JSON mode provides full output for programmatic processing.

use crate::domain::message::{ContentBlock, Message};
use colored::*;
use regex::Regex;
use serde_json::Value;

/// Format a message for display based on the output mode
///
/// # Arguments
///
/// * `msg` - The message to format
/// * `json_mode` - If true, output full JSON debug format; if false, use interactive colored format
/// * `prefix` - Optional prefix to add (e.g., "[Retry 1]")
///
/// # Returns
///
/// Formatted string ready for display
pub fn format_message(msg: &Message, json_mode: bool, prefix: Option<&str>) -> String {
    if json_mode {
        // JSON mode: Full debug output
        if let Some(p) = prefix {
            format!("  [{}] Message: {:?}", p, msg)
        } else {
            format!("  Message: {:?}", msg)
        }
    } else {
        // Interactive mode: Colored, condensed output
        format_message_interactive(msg, prefix)
    }
}

/// Format message for interactive terminal display
/// - Colored output for different message types
/// - Limited to 2 lines maximum
/// - JSON payloads are parsed and only relevant fields shown
fn format_message_interactive(msg: &Message, prefix: Option<&str>) -> String {
    let prefix_str = if let Some(p) = prefix {
        format!("[{}] ", p).yellow().bold().to_string()
    } else {
        String::new()
    };

    match msg {
        Message::Assistant(assistant_msg) => {
            format_assistant_message(assistant_msg, &prefix_str)
        }
        Message::User(user_msg) => {
            format!("{}{}  {}",
                prefix_str,
                "User:".cyan().bold(),
                truncate_and_format(&format!("{:?}", user_msg.message.content), 80)
            )
        }
        Message::System(system_msg) => {
            format!("{}{}  {}: {}",
                prefix_str,
                "System".blue().bold(),
                system_msg.subtype.dimmed(),
                format_json_value(&system_msg.data, 60)
            )
        }
        Message::Result(result_msg) => {
            format_result_message(result_msg, &prefix_str)
        }
        Message::StreamEvent(stream_msg) => {
            format!("{}{}  {}",
                prefix_str,
                "Stream".magenta().bold(),
                format_json_value(&stream_msg.event, 80)
            )
        }
    }
}

/// Format assistant message with content blocks
fn format_assistant_message(msg: &crate::domain::message::AssistantMessage, prefix: &str) -> String {
    let mut output = String::new();

    for (idx, block) in msg.message.content.iter().enumerate() {
        if idx > 0 {
            output.push('\n');
        }

        let block_str = match block {
            ContentBlock::Text { text } => {
                format!("{}{}  {}",
                    prefix,
                    "Assistant:".green().bold(),
                    truncate_and_format(text, 100)
                )
            }
            ContentBlock::Thinking { thinking, .. } => {
                format!("{}{}  {}",
                    prefix,
                    "Thinking:".yellow().bold(),
                    truncate_and_format(thinking, 100).dimmed()
                )
            }
            ContentBlock::ToolUse { name, input, .. } => {
                format!("{}{}  {} {}",
                    prefix,
                    "Tool:".cyan().bold(),
                    name.bright_white(),
                    format_json_value(input, 70).dimmed()
                )
            }
            ContentBlock::ToolResult { tool_use_id, content, is_error } => {
                let status = if is_error.unwrap_or(false) {
                    "Error".red()
                } else {
                    "Result".green()
                };

                let content_str = if let Some(c) = content {
                    format_json_value(c, 70)
                } else {
                    "(empty)".dimmed().to_string()
                };

                format!("{}{}  [{}] {}",
                    prefix,
                    status.bold(),
                    tool_use_id.chars().take(8).collect::<String>().dimmed(),
                    content_str
                )
            }
        };

        output.push_str(&block_str);
    }

    // Ensure we don't exceed 2 lines
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() > 2 {
        format!("{}\n{} {}",
            lines[0],
            lines[1],
            format!("(+{} more blocks)", lines.len() - 2).dimmed()
        )
    } else {
        output
    }
}

/// Format result message
fn format_result_message(msg: &crate::domain::message::ResultMessage, prefix: &str) -> String {
    let status = if msg.is_error {
        "Error".red()
    } else {
        "Complete".green()
    };

    let duration = format!("{:.2}s", msg.duration_ms as f64 / 1000.0);
    let turns = format!("{} turns", msg.num_turns);

    let cost = if let Some(c) = msg.total_cost_usd {
        format!(" ${:.4}", c).yellow().to_string()
    } else {
        String::new()
    };

    let result_preview = if let Some(ref r) = msg.result {
        format!(" - {}", truncate_and_format(r, 50).dimmed())
    } else {
        String::new()
    };

    format!("{}{}  {} | {} | {}{}{}",
        prefix,
        status.bold(),
        duration.bright_white(),
        turns.dimmed(),
        cost,
        result_preview,
        format_json_value(&msg.usage.clone().unwrap_or(Value::Null), 40).dimmed()
    )
}

/// Format a JSON value, extracting only relevant fields and limiting length
fn format_json_value(value: &Value, max_len: usize) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            // Check if string contains JSON
            if let Ok(parsed) = serde_json::from_str::<Value>(s) {
                format_json_value(&parsed, max_len)
            } else {
                truncate_and_format(s, max_len)
            }
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else if arr.len() == 1 {
                format_json_value(&arr[0], max_len)
            } else {
                format!("[{} items]", arr.len())
            }
        }
        Value::Object(obj) => {
            // Extract most relevant fields
            let relevant_keys = extract_relevant_fields(obj);

            if relevant_keys.is_empty() {
                if obj.is_empty() {
                    "{}".to_string()
                } else {
                    format!("{{{} fields}}", obj.len())
                }
            } else {
                let parts: Vec<String> = relevant_keys.iter()
                    .take(2)  // Only show up to 2 fields
                    .filter_map(|key| {
                        obj.get(key).map(|val| {
                            let val_str = match val {
                                Value::String(s) => truncate_and_format(s, 30),
                                Value::Number(n) => n.to_string(),
                                Value::Bool(b) => b.to_string(),
                                Value::Array(a) => format!("[{}]", a.len()),
                                Value::Object(o) => format!("{{{}}}", o.len()),
                                Value::Null => "null".to_string(),
                            };
                            format!("{}: {}", key, val_str)
                        })
                    })
                    .collect();

                let result = parts.join(", ");
                if result.len() > max_len {
                    // Safe UTF-8 truncation using char boundaries
                    let truncated: String = result.chars().take(max_len.saturating_sub(3)).collect();
                    format!("{}...", truncated)
                } else {
                    result
                }
            }
        }
    }
}

/// Extract most relevant fields from a JSON object
/// Priority order: error, message, result, status, name, type, data, content, value
fn extract_relevant_fields(obj: &serde_json::Map<String, Value>) -> Vec<String> {
    let priority_keys = [
        "error", "message", "result", "status",
        "name", "type", "data", "content", "value",
        "text", "output", "response", "id"
    ];

    let mut found = Vec::new();

    // First, collect priority keys that exist
    for key in &priority_keys {
        if obj.contains_key(*key) {
            found.push(key.to_string());
        }
    }

    // If we found priority keys, return them (up to 3)
    if !found.is_empty() {
        return found.into_iter().take(3).collect();
    }

    // Otherwise, return first few keys
    obj.keys()
        .take(3)
        .cloned()
        .collect()
}

/// Truncate text and add ellipsis if needed
fn truncate_and_format(text: &str, max_len: usize) -> String {
    // Remove excessive whitespace and newlines
    let re = Regex::new(r"\s+").unwrap();
    let cleaned = re.replace_all(text.trim(), " ");

    if cleaned.len() > max_len {
        // Safe UTF-8 truncation using char boundaries
        let truncated: String = cleaned.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    } else {
        cleaned.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::message::*;

    #[test]
    fn test_format_json_value_object() {
        let json = serde_json::json!({
            "error": "Test error",
            "status": "failed",
            "other": "data"
        });

        let formatted = format_json_value(&json, 100);
        assert!(formatted.contains("error"));
        assert!(formatted.contains("Test error"));
    }

    #[test]
    fn test_format_json_value_nested_string() {
        let inner = serde_json::json!({"message": "Hello"});
        let json = Value::String(serde_json::to_string(&inner).unwrap());

        let formatted = format_json_value(&json, 100);
        assert!(formatted.contains("message"));
    }

    #[test]
    fn test_truncate_and_format() {
        let long_text = "This is a very long text that should be truncated to fit within the maximum length limit";
        let formatted = truncate_and_format(long_text, 30);

        assert!(formatted.len() <= 30);
        assert!(formatted.ends_with("..."));
    }

    #[test]
    fn test_truncate_and_format_whitespace() {
        let text = "Text   with\n\nmultiple\t\twhitespace";
        let formatted = truncate_and_format(text, 100);

        assert!(!formatted.contains("\n"));
        assert!(!formatted.contains("\t"));
    }

    #[test]
    fn test_extract_relevant_fields() {
        let mut obj = serde_json::Map::new();
        obj.insert("error".to_string(), Value::String("test".to_string()));
        obj.insert("random".to_string(), Value::String("data".to_string()));
        obj.insert("message".to_string(), Value::String("msg".to_string()));

        let fields = extract_relevant_fields(&obj);

        // Should prioritize "error" and "message" over "random"
        assert!(fields.contains(&"error".to_string()));
        assert!(fields.contains(&"message".to_string()));
    }

    #[test]
    fn test_format_message_json_mode() {
        let msg = Message::User(UserMessage {
            message: UserMessageContent {
                role: "user".to_string(),
                content: ContentValue::Text("Hello".to_string()),
            },
            parent_tool_use_id: None,
        });

        let formatted = format_message(&msg, true, None);
        assert!(formatted.starts_with("  Message:"));
    }

    #[test]
    fn test_format_message_interactive_mode() {
        let msg = Message::User(UserMessage {
            message: UserMessageContent {
                role: "user".to_string(),
                content: ContentValue::Text("Hello".to_string()),
            },
            parent_tool_use_id: None,
        });

        let formatted = format_message(&msg, false, None);
        // In interactive mode, should not contain debug format markers
        assert!(!formatted.contains("UserMessage"));
    }

    #[test]
    fn test_truncate_with_multibyte_utf8_chars() {
        // Test with box-drawing characters (3-byte UTF-8 sequences)
        let text = "╔══════════════════════════════════════════════════════════════╗ ║ LOOP TYPES TESTING ║ ╚";
        let formatted = truncate_and_format(text, 30);

        // Should truncate safely without panicking
        // Check character count, not byte count (multi-byte chars make byte length larger)
        assert!(formatted.chars().count() <= 33); // 30 chars + "..."
        assert!(formatted.ends_with("..."));

        // Test that it didn't panic on multi-byte char boundary
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_format_json_value_with_multibyte_chars() {
        let json = serde_json::json!({
            "message": "╔══════════════════════════════════════════════════════════════╗ Test"
        });

        let formatted = format_json_value(&json, 30);

        // Should truncate safely without panicking
        assert!(!formatted.is_empty());
        // The result might be truncated, but should not panic
    }
}

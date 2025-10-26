use periplon_sdk::domain::{parse_message, ContentBlock, Message};
use serde_json::json;

#[test]
fn test_user_message_parsing() {
    let json = json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": "Hello, world!"
        }
    });

    let msg = parse_message(json).unwrap();
    assert!(matches!(msg, Message::User(_)));

    if let Message::User(user_msg) = msg {
        assert_eq!(user_msg.message.role, "user");
    }
}

#[test]
fn test_assistant_message_parsing() {
    let json = json!({
        "type": "assistant",
        "message": {
            "model": "claude-sonnet-4-5",
            "content": [
                {"type": "text", "text": "Hello"}
            ]
        }
    });

    let msg = parse_message(json).unwrap();
    assert!(matches!(msg, Message::Assistant(_)));

    if let Message::Assistant(assistant_msg) = msg {
        assert_eq!(assistant_msg.message.model, "claude-sonnet-4-5");
        assert_eq!(assistant_msg.message.content.len(), 1);

        if let ContentBlock::Text { text } = &assistant_msg.message.content[0] {
            assert_eq!(text, "Hello");
        } else {
            panic!("Expected text content block");
        }
    }
}

#[test]
fn test_tool_use_parsing() {
    let json = json!({
        "type": "assistant",
        "message": {
            "model": "claude-sonnet-4-5",
            "content": [
                {
                    "type": "tool_use",
                    "id": "tool_123",
                    "name": "Read",
                    "input": {"file_path": "/test.txt"}
                }
            ]
        }
    });

    let msg = parse_message(json).unwrap();

    if let Message::Assistant(assistant_msg) = msg {
        if let ContentBlock::ToolUse { id, name, input } = &assistant_msg.message.content[0] {
            assert_eq!(id, "tool_123");
            assert_eq!(name, "Read");
            assert_eq!(
                input.get("file_path").unwrap().as_str().unwrap(),
                "/test.txt"
            );
        } else {
            panic!("Expected tool_use content block");
        }
    }
}

#[test]
fn test_result_message_parsing() {
    let json = json!({
        "type": "result",
        "subtype": "final",
        "duration_ms": 1234,
        "duration_api_ms": 567,
        "is_error": false,
        "num_turns": 3,
        "session_id": "session_abc",
        "total_cost_usd": 0.0042,
        "usage": {
            "input_tokens": 100,
            "output_tokens": 50
        },
        "result": "Success"
    });

    let msg = parse_message(json).unwrap();
    assert!(matches!(msg, Message::Result(_)));

    if let Message::Result(result_msg) = msg {
        assert_eq!(result_msg.duration_ms, 1234);
        assert_eq!(result_msg.num_turns, 3);
        assert_eq!(result_msg.session_id, "session_abc");
        assert_eq!(result_msg.total_cost_usd, Some(0.0042));
        assert!(!result_msg.is_error);
    }
}

#[test]
fn test_message_user_helper() {
    let msg = Message::user("Test prompt");

    if let Message::User(user_msg) = msg {
        assert_eq!(user_msg.message.role, "user");
    } else {
        panic!("Expected user message");
    }
}

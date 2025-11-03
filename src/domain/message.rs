use serde::{Deserialize, Serialize};

/// Main message enum (discriminated union)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "user")]
    User(UserMessage),
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
    #[serde(rename = "system")]
    System(SystemMessage),
    #[serde(rename = "result")]
    Result(ResultMessage),
    #[serde(rename = "stream_event")]
    StreamEvent(StreamEventMessage),
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Message::User(UserMessage {
            message: UserMessageContent {
                role: "user".to_string(),
                content: ContentValue::Text(content.into()),
            },
            parent_tool_use_id: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub message: UserMessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessageContent {
    pub role: String,
    pub content: ContentValue,
}

/// Content can be either string or array of blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentValue {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub message: AssistantMessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessageContent {
    pub model: String,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub subtype: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMessage {
    pub subtype: String,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
    pub is_error: bool,
    pub num_turns: u32,
    pub session_id: String,
    pub total_cost_usd: Option<f64>,
    pub usage: Option<serde_json::Value>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEventMessage {
    pub uuid: String,
    pub session_id: String,
    pub event: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "thinking")]
    Thinking { thinking: String, signature: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },

    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

/// Parse a JSON value into a Message
pub fn parse_message(value: serde_json::Value) -> crate::error::Result<Message> {
    serde_json::from_value(value).map_err(|e| crate::error::Error::ParseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_message_user_convenience_constructor() {
        let msg = Message::user("Hello world");
        match msg {
            Message::User(user_msg) => {
                assert_eq!(user_msg.message.role, "user");
                match &user_msg.message.content {
                    ContentValue::Text(text) => assert_eq!(text, "Hello world"),
                    _ => panic!("Expected Text content"),
                }
                assert!(user_msg.parent_tool_use_id.is_none());
            }
            _ => panic!("Expected User message"),
        }
    }

    #[test]
    fn test_parse_message_user_text() {
        let json = json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": "Test message"
            }
        });
        let msg = parse_message(json).unwrap();
        assert!(matches!(msg, Message::User(_)));
    }

    #[test]
    fn test_parse_message_user_blocks() {
        let json = json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": [
                    {"type": "text", "text": "Hello"}
                ]
            }
        });
        let msg = parse_message(json).unwrap();
        match msg {
            Message::User(user_msg) => match &user_msg.message.content {
                ContentValue::Blocks(blocks) => assert_eq!(blocks.len(), 1),
                _ => panic!("Expected Blocks content"),
            },
            _ => panic!("Expected User message"),
        }
    }

    #[test]
    fn test_parse_message_assistant() {
        let json = json!({
            "type": "assistant",
            "message": {
                "model": "claude-sonnet-4-5",
                "content": [
                    {"type": "text", "text": "Response"}
                ]
            }
        });
        let msg = parse_message(json).unwrap();
        match msg {
            Message::Assistant(assistant_msg) => {
                assert_eq!(assistant_msg.message.model, "claude-sonnet-4-5");
                assert_eq!(assistant_msg.message.content.len(), 1);
            }
            _ => panic!("Expected Assistant message"),
        }
    }

    #[test]
    fn test_parse_message_system() {
        let json = json!({
            "type": "system",
            "subtype": "info",
            "data": {"key": "value"}
        });
        let msg = parse_message(json).unwrap();
        match msg {
            Message::System(system_msg) => {
                assert_eq!(system_msg.subtype, "info");
                assert_eq!(system_msg.data["key"], "value");
            }
            _ => panic!("Expected System message"),
        }
    }

    #[test]
    fn test_parse_message_result() {
        let json = json!({
            "type": "result",
            "subtype": "completion",
            "duration_ms": 1000,
            "duration_api_ms": 800,
            "is_error": false,
            "num_turns": 5,
            "session_id": "sess-123",
            "total_cost_usd": 0.05,
            "usage": {"input_tokens": 100, "output_tokens": 50},
            "result": "Success"
        });
        let msg = parse_message(json).unwrap();
        match msg {
            Message::Result(result_msg) => {
                assert_eq!(result_msg.subtype, "completion");
                assert_eq!(result_msg.duration_ms, 1000);
                assert_eq!(result_msg.duration_api_ms, 800);
                assert!(!result_msg.is_error);
                assert_eq!(result_msg.num_turns, 5);
                assert_eq!(result_msg.session_id, "sess-123");
                assert_eq!(result_msg.total_cost_usd, Some(0.05));
                assert_eq!(result_msg.result, Some("Success".to_string()));
            }
            _ => panic!("Expected Result message"),
        }
    }

    #[test]
    fn test_parse_message_stream_event() {
        let json = json!({
            "type": "stream_event",
            "uuid": "evt-123",
            "session_id": "sess-456",
            "event": {"type": "token", "data": "word"}
        });
        let msg = parse_message(json).unwrap();
        match msg {
            Message::StreamEvent(event_msg) => {
                assert_eq!(event_msg.uuid, "evt-123");
                assert_eq!(event_msg.session_id, "sess-456");
                assert_eq!(event_msg.event["type"], "token");
            }
            _ => panic!("Expected StreamEvent message"),
        }
    }

    #[test]
    fn test_parse_message_invalid() {
        let json = json!({
            "type": "unknown",
            "data": "test"
        });
        let result = parse_message(json);
        assert!(result.is_err());
        match result {
            Err(crate::error::Error::ParseError(msg)) => {
                assert!(msg.contains("unknown variant"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_content_block_text() {
        let json = json!({"type": "text", "text": "Hello"});
        let block: ContentBlock = serde_json::from_value(json).unwrap();
        match block {
            ContentBlock::Text { text } => assert_eq!(text, "Hello"),
            _ => panic!("Expected Text block"),
        }
    }

    #[test]
    fn test_content_block_thinking() {
        let json = json!({
            "type": "thinking",
            "thinking": "Analyzing...",
            "signature": "sig-123"
        });
        let block: ContentBlock = serde_json::from_value(json).unwrap();
        match block {
            ContentBlock::Thinking {
                thinking,
                signature,
            } => {
                assert_eq!(thinking, "Analyzing...");
                assert_eq!(signature, "sig-123");
            }
            _ => panic!("Expected Thinking block"),
        }
    }

    #[test]
    fn test_content_block_tool_use() {
        let json = json!({
            "type": "tool_use",
            "id": "tool-1",
            "name": "Read",
            "input": {"path": "file.txt"}
        });
        let block: ContentBlock = serde_json::from_value(json).unwrap();
        match block {
            ContentBlock::ToolUse { id, name, input } => {
                assert_eq!(id, "tool-1");
                assert_eq!(name, "Read");
                assert_eq!(input["path"], "file.txt");
            }
            _ => panic!("Expected ToolUse block"),
        }
    }

    #[test]
    fn test_content_block_tool_result_success() {
        let json = json!({
            "type": "tool_result",
            "tool_use_id": "tool-1",
            "content": {"data": "result"},
            "is_error": false
        });
        let block: ContentBlock = serde_json::from_value(json).unwrap();
        match block {
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => {
                assert_eq!(tool_use_id, "tool-1");
                assert_eq!(content.unwrap()["data"], "result");
                assert_eq!(is_error, Some(false));
            }
            _ => panic!("Expected ToolResult block"),
        }
    }

    #[test]
    fn test_content_block_tool_result_error() {
        let json = json!({
            "type": "tool_result",
            "tool_use_id": "tool-2",
            "is_error": true
        });
        let block: ContentBlock = serde_json::from_value(json).unwrap();
        match block {
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => {
                assert_eq!(tool_use_id, "tool-2");
                assert!(content.is_none());
                assert_eq!(is_error, Some(true));
            }
            _ => panic!("Expected ToolResult block"),
        }
    }

    #[test]
    fn test_user_message_with_parent_tool_use_id() {
        let json = json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": "Follow-up"
            },
            "parent_tool_use_id": "tool-123"
        });
        let msg = parse_message(json).unwrap();
        match msg {
            Message::User(user_msg) => {
                assert_eq!(user_msg.parent_tool_use_id, Some("tool-123".to_string()));
            }
            _ => panic!("Expected User message"),
        }
    }

    #[test]
    fn test_assistant_message_with_parent_tool_use_id() {
        let json = json!({
            "type": "assistant",
            "message": {
                "model": "claude-3-5-sonnet",
                "content": []
            },
            "parent_tool_use_id": "tool-456"
        });
        let msg = parse_message(json).unwrap();
        match msg {
            Message::Assistant(assistant_msg) => {
                assert_eq!(
                    assistant_msg.parent_tool_use_id,
                    Some("tool-456".to_string())
                );
            }
            _ => panic!("Expected Assistant message"),
        }
    }

    #[test]
    fn test_stream_event_with_parent_tool_use_id() {
        let json = json!({
            "type": "stream_event",
            "uuid": "evt-789",
            "session_id": "sess-789",
            "event": {},
            "parent_tool_use_id": "tool-789"
        });
        let msg = parse_message(json).unwrap();
        match msg {
            Message::StreamEvent(event_msg) => {
                assert_eq!(event_msg.parent_tool_use_id, Some("tool-789".to_string()));
            }
            _ => panic!("Expected StreamEvent message"),
        }
    }

    #[test]
    fn test_result_message_minimal() {
        let json = json!({
            "type": "result",
            "subtype": "error",
            "duration_ms": 500,
            "duration_api_ms": 400,
            "is_error": true,
            "num_turns": 1,
            "session_id": "sess-error"
        });
        let msg = parse_message(json).unwrap();
        match msg {
            Message::Result(result_msg) => {
                assert_eq!(result_msg.subtype, "error");
                assert!(result_msg.is_error);
                assert!(result_msg.total_cost_usd.is_none());
                assert!(result_msg.usage.is_none());
                assert!(result_msg.result.is_none());
            }
            _ => panic!("Expected Result message"),
        }
    }

    #[test]
    fn test_message_serialization_roundtrip() {
        let original = Message::user("Test roundtrip");
        let json = serde_json::to_value(&original).unwrap();
        let parsed = parse_message(json).unwrap();

        match (original, parsed) {
            (Message::User(orig), Message::User(parsed)) => {
                assert_eq!(orig.message.role, parsed.message.role);
            }
            _ => panic!("Serialization roundtrip failed"),
        }
    }
}

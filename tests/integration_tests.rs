use futures::StreamExt;
use periplon_sdk::adapters::secondary::MockTransport;
use periplon_sdk::application::Query;
use periplon_sdk::domain::Message;
use serde_json::json;

#[tokio::test]
async fn test_mock_transport_basic() {
    let messages = vec![
        json!({
            "type": "assistant",
            "message": {
                "model": "claude-sonnet-4-5",
                "content": [
                    {"type": "text", "text": "Hello!"}
                ]
            }
        }),
        json!({
            "type": "result",
            "subtype": "final",
            "duration_ms": 100,
            "duration_api_ms": 50,
            "is_error": false,
            "num_turns": 1,
            "session_id": "test",
            "total_cost_usd": 0.001
        }),
    ];

    let transport = Box::new(MockTransport::new(messages));
    let (mut query, write_rx) = Query::new(transport, false, None, None);
    query.start(write_rx).await.unwrap();

    let stream = query.receive_messages();
    futures::pin_mut!(stream);
    let mut message_count = 0;

    while let Some(msg) = stream.next().await {
        message_count += 1;
        match msg {
            Message::Assistant(_) => {
                assert_eq!(message_count, 1);
            }
            Message::Result(_) => {
                assert_eq!(message_count, 2);
            }
            _ => panic!("Unexpected message type"),
        }
    }

    assert_eq!(message_count, 2);
}

#[tokio::test]
async fn test_query_layer_message_routing() {
    let messages = vec![
        json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": "Test"
            }
        }),
        json!({
            "type": "assistant",
            "message": {
                "model": "claude-sonnet-4-5",
                "content": [
                    {"type": "text", "text": "Response"}
                ]
            }
        }),
    ];

    let transport = Box::new(MockTransport::new(messages));
    let (mut query, write_rx) = Query::new(transport, false, None, None);
    query.start(write_rx).await.unwrap();

    let stream = query.receive_messages();
    futures::pin_mut!(stream);
    let mut types = Vec::new();

    while let Some(msg) = stream.next().await {
        match msg {
            Message::User(_) => types.push("user"),
            Message::Assistant(_) => types.push("assistant"),
            _ => types.push("other"),
        }
    }

    assert_eq!(types, vec!["user", "assistant"]);
}

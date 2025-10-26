# Testing Guide

## Overview

The SDK includes comprehensive testing support with unit tests, integration tests, and mock adapters.

## Running Tests

### All Tests

```bash
# Run all tests
cargo test

# Run with output visible
cargo test -- --nocapture

# Run with all features
cargo test --all-features
```

### Specific Tests

```bash
# Run specific test
cargo test test_message_parsing

# Run tests matching pattern
cargo test message

# Run integration tests only
cargo test --test integration_tests

# Run doc tests
cargo test --doc
```

### Test Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## Test Categories

### 1. Unit Tests

Test individual components in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_parsing() {
        let json = r#"{
            "type": "user",
            "message": {
                "role": "user",
                "content": [{"type": "text", "text": "Hello"}]
            }
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, Message::User(_)));
    }

    #[test]
    fn test_permission_evaluation() {
        let request = PermissionRequest {
            tool_name: "Read".to_string(),
            mode: "default".to_string(),
        };

        let result = evaluate_permission(&request);
        assert!(result.is_ok());
    }
}
```

### 2. Integration Tests

Test component interactions:

```rust
// tests/integration_tests.rs

use periplon_sdk::{query, Message, ContentBlock};
use futures::StreamExt;

#[tokio::test]
async fn test_simple_query() {
    let mut stream = query("What is 2 + 2?", None).await.unwrap();

    let mut found_response = false;
    while let Some(msg) = stream.next().await {
        if let Message::Assistant(assistant_msg) = msg {
            found_response = true;
            assert!(!assistant_msg.message.content.is_empty());
        }
    }

    assert!(found_response, "Should receive assistant response");
}

#[tokio::test]
async fn test_interactive_client() {
    let mut client = PeriplonSDKClient::new(AgentOptions::default());
    client.connect(None).await.unwrap();

    client.query("Hello").await.unwrap();
    let mut stream = client.receive_response().unwrap();

    let mut received_message = false;
    while let Some(msg) = stream.next().await {
        if matches!(msg, Message::Assistant(_)) {
            received_message = true;
        }
    }

    assert!(received_message);
    client.disconnect().await.unwrap();
}
```

### 3. Mock Tests

Use mock adapters for deterministic testing:

```rust
use periplon_sdk::adapters::secondary::MockTransport;
use periplon_sdk::application::Query;
use serde_json::json;

#[tokio::test]
async fn test_with_mock_transport() {
    let messages = vec![
        json!({
            "type": "assistant",
            "message": {
                "model": "claude-sonnet-4-5",
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "Hello!"}
                ]
            }
        }),
        json!({
            "type": "result",
            "total_cost_usd": 0.01,
            "status": "success"
        }),
    ];

    let transport = Box::new(MockTransport::new(messages));
    let mut query = Query::new(transport, false, None, None);

    query.start().await.unwrap();

    let mut found_assistant = false;
    let mut found_result = false;

    while let Some(msg) = query.next().await {
        match msg.unwrap() {
            Message::Assistant(_) => found_assistant = true,
            Message::Result(_) => found_result = true,
            _ => {}
        }
    }

    assert!(found_assistant);
    assert!(found_result);
}
```

## MockTransport Usage

### Basic Mock

```rust
use periplon_sdk::adapters::secondary::MockTransport;

let responses = vec![
    json!({"type": "assistant", "message": {...}}),
    json!({"type": "result", "status": "success"}),
];

let transport = Box::new(MockTransport::new(responses));
```

### Complex Mock Scenarios

```rust
#[tokio::test]
async fn test_tool_use_flow() {
    let messages = vec![
        // Assistant requests tool
        json!({
            "type": "assistant",
            "message": {
                "model": "claude-sonnet-4-5",
                "role": "assistant",
                "content": [{
                    "type": "tool_use",
                    "id": "toolu_123",
                    "name": "Read",
                    "input": {"file_path": "test.txt"}
                }]
            }
        }),
        // Tool result
        json!({
            "type": "assistant",
            "message": {
                "role": "assistant",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": "toolu_123",
                    "content": "file contents"
                }]
            }
        }),
        // Final response
        json!({
            "type": "assistant",
            "message": {
                "role": "assistant",
                "content": [{
                    "type": "text",
                    "text": "The file contains..."
                }]
            }
        }),
    ];

    let transport = Box::new(MockTransport::new(messages));
    // Test tool use flow
}
```

## DSL Testing

### Workflow Validation Tests

```rust
use periplon_sdk::dsl::{parse_workflow, validate_workflow};

#[test]
fn test_valid_workflow() {
    let yaml = r#"
name: "Test Workflow"
version: "1.0.0"
agents:
  test_agent:
    description: "Test"
tasks:
  test_task:
    description: "Test task"
    agent: "test_agent"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    let result = validate_workflow(&workflow);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_agent_reference() {
    let yaml = r#"
name: "Test Workflow"
version: "1.0.0"
agents:
  test_agent:
    description: "Test"
tasks:
  test_task:
    description: "Test task"
    agent: "nonexistent_agent"  # Invalid reference
"#;

    let workflow = parse_workflow(yaml).unwrap();
    let result = validate_workflow(&workflow);
    assert!(result.is_err());
}
```

### Task Graph Tests

```rust
use periplon_sdk::dsl::task_graph::TaskGraph;

#[test]
fn test_dependency_resolution() {
    let mut graph = TaskGraph::new();

    graph.add_task("task1", vec![]);
    graph.add_task("task2", vec!["task1"]);
    graph.add_task("task3", vec!["task1", "task2"]);

    let order = graph.topological_sort().unwrap();

    assert_eq!(order[0], "task1");
    assert_eq!(order[1], "task2");
    assert_eq!(order[2], "task3");
}

#[test]
fn test_cycle_detection() {
    let mut graph = TaskGraph::new();

    graph.add_task("task1", vec!["task2"]);
    graph.add_task("task2", vec!["task1"]);

    let result = graph.topological_sort();
    assert!(result.is_err());
}
```

### Loop Tests

```rust
use periplon_sdk::dsl::loops::{LoopConfig, CollectionSource};

#[tokio::test]
async fn test_for_each_loop() {
    let config = LoopConfig {
        loop_type: LoopType::ForEach,
        collection: Some(CollectionSource::Inline {
            items: vec![
                json!(1),
                json!(2),
                json!(3),
            ],
        }),
        iterator: Some("item".to_string()),
        ..Default::default()
    };

    let executor = LoopExecutor::new(config);
    let results = executor.execute().await.unwrap();

    assert_eq!(results.len(), 3);
}
```

## Benchmarking

### Setup

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench dsl_parsing

# Generate report
cargo bench -- --save-baseline main
```

### Writing Benchmarks

```rust
// benches/dsl_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use periplon_sdk::dsl::parse_workflow;

fn benchmark_parsing(c: &mut Criterion) {
    let yaml = include_str!("../examples/dsl_workflows/complex.yaml");

    c.bench_function("parse_complex_workflow", |b| {
        b.iter(|| parse_workflow(black_box(yaml)))
    });
}

criterion_group!(benches, benchmark_parsing);
criterion_main!(benches);
```

## Test Utilities

### Custom Test Helpers

```rust
// tests/common/mod.rs

pub fn create_test_message(text: &str) -> Message {
    Message::User(UserMessage {
        message: UserMessageContent {
            role: "user".to_string(),
            content: vec![ContentBlock::Text {
                text: text.to_string(),
            }],
        },
    })
}

pub fn create_test_options() -> AgentOptions {
    AgentOptions {
        permission_mode: Some("acceptEdits".to_string()),
        max_turns: Some(5),
        ..Default::default()
    }
}
```

### Async Test Helpers

```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_with_timeout() {
    let result = timeout(
        Duration::from_secs(5),
        query("test", None)
    ).await;

    assert!(result.is_ok(), "Query timed out");
}
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt -- --check
```

## Best Practices

1. **Isolate Tests**: Use mocks to avoid external dependencies
2. **Test Edge Cases**: Cover error conditions and boundary cases
3. **Fast Tests**: Keep unit tests fast; use integration tests sparingly
4. **Clear Names**: Use descriptive test function names
5. **Arrange-Act-Assert**: Structure tests clearly
6. **Don't Test Implementation**: Test behavior, not internals
7. **Use Fixtures**: Share test data with helper functions
8. **Clean Up**: Ensure tests clean up resources

### Example: Well-Structured Test

```rust
#[tokio::test]
async fn test_query_handles_invalid_cli_response() {
    // Arrange
    let invalid_messages = vec![
        json!({"invalid": "structure"}),
    ];
    let transport = Box::new(MockTransport::new(invalid_messages));
    let mut query = Query::new(transport, false, None, None);

    // Act
    query.start().await.unwrap();
    let result = query.next().await;

    // Assert
    assert!(result.is_some());
    assert!(matches!(result.unwrap(), Err(Error::InvalidMessage(_))));
}
```

## Debugging Tests

### Enable Logging

```rust
// At the start of your test
use tracing_subscriber;

#[tokio::test]
async fn test_with_logging() {
    tracing_subscriber::fmt::init();

    // Your test code
}
```

### Print Test Output

```bash
# Show stdout/stderr
cargo test -- --nocapture

# Show test names
cargo test -- --show-output
```

### Run Single Test with Debugging

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name -- --nocapture

# Run with full backtrace
RUST_BACKTRACE=full cargo test test_name
```

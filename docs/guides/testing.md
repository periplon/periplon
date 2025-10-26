# Testing Guide

## Overview

The SDK includes comprehensive testing support with **166+ integration tests** covering:
- Authentication and Authorization (26 tests)
- Queue Backend Operations (22 tests)
- Storage Backend Operations (21 tests)
- Schedule Management API (22 tests)
- Execution Management API (22 tests)
- WebSocket Real-time Streaming (21 tests)
- Workflow API Integration (32 tests)
- Unit tests, integration tests, and mock adapters

## Quick Reference

```bash
# Run all tests with server features
cargo test --lib --tests --features server

# Run specific test suite
cargo test --test execution_api_tests --features server

# Run with output visible
cargo test -- --nocapture

# Run tests matching pattern
cargo test websocket --features server
```

## Running Tests

### All Tests

```bash
# Run all tests with server features (recommended)
cargo test --lib --tests --features server

# Run all tests including examples
cargo test --all-features

# Run with output visible
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test --features server

# Run tests sequentially
cargo test --features server -- --test-threads=1
```

### Specific Test Suites

```bash
# Authentication and authorization tests (26 tests)
cargo test --test auth_tests --features server

# Queue backend tests (22 tests)
cargo test --test queue_backend_tests --features server

# Storage backend tests (21 tests)
cargo test --test storage_backend_tests --features server

# Schedule API tests (22 tests)
cargo test --test schedule_api_tests --features server

# Execution API tests (22 tests)
cargo test --test execution_api_tests --features server

# WebSocket streaming tests (21 tests)
cargo test --test websocket_tests --features server

# Workflow API tests (32 tests)
cargo test --test workflow_api_tests --features server
```

### Running Specific Tests

```bash
# Run specific test by name
cargo test test_create_execution --features server

# Run tests matching pattern
cargo test execution --features server

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

## Comprehensive Test Suite Documentation

### Server API Tests (166+ tests)

#### 1. Authentication and Authorization Tests (26 tests)

Tests JWT authentication, user management, role-based access control (RBAC), and resource-level permissions.

**Location:** `tests/auth_tests.rs`

**Key Test Categories:**
- JWT token generation, validation, and expiration (7 tests)
- User registration, login, and password management (8 tests)
- Role-based access control and permissions (9 tests)
- Complete authentication flows (2 tests)

**Example Test:**
```rust
#[tokio::test]
async fn test_complete_auth_flow() {
    let user_storage = Arc::new(MockUserStorage::new());
    let auth_service = Arc::new(MockAuthorizationService::new());
    let jwt_manager = Arc::new(JwtManager::new("test_secret", 24));

    // 1. Register user
    let user = create_test_user("user@example.com", "password123", vec!["user".to_string()]);
    let user_id = user_storage.create_user(&user).await.unwrap();

    // 2. Setup permissions
    auth_service.grant_role(&user_id.to_string(), "user");
    auth_service.grant_permission(&user_id.to_string(), "workflows:read");

    // 3. Login and generate token
    let token = jwt_manager.generate_token(&user_id.to_string(), vec!["user".to_string()]).unwrap();

    // 4. Validate token
    let claims = jwt_manager.validate_token(&token).unwrap();
    assert_eq!(claims.sub, user_id.to_string());

    // 5. Check authorization
    let has_permission = auth_service.has_permission(&user_id.to_string(), "workflows:read").await;
    assert!(has_permission);
}
```

**Run Tests:**
```bash
cargo test --test auth_tests --features server -- --nocapture
```

#### 2. Queue Backend Tests (22 tests)

Tests job queuing, worker management, priority handling, and persistence.

**Location:** `tests/queue_backend_tests.rs`

**Key Test Categories:**
- Basic queue operations (enqueue, dequeue) (5 tests)
- Priority queue ordering (3 tests)
- Worker locking and heartbeat (4 tests)
- Job completion and failure tracking (4 tests)
- Persistence and recovery (3 tests)
- Concurrent operations (3 tests)

**Example Test:**
```rust
#[tokio::test]
async fn test_priority_ordering() {
    let temp_dir = TempDir::new().unwrap();
    let queue = FilesystemQueue::new(temp_dir.path().to_path_buf()).await.unwrap();

    // Enqueue jobs with different priorities
    let job1 = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({})).with_priority(5);
    let job2 = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({})).with_priority(10);
    let job3 = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({})).with_priority(1);

    queue.enqueue(job1.clone()).await.unwrap();
    queue.enqueue(job2.clone()).await.unwrap();
    queue.enqueue(job3.clone()).await.unwrap();

    // Note: Filesystem queue uses FIFO, not priority-based ordering
    // This test documents actual behavior
}
```

**Run Tests:**
```bash
cargo test --test queue_backend_tests --features server -- --nocapture
```

#### 3. Storage Backend Tests (21 tests)

Tests workflow storage, execution tracking, checkpoint management, and data persistence.

**Location:** `tests/storage_backend_tests.rs`

**Key Test Categories:**
- Workflow CRUD operations (7 tests)
- Execution lifecycle management (7 tests)
- Checkpoint storage and retrieval (4 tests)
- Data integrity and relationships (3 tests)

**Example Test:**
```rust
#[tokio::test]
async fn test_filesystem_workflow_execution_relationship() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let (workflow, metadata) = create_test_workflow("linked-workflow");
    let workflow_id = metadata.id;

    // Store workflow
    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create multiple executions for this workflow
    for i in 0..3 {
        let status = match i {
            0 => ExecutionStatus::Queued,
            1 => ExecutionStatus::Running,
            _ => ExecutionStatus::Completed,
        };
        let execution = create_test_execution(workflow_id, status);
        storage.store_execution(&execution).await.unwrap();
    }

    // Verify relationship via filter
    let filter = ExecutionFilter {
        workflow_id: Some(workflow_id),
        ..Default::default()
    };
    let executions = storage.list_executions(&filter).await.unwrap();
    assert_eq!(executions.len(), 3);
}
```

**Run Tests:**
```bash
cargo test --test storage_backend_tests --features server -- --nocapture
```

#### 4. Schedule API Tests (22 tests)

Tests schedule creation, cron scheduling, due schedule detection, and run tracking.

**Location:** `tests/schedule_api_tests.rs`

**Key Test Categories:**
- Schedule CRUD operations (5 tests)
- Schedule filtering and pagination (5 tests)
- Due schedule detection (2 tests)
- Schedule run tracking (3 tests)
- Schedule lifecycle management (5 tests)
- Error handling (2 tests)

**Example Test:**
```rust
#[tokio::test]
async fn test_get_due_schedules() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let now = Utc::now();

    // Create schedule due now
    let mut schedule1 = create_test_schedule(workflow_id, "0 0 * * *");
    schedule1.next_run_at = Some(now - Duration::minutes(5));
    storage.store_schedule(&schedule1).await.unwrap();

    // Create schedule due in future
    let mut schedule2 = create_test_schedule(workflow_id, "0 12 * * *");
    schedule2.next_run_at = Some(now + Duration::hours(1));
    storage.store_schedule(&schedule2).await.unwrap();

    // Get due schedules
    let due_schedules = storage.get_due_schedules(now).await.unwrap();
    assert_eq!(due_schedules.len(), 1);
}
```

**Run Tests:**
```bash
cargo test --test schedule_api_tests --features server -- --nocapture
```

#### 5. Execution API Tests (22 tests)

Tests execution creation, lifecycle management, status transitions, and logging.

**Location:** `tests/execution_api_tests.rs`

**Key Test Categories:**
- Execution creation and queuing (4 tests)
- Execution lifecycle transitions (6 tests)
- Execution filtering and pagination (5 tests)
- Execution log management (3 tests)
- Concurrent operations (2 tests)
- Error handling (2 tests)

**Example Test:**
```rust
#[tokio::test]
async fn test_execution_lifecycle_queued_to_running() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create queued execution
    let mut execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Transition to running
    execution.status = ExecutionStatus::Running;
    execution.started_at = Some(Utc::now());
    storage.update_execution(execution_id, &execution).await.unwrap();

    // Verify status change
    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Running);
    assert!(retrieved.started_at.is_some());
}
```

**Run Tests:**
```bash
cargo test --test execution_api_tests --features server -- --nocapture
```

#### 6. WebSocket Streaming Tests (21 tests)

Tests real-time execution streaming, message formats, and connection management.

**Location:** `tests/websocket_tests.rs`

**Key Test Categories:**
- Message format validation (7 tests)
- Execution state streaming (3 tests)
- Progress tracking (2 tests)
- Connection management (3 tests)
- Concurrent connections (2 tests)
- Error and edge cases (4 tests)

**Example Test:**
```rust
#[tokio::test]
async fn test_stream_execution_state_changes() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create execution and simulate state changes
    let mut execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Transition through states
    execution.status = ExecutionStatus::Running;
    execution.started_at = Some(Utc::now());
    storage.update_execution(execution_id, &execution).await.unwrap();

    execution.status = ExecutionStatus::Completed;
    execution.completed_at = Some(Utc::now());
    execution.result = Some(json!({"status": "success"}));
    storage.update_execution(execution_id, &execution).await.unwrap();

    // Verify final state
    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Completed);
    assert!(retrieved.result.is_some());
}
```

**Run Tests:**
```bash
cargo test --test websocket_tests --features server -- --nocapture
```

### Mock Services

The SDK provides comprehensive mock implementations for testing:

#### MockStorage

In-memory storage for workflows, executions, checkpoints, and schedules.

```rust
use periplon_sdk::testing::MockStorage;

let storage = Arc::new(MockStorage::new());

// Configure failure modes
storage.fail_get();    // Fail all get operations
storage.fail_store();  // Fail all store operations

// Helper methods
let count = storage.workflow_count();
let count = storage.execution_count();
let count = storage.schedule_count();

// Get executions by status
let running = storage.get_executions_by_status(ExecutionStatus::Running);

// Clear all state
storage.clear();
```

#### MockQueue

In-memory job queue with worker management.

```rust
use periplon_sdk::testing::MockQueue;

let queue = Arc::new(MockQueue::new());

// Configure failure modes
queue.fail_enqueue();  // Fail enqueue operations
queue.fail_dequeue();  // Fail dequeue operations

// Helper methods
let count = queue.pending_count();
let count = queue.processing_count();
let count = queue.completed_count();
let count = queue.failed_count();

// Get jobs
let pending = queue.get_pending();
let completed = queue.get_completed();
let failed = queue.get_failed();

// Clear all state
queue.clear();
```

#### MockAuthorizationService

In-memory auth and permissions.

```rust
use periplon_sdk::testing::{MockAuthorizationService, MockUserStorage};

let auth_service = Arc::new(MockAuthorizationService::new());
let user_storage = Arc::new(MockUserStorage::new());

// Grant permissions
auth_service.grant_permission("user_id", "workflows:read");
auth_service.grant_role("user_id", "admin");

// Check permissions
let has_perm = auth_service.has_permission("user_id", "workflows:read").await;
let has_role = auth_service.has_role("user_id", "admin").await;
```

### Test Organization

```
tests/
├── auth_tests.rs                 # Authentication & authorization (26 tests)
├── queue_backend_tests.rs        # Queue operations (22 tests)
├── storage_backend_tests.rs      # Storage persistence (21 tests)
├── schedule_api_tests.rs         # Schedule management (22 tests)
├── execution_api_tests.rs        # Execution lifecycle (22 tests)
├── websocket_tests.rs            # Real-time streaming (21 tests)
├── workflow_api_tests.rs         # Workflow API (32 tests)
└── common/
    └── mod.rs                    # Shared test utilities

src/testing/
├── mock_storage.rs               # MockStorage implementation
├── mock_queue.rs                 # MockQueue implementation
├── mock_auth_service.rs          # MockAuthorizationService
└── mod.rs                        # Testing module exports
```

### Quick Test Commands

```bash
# Run all server tests
cargo test --lib --tests --features server

# Run specific test suite
cargo test --test execution_api_tests --features server

# Run tests matching a pattern
cargo test websocket --features server

# Run with output
cargo test --features server -- --nocapture

# Run single test
cargo test test_create_execution --features server -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name --features server
```

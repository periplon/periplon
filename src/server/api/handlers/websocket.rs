// WebSocket handlers for real-time execution streaming

#[cfg(feature = "server")]
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
#[cfg(feature = "server")]
use futures::{sink::SinkExt, stream::StreamExt};
#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use std::sync::Arc;
#[cfg(feature = "server")]
use tokio::sync::broadcast;
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use crate::server::Storage;

/// WebSocket message types for execution streaming
#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExecutionStreamMessage {
    /// Execution started
    Started {
        execution_id: Uuid,
        workflow_id: Uuid,
        started_at: String,
    },
    /// Task execution update
    TaskUpdate {
        execution_id: Uuid,
        task_id: String,
        status: String,
        message: Option<String>,
    },
    /// Log message
    Log {
        execution_id: Uuid,
        timestamp: String,
        level: String,
        message: String,
    },
    /// Execution progress
    Progress {
        execution_id: Uuid,
        completed_tasks: usize,
        total_tasks: usize,
        percent: f64,
    },
    /// Execution completed
    Completed {
        execution_id: Uuid,
        status: String,
        completed_at: String,
        result: Option<serde_json::Value>,
    },
    /// Execution failed
    Failed {
        execution_id: Uuid,
        error: String,
        failed_at: String,
    },
    /// Keep-alive ping
    Ping { timestamp: String },
    /// Keep-alive pong
    Pong { timestamp: String },
}

/// WebSocket handler for streaming execution updates
#[cfg(feature = "server")]
pub async fn execution_stream(
    ws: WebSocketUpgrade,
    Path(execution_id): Path<String>,
    State(storage): State<Arc<dyn Storage>>,
) -> Response {
    let execution_id = match Uuid::parse_str(&execution_id) {
        Ok(id) => id,
        Err(_) => {
            // Return 400 for invalid UUID
            return (StatusCode::BAD_REQUEST, "Invalid execution ID").into_response();
        }
    };

    // Check if execution exists
    let execution_exists = storage
        .get_execution(execution_id)
        .await
        .unwrap_or(None)
        .is_some();

    if !execution_exists {
        return (StatusCode::NOT_FOUND, "Execution not found").into_response();
    }

    ws.on_upgrade(move |socket| handle_execution_stream(socket, execution_id, storage))
}

/// Handle WebSocket connection for execution streaming
#[cfg(feature = "server")]
async fn handle_execution_stream(socket: WebSocket, execution_id: Uuid, storage: Arc<dyn Storage>) {
    let (mut sender, mut receiver) = socket.split();

    // Create a broadcast channel for this execution
    let (tx, mut rx) = broadcast::channel::<ExecutionStreamMessage>(100);

    // Send initial execution state
    if let Ok(Some(execution)) = storage.get_execution(execution_id).await {
        let initial_msg = ExecutionStreamMessage::Started {
            execution_id,
            workflow_id: execution.workflow_id,
            started_at: execution
                .started_at
                .map(|t| t.to_rfc3339())
                .unwrap_or_else(|| "pending".to_string()),
        };

        if let Ok(json) = serde_json::to_string(&initial_msg) {
            let _ = sender.send(Message::Text(json)).await;
        }

        // Send current status if completed or failed
        if let Some(completed_at) = execution.completed_at {
            let status_msg = match execution.status {
                crate::server::ExecutionStatus::Completed => ExecutionStreamMessage::Completed {
                    execution_id,
                    status: "completed".to_string(),
                    completed_at: completed_at.to_rfc3339(),
                    result: execution.result,
                },
                crate::server::ExecutionStatus::Failed => ExecutionStreamMessage::Failed {
                    execution_id,
                    error: execution
                        .error
                        .unwrap_or_else(|| "Unknown error".to_string()),
                    failed_at: completed_at.to_rfc3339(),
                },
                _ => ExecutionStreamMessage::Progress {
                    execution_id,
                    completed_tasks: 0,
                    total_tasks: 0,
                    percent: 0.0,
                },
            };

            if let Ok(json) = serde_json::to_string(&status_msg) {
                let _ = sender.send(Message::Text(json)).await;
            }
        }
    }

    // Spawn a task to poll for execution updates
    let storage_clone = Arc::clone(&storage);
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        poll_execution_updates(execution_id, storage_clone, tx_clone).await;
    });

    // Handle both incoming WebSocket messages and broadcast messages
    loop {
        tokio::select! {
            // Forward broadcast messages to WebSocket
            msg = rx.recv() => {
                match msg {
                    Ok(msg) => {
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        // Channel closed
                        break;
                    }
                }
            }
            // Handle incoming WebSocket messages (pings, close)
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // Received pong, connection is alive
                    }
                    Some(Ok(Message::Close(_))) => {
                        break;
                    }
                    Some(Err(_)) | None => {
                        break;
                    }
                    _ => {
                        // Ignore other messages
                    }
                }
            }
        }
    }
}

/// Poll for execution updates and send them via broadcast channel
#[cfg(feature = "server")]
async fn poll_execution_updates(
    execution_id: Uuid,
    storage: Arc<dyn Storage>,
    tx: broadcast::Sender<ExecutionStreamMessage>,
) {
    let mut last_log_count = 0;
    let mut last_status = None;

    loop {
        // Sleep between polls
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Get execution state
        let execution = match storage.get_execution(execution_id).await {
            Ok(Some(exec)) => exec,
            _ => continue,
        };

        // Check if status changed
        if Some(&execution.status) != last_status.as_ref() {
            last_status = Some(execution.status.clone());

            match execution.status {
                crate::server::ExecutionStatus::Running => {
                    let msg = ExecutionStreamMessage::Progress {
                        execution_id,
                        completed_tasks: 0,
                        total_tasks: 0,
                        percent: 0.0,
                    };
                    let _ = tx.send(msg);
                }
                crate::server::ExecutionStatus::Completed => {
                    let msg = ExecutionStreamMessage::Completed {
                        execution_id,
                        status: "completed".to_string(),
                        completed_at: execution
                            .completed_at
                            .map(|t| t.to_rfc3339())
                            .unwrap_or_default(),
                        result: execution.result,
                    };
                    let _ = tx.send(msg);
                    break; // Stop polling when completed
                }
                crate::server::ExecutionStatus::Failed => {
                    let msg = ExecutionStreamMessage::Failed {
                        execution_id,
                        error: execution
                            .error
                            .unwrap_or_else(|| "Unknown error".to_string()),
                        failed_at: execution
                            .completed_at
                            .map(|t| t.to_rfc3339())
                            .unwrap_or_default(),
                    };
                    let _ = tx.send(msg);
                    break; // Stop polling when failed
                }
                crate::server::ExecutionStatus::Cancelled => {
                    let msg = ExecutionStreamMessage::Failed {
                        execution_id,
                        error: "Execution cancelled".to_string(),
                        failed_at: execution
                            .completed_at
                            .map(|t| t.to_rfc3339())
                            .unwrap_or_default(),
                    };
                    let _ = tx.send(msg);
                    break; // Stop polling when cancelled
                }
                _ => {}
            }
        }

        // Get new logs
        if let Ok(logs) = storage.get_execution_logs(execution_id, Some(100)).await {
            if logs.len() > last_log_count {
                // Send new logs
                for log in &logs[last_log_count..] {
                    let msg = ExecutionStreamMessage::Log {
                        execution_id,
                        timestamp: log.timestamp.to_rfc3339(),
                        level: log.level.clone(),
                        message: log.message.clone(),
                    };
                    let _ = tx.send(msg);
                }
                last_log_count = logs.len();
            }
        }

        // Send periodic ping to keep connection alive
        if tx.receiver_count() > 0 {
            let msg = ExecutionStreamMessage::Ping {
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            let _ = tx.send(msg);
        } else {
            // No more receivers, stop polling
            break;
        }
    }
}

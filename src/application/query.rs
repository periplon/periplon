use crate::domain::control::*;
use crate::domain::hook::{HookCallback, HookContext};
use crate::domain::message::{parse_message, Message};
use crate::domain::{HookEvent, HookMatcher, PermissionResult, ToolPermissionContext};
use crate::error::{Error, Result};
use crate::options::CanUseToolCallback;
use crate::ports::secondary::Transport;
use futures::{Stream, StreamExt};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

pub struct Query {
    transport: Arc<Mutex<Box<dyn Transport>>>,
    is_streaming_mode: bool,

    // Callbacks
    can_use_tool: Option<CanUseToolCallback>,
    hook_callbacks: Arc<Mutex<HashMap<String, HookCallback>>>,

    // Control protocol state
    pending_responses: Arc<Mutex<HashMap<String, oneshot::Sender<serde_json::Value>>>>,
    request_counter: Arc<Mutex<u64>>,

    // Message stream
    message_tx: Option<mpsc::UnboundedSender<Message>>,
    message_rx: Arc<Mutex<mpsc::UnboundedReceiver<Message>>>,

    // Write queue to avoid deadlock
    write_tx: mpsc::UnboundedSender<String>,

    // Task handles
    read_task: Option<tokio::task::JoinHandle<()>>,
}

impl Query {
    pub fn new(
        transport: Box<dyn Transport>,
        is_streaming_mode: bool,
        can_use_tool: Option<CanUseToolCallback>,
        hooks: Option<HashMap<HookEvent, Vec<HookMatcher>>>,
    ) -> (Self, mpsc::UnboundedReceiver<String>) {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (write_tx, write_rx) = mpsc::unbounded_channel();

        // Convert hooks to internal format with callback IDs
        let hook_callbacks = if let Some(hooks_map) = hooks {
            let mut callbacks = HashMap::new();
            let mut next_id = 0;

            for (_event, matchers) in hooks_map {
                for matcher in matchers {
                    for callback in matcher.hooks {
                        let id = format!("hook_{}", next_id);
                        next_id += 1;
                        callbacks.insert(id, callback);
                    }
                }
            }

            callbacks
        } else {
            HashMap::new()
        };

        let query = Self {
            transport: Arc::new(Mutex::new(transport)),
            is_streaming_mode,
            can_use_tool,
            hook_callbacks: Arc::new(Mutex::new(hook_callbacks)),
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            request_counter: Arc::new(Mutex::new(0)),
            message_tx: Some(message_tx),
            message_rx: Arc::new(Mutex::new(message_rx)),
            write_tx,
            read_task: None,
        };

        (query, write_rx)
    }

    /// Initialize control protocol
    pub async fn initialize(
        &mut self,
        hooks_config: Option<HashMap<String, Vec<HookMatcherConfig>>>,
    ) -> Result<serde_json::Value> {
        if !self.is_streaming_mode {
            return Ok(json!({}));
        }

        let request = ControlRequestBody::Initialize {
            hooks: hooks_config,
        };

        self.send_control_request(request).await
    }

    /// Start reading messages from transport
    pub async fn start(&mut self, mut write_rx: mpsc::UnboundedReceiver<String>) -> Result<()> {
        let transport = Arc::clone(&self.transport);
        let message_tx = self.message_tx.take().ok_or(Error::AlreadyStarted)?;
        let pending_responses = Arc::clone(&self.pending_responses);
        let can_use_tool = self.can_use_tool.clone();
        let hook_callbacks = Arc::clone(&self.hook_callbacks);
        let write_tx = self.write_tx.clone();

        // Extract stdin Arc before locking transport for reading
        // This allows write task to write independently
        let stdin_opt = {
            let mut transport_lock = transport.lock().await;
            // Try to downcast to SubprocessCLITransport to get stdin
            use crate::adapters::secondary::SubprocessCLITransport;
            let transport_ref: &mut dyn crate::ports::secondary::Transport = &mut **transport_lock;
            let transport_ptr = transport_ref as *mut dyn crate::ports::secondary::Transport;
            let subprocess_ptr = transport_ptr as *mut SubprocessCLITransport;
            unsafe { (*subprocess_ptr).get_stdin() }
        };

        // Spawn separate write task with direct stdin access
        if let Some(stdin_arc) = stdin_opt {
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                while let Some(msg) = write_rx.recv().await {
                    let mut stdin = stdin_arc.lock().await;
                    if let Err(e) = stdin.write_all(msg.as_bytes()).await {
                        eprintln!("Write error: {}", e);
                        break;
                    }
                    if let Err(e) = stdin.flush().await {
                        eprintln!("Flush error: {}", e);
                        break;
                    }
                }
            });
        }

        // Spawn read task
        let task = tokio::spawn(async move {
            let mut transport_lock = transport.lock().await;
            let mut stream = transport_lock.read_messages();

            while let Some(result) = stream.next().await {
                match result {
                    Ok(value) => {
                        // Route message based on type
                        if let Some(msg_type) = value.get("type").and_then(|v| v.as_str()) {
                            match msg_type {
                                "control_response" => {
                                    // Handle control response
                                    if let Ok(response) =
                                        serde_json::from_value::<ControlResponse>(value)
                                    {
                                        Self::handle_control_response(response, &pending_responses)
                                            .await;
                                    }
                                }

                                "control_request" => {
                                    // Handle incoming control request
                                    if let Ok(request) =
                                        serde_json::from_value::<IncomingControlRequest>(value)
                                    {
                                        tokio::spawn(Self::handle_incoming_control_request(
                                            request,
                                            write_tx.clone(),
                                            can_use_tool.clone(),
                                            Arc::clone(&hook_callbacks),
                                        ));
                                    }
                                }

                                _ => {
                                    // Regular SDK message
                                    if let Ok(message) = parse_message(value) {
                                        let _ = message_tx.send(message);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                }
            }
        });

        self.read_task = Some(task);
        Ok(())
    }

    /// Send control request to CLI
    async fn send_control_request(&self, request: ControlRequestBody) -> Result<serde_json::Value> {
        if !self.is_streaming_mode {
            return Err(Error::NotStreamingMode);
        }

        // Generate request ID
        let mut counter = self.request_counter.lock().await;
        *counter += 1;
        let request_id = format!("req_{}_{:x}", *counter, rand::random::<u32>());
        drop(counter);

        // Create response channel
        let (tx, rx) = oneshot::channel();
        self.pending_responses
            .lock()
            .await
            .insert(request_id.clone(), tx);

        // Build and send request
        let control_request = ControlRequest {
            msg_type: "control_request".to_string(),
            request_id: request_id.clone(),
            request,
        };

        let json_str = serde_json::to_string(&control_request)?;
        self.write_tx
            .send(format!("{}\n", json_str))
            .map_err(|_| Error::ChannelClosed)?;

        // Wait for response with timeout
        let response = tokio::time::timeout(tokio::time::Duration::from_secs(60), rx).await??;

        Ok(response)
    }

    /// Handle control response from CLI
    async fn handle_control_response(
        response: ControlResponse,
        pending_responses: &Arc<Mutex<HashMap<String, oneshot::Sender<serde_json::Value>>>>,
    ) {
        match response.response {
            ControlResponseBody::Success {
                request_id,
                response: data,
            } => {
                if let Some(tx) = pending_responses.lock().await.remove(&request_id) {
                    let _ = tx.send(data.unwrap_or(json!({})));
                }
            }
            ControlResponseBody::Error { request_id, error } => {
                pending_responses.lock().await.remove(&request_id);
                eprintln!("Control request error: {}", error);
            }
        }
    }

    /// Handle incoming control request from CLI
    async fn handle_incoming_control_request(
        request: IncomingControlRequest,
        write_tx: mpsc::UnboundedSender<String>,
        can_use_tool: Option<CanUseToolCallback>,
        hook_callbacks: Arc<Mutex<HashMap<String, HookCallback>>>,
    ) {
        let request_id = request.request_id.clone();

        let result = match request.request {
            IncomingControlRequestBody::CanUseTool {
                tool_name,
                input,
                permission_suggestions,
                ..
            } => {
                if let Some(callback) = can_use_tool {
                    let context = ToolPermissionContext {
                        signal: None,
                        suggestions: permission_suggestions.unwrap_or_default(),
                    };

                    match callback(tool_name, input.clone(), context).await {
                        PermissionResult::Allow {
                            updated_input,
                            updated_permissions,
                        } => Ok(json!({
                            "behavior": "allow",
                            "updatedInput": updated_input.unwrap_or(input),
                            "updatedPermissions": updated_permissions,
                        })),
                        PermissionResult::Deny { message, interrupt } => Ok(json!({
                            "behavior": "deny",
                            "message": message,
                            "interrupt": interrupt,
                        })),
                    }
                } else {
                    Err("canUseTool callback not provided".to_string())
                }
            }

            IncomingControlRequestBody::HookCallback {
                callback_id,
                input,
                tool_use_id,
            } => {
                if let Some(callback) = hook_callbacks.lock().await.get(&callback_id) {
                    let context = HookContext { signal: None };
                    let output = callback(input, tool_use_id, context).await;
                    serde_json::to_value(output).map_err(|e| e.to_string())
                } else {
                    Err(format!("Hook callback not found: {}", callback_id))
                }
            }

            IncomingControlRequestBody::McpMessage { .. } => Err("MCP not implemented".to_string()),
        };

        // Send response
        let response = match result {
            Ok(data) => ControlResponse {
                msg_type: "control_response".to_string(),
                response: ControlResponseBody::Success {
                    request_id,
                    response: Some(data),
                },
            },
            Err(error) => ControlResponse {
                msg_type: "control_response".to_string(),
                response: ControlResponseBody::Error { request_id, error },
            },
        };

        let json_str = serde_json::to_string(&response).unwrap();
        let _ = write_tx.send(format!("{}\n", json_str));
    }

    /// Receive messages from the stream
    pub fn receive_messages(&self) -> impl Stream<Item = Message> + '_ {
        async_stream::stream! {
            let mut rx = self.message_rx.lock().await;
            while let Some(msg) = rx.recv().await {
                yield msg;
            }
        }
    }

    /// Send interrupt request
    pub async fn interrupt(&self) -> Result<()> {
        self.send_control_request(ControlRequestBody::Interrupt)
            .await?;
        Ok(())
    }

    /// Set permission mode
    pub async fn set_permission_mode(&self, mode: String) -> Result<()> {
        self.send_control_request(ControlRequestBody::SetPermissionMode { mode })
            .await?;
        Ok(())
    }

    /// Set model
    pub async fn set_model(&self, model: Option<String>) -> Result<()> {
        self.send_control_request(ControlRequestBody::SetModel { model })
            .await?;
        Ok(())
    }

    /// Close query and transport
    pub async fn close(&mut self) -> Result<()> {
        if let Some(task) = self.read_task.take() {
            task.abort();
        }
        self.transport.lock().await.close().await
    }

    /// Get transport reference
    pub fn transport(&self) -> Arc<Mutex<Box<dyn Transport>>> {
        Arc::clone(&self.transport)
    }

    /// Write a message to the transport
    pub fn write(&self, data: impl Into<String>) -> Result<()> {
        self.write_tx
            .send(data.into())
            .map_err(|_| Error::ChannelClosed)
    }
}

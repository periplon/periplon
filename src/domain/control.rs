use super::hook::HookInput;
use super::permission::PermissionUpdate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Control request (SDK → CLI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlRequest {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub request_id: String,
    pub request: ControlRequestBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "subtype")]
pub enum ControlRequestBody {
    #[serde(rename = "initialize")]
    Initialize {
        #[serde(skip_serializing_if = "Option::is_none")]
        hooks: Option<HashMap<String, Vec<HookMatcherConfig>>>,
    },

    #[serde(rename = "interrupt")]
    Interrupt,

    #[serde(rename = "set_permission_mode")]
    SetPermissionMode { mode: String },

    #[serde(rename = "set_model")]
    SetModel {
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMatcherConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,
    #[serde(rename = "hookCallbackIds")]
    pub hook_callback_ids: Vec<String>,
}

/// Control response (CLI → SDK)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResponse {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub response: ControlResponseBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "subtype")]
pub enum ControlResponseBody {
    #[serde(rename = "success")]
    Success {
        request_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        response: Option<serde_json::Value>,
    },

    #[serde(rename = "error")]
    Error { request_id: String, error: String },
}

/// Incoming control request (CLI → SDK)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingControlRequest {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub request_id: String,
    pub request: IncomingControlRequestBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "subtype")]
pub enum IncomingControlRequestBody {
    #[serde(rename = "can_use_tool")]
    CanUseTool {
        tool_name: String,
        input: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        permission_suggestions: Option<Vec<PermissionUpdate>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        blocked_path: Option<String>,
    },

    #[serde(rename = "hook_callback")]
    HookCallback {
        callback_id: String,
        input: HookInput,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_use_id: Option<String>,
    },

    #[serde(rename = "mcp_message")]
    McpMessage {
        server_name: String,
        message: serde_json::Value,
    },
}

#[derive(Debug, Clone)]
pub struct ControlConfig {
    pub hooks: Option<HashMap<String, Vec<HookMatcherConfig>>>,
}

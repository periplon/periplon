use serde::{Deserialize, Serialize};

pub type HookEvent = String;

#[derive(Clone)]
pub struct HookMatcher {
    pub matcher: Option<String>,
    pub hooks: Vec<HookCallback>,
}

pub struct HookContext {
    pub signal: Option<()>,
}

/// Hook input types (discriminated union)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hook_event_name")]
pub enum HookInput {
    PreToolUse {
        session_id: String,
        transcript_path: String,
        cwd: String,
        permission_mode: Option<String>,
        tool_name: String,
        tool_input: serde_json::Value,
    },
    PostToolUse {
        session_id: String,
        transcript_path: String,
        cwd: String,
        permission_mode: Option<String>,
        tool_name: String,
        tool_input: serde_json::Value,
        tool_response: serde_json::Value,
    },
    UserPromptSubmit {
        session_id: String,
        transcript_path: String,
        cwd: String,
        permission_mode: Option<String>,
        prompt: String,
    },
    Stop {
        session_id: String,
        transcript_path: String,
        cwd: String,
        permission_mode: Option<String>,
        stop_hook_active: bool,
    },
    SubagentStop {
        session_id: String,
        transcript_path: String,
        cwd: String,
        permission_mode: Option<String>,
        stop_hook_active: bool,
    },
    PreCompact {
        session_id: String,
        transcript_path: String,
        cwd: String,
        permission_mode: Option<String>,
        trigger: String,
        custom_instructions: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HookJSONOutput {
    Async {
        #[serde(rename = "async")]
        is_async: bool,
        #[serde(rename = "asyncTimeout", skip_serializing_if = "Option::is_none")]
        async_timeout: Option<u32>,
    },
    Sync {
        #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
        should_continue: Option<bool>,
        #[serde(rename = "suppressOutput", skip_serializing_if = "Option::is_none")]
        suppress_output: Option<bool>,
        #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
        stop_reason: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        decision: Option<String>,
        #[serde(rename = "systemMessage", skip_serializing_if = "Option::is_none")]
        system_message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
        #[serde(rename = "hookSpecificOutput", skip_serializing_if = "Option::is_none")]
        hook_specific_output: Option<HookSpecificOutput>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hookEventName")]
pub enum HookSpecificOutput {
    PreToolUse {
        #[serde(rename = "permissionDecision", skip_serializing_if = "Option::is_none")]
        permission_decision: Option<String>,
        #[serde(
            rename = "permissionDecisionReason",
            skip_serializing_if = "Option::is_none"
        )]
        permission_decision_reason: Option<String>,
        #[serde(rename = "updatedInput", skip_serializing_if = "Option::is_none")]
        updated_input: Option<serde_json::Value>,
    },
    PostToolUse {
        #[serde(rename = "additionalContext", skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
    UserPromptSubmit {
        #[serde(rename = "additionalContext", skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
}

pub type HookCallback = std::sync::Arc<
    dyn Fn(
            HookInput,
            Option<String>,
            HookContext,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = HookJSONOutput> + Send>>
        + Send
        + Sync,
>;

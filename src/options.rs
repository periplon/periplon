use crate::domain::{
    HookEvent, HookMatcher, PermissionMode, PermissionResult, Provider, ToolPermissionContext,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

pub type SettingSource = String;

#[derive(Clone, Default)]
pub struct AgentOptions {
    pub provider: Option<Provider>,
    pub allowed_tools: Vec<String>,
    pub disallowed_tools: Vec<String>,
    pub system_prompt: Option<SystemPromptConfig>,
    pub permission_mode: Option<PermissionMode>,
    pub max_turns: Option<u32>,
    pub model: Option<String>,
    pub cwd: Option<PathBuf>,
    pub create_cwd: bool,
    pub cli_path: Option<PathBuf>,

    // MCP Servers
    pub mcp_servers: HashMap<String, McpServerConfig>,

    // Advanced options
    pub continue_conversation: bool,
    pub resume: Option<String>,
    pub permission_prompt_tool_name: Option<String>,
    pub add_dirs: Vec<PathBuf>,
    pub env: HashMap<String, String>,
    pub extra_args: HashMap<String, Option<String>>,
    pub max_buffer_size: Option<usize>,

    // Callbacks
    pub can_use_tool: Option<CanUseToolCallback>,
    pub hooks: Option<HashMap<HookEvent, Vec<HookMatcher>>>,
    pub stderr: Option<StderrCallback>,

    // Session options
    pub include_partial_messages: bool,
    pub fork_session: bool,
    pub agents: HashMap<String, AgentDefinition>,
    pub setting_sources: Option<Vec<SettingSource>>,
}

#[derive(Debug, Clone)]
pub enum SystemPromptConfig {
    Text(String),
    Preset {
        preset: String,
        append: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub description: String,
    pub prompt: String,
    pub tools: Option<Vec<String>>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpServerConfig {
    #[serde(rename = "stdio")]
    Stdio {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        env: Option<HashMap<String, String>>,
    },

    #[serde(rename = "sse")]
    Sse {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },

    #[serde(rename = "http")]
    Http {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },

    #[serde(rename = "sdk")]
    Sdk { name: String },
}

pub type CanUseToolCallback = std::sync::Arc<
    dyn Fn(
            String,
            serde_json::Value,
            ToolPermissionContext,
        ) -> Pin<Box<dyn Future<Output = PermissionResult> + Send>>
        + Send
        + Sync,
>;

pub type StderrCallback = std::sync::Arc<dyn Fn(String) + Send + Sync>;

// Manual Debug implementation for AgentOptions since callbacks don't implement Debug
impl std::fmt::Debug for AgentOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentOptions")
            .field("provider", &self.provider)
            .field("allowed_tools", &self.allowed_tools)
            .field("disallowed_tools", &self.disallowed_tools)
            .field("system_prompt", &self.system_prompt)
            .field("permission_mode", &self.permission_mode)
            .field("max_turns", &self.max_turns)
            .field("model", &self.model)
            .field("cwd", &self.cwd)
            .field("create_cwd", &self.create_cwd)
            .field("cli_path", &self.cli_path)
            .field("mcp_servers", &self.mcp_servers)
            .field("continue_conversation", &self.continue_conversation)
            .field("resume", &self.resume)
            .field(
                "permission_prompt_tool_name",
                &self.permission_prompt_tool_name,
            )
            .field("add_dirs", &self.add_dirs)
            .field("env", &self.env)
            .field("extra_args", &self.extra_args)
            .field("max_buffer_size", &self.max_buffer_size)
            .field(
                "can_use_tool",
                &self.can_use_tool.as_ref().map(|_| "<callback>"),
            )
            .field(
                "hooks",
                &self.hooks.as_ref().map(|h| format!("{} hooks", h.len())),
            )
            .field("stderr", &self.stderr.as_ref().map(|_| "<callback>"))
            .field("include_partial_messages", &self.include_partial_messages)
            .field("fork_session", &self.fork_session)
            .field("agents", &self.agents)
            .field("setting_sources", &self.setting_sources)
            .finish()
    }
}

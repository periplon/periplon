use crate::domain::{
    HookEvent, HookMatcher, PermissionMode, PermissionResult, ToolPermissionContext,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

pub type SettingSource = String;

#[derive(Clone, Default)]
pub struct AgentOptions {
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

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_options_default() {
        let options = AgentOptions::default();

        assert!(options.allowed_tools.is_empty());
        assert!(options.disallowed_tools.is_empty());
        assert!(options.system_prompt.is_none());
        assert!(options.permission_mode.is_none());
        assert!(options.max_turns.is_none());
        assert!(options.model.is_none());
        assert!(options.cwd.is_none());
        assert!(!options.create_cwd);
        assert!(options.cli_path.is_none());
        assert!(options.mcp_servers.is_empty());
        assert!(!options.continue_conversation);
        assert!(options.resume.is_none());
        assert!(options.permission_prompt_tool_name.is_none());
        assert!(options.add_dirs.is_empty());
        assert!(options.env.is_empty());
        assert!(options.extra_args.is_empty());
        assert!(options.max_buffer_size.is_none());
        assert!(options.can_use_tool.is_none());
        assert!(options.hooks.is_none());
        assert!(options.stderr.is_none());
        assert!(!options.include_partial_messages);
        assert!(!options.fork_session);
        assert!(options.agents.is_empty());
        assert!(options.setting_sources.is_none());
    }

    #[test]
    fn test_agent_options_with_tools() {
        let options = AgentOptions {
            allowed_tools: vec!["Read".to_string(), "Write".to_string()],
            disallowed_tools: vec!["Bash".to_string()],
            ..Default::default()
        };

        assert_eq!(options.allowed_tools.len(), 2);
        assert_eq!(options.disallowed_tools.len(), 1);
        assert!(options.allowed_tools.contains(&"Read".to_string()));
        assert!(options.disallowed_tools.contains(&"Bash".to_string()));
    }

    #[test]
    fn test_agent_options_with_permission_mode() {
        let mut options = AgentOptions::default();
        options.permission_mode = Some("acceptEdits".to_string());

        assert_eq!(options.permission_mode.as_ref().unwrap(), "acceptEdits");
    }

    #[test]
    fn test_agent_options_with_system_prompt_text() {
        let mut options = AgentOptions::default();
        options.system_prompt = Some(SystemPromptConfig::Text("Test prompt".to_string()));

        if let Some(SystemPromptConfig::Text(text)) = &options.system_prompt {
            assert_eq!(text, "Test prompt");
        } else {
            panic!("Expected Text variant");
        }
    }

    #[test]
    fn test_agent_options_with_system_prompt_preset() {
        let mut options = AgentOptions::default();
        options.system_prompt = Some(SystemPromptConfig::Preset {
            preset: "default".to_string(),
            append: Some("Extra instructions".to_string()),
        });

        if let Some(SystemPromptConfig::Preset { preset, append }) = &options.system_prompt {
            assert_eq!(preset, "default");
            assert_eq!(append.as_ref().unwrap(), "Extra instructions");
        } else {
            panic!("Expected Preset variant");
        }
    }

    #[test]
    fn test_agent_options_with_max_turns() {
        let mut options = AgentOptions::default();
        options.max_turns = Some(10);

        assert_eq!(options.max_turns, Some(10));
    }

    #[test]
    fn test_agent_options_with_model() {
        let mut options = AgentOptions::default();
        options.model = Some("claude-sonnet-4-5".to_string());

        assert_eq!(options.model.as_ref().unwrap(), "claude-sonnet-4-5");
    }

    #[test]
    fn test_agent_options_with_cwd() {
        let mut options = AgentOptions::default();
        options.cwd = Some(PathBuf::from("/tmp/test"));
        options.create_cwd = true;

        assert_eq!(options.cwd.as_ref().unwrap(), &PathBuf::from("/tmp/test"));
        assert!(options.create_cwd);
    }

    #[test]
    fn test_agent_options_with_mcp_servers_stdio() {
        let mut options = AgentOptions::default();
        options.mcp_servers.insert(
            "test-server".to_string(),
            McpServerConfig::Stdio {
                command: "node".to_string(),
                args: Some(vec!["server.js".to_string()]),
                env: Some(HashMap::from([(
                    "API_KEY".to_string(),
                    "secret".to_string(),
                )])),
            },
        );

        assert_eq!(options.mcp_servers.len(), 1);
        if let Some(McpServerConfig::Stdio { command, args, env }) =
            options.mcp_servers.get("test-server")
        {
            assert_eq!(command, "node");
            assert_eq!(args.as_ref().unwrap()[0], "server.js");
            assert_eq!(env.as_ref().unwrap().get("API_KEY").unwrap(), "secret");
        } else {
            panic!("Expected Stdio config");
        }
    }

    #[test]
    fn test_agent_options_with_mcp_servers_sse() {
        let mut options = AgentOptions::default();
        options.mcp_servers.insert(
            "sse-server".to_string(),
            McpServerConfig::Sse {
                url: "http://localhost:3000/sse".to_string(),
                headers: Some(HashMap::from([(
                    "Authorization".to_string(),
                    "Bearer token".to_string(),
                )])),
            },
        );

        if let Some(McpServerConfig::Sse { url, headers }) = options.mcp_servers.get("sse-server") {
            assert_eq!(url, "http://localhost:3000/sse");
            assert_eq!(
                headers.as_ref().unwrap().get("Authorization").unwrap(),
                "Bearer token"
            );
        } else {
            panic!("Expected SSE config");
        }
    }

    #[test]
    fn test_agent_options_with_mcp_servers_http() {
        let mut options = AgentOptions::default();
        options.mcp_servers.insert(
            "http-server".to_string(),
            McpServerConfig::Http {
                url: "http://localhost:4000".to_string(),
                headers: None,
            },
        );

        if let Some(McpServerConfig::Http { url, headers }) = options.mcp_servers.get("http-server")
        {
            assert_eq!(url, "http://localhost:4000");
            assert!(headers.is_none());
        } else {
            panic!("Expected HTTP config");
        }
    }

    #[test]
    fn test_agent_options_with_mcp_servers_sdk() {
        let mut options = AgentOptions::default();
        options.mcp_servers.insert(
            "sdk-server".to_string(),
            McpServerConfig::Sdk {
                name: "my-mcp-sdk".to_string(),
            },
        );

        if let Some(McpServerConfig::Sdk { name }) = options.mcp_servers.get("sdk-server") {
            assert_eq!(name, "my-mcp-sdk");
        } else {
            panic!("Expected SDK config");
        }
    }

    #[test]
    fn test_agent_options_with_env() {
        let mut options = AgentOptions::default();
        options
            .env
            .insert("PATH".to_string(), "/usr/bin".to_string());
        options
            .env
            .insert("HOME".to_string(), "/home/user".to_string());

        assert_eq!(options.env.len(), 2);
        assert_eq!(options.env.get("PATH").unwrap(), "/usr/bin");
        assert_eq!(options.env.get("HOME").unwrap(), "/home/user");
    }

    #[test]
    fn test_agent_options_with_extra_args() {
        let mut options = AgentOptions::default();
        options.extra_args.insert("--verbose".to_string(), None);
        options
            .extra_args
            .insert("--output".to_string(), Some("file.txt".to_string()));

        assert_eq!(options.extra_args.len(), 2);
        assert!(options.extra_args.get("--verbose").unwrap().is_none());
        assert_eq!(
            options
                .extra_args
                .get("--output")
                .unwrap()
                .as_ref()
                .unwrap(),
            "file.txt"
        );
    }

    #[test]
    fn test_agent_options_with_agent_definitions() {
        let mut options = AgentOptions::default();
        options.agents.insert(
            "researcher".to_string(),
            AgentDefinition {
                description: "Research agent".to_string(),
                prompt: "You are a researcher".to_string(),
                tools: Some(vec!["WebSearch".to_string()]),
                model: Some("claude-opus".to_string()),
            },
        );

        assert_eq!(options.agents.len(), 1);
        let agent = options.agents.get("researcher").unwrap();
        assert_eq!(agent.description, "Research agent");
        assert_eq!(agent.prompt, "You are a researcher");
        assert_eq!(agent.tools.as_ref().unwrap()[0], "WebSearch");
        assert_eq!(agent.model.as_ref().unwrap(), "claude-opus");
    }

    #[test]
    fn test_agent_options_clone() {
        let mut options1 = AgentOptions::default();
        options1.allowed_tools = vec!["Read".to_string()];
        options1.max_turns = Some(5);

        let options2 = options1.clone();

        assert_eq!(options2.allowed_tools, vec!["Read".to_string()]);
        assert_eq!(options2.max_turns, Some(5));
    }

    #[test]
    fn test_agent_options_debug() {
        let options = AgentOptions {
            allowed_tools: vec!["Read".to_string()],
            permission_mode: Some("default".to_string()),
            max_turns: Some(10),
            ..Default::default()
        };

        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("allowed_tools"));
        assert!(debug_str.contains("Read"));
        assert!(debug_str.contains("max_turns"));
    }

    #[test]
    fn test_system_prompt_config_debug() {
        let text_config = SystemPromptConfig::Text("Test".to_string());
        let debug_str = format!("{:?}", text_config);
        assert!(debug_str.contains("Text"));
        assert!(debug_str.contains("Test"));

        let preset_config = SystemPromptConfig::Preset {
            preset: "default".to_string(),
            append: Some("Extra".to_string()),
        };
        let debug_str = format!("{:?}", preset_config);
        assert!(debug_str.contains("Preset"));
        assert!(debug_str.contains("default"));
        assert!(debug_str.contains("Extra"));
    }

    #[test]
    fn test_agent_definition_serde() {
        let agent = AgentDefinition {
            description: "Test agent".to_string(),
            prompt: "Test prompt".to_string(),
            tools: Some(vec!["Read".to_string(), "Write".to_string()]),
            model: Some("claude-sonnet".to_string()),
        };

        // Test serialization
        let json = serde_json::to_string(&agent).unwrap();
        assert!(json.contains("Test agent"));
        assert!(json.contains("Test prompt"));

        // Test deserialization
        let deserialized: AgentDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.description, agent.description);
        assert_eq!(deserialized.prompt, agent.prompt);
        assert_eq!(deserialized.tools, agent.tools);
        assert_eq!(deserialized.model, agent.model);
    }

    #[test]
    fn test_mcp_server_config_serde_stdio() {
        let config = McpServerConfig::Stdio {
            command: "node".to_string(),
            args: Some(vec!["server.js".to_string()]),
            env: Some(HashMap::from([("KEY".to_string(), "value".to_string())])),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("stdio"));
        assert!(json.contains("node"));

        let deserialized: McpServerConfig = serde_json::from_str(&json).unwrap();
        if let McpServerConfig::Stdio { command, .. } = deserialized {
            assert_eq!(command, "node");
        } else {
            panic!("Expected Stdio variant");
        }
    }

    #[test]
    fn test_mcp_server_config_serde_sse() {
        let config = McpServerConfig::Sse {
            url: "http://localhost:3000".to_string(),
            headers: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: McpServerConfig = serde_json::from_str(&json).unwrap();

        if let McpServerConfig::Sse { url, .. } = deserialized {
            assert_eq!(url, "http://localhost:3000");
        } else {
            panic!("Expected SSE variant");
        }
    }

    #[test]
    fn test_mcp_server_config_serde_http() {
        let config = McpServerConfig::Http {
            url: "https://api.example.com".to_string(),
            headers: Some(HashMap::from([(
                "Authorization".to_string(),
                "Bearer token".to_string(),
            )])),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: McpServerConfig = serde_json::from_str(&json).unwrap();

        if let McpServerConfig::Http { url, headers } = deserialized {
            assert_eq!(url, "https://api.example.com");
            assert!(headers.is_some());
        } else {
            panic!("Expected HTTP variant");
        }
    }

    #[test]
    fn test_mcp_server_config_serde_sdk() {
        let config = McpServerConfig::Sdk {
            name: "my-sdk".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: McpServerConfig = serde_json::from_str(&json).unwrap();

        if let McpServerConfig::Sdk { name } = deserialized {
            assert_eq!(name, "my-sdk");
        } else {
            panic!("Expected SDK variant");
        }
    }

    #[test]
    fn test_agent_options_with_callbacks() {
        use std::sync::Arc;

        let mut options = AgentOptions::default();

        // Add a permission callback
        options.can_use_tool = Some(Arc::new(|_tool, _input, _ctx| {
            Box::pin(async move {
                PermissionResult::Allow {
                    updated_input: None,
                    updated_permissions: None,
                }
            })
        }));

        // Add a stderr callback
        options.stderr = Some(Arc::new(|msg| {
            eprintln!("STDERR: {}", msg);
        }));

        assert!(options.can_use_tool.is_some());
        assert!(options.stderr.is_some());

        // Debug should show placeholders for callbacks
        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("<callback>"));
    }

    #[test]
    fn test_agent_options_with_hooks() {
        use crate::domain::HookJSONOutput;
        use std::sync::Arc;

        let mut options = AgentOptions::default();
        let mut hooks = HashMap::new();

        // Create a hook callback
        let hook_fn: crate::domain::HookCallback = Arc::new(|_input, _session, _ctx| {
            Box::pin(async move {
                HookJSONOutput::Sync {
                    should_continue: Some(true),
                    suppress_output: None,
                    stop_reason: None,
                    decision: None,
                    system_message: None,
                    reason: None,
                    hook_specific_output: None,
                }
            })
        });

        // Create a simple hook matcher
        let matcher = HookMatcher {
            matcher: Some("*".to_string()),
            hooks: vec![hook_fn],
        };

        hooks.insert("pre_tool_use".to_string(), vec![matcher]);
        options.hooks = Some(hooks);

        assert!(options.hooks.is_some());
        assert_eq!(options.hooks.as_ref().unwrap().len(), 1);

        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("1 hooks"));
    }
}

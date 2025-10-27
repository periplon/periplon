use periplon_sdk::options::{AgentDefinition, AgentOptions, McpServerConfig, SystemPromptConfig};
use std::collections::HashMap;
use std::path::PathBuf;

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
    let mut options = AgentOptions::default();
    options.allowed_tools = vec!["Read".to_string(), "Write".to_string()];
    options.disallowed_tools = vec!["Bash".to_string()];

    assert_eq!(options.allowed_tools.len(), 2);
    assert_eq!(options.disallowed_tools.len(), 1);
    assert!(options.allowed_tools.contains(&"Read".to_string()));
    assert!(options.disallowed_tools.contains(&"Bash".to_string()));
}

#[test]
fn test_agent_options_with_system_prompt_text() {
    let mut options = AgentOptions::default();
    options.system_prompt = Some(SystemPromptConfig::Text("Custom prompt".to_string()));

    assert!(options.system_prompt.is_some());
    match &options.system_prompt {
        Some(SystemPromptConfig::Text(text)) => {
            assert_eq!(text, "Custom prompt");
        }
        _ => panic!("Expected text system prompt"),
    }
}

#[test]
fn test_agent_options_with_system_prompt_preset() {
    let mut options = AgentOptions::default();
    options.system_prompt = Some(SystemPromptConfig::Preset {
        preset: "code-review".to_string(),
        append: Some("Focus on security".to_string()),
    });

    assert!(options.system_prompt.is_some());
    match &options.system_prompt {
        Some(SystemPromptConfig::Preset { preset, append }) => {
            assert_eq!(preset, "code-review");
            assert_eq!(append.as_ref().unwrap(), "Focus on security");
        }
        _ => panic!("Expected preset system prompt"),
    }
}

#[test]
fn test_agent_options_with_permission_modes() {
    let test_cases = vec!["default", "acceptEdits", "plan", "bypassPermissions"];

    for mode in test_cases {
        let mut options = AgentOptions::default();
        options.permission_mode = Some(mode.to_string());
        assert_eq!(options.permission_mode.as_ref().unwrap(), mode);
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
    let mut env = HashMap::new();
    env.insert("API_KEY".to_string(), "secret".to_string());

    options.mcp_servers.insert(
        "test-server".to_string(),
        McpServerConfig::Stdio {
            command: "node".to_string(),
            args: Some(vec!["server.js".to_string()]),
            env: Some(env),
        },
    );

    assert_eq!(options.mcp_servers.len(), 1);
    match options.mcp_servers.get("test-server").unwrap() {
        McpServerConfig::Stdio { command, args, env } => {
            assert_eq!(command, "node");
            assert_eq!(args.as_ref().unwrap().len(), 1);
            assert_eq!(env.as_ref().unwrap().get("API_KEY").unwrap(), "secret");
        }
        _ => panic!("Expected stdio config"),
    }
}

#[test]
fn test_agent_options_with_mcp_servers_sse() {
    let mut options = AgentOptions::default();
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token".to_string());

    options.mcp_servers.insert(
        "sse-server".to_string(),
        McpServerConfig::Sse {
            url: "http://localhost:8080".to_string(),
            headers: Some(headers),
        },
    );

    assert_eq!(options.mcp_servers.len(), 1);
    match options.mcp_servers.get("sse-server").unwrap() {
        McpServerConfig::Sse { url, headers } => {
            assert_eq!(url, "http://localhost:8080");
            assert!(headers.is_some());
        }
        _ => panic!("Expected SSE config"),
    }
}

#[test]
fn test_agent_options_with_mcp_servers_http() {
    let mut options = AgentOptions::default();

    options.mcp_servers.insert(
        "http-server".to_string(),
        McpServerConfig::Http {
            url: "http://api.example.com".to_string(),
            headers: None,
        },
    );

    assert_eq!(options.mcp_servers.len(), 1);
    match options.mcp_servers.get("http-server").unwrap() {
        McpServerConfig::Http { url, headers } => {
            assert_eq!(url, "http://api.example.com");
            assert!(headers.is_none());
        }
        _ => panic!("Expected HTTP config"),
    }
}

#[test]
fn test_agent_options_with_mcp_servers_sdk() {
    let mut options = AgentOptions::default();

    options.mcp_servers.insert(
        "sdk-server".to_string(),
        McpServerConfig::Sdk {
            name: "my-sdk".to_string(),
        },
    );

    assert_eq!(options.mcp_servers.len(), 1);
    match options.mcp_servers.get("sdk-server").unwrap() {
        McpServerConfig::Sdk { name } => {
            assert_eq!(name, "my-sdk");
        }
        _ => panic!("Expected SDK config"),
    }
}

#[test]
fn test_agent_options_with_continue_conversation() {
    let mut options = AgentOptions::default();
    options.continue_conversation = true;

    assert!(options.continue_conversation);
}

#[test]
fn test_agent_options_with_resume() {
    let mut options = AgentOptions::default();
    options.resume = Some("session-123".to_string());

    assert_eq!(options.resume.as_ref().unwrap(), "session-123");
}

#[test]
fn test_agent_options_with_add_dirs() {
    let mut options = AgentOptions::default();
    options.add_dirs = vec![PathBuf::from("/path/1"), PathBuf::from("/path/2")];

    assert_eq!(options.add_dirs.len(), 2);
}

#[test]
fn test_agent_options_with_env() {
    let mut options = AgentOptions::default();
    options.env.insert("VAR1".to_string(), "value1".to_string());
    options.env.insert("VAR2".to_string(), "value2".to_string());

    assert_eq!(options.env.len(), 2);
    assert_eq!(options.env.get("VAR1").unwrap(), "value1");
}

#[test]
fn test_agent_options_with_extra_args() {
    let mut options = AgentOptions::default();
    options
        .extra_args
        .insert("flag".to_string(), Some("value".to_string()));
    options.extra_args.insert("switch".to_string(), None);

    assert_eq!(options.extra_args.len(), 2);
    assert!(options.extra_args.get("flag").unwrap().is_some());
    assert!(options.extra_args.get("switch").unwrap().is_none());
}

#[test]
fn test_agent_options_with_max_buffer_size() {
    let mut options = AgentOptions::default();
    options.max_buffer_size = Some(1024 * 1024);

    assert_eq!(options.max_buffer_size, Some(1024 * 1024));
}

#[test]
fn test_agent_options_with_session_options() {
    let mut options = AgentOptions::default();
    options.include_partial_messages = true;
    options.fork_session = true;

    assert!(options.include_partial_messages);
    assert!(options.fork_session);
}

#[test]
fn test_agent_options_with_agents() {
    let mut options = AgentOptions::default();

    let agent = AgentDefinition {
        description: "Test agent".to_string(),
        prompt: "You are a helpful assistant".to_string(),
        tools: Some(vec!["Read".to_string(), "Write".to_string()]),
        model: Some("claude-sonnet-4-5".to_string()),
    };

    options.agents.insert("test-agent".to_string(), agent);

    assert_eq!(options.agents.len(), 1);
    let retrieved = options.agents.get("test-agent").unwrap();
    assert_eq!(retrieved.description, "Test agent");
    assert_eq!(retrieved.tools.as_ref().unwrap().len(), 2);
}

#[test]
fn test_agent_options_with_setting_sources() {
    let mut options = AgentOptions::default();
    options.setting_sources = Some(vec![
        "default".to_string(),
        "user".to_string(),
        "project".to_string(),
    ]);

    assert_eq!(options.setting_sources.as_ref().unwrap().len(), 3);
}

#[test]
fn test_agent_options_clone() {
    let mut options = AgentOptions::default();
    options.model = Some("claude-sonnet-4-5".to_string());
    options.max_turns = Some(5);

    let cloned = options.clone();

    assert_eq!(cloned.model, options.model);
    assert_eq!(cloned.max_turns, options.max_turns);
}

#[test]
fn test_agent_options_debug() {
    let mut options = AgentOptions::default();
    options.model = Some("claude-sonnet-4-5".to_string());
    options.max_turns = Some(10);

    let debug_str = format!("{:?}", options);

    assert!(debug_str.contains("AgentOptions"));
    assert!(debug_str.contains("claude-sonnet-4-5"));
    assert!(debug_str.contains("10"));
}

#[test]
fn test_system_prompt_config_text_debug() {
    let config = SystemPromptConfig::Text("Test prompt".to_string());
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("Text"));
    assert!(debug_str.contains("Test prompt"));
}

#[test]
fn test_system_prompt_config_preset_debug() {
    let config = SystemPromptConfig::Preset {
        preset: "code-review".to_string(),
        append: Some("Focus on tests".to_string()),
    };
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("Preset"));
    assert!(debug_str.contains("code-review"));
    assert!(debug_str.contains("Focus on tests"));
}

#[test]
fn test_agent_definition_serialization() {
    let agent = AgentDefinition {
        description: "Test".to_string(),
        prompt: "Prompt".to_string(),
        tools: Some(vec!["Read".to_string()]),
        model: Some("sonnet".to_string()),
    };

    let json = serde_json::to_string(&agent).unwrap();
    let deserialized: AgentDefinition = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.description, agent.description);
    assert_eq!(deserialized.prompt, agent.prompt);
}

#[test]
fn test_mcp_server_config_stdio_serialization() {
    let config = McpServerConfig::Stdio {
        command: "node".to_string(),
        args: Some(vec!["server.js".to_string()]),
        env: None,
    };

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("stdio"));
    assert!(json.contains("node"));

    let deserialized: McpServerConfig = serde_json::from_str(&json).unwrap();
    match deserialized {
        McpServerConfig::Stdio { command, .. } => {
            assert_eq!(command, "node");
        }
        _ => panic!("Expected Stdio"),
    }
}

#[test]
fn test_mcp_server_config_sse_serialization() {
    let config = McpServerConfig::Sse {
        url: "http://localhost".to_string(),
        headers: None,
    };

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("sse"));

    let deserialized: McpServerConfig = serde_json::from_str(&json).unwrap();
    match deserialized {
        McpServerConfig::Sse { url, .. } => {
            assert_eq!(url, "http://localhost");
        }
        _ => panic!("Expected SSE"),
    }
}

#[test]
fn test_mcp_server_config_http_serialization() {
    let config = McpServerConfig::Http {
        url: "http://api.test".to_string(),
        headers: None,
    };

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("http"));

    let deserialized: McpServerConfig = serde_json::from_str(&json).unwrap();
    match deserialized {
        McpServerConfig::Http { url, .. } => {
            assert_eq!(url, "http://api.test");
        }
        _ => panic!("Expected HTTP"),
    }
}

#[test]
fn test_mcp_server_config_sdk_serialization() {
    let config = McpServerConfig::Sdk {
        name: "test-sdk".to_string(),
    };

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("sdk"));
    assert!(json.contains("test-sdk"));

    let deserialized: McpServerConfig = serde_json::from_str(&json).unwrap();
    match deserialized {
        McpServerConfig::Sdk { name } => {
            assert_eq!(name, "test-sdk");
        }
        _ => panic!("Expected SDK"),
    }
}

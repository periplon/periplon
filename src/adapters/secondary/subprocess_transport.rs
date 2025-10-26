use crate::error::{Error, Result};
use crate::options::{AgentOptions, SystemPromptConfig};
use crate::ports::secondary::Transport;
use async_trait::async_trait;
use futures::Stream;
use serde_json::json;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

const MINIMUM_CLI_VERSION: &str = "2.0.0";

use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

pub struct SubprocessCLITransport {
    cli_path: PathBuf,
    prompt: PromptType,
    options: AgentOptions,
    cwd: Option<PathBuf>,

    // Process state
    process: Option<Child>,
    stdin: Option<Arc<TokioMutex<ChildStdin>>>,
    stdout: Option<BufReader<ChildStdout>>,
    stderr_task: Option<tokio::task::JoinHandle<()>>,

    ready: bool,
    max_buffer_size: usize,
}

impl SubprocessCLITransport {
    /// Get a clone of the stdin Arc for independent locking
    pub fn get_stdin(&self) -> Option<Arc<TokioMutex<ChildStdin>>> {
        self.stdin.clone()
    }
}

pub enum PromptType {
    String(String),
    Stream,
}

impl SubprocessCLITransport {
    pub fn new(prompt: PromptType, options: AgentOptions) -> Self {
        let cli_path = options.cli_path.clone().unwrap_or_else(|| {
            Self::find_cli().unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                eprintln!("\nTroubleshooting:");
                eprintln!("  1. Ensure the CLI is installed (npm install -g @anthropics/claude)");
                eprintln!("  2. Check if 'claude' is in your PATH or set as an alias");
                eprintln!("  3. Specify cli_path explicitly in AgentOptions");
                eprintln!("\nSearched locations:");
                if let Ok(home) = std::env::var("HOME") {
                    eprintln!("  - PATH and shell aliases");
                    eprintln!("  - {}/.npm-global/bin/claude", home);
                    eprintln!("  - /usr/local/bin/claude");
                    eprintln!("  - {}/.local/bin/claude", home);
                    eprintln!("  - {}/node_modules/.bin/claude", home);
                    eprintln!("  - {}/.yarn/bin/claude", home);
                }
                panic!("CLI not found");
            })
        });

        let max_buffer_size = options.max_buffer_size.unwrap_or(1024 * 1024);
        let cwd = options.cwd.clone();

        Self {
            cli_path,
            prompt,
            options,
            cwd,
            process: None,
            stdin: None,
            stdout: None,
            stderr_task: None,
            ready: false,
            max_buffer_size,
        }
    }

    /// Find CLI binary in common locations
    fn find_cli() -> Result<PathBuf> {
        let debug = std::env::var("CLAUDE_SDK_DEBUG").is_ok();

        // 1. Check PATH (which resolves aliases and symlinks)
        if debug {
            eprintln!("[DEBUG] Searching for 'claude' in PATH...");
        }
        if let Ok(path) = which::which("claude") {
            if debug {
                eprintln!("[DEBUG] Found in PATH: {:?}", path);
            }
            // Canonicalize to resolve symlinks and aliases
            if let Ok(canonical) = path.canonicalize() {
                if debug {
                    eprintln!("[DEBUG] Canonicalized to: {:?}", canonical);
                }
                return Ok(canonical);
            }
            return Ok(path);
        }

        // 2. Check shell aliases by spawning a shell
        if debug {
            eprintln!("[DEBUG] Attempting to resolve shell alias...");
        }
        if let Ok(resolved) = Self::resolve_shell_alias("claude") {
            if debug {
                eprintln!("[DEBUG] Resolved alias to: {:?}", resolved);
            }
            return Ok(resolved);
        }

        // 3. Check common install locations
        if debug {
            eprintln!("[DEBUG] Checking common install locations...");
        }
        let home = std::env::var("HOME").map_err(|_| Error::CliNotFound)?;
        let locations = vec![
            format!("{}/.npm-global/bin/claude", home),
            "/usr/local/bin/claude".to_string(),
            format!("{}/.local/bin/claude", home),
            format!("{}/node_modules/.bin/claude", home),
            format!("{}/.yarn/bin/claude", home),
        ];

        for loc in &locations {
            let path = PathBuf::from(loc);
            if debug {
                eprintln!("[DEBUG] Checking: {:?} (exists: {})", path, path.exists());
            }
            if path.exists() && path.is_file() {
                if debug {
                    eprintln!("[DEBUG] Found at: {:?}", path);
                }
                return Ok(path);
            }
        }

        Err(Error::CliNotFound)
    }

    /// Resolve shell alias by spawning a shell
    fn resolve_shell_alias(cmd: &str) -> Result<PathBuf> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let debug = std::env::var("CLAUDE_SDK_DEBUG").is_ok();

        // Try multiple approaches to resolve the command
        let commands = vec![
            format!("type -P {} 2>/dev/null", cmd), // bash/zsh - resolve to path
            format!("command -v {} 2>/dev/null", cmd), // POSIX - find command
            format!("which {} 2>/dev/null", cmd),   // fallback - which utility
        ];

        for cmd_str in &commands {
            let output = std::process::Command::new(&shell)
                .arg("-i")
                .arg("-c")
                .arg(cmd_str)
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if debug {
                        eprintln!(
                            "[DEBUG] Shell command '{}' returned: {:?}",
                            cmd_str, path_str
                        );
                    }

                    if !path_str.is_empty() {
                        // Check if the output is an alias definition (e.g., "alias foo='bar'")
                        let resolved_path =
                            if let Some(alias_path) = Self::extract_alias_path(&path_str) {
                                if debug {
                                    eprintln!("[DEBUG] Extracted alias path: {:?}", alias_path);
                                }
                                Self::expand_tilde(&alias_path)
                            } else {
                                Self::expand_tilde(&path_str)
                            };

                        if debug {
                            eprintln!("[DEBUG] Expanded path: {:?}", resolved_path);
                        }

                        let path = PathBuf::from(&resolved_path);
                        if path.exists() && path.is_file() {
                            return Ok(path);
                        }
                    }
                }
            }
        }

        Err(Error::CliNotFound)
    }

    /// Extract the actual path from an alias definition
    /// Examples:
    /// - "alias claude='~/.claude/local/claude'" -> "~/.claude/local/claude"
    /// - "~/.claude/local/claude" -> "~/.claude/local/claude"
    fn extract_alias_path(alias_str: &str) -> Option<String> {
        // Match patterns like: alias name='path' or alias name="path"
        if let Some(start) = alias_str.find('\'') {
            if let Some(end) = alias_str.rfind('\'') {
                if start < end {
                    return Some(alias_str[start + 1..end].to_string());
                }
            }
        }
        if let Some(start) = alias_str.find('"') {
            if let Some(end) = alias_str.rfind('"') {
                if start < end {
                    return Some(alias_str[start + 1..end].to_string());
                }
            }
        }
        None
    }

    /// Expand tilde (~) to home directory
    fn expand_tilde(path: &str) -> String {
        if path.starts_with("~/") {
            if let Ok(home) = std::env::var("HOME") {
                return path.replacen("~", &home, 1);
            }
        }
        path.to_string()
    }

    /// Build CLI command with arguments
    fn build_command(&self) -> Vec<String> {
        let mut cmd = vec![
            self.cli_path.to_string_lossy().to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
        ];

        // System prompt
        match &self.options.system_prompt {
            Some(SystemPromptConfig::Text(text)) => {
                cmd.extend(["--system-prompt".to_string(), text.clone()]);
            }
            Some(SystemPromptConfig::Preset {
                preset: _,
                append: Some(text),
            }) => {
                cmd.extend(["--append-system-prompt".to_string(), text.clone()]);
            }
            _ => {}
        }

        // Tools
        if !self.options.allowed_tools.is_empty() {
            cmd.extend([
                "--allowedTools".to_string(),
                self.options.allowed_tools.join(","),
            ]);
        }
        if !self.options.disallowed_tools.is_empty() {
            cmd.extend([
                "--disallowedTools".to_string(),
                self.options.disallowed_tools.join(","),
            ]);
        }

        // Other options
        if let Some(max_turns) = self.options.max_turns {
            cmd.extend(["--max-turns".to_string(), max_turns.to_string()]);
        }
        if let Some(model) = &self.options.model {
            cmd.extend(["--model".to_string(), model.clone()]);
        }
        if let Some(mode) = &self.options.permission_mode {
            cmd.extend(["--permission-mode".to_string(), mode.clone()]);
        }
        if let Some(tool_name) = &self.options.permission_prompt_tool_name {
            cmd.extend(["--permission-prompt-tool".to_string(), tool_name.clone()]);
        }

        // Session options
        if self.options.continue_conversation {
            cmd.push("--continue".to_string());
        }
        if let Some(resume) = &self.options.resume {
            cmd.extend(["--resume".to_string(), resume.clone()]);
        }
        if self.options.include_partial_messages {
            cmd.push("--include-partial-messages".to_string());
        }
        if self.options.fork_session {
            cmd.push("--fork-session".to_string());
        }

        // Directories
        for dir in &self.options.add_dirs {
            cmd.extend(["--add-dir".to_string(), dir.to_string_lossy().to_string()]);
        }

        // MCP servers
        if !self.options.mcp_servers.is_empty() {
            let servers_json = serde_json::to_string(&json!({
                "mcpServers": self.options.mcp_servers
            }))
            .unwrap();
            cmd.extend(["--mcp-config".to_string(), servers_json]);
        }

        // Agents
        if !self.options.agents.is_empty() {
            let agents_json = serde_json::to_string(&self.options.agents).unwrap();
            cmd.extend(["--agents".to_string(), agents_json]);
        }

        // Setting sources
        if let Some(sources) = &self.options.setting_sources {
            cmd.extend(["--setting-sources".to_string(), sources.join(",")]);
        }

        // Extra args
        for (flag, value) in &self.options.extra_args {
            if let Some(val) = value {
                cmd.extend([format!("--{}", flag), val.clone()]);
            } else {
                cmd.push(format!("--{}", flag));
            }
        }

        // Input mode
        match &self.prompt {
            PromptType::String(text) => {
                cmd.extend(["--print".to_string(), "--".to_string(), text.clone()]);
            }
            PromptType::Stream => {
                cmd.extend(["--input-format".to_string(), "stream-json".to_string()]);
            }
        }

        cmd
    }

    /// Spawn stderr handler task
    async fn spawn_stderr_handler(&mut self, stderr: ChildStderr) {
        if let Some(callback) = self.options.stderr.clone() {
            let task = tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();

                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break;
                    }
                    let trimmed = line.trim_end();
                    if !trimmed.is_empty() {
                        callback(trimmed.to_string());
                    }
                    line.clear();
                }
            });

            self.stderr_task = Some(task);
        }
    }

    /// Check CLI version
    async fn check_cli_version(&self) -> Result<()> {
        if std::env::var("PERIPLON_SKIP_VERSION_CHECK").is_ok() {
            return Ok(());
        }

        let output = tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            Command::new(&self.cli_path).arg("-v").output(),
        )
        .await??;

        let version_str = String::from_utf8_lossy(&output.stdout);
        if let Some(captures) = regex::Regex::new(r"([0-9]+\.[0-9]+\.[0-9]+)")
            .unwrap()
            .captures(&version_str)
        {
            let version = captures.get(1).unwrap().as_str();
            let parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
            let min_parts = vec![2, 0, 0];

            if parts < min_parts {
                eprintln!(
                    "Warning: CLI version {} is unsupported. Minimum required: {}",
                    version, MINIMUM_CLI_VERSION
                );
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Transport for SubprocessCLITransport {
    async fn connect(&mut self) -> Result<()> {
        if self.process.is_some() {
            return Ok(());
        }

        self.check_cli_version().await?;

        let cmd_args = self.build_command();
        let mut command = Command::new(&cmd_args[0]);
        command.args(&cmd_args[1..]);

        // Set environment
        let mut env = self.options.env.clone();
        env.insert("CLAUDE_CODE_ENTRYPOINT".to_string(), "sdk-rust".to_string());
        env.insert(
            "PERIPLON_VERSION".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        command.envs(env);

        // Set working directory
        if let Some(cwd) = &self.cwd {
            // Create directory if requested and it doesn't exist
            if self.options.create_cwd && !cwd.exists() {
                std::fs::create_dir_all(cwd).map_err(|e| {
                    Error::InvalidInput(format!(
                        "Failed to create working directory '{}': {}",
                        cwd.display(),
                        e
                    ))
                })?;
            }

            command.current_dir(cwd);
        }

        // Spawn with piped I/O
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let mut child = command.spawn()?;

        // Take ownership of stdio
        let stdin = child.stdin.take().ok_or(Error::StdioError)?;
        let stdout = child.stdout.take().ok_or(Error::StdioError)?;
        let stderr = child.stderr.take().ok_or(Error::StdioError)?;

        self.stdout = Some(BufReader::new(stdout));
        self.spawn_stderr_handler(stderr).await;

        // For streaming mode, keep stdin open; for string mode, close it
        match &self.prompt {
            PromptType::Stream => {
                self.stdin = Some(Arc::new(TokioMutex::new(stdin)));
            }
            PromptType::String(_) => {
                drop(stdin); // Close immediately
            }
        }

        self.process = Some(child);
        self.ready = true;

        Ok(())
    }

    async fn write(&mut self, data: &str) -> Result<()> {
        if !self.ready {
            return Err(Error::NotReady);
        }

        let stdin_arc = self.stdin.as_ref().ok_or(Error::StdioError)?;
        let mut stdin = stdin_arc.lock().await;
        stdin.write_all(data.as_bytes()).await?;
        stdin.flush().await?;

        Ok(())
    }

    fn read_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>> {
        Box::pin(async_stream::try_stream! {
            let stdout = self.stdout.as_mut().ok_or(Error::StdioError)?;
            let mut json_buffer = String::new();
            let mut line = String::new();

            loop {
                line.clear();
                let n = stdout.read_line(&mut line).await?;
                if n == 0 {
                    break;
                } // EOF

                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Speculative JSON parsing with buffering
                json_buffer.push_str(trimmed);

                if json_buffer.len() > self.max_buffer_size {
                    Err(Error::BufferOverflow)?;
                }

                match serde_json::from_str::<serde_json::Value>(&json_buffer) {
                    Ok(data) => {
                        json_buffer.clear();
                        yield data;
                    }
                    Err(_) => {
                        // Incomplete JSON, keep buffering
                        continue;
                    }
                }
            }

            // Check process exit code
            if let Some(process) = &mut self.process {
                let status = process.wait().await?;
                if !status.success() {
                    let code = status.code().unwrap_or(-1);
                    Err(Error::ProcessFailed { exit_code: code })?;
                }
            }
        })
    }

    async fn end_input(&mut self) -> Result<()> {
        if let Some(stdin) = self.stdin.take() {
            drop(stdin);
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.ready = false;

        // Close stdin
        if let Some(stdin) = self.stdin.take() {
            drop(stdin);
        }

        // Cancel stderr task
        if let Some(task) = self.stderr_task.take() {
            task.abort();
        }

        // Terminate process
        if let Some(mut process) = self.process.take() {
            let _ = process.kill().await;
            let _ = process.wait().await;
        }

        self.stdout = None;

        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

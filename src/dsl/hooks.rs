//! Workflow Hooks System
//!
//! This module provides hooks that execute custom actions at different workflow stages.

use crate::dsl::schema::HookCommand;
use crate::error::{Error, Result};
use std::process::Stdio;
use tokio::process::Command;

/// Hook execution context with metadata
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Workflow name
    pub workflow_name: String,
    /// Current stage (if applicable)
    pub stage: Option<String>,
    /// Error message (if in error hook)
    pub error: Option<String>,
}

impl HookContext {
    /// Create a new hook context
    pub fn new(workflow_name: String) -> Self {
        Self {
            workflow_name,
            stage: None,
            error: None,
        }
    }

    /// Set the current stage
    pub fn with_stage(mut self, stage: String) -> Self {
        self.stage = Some(stage);
        self
    }

    /// Set error information
    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }
}

/// Workflow hooks executor
pub struct HooksExecutor;

impl HooksExecutor {
    /// Execute a list of hook commands
    ///
    /// # Arguments
    ///
    /// * `hooks` - List of hook commands to execute
    /// * `context` - Execution context with metadata
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub async fn execute_hooks(hooks: &[HookCommand], context: &HookContext) -> Result<()> {
        for hook in hooks {
            Self::execute_hook(hook, context).await?;
        }
        Ok(())
    }

    /// Execute a single hook command
    async fn execute_hook(hook: &HookCommand, context: &HookContext) -> Result<()> {
        let (command, description) = match hook {
            HookCommand::Command(cmd) => (cmd.as_str(), None),
            HookCommand::CommandSpec {
                command,
                description,
            } => (command.as_str(), description.as_ref()),
        };

        println!(
            "Executing hook{}: {}",
            description.map(|d| format!(" ({})", d)).unwrap_or_default(),
            command
        );

        // Execute the command
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .env("WORKFLOW_NAME", &context.workflow_name)
            .env("WORKFLOW_STAGE", context.stage.as_deref().unwrap_or(""))
            .env("WORKFLOW_ERROR", context.error.as_deref().unwrap_or(""))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| Error::InvalidInput(format!("Failed to execute hook: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::InvalidInput(format!(
                "Hook command failed: {}",
                stderr
            )));
        }

        // Print output if any
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("Hook output: {}", stdout.trim());
        }

        Ok(())
    }

    /// Execute pre-workflow hooks
    pub async fn execute_pre_workflow(hooks: &[HookCommand], workflow_name: &str) -> Result<()> {
        let context = HookContext::new(workflow_name.to_string());
        println!("Running pre-workflow hooks...");
        Self::execute_hooks(hooks, &context).await
    }

    /// Execute post-workflow hooks
    pub async fn execute_post_workflow(hooks: &[HookCommand], workflow_name: &str) -> Result<()> {
        let context = HookContext::new(workflow_name.to_string());
        println!("Running post-workflow hooks...");
        Self::execute_hooks(hooks, &context).await
    }

    /// Execute stage completion hooks
    pub async fn execute_stage_complete(
        hooks: &[HookCommand],
        workflow_name: &str,
        stage: &str,
    ) -> Result<()> {
        let context = HookContext::new(workflow_name.to_string()).with_stage(stage.to_string());
        println!("Running stage complete hooks for '{}'...", stage);
        Self::execute_hooks(hooks, &context).await
    }

    /// Execute error hooks
    pub async fn execute_error(
        hooks: &[HookCommand],
        workflow_name: &str,
        error: &str,
    ) -> Result<()> {
        let context = HookContext::new(workflow_name.to_string()).with_error(error.to_string());
        println!("Running error hooks...");
        Self::execute_hooks(hooks, &context).await
    }
}

/// Error recovery strategy
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the task with optional configuration
    Retry {
        max_attempts: u32,
        config: Option<crate::dsl::schema::ErrorHandlingSpec>,
    },
    /// Skip the task and continue
    Skip,
    /// Use fallback agent
    Fallback { agent_name: String },
    /// Abort the entire workflow
    Abort,
}

/// Error recovery manager
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Determine recovery strategy based on task configuration
    pub fn get_strategy(
        retry_count: Option<u32>,
        fallback_agent: Option<&str>,
    ) -> RecoveryStrategy {
        if let Some(agent) = fallback_agent {
            RecoveryStrategy::Fallback {
                agent_name: agent.to_string(),
            }
        } else if let Some(max_attempts) = retry_count {
            RecoveryStrategy::Retry {
                max_attempts,
                config: None, // Populated later from ErrorHandlingSpec
            }
        } else {
            RecoveryStrategy::Abort
        }
    }

    /// Determine recovery strategy from ErrorHandlingSpec
    pub fn get_strategy_from_spec(
        spec: Option<&crate::dsl::schema::ErrorHandlingSpec>,
    ) -> RecoveryStrategy {
        if let Some(error_spec) = spec {
            if let Some(agent) = &error_spec.fallback_agent {
                RecoveryStrategy::Fallback {
                    agent_name: agent.clone(),
                }
            } else {
                // Check if retry count is > 0
                let has_retry = error_spec.retry > 0;

                if has_retry {
                    RecoveryStrategy::Retry {
                        max_attempts: error_spec.retry,
                        config: Some(error_spec.clone()),
                    }
                } else {
                    RecoveryStrategy::Abort
                }
            }
        } else {
            RecoveryStrategy::Abort
        }
    }

    /// Check if a retry should be attempted
    pub fn should_retry(strategy: &RecoveryStrategy, current_attempt: u32) -> bool {
        match strategy {
            RecoveryStrategy::Retry { max_attempts, .. } => current_attempt <= *max_attempts,
            _ => false,
        }
    }

    /// Get fallback agent if available
    pub fn get_fallback_agent(strategy: &RecoveryStrategy) -> Option<&str> {
        match strategy {
            RecoveryStrategy::Fallback { agent_name } => Some(agent_name),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_context_creation() {
        let context = HookContext::new("test_workflow".to_string());
        assert_eq!(context.workflow_name, "test_workflow");
        assert!(context.stage.is_none());
        assert!(context.error.is_none());
    }

    #[test]
    fn test_hook_context_with_stage() {
        let context =
            HookContext::new("test_workflow".to_string()).with_stage("stage1".to_string());
        assert_eq!(context.stage, Some("stage1".to_string()));
    }

    #[test]
    fn test_hook_context_with_error() {
        let context =
            HookContext::new("test_workflow".to_string()).with_error("error msg".to_string());
        assert_eq!(context.error, Some("error msg".to_string()));
    }

    #[tokio::test]
    async fn test_execute_simple_hook() {
        let hooks = vec![HookCommand::Command("echo 'test'".to_string())];
        let context = HookContext::new("test".to_string());

        let result = HooksExecutor::execute_hooks(&hooks, &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_hook_with_description() {
        let hooks = vec![HookCommand::CommandSpec {
            command: "echo 'test'".to_string(),
            description: Some("Test hook".to_string()),
        }];
        let context = HookContext::new("test".to_string());

        let result = HooksExecutor::execute_hooks(&hooks, &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_failing_hook() {
        let hooks = vec![HookCommand::Command("exit 1".to_string())];
        let context = HookContext::new("test".to_string());

        let result = HooksExecutor::execute_hooks(&hooks, &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_recovery_strategy_retry() {
        let strategy = ErrorRecovery::get_strategy(Some(3), None);
        match strategy {
            RecoveryStrategy::Retry { max_attempts, .. } => {
                assert_eq!(max_attempts, 3)
            }
            _ => panic!("Wrong strategy"),
        }
    }

    #[test]
    fn test_recovery_strategy_fallback() {
        let strategy = ErrorRecovery::get_strategy(None, Some("fallback_agent"));
        match strategy {
            RecoveryStrategy::Fallback { agent_name } => {
                assert_eq!(agent_name, "fallback_agent")
            }
            _ => panic!("Wrong strategy"),
        }
    }

    #[test]
    fn test_should_retry() {
        let strategy = RecoveryStrategy::Retry {
            max_attempts: 3,
            config: None,
        };
        // With max_attempts=3 (3 retries), we should retry for attempts 1, 2, and 3
        assert!(ErrorRecovery::should_retry(&strategy, 0)); // Initial attempt can retry
        assert!(ErrorRecovery::should_retry(&strategy, 1)); // First retry
        assert!(ErrorRecovery::should_retry(&strategy, 2)); // Second retry
        assert!(ErrorRecovery::should_retry(&strategy, 3)); // Third retry
        assert!(!ErrorRecovery::should_retry(&strategy, 4)); // No fourth retry
    }
}

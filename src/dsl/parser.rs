//! YAML Parser for DSL Workflows
//!
//! This module provides functionality to parse YAML files into DSL workflow structures.

use crate::dsl::fetcher::{fetch_subflow, SubflowCache};
use crate::dsl::schema::{DSLWorkflow, SubflowSpec};
use crate::error::{Error, Result};
use std::collections::HashSet;
use std::path::Path;

/// Parse a YAML string into a DSL workflow
///
/// # Arguments
///
/// * `yaml_str` - YAML string to parse
///
/// # Returns
///
/// Result containing the parsed workflow or an error
///
/// # Example
///
/// ```no_run
/// use periplon_sdk::dsl::parse_workflow;
///
/// let yaml = r#"
/// name: "Test Workflow"
/// version: "1.0.0"
/// agents:
///   test_agent:
///     description: "A test agent"
///     tools:
///       - Read
/// "#;
///
/// let workflow = parse_workflow(yaml).unwrap();
/// assert_eq!(workflow.name, "Test Workflow");
/// ```
pub fn parse_workflow(yaml_str: &str) -> Result<DSLWorkflow> {
    serde_yaml::from_str(yaml_str)
        .map_err(|e| Error::InvalidInput(format!("Failed to parse YAML workflow: {}", e)))
}

/// Parse a YAML file into a DSL workflow
///
/// # Arguments
///
/// * `path` - Path to the YAML file
///
/// # Returns
///
/// Result containing the parsed workflow or an error
///
/// # Example
///
/// ```no_run
/// use periplon_sdk::dsl::parse_workflow_file;
///
/// let workflow = parse_workflow_file("workflow.yaml").unwrap();
/// println!("Loaded workflow: {}", workflow.name);
/// ```
pub fn parse_workflow_file<P: AsRef<Path>>(path: P) -> Result<DSLWorkflow> {
    let contents = std::fs::read_to_string(path.as_ref())
        .map_err(|e| Error::InvalidInput(format!("Failed to read workflow file: {}", e)))?;

    parse_workflow(&contents)
}

/// Serialize a DSL workflow to YAML string
///
/// # Arguments
///
/// * `workflow` - The workflow to serialize
///
/// # Returns
///
/// Result containing the YAML string or an error
pub fn serialize_workflow(workflow: &DSLWorkflow) -> Result<String> {
    serde_yaml::to_string(workflow)
        .map_err(|e| Error::InvalidInput(format!("Failed to serialize workflow to YAML: {}", e)))
}

/// Write a DSL workflow to a YAML file
///
/// # Arguments
///
/// * `workflow` - The workflow to write
/// * `path` - Path to write the YAML file
///
/// # Returns
///
/// Result indicating success or error
pub fn write_workflow_file<P: AsRef<Path>>(workflow: &DSLWorkflow, path: P) -> Result<()> {
    let yaml_str = serialize_workflow(workflow)?;

    std::fs::write(path.as_ref(), yaml_str)
        .map_err(|e| Error::InvalidInput(format!("Failed to write workflow file: {}", e)))
}

/// Parse a workflow file and resolve all subflows
///
/// # Arguments
///
/// * `path` - Path to the main workflow file
///
/// # Returns
///
/// Result containing the merged workflow with all subflows resolved
///
/// # Example
///
/// ```no_run
/// use periplon_sdk::dsl::parse_workflow_with_subflows;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workflow = parse_workflow_with_subflows("workflow.yaml").await?;
/// println!("Loaded workflow with {} subflows", workflow.subflows.len());
/// # Ok(())
/// # }
/// ```
pub async fn parse_workflow_with_subflows<P: AsRef<Path>>(path: P) -> Result<DSLWorkflow> {
    let path_ref = path.as_ref();
    let base_path = path_ref.parent().map(|p| p.to_path_buf());
    let mut workflow = parse_workflow_file(path_ref)?;
    let cache = SubflowCache::new();

    resolve_subflows(&mut workflow, base_path.as_deref(), &cache).await?;

    Ok(workflow)
}

/// Resolve all subflows in a workflow by fetching and merging external sources
///
/// # Arguments
///
/// * `workflow` - The workflow to resolve subflows for
/// * `base_path` - Base path for resolving relative file paths
/// * `cache` - Cache for storing fetched subflows
///
/// # Returns
///
/// Result indicating success or error
async fn resolve_subflows(
    workflow: &mut DSLWorkflow,
    base_path: Option<&Path>,
    cache: &SubflowCache,
) -> Result<()> {
    let mut visited = HashSet::new();
    resolve_subflows_recursive(workflow, base_path, cache, &mut visited, vec![]).await
}

/// Recursively resolve subflows with cycle detection
fn resolve_subflows_recursive<'a>(
    workflow: &'a mut DSLWorkflow,
    base_path: Option<&'a Path>,
    cache: &'a SubflowCache,
    visited: &'a mut HashSet<String>,
    path: Vec<String>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
    Box::pin(async move {
        let subflow_ids: Vec<String> = workflow.subflows.keys().cloned().collect();

        for subflow_id in subflow_ids {
            // Check for circular dependencies
            if path.contains(&subflow_id) {
                return Err(Error::InvalidInput(format!(
                    "Circular subflow dependency detected: {} -> {}",
                    path.join(" -> "),
                    subflow_id
                )));
            }

            // Skip if already processed
            if visited.contains(&subflow_id) {
                continue;
            }

            let subflow = workflow.subflows.get(&subflow_id).unwrap().clone();

            // If subflow has a source, fetch and merge it
            if let Some(source) = &subflow.source {
                let mut fetched_workflow = fetch_subflow(source, base_path, cache).await?;

                // Recursively resolve subflows in the fetched workflow
                let mut new_path = path.clone();
                new_path.push(subflow_id.clone());
                resolve_subflows_recursive(
                    &mut fetched_workflow,
                    base_path,
                    cache,
                    visited,
                    new_path,
                )
                .await?;

                // Merge the fetched workflow into the subflow spec
                let merged_subflow = SubflowSpec {
                    description: subflow.description.clone(),
                    source: subflow.source.clone(),
                    agents: fetched_workflow.agents,
                    tasks: fetched_workflow.tasks,
                    inputs: subflow.inputs.clone(),
                    outputs: subflow.outputs.clone(),
                };

                workflow.subflows.insert(subflow_id.clone(), merged_subflow);
            }

            visited.insert(subflow_id);
        }

        Ok(())
    })
}

/// Merge a subflow's agents and tasks into the main workflow with namespacing
///
/// This function is useful when you want to inline a subflow into the main workflow.
/// Agents and tasks from the subflow will be prefixed with "subflow_id." to avoid conflicts.
///
/// # Arguments
///
/// * `workflow` - The main workflow to merge into
/// * `subflow_id` - The ID of the subflow to merge
///
/// # Returns
///
/// Result indicating success or error
pub fn merge_subflow_inline(workflow: &mut DSLWorkflow, subflow_id: &str) -> Result<()> {
    let subflow = workflow
        .subflows
        .get(subflow_id)
        .ok_or_else(|| Error::InvalidInput(format!("Subflow '{}' not found", subflow_id)))?
        .clone();

    // Merge agents with namespacing
    for (agent_id, agent_spec) in subflow.agents {
        let namespaced_id = format!("{}.{}", subflow_id, agent_id);
        workflow.agents.insert(namespaced_id, agent_spec);
    }

    // Merge tasks with namespacing
    for (task_id, mut task_spec) in subflow.tasks {
        let namespaced_id = format!("{}.{}", subflow_id, task_id);

        // Update agent references in tasks
        if let Some(ref agent) = task_spec.agent {
            task_spec.agent = Some(format!("{}.{}", subflow_id, agent));
        }

        // Update dependencies to use namespaced IDs
        task_spec.depends_on = task_spec
            .depends_on
            .iter()
            .map(|dep| format!("{}.{}", subflow_id, dep))
            .collect();

        workflow.tasks.insert(namespaced_id, task_spec);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_workflow() {
        let yaml = r#"
name: "Minimal Workflow"
version: "1.0.0"
"#;
        let workflow = parse_workflow(yaml).unwrap();
        assert_eq!(workflow.name, "Minimal Workflow");
        assert_eq!(workflow.version, "1.0.0");
        assert!(workflow.agents.is_empty());
        assert!(workflow.tasks.is_empty());
    }

    #[test]
    fn test_parse_workflow_with_agent() {
        let yaml = r#"
name: "Test Workflow"
version: "1.0.0"
agents:
  researcher:
    description: "Research agent"
    model: "claude-sonnet-4-5"
    tools:
      - Read
      - WebSearch
    permissions:
      mode: "default"
"#;
        let workflow = parse_workflow(yaml).unwrap();
        assert_eq!(workflow.agents.len(), 1);

        let agent = workflow.agents.get("researcher").unwrap();
        assert_eq!(agent.description, "Research agent");
        assert_eq!(agent.model, Some("claude-sonnet-4-5".to_string()));
        assert_eq!(agent.tools.len(), 2);
        assert_eq!(agent.permissions.mode, "default");
    }

    #[test]
    fn test_parse_workflow_with_tasks() {
        let yaml = r#"
name: "Task Workflow"
version: "1.0.0"
tasks:
  task1:
    description: "First task"
    agent: "researcher"
    priority: 1
  task2:
    description: "Second task"
    agent: "coder"
    priority: 2
    depends_on:
      - task1
"#;
        let workflow = parse_workflow(yaml).unwrap();
        assert_eq!(workflow.tasks.len(), 2);

        let task1 = workflow.tasks.get("task1").unwrap();
        assert_eq!(task1.description, "First task");
        assert_eq!(task1.priority, 1);

        let task2 = workflow.tasks.get("task2").unwrap();
        assert_eq!(task2.description, "Second task");
        assert_eq!(task2.depends_on.len(), 1);
        assert_eq!(task2.depends_on[0], "task1");
    }

    #[test]
    fn test_parse_hierarchical_tasks() {
        let yaml = r#"
name: "Hierarchical Workflow"
version: "1.0.0"
tasks:
  parent_task:
    description: "Parent task"
    agent: "coder"
    subtasks:
      - child_task1:
          description: "First child"
          agent: "coder"
      - child_task2:
          description: "Second child"
          agent: "reviewer"
"#;
        let workflow = parse_workflow(yaml).unwrap();

        let parent = workflow.tasks.get("parent_task").unwrap();
        assert_eq!(parent.subtasks.len(), 2);
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = r#"
name: "Invalid
version: 1.0.0
"#;
        let result = parse_workflow(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_workflow() {
        let yaml = r#"
name: "Test Workflow"
version: "1.0.0"
"#;
        let workflow = parse_workflow(yaml).unwrap();
        let serialized = serialize_workflow(&workflow).unwrap();

        assert!(serialized.contains("Test Workflow"));
        assert!(serialized.contains("1.0.0"));
    }

    #[test]
    fn test_roundtrip_serialization() {
        let yaml = r#"
name: "Roundtrip Test"
version: "1.0.0"
agents:
  test_agent:
    description: "Test agent"
    tools:
      - Read
    permissions:
      mode: "default"
"#;
        let workflow = parse_workflow(yaml).unwrap();
        let serialized = serialize_workflow(&workflow).unwrap();
        let roundtrip = parse_workflow(&serialized).unwrap();

        assert_eq!(workflow.name, roundtrip.name);
        assert_eq!(workflow.version, roundtrip.version);
        assert_eq!(workflow.agents.len(), roundtrip.agents.len());
    }

    #[test]
    fn test_parse_workflow_with_imports() {
        let yaml = r#"
name: "Workflow with Imports"
version: "1.0.0"
imports:
  google: "google-workspace@1.0.0"
  slack: "slack-integrations@2.1.0"
  github: "github-actions@3.0.0"
"#;
        let workflow = parse_workflow(yaml).unwrap();
        assert_eq!(workflow.imports.len(), 3);
        assert_eq!(
            workflow.imports.get("google"),
            Some(&"google-workspace@1.0.0".to_string())
        );
        assert_eq!(
            workflow.imports.get("slack"),
            Some(&"slack-integrations@2.1.0".to_string())
        );
        assert_eq!(
            workflow.imports.get("github"),
            Some(&"github-actions@3.0.0".to_string())
        );
    }

    #[test]
    fn test_parse_workflow_with_uses_workflow() {
        let yaml = r#"
name: "Workflow with uses_workflow"
version: "1.0.0"
imports:
  google: "google-workspace@1.0.0"
tasks:
  upload:
    description: "Upload files to Google Drive"
    uses_workflow: "google:upload-files"
    inputs:
      folder_id: "abc123"
      files: ["doc1.pdf", "doc2.pdf"]
"#;
        let workflow = parse_workflow(yaml).unwrap();
        assert_eq!(workflow.imports.len(), 1);
        assert_eq!(workflow.tasks.len(), 1);

        let task = workflow.tasks.get("upload").unwrap();
        assert_eq!(task.description, "Upload files to Google Drive");
        assert_eq!(task.uses_workflow, Some("google:upload-files".to_string()));
        assert_eq!(task.inputs.len(), 2);
    }

    #[test]
    fn test_serialize_workflow_with_imports_and_uses_workflow() {
        use crate::dsl::schema::{DSLWorkflow, TaskSpec};
        use std::collections::HashMap;

        let mut workflow = DSLWorkflow {
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        // Add import
        workflow
            .imports
            .insert("google".to_string(), "google-workspace@1.0.0".to_string());

        // Add task with uses_workflow
        let task = TaskSpec {
            description: "Upload files".to_string(),
            uses_workflow: Some("google:upload-files".to_string()),
            inputs: [("folder_id".to_string(), serde_json::json!("abc123"))]
                .iter()
                .cloned()
                .collect(),
            ..Default::default()
        };
        workflow.tasks.insert("upload".to_string(), task);

        // Serialize and parse back
        let yaml = serialize_workflow(&workflow).unwrap();
        let parsed = parse_workflow(&yaml).unwrap();

        // Verify
        assert_eq!(parsed.imports.len(), 1);
        assert_eq!(parsed.tasks.len(), 1);
        let parsed_task = parsed.tasks.get("upload").unwrap();
        assert_eq!(
            parsed_task.uses_workflow,
            Some("google:upload-files".to_string())
        );
    }

    #[test]
    fn test_parse_workflow_without_imports() {
        let yaml = r#"
name: "Workflow without Imports"
version: "1.0.0"
agents:
  researcher:
    description: "Research agent"
"#;
        let workflow = parse_workflow(yaml).unwrap();
        assert!(workflow.imports.is_empty());
    }

    #[test]
    fn test_roundtrip_with_imports_and_uses_workflow() {
        let yaml = r#"
name: "Complex Workflow"
version: "1.0.0"
imports:
  google: "google-workspace@1.0.0"
  slack: "slack-api@2.0.0"
agents:
  notifier:
    description: "Notification agent"
    tools:
      - Read
tasks:
  upload:
    description: "Upload files"
    uses_workflow: "google:upload-files"
    inputs:
      folder_id: "folder123"
  notify:
    description: "Send notification"
    uses_workflow: "slack:send-message"
    inputs:
      channel: "general"
      message: "Files uploaded!"
    depends_on:
      - upload
"#;
        let workflow = parse_workflow(yaml).unwrap();
        let serialized = serialize_workflow(&workflow).unwrap();
        let roundtrip = parse_workflow(&serialized).unwrap();

        assert_eq!(workflow.name, roundtrip.name);
        assert_eq!(workflow.imports.len(), roundtrip.imports.len());
        assert_eq!(workflow.tasks.len(), roundtrip.tasks.len());

        let task1 = roundtrip.tasks.get("upload").unwrap();
        assert_eq!(task1.uses_workflow, Some("google:upload-files".to_string()));

        let task2 = roundtrip.tasks.get("notify").unwrap();
        assert_eq!(task2.uses_workflow, Some("slack:send-message".to_string()));
        assert_eq!(task2.depends_on, vec!["upload".to_string()]);
    }

    // ========================================================================
    // Notification Parsing Tests
    // ========================================================================

    #[test]
    fn test_parse_simple_string_notification() {
        let yaml = r#"
name: "Notification Test"
version: "1.0.0"
tasks:
  build:
    description: "Build the project"
    agent: "builder"
    on_complete:
      notify: "Build completed successfully"
"#;
        let workflow = parse_workflow(yaml).unwrap();
        let task = workflow.tasks.get("build").unwrap();

        assert!(task.on_complete.is_some());
        let action = task.on_complete.as_ref().unwrap();
        assert!(action.notify.is_some());

        match action.notify.as_ref().unwrap() {
            crate::dsl::schema::NotificationSpec::Simple(msg) => {
                assert_eq!(msg, "Build completed successfully");
            }
            _ => panic!("Expected simple notification"),
        }
    }

    #[test]
    fn test_parse_structured_notification_with_console() {
        let yaml = r#"
name: "Structured Notification Test"
version: "1.0.0"
tasks:
  deploy:
    description: "Deploy application"
    agent: "deployer"
    on_complete:
      notify:
        message: "Deployment completed"
        title: "Production Deployment"
        priority: high
        channels:
          - type: console
            colored: true
            timestamp: true
"#;
        let workflow = parse_workflow(yaml).unwrap();
        let task = workflow.tasks.get("deploy").unwrap();

        assert!(task.on_complete.is_some());
        let action = task.on_complete.as_ref().unwrap();
        assert!(action.notify.is_some());

        match action.notify.as_ref().unwrap() {
            crate::dsl::schema::NotificationSpec::Structured {
                message,
                title,
                priority,
                channels,
                ..
            } => {
                assert_eq!(message, "Deployment completed");
                assert_eq!(title.as_ref().unwrap(), "Production Deployment");
                assert_eq!(
                    *priority.as_ref().unwrap(),
                    crate::dsl::schema::NotificationPriority::High
                );
                assert_eq!(channels.len(), 1);

                match &channels[0] {
                    crate::dsl::schema::NotificationChannel::Console { colored, timestamp } => {
                        assert!(colored);
                        assert!(timestamp);
                    }
                    _ => panic!("Expected Console channel"),
                }
            }
            _ => panic!("Expected structured notification"),
        }
    }

    #[test]
    fn test_parse_notification_with_ntfy() {
        let yaml = r#"
name: "Ntfy Notification Test"
version: "1.0.0"
tasks:
  backup:
    description: "Backup database"
    agent: "backup_agent"
    on_complete:
      notify:
        message: "Backup completed successfully"
        channels:
          - type: ntfy
            server: "https://ntfy.sh"
            topic: "my-backups"
            priority: 4
            tags: ["white_check_mark", "floppy_disk"]
            markdown: true
"#;
        let workflow = parse_workflow(yaml).unwrap();
        let task = workflow.tasks.get("backup").unwrap();

        assert!(task.on_complete.is_some());
        let action = task.on_complete.as_ref().unwrap();

        match action.notify.as_ref().unwrap() {
            crate::dsl::schema::NotificationSpec::Structured { channels, .. } => {
                assert_eq!(channels.len(), 1);

                match &channels[0] {
                    crate::dsl::schema::NotificationChannel::Ntfy {
                        server,
                        topic,
                        priority,
                        tags,
                        markdown,
                        ..
                    } => {
                        assert_eq!(server, "https://ntfy.sh");
                        assert_eq!(topic, "my-backups");
                        assert_eq!(*priority, Some(4));
                        assert_eq!(tags.len(), 2);
                        assert!(markdown);
                    }
                    _ => panic!("Expected Ntfy channel"),
                }
            }
            _ => panic!("Expected structured notification"),
        }
    }

    #[test]
    fn test_parse_notification_with_slack() {
        let yaml = r##"
name: "Slack Notification Test"
version: "1.0.0"
tasks:
  build:
    description: "Build project"
    agent: "builder"
    on_complete:
      notify:
        message: "Build completed"
        channels:
          - type: slack
            credential: "${secret.slack_webhook}"
            channel: "#builds"
            method: webhook
"##;
        let workflow = parse_workflow(yaml).unwrap();
        let task = workflow.tasks.get("build").unwrap();

        match task.on_complete.as_ref().unwrap().notify.as_ref().unwrap() {
            crate::dsl::schema::NotificationSpec::Structured { channels, .. } => {
                match &channels[0] {
                    crate::dsl::schema::NotificationChannel::Slack {
                        credential,
                        channel,
                        method,
                        ..
                    } => {
                        assert_eq!(credential, "${secret.slack_webhook}");
                        assert_eq!(channel, "#builds");
                        assert_eq!(*method, crate::dsl::schema::SlackMethod::Webhook);
                    }
                    _ => panic!("Expected Slack channel"),
                }
            }
            _ => panic!("Expected structured notification"),
        }
    }

    #[test]
    fn test_parse_notification_with_discord() {
        let yaml = r##"
name: "Discord Notification Test"
version: "1.0.0"
tasks:
  test:
    description: "Run tests"
    agent: "tester"
    on_complete:
      notify:
        message: "Tests passed"
        channels:
          - type: discord
            webhook_url: "${secret.discord_webhook}"
            username: "Test Bot"
            tts: false
            embed:
              title: "Test Results"
              description: "All tests passed successfully"
              color: 65280
"##;
        let workflow = parse_workflow(yaml).unwrap();
        let task = workflow.tasks.get("test").unwrap();

        match task.on_complete.as_ref().unwrap().notify.as_ref().unwrap() {
            crate::dsl::schema::NotificationSpec::Structured { channels, .. } => {
                match &channels[0] {
                    crate::dsl::schema::NotificationChannel::Discord {
                        webhook_url,
                        username,
                        tts,
                        embed,
                        ..
                    } => {
                        assert_eq!(webhook_url, "${secret.discord_webhook}");
                        assert_eq!(username.as_ref().unwrap(), "Test Bot");
                        assert!(!tts);
                        assert!(embed.is_some());

                        let embed_data = embed.as_ref().unwrap();
                        assert_eq!(embed_data.title.as_ref().unwrap(), "Test Results");
                        assert_eq!(embed_data.color, Some(65280));
                    }
                    _ => panic!("Expected Discord channel"),
                }
            }
            _ => panic!("Expected structured notification"),
        }
    }

    #[test]
    fn test_parse_notification_with_webhook() {
        let yaml = r##"
name: "Webhook Notification Test"
version: "1.0.0"
tasks:
  deploy:
    description: "Deploy application"
    agent: "deployer"
    on_complete:
      notify:
        message: "Deployment successful"
        channels:
          - type: webhook
            url: "https://example.com/webhook"
            method: POST
            headers:
              Content-Type: "application/json"
            auth:
              type: bearer
              token: "${secret.api_token}"
            body_template: '{"event": "deploy", "message": "{message}", "title": "{title}"}'
            retry:
              max_attempts: 3
              delay_secs: 5
              exponential_backoff: true
"##;
        let workflow = parse_workflow(yaml).unwrap();
        let task = workflow.tasks.get("deploy").unwrap();

        match task.on_complete.as_ref().unwrap().notify.as_ref().unwrap() {
            crate::dsl::schema::NotificationSpec::Structured { channels, .. } => {
                match &channels[0] {
                    crate::dsl::schema::NotificationChannel::Webhook {
                        url,
                        method,
                        headers,
                        auth,
                        body_template,
                        retry,
                        ..
                    } => {
                        assert_eq!(url, "https://example.com/webhook");
                        assert!(matches!(method, crate::dsl::schema::HttpMethod::Post));
                        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
                        assert!(auth.is_some());
                        assert!(body_template.is_some());

                        let retry_config = retry.as_ref().unwrap();
                        assert_eq!(retry_config.max_attempts, 3);
                        assert_eq!(retry_config.delay_secs, 5);
                        assert!(retry_config.exponential_backoff);
                    }
                    _ => panic!("Expected Webhook channel"),
                }
            }
            _ => panic!("Expected structured notification"),
        }
    }

    #[test]
    fn test_parse_notification_with_file() {
        let yaml = r#"
name: "File Notification Test"
version: "1.0.0"
tasks:
  process:
    description: "Process data"
    agent: "processor"
    on_complete:
      notify:
        message: "Processing complete"
        channels:
          - type: file
            path: "./logs/notifications.log"
            append: true
            timestamp: true
            format: json
"#;
        let workflow = parse_workflow(yaml).unwrap();
        let task = workflow.tasks.get("process").unwrap();

        match task.on_complete.as_ref().unwrap().notify.as_ref().unwrap() {
            crate::dsl::schema::NotificationSpec::Structured { channels, .. } => {
                match &channels[0] {
                    crate::dsl::schema::NotificationChannel::File {
                        path,
                        append,
                        timestamp,
                        format,
                    } => {
                        assert_eq!(path, "./logs/notifications.log");
                        assert!(append);
                        assert!(timestamp);
                        assert_eq!(*format, crate::dsl::schema::FileNotificationFormat::Json);
                    }
                    _ => panic!("Expected File channel"),
                }
            }
            _ => panic!("Expected structured notification"),
        }
    }

    #[test]
    fn test_parse_notification_with_multiple_channels() {
        let yaml = r#"
name: "Multi-Channel Notification Test"
version: "1.0.0"
tasks:
  critical:
    description: "Critical task"
    agent: "critical_agent"
    on_complete:
      notify:
        message: "Critical task completed"
        title: "System Alert"
        priority: critical
        channels:
          - type: console
            colored: true
            timestamp: true
          - type: ntfy
            server: "https://ntfy.sh"
            topic: "alerts"
            priority: 5
          - type: file
            path: "./alerts.log"
            append: true
            timestamp: true
            format: jsonlines
"#;
        let workflow = parse_workflow(yaml).unwrap();
        let task = workflow.tasks.get("critical").unwrap();

        match task.on_complete.as_ref().unwrap().notify.as_ref().unwrap() {
            crate::dsl::schema::NotificationSpec::Structured {
                message,
                title,
                priority,
                channels,
                ..
            } => {
                assert_eq!(message, "Critical task completed");
                assert_eq!(title.as_ref().unwrap(), "System Alert");
                assert_eq!(
                    *priority.as_ref().unwrap(),
                    crate::dsl::schema::NotificationPriority::Critical
                );
                assert_eq!(channels.len(), 3);

                // Verify channel types
                assert!(matches!(
                    channels[0],
                    crate::dsl::schema::NotificationChannel::Console { .. }
                ));
                assert!(matches!(
                    channels[1],
                    crate::dsl::schema::NotificationChannel::Ntfy { .. }
                ));
                assert!(matches!(
                    channels[2],
                    crate::dsl::schema::NotificationChannel::File { .. }
                ));
            }
            _ => panic!("Expected structured notification"),
        }
    }

    #[test]
    fn test_parse_workflow_level_notification_defaults() {
        let yaml = r#"
name: "Workflow with Notification Defaults"
version: "1.0.0"
notifications:
  notify_on_completion: true
  notify_on_failure: true
  notify_on_start: false
  notify_on_workflow_completion: true
  default_channels:
    - type: console
      colored: true
      timestamp: true
    - type: ntfy
      server: "https://ntfy.sh"
      topic: "workflow-updates"
agents:
  worker:
    description: "Worker agent"
tasks:
  work:
    description: "Do work"
    agent: "worker"
"#;
        let workflow = parse_workflow(yaml).unwrap();

        assert!(workflow.notifications.is_some());
        let notif_defaults = workflow.notifications.as_ref().unwrap();

        assert!(notif_defaults.notify_on_completion);
        assert!(notif_defaults.notify_on_failure);
        assert!(!notif_defaults.notify_on_start);
        assert!(notif_defaults.notify_on_workflow_completion);
        assert_eq!(notif_defaults.default_channels.len(), 2);
    }

    #[test]
    fn test_parse_notification_with_metadata() {
        let yaml = r#"
name: "Notification with Metadata"
version: "1.0.0"
tasks:
  task1:
    description: "Test task"
    agent: "test_agent"
    on_complete:
      notify:
        message: "Task completed"
        title: "Test Notification"
        priority: normal
        metadata:
          environment: "production"
          version: "1.2.3"
          component: "api"
        channels:
          - type: console
            colored: true
            timestamp: true
"#;
        let workflow = parse_workflow(yaml).unwrap();
        let task = workflow.tasks.get("task1").unwrap();

        match task.on_complete.as_ref().unwrap().notify.as_ref().unwrap() {
            crate::dsl::schema::NotificationSpec::Structured {
                message, metadata, ..
            } => {
                assert_eq!(message, "Task completed");
                assert_eq!(metadata.len(), 3);
                assert_eq!(metadata.get("environment").unwrap(), "production");
                assert_eq!(metadata.get("version").unwrap(), "1.2.3");
                assert_eq!(metadata.get("component").unwrap(), "api");
            }
            _ => panic!("Expected structured notification"),
        }
    }

    #[test]
    fn test_roundtrip_notification_serialization() {
        let yaml = r#"
name: "Notification Roundtrip Test"
version: "1.0.0"
tasks:
  test:
    description: "Test task"
    agent: "tester"
    on_complete:
      notify:
        message: "Test completed"
        title: "Test Results"
        priority: high
        channels:
          - type: console
            colored: true
            timestamp: true
          - type: ntfy
            server: "https://ntfy.sh"
            topic: "test-results"
            priority: 4
            tags: ["test", "success"]
            markdown: true
"#;
        let workflow = parse_workflow(yaml).unwrap();
        let serialized = serialize_workflow(&workflow).unwrap();
        let roundtrip = parse_workflow(&serialized).unwrap();

        // Verify task notification is preserved
        let original_task = workflow.tasks.get("test").unwrap();
        let roundtrip_task = roundtrip.tasks.get("test").unwrap();

        assert!(original_task.on_complete.is_some());
        assert!(roundtrip_task.on_complete.is_some());

        let original_notify = &original_task.on_complete.as_ref().unwrap().notify;
        let roundtrip_notify = &roundtrip_task.on_complete.as_ref().unwrap().notify;

        assert!(original_notify.is_some());
        assert!(roundtrip_notify.is_some());
    }

    #[test]
    fn test_parse_notification_with_email() {
        let yaml = r##"
name: "Email Notification Test"
version: "1.0.0"
tasks:
  report:
    description: "Generate report"
    agent: "reporter"
    on_complete:
      notify:
        message: "Report ready"
        channels:
          - type: email
            to: ["admin@example.com", "team@example.com"]
            subject: "Daily Report Generated"
            smtp:
              host: "smtp.example.com"
              port: 587
              username: "${secret.smtp_user}"
              password: "${secret.smtp_pass}"
              from: "noreply@example.com"
              use_tls: true
"##;
        let workflow = parse_workflow(yaml).unwrap();
        let task = workflow.tasks.get("report").unwrap();

        match task.on_complete.as_ref().unwrap().notify.as_ref().unwrap() {
            crate::dsl::schema::NotificationSpec::Structured { channels, .. } => {
                match &channels[0] {
                    crate::dsl::schema::NotificationChannel::Email {
                        to, subject, smtp, ..
                    } => {
                        assert_eq!(to.len(), 2);
                        assert_eq!(subject.as_ref().unwrap(), "Daily Report Generated");
                        assert_eq!(smtp.host, "smtp.example.com");
                        assert_eq!(smtp.port, 587);
                        assert!(smtp.use_tls);
                    }
                    _ => panic!("Expected Email channel"),
                }
            }
            _ => panic!("Expected structured notification"),
        }
    }

    #[test]
    fn test_parse_notification_backward_compatibility() {
        // Test that old simple string format still works
        let yaml = r#"
name: "Backward Compatibility Test"
version: "1.0.0"
tasks:
  old_style:
    description: "Old style notification"
    agent: "worker"
    on_complete:
      notify: "Task done!"
  new_style:
    description: "New style notification"
    agent: "worker"
    on_complete:
      notify:
        message: "Task completed"
        channels:
          - type: console
            colored: true
            timestamp: true
"#;
        let workflow = parse_workflow(yaml).unwrap();

        // Verify old style
        let old_task = workflow.tasks.get("old_style").unwrap();
        match old_task
            .on_complete
            .as_ref()
            .unwrap()
            .notify
            .as_ref()
            .unwrap()
        {
            crate::dsl::schema::NotificationSpec::Simple(msg) => {
                assert_eq!(msg, "Task done!");
            }
            _ => panic!("Expected simple notification"),
        }

        // Verify new style
        let new_task = workflow.tasks.get("new_style").unwrap();
        match new_task
            .on_complete
            .as_ref()
            .unwrap()
            .notify
            .as_ref()
            .unwrap()
        {
            crate::dsl::schema::NotificationSpec::Structured { message, .. } => {
                assert_eq!(message, "Task completed");
            }
            _ => panic!("Expected structured notification"),
        }
    }

    // ========================================================================
    // File Operations Tests
    // ========================================================================

    #[test]
    fn test_write_workflow_file() {
        use tempfile::NamedTempFile;

        let yaml = r#"
name: "Test Workflow"
version: "1.0.0"
agents:
  test_agent:
    description: "Test agent"
"#;
        let workflow = parse_workflow(yaml).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Write workflow to file
        write_workflow_file(&workflow, path).unwrap();

        // Read it back and verify
        let read_workflow = parse_workflow_file(path).unwrap();
        assert_eq!(read_workflow.name, workflow.name);
        assert_eq!(read_workflow.version, workflow.version);
    }

    #[test]
    fn test_write_workflow_file_invalid_path() {
        let yaml = r#"
name: "Test"
version: "1.0.0"
"#;
        let workflow = parse_workflow(yaml).unwrap();

        // Try to write to invalid path
        let result = write_workflow_file(&workflow, "/nonexistent/dir/file.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_workflow_file_not_found() {
        let result = parse_workflow_file("/nonexistent/file.yaml");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read workflow file"));
    }

    #[test]
    fn test_merge_subflow_inline() {
        // Test merge_subflow_inline with a YAML workflow
        let yaml = r#"
name: "Main Workflow"
version: "1.0.0"
subflows:
  test_subflow:
    description: "Test subflow"
    agents:
      subflow_agent:
        description: "Subflow agent"
    tasks:
      subflow_task:
        description: "Subflow task"
        agent: "subflow_agent"
"#;
        let mut workflow = parse_workflow(yaml).unwrap();

        // Merge subflow inline
        merge_subflow_inline(&mut workflow, "test_subflow").unwrap();

        // Verify agents were merged with namespacing
        assert!(workflow.agents.contains_key("test_subflow.subflow_agent"));

        // Verify tasks were merged with namespacing
        assert!(workflow.tasks.contains_key("test_subflow.subflow_task"));

        // Verify agent references were updated
        let task = workflow.tasks.get("test_subflow.subflow_task").unwrap();
        assert_eq!(
            task.agent,
            Some("test_subflow.subflow_agent".to_string())
        );
    }

    #[test]
    fn test_merge_subflow_inline_with_dependencies() {
        let yaml = r#"
name: "Main Workflow"
version: "1.0.0"
subflows:
  sub:
    description: "Test subflow"
    tasks:
      task1:
        description: "Task 1"
      task2:
        description: "Task 2"
        depends_on:
          - task1
"#;
        let mut workflow = parse_workflow(yaml).unwrap();

        // Merge subflow
        merge_subflow_inline(&mut workflow, "sub").unwrap();

        // Verify dependencies were namespaced
        let task2 = workflow.tasks.get("sub.task2").unwrap();
        assert_eq!(task2.depends_on, vec!["sub.task1".to_string()]);
    }

    #[test]
    fn test_merge_subflow_inline_nonexistent() {
        let yaml = r#"
name: "Main"
version: "1.0.0"
"#;
        let mut workflow = parse_workflow(yaml).unwrap();

        let result = merge_subflow_inline(&mut workflow, "nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Subflow 'nonexistent' not found"));
    }

    #[tokio::test]
    async fn test_parse_workflow_with_subflows_not_found() {
        let result = parse_workflow_with_subflows("/nonexistent/workflow.yaml").await;
        assert!(result.is_err());
    }
}

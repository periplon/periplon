//! Workflow generation using AI
use super::providers::AiProvider;
use crate::error::Result;

/// Generate a workflow block from natural language description
pub async fn generate_workflow_block(
    provider: &dyn AiProvider,
    description: &str,
) -> Result<String> {
    let system_prompt = r#"You are a Periplon DSL workflow expert. Generate YAML workflow blocks based on user descriptions.

Output only valid YAML that can be directly inserted into a Periplon workflow. Do not include explanations or markdown formatting.

Example format:
```yaml
tasks:
  task_id:
    description: "Description"
    agent: "agent_name"
    depends_on: [dependency]
    inputs:
      key: "value"
```"#;

    let prompt = format!(
        "Generate a Periplon DSL workflow block for the following:\n\n{}",
        description
    );

    let response = provider
        .generate_with_system(system_prompt, &prompt)
        .await?;

    // Extract YAML from response (remove markdown code blocks if present)
    let yaml = extract_yaml(&response.text);

    Ok(yaml)
}

/// Generate a single task definition
pub async fn generate_task(
    provider: &dyn AiProvider,
    task_name: &str,
    description: &str,
) -> Result<String> {
    let system_prompt = r#"You are a Periplon DSL workflow expert. Generate individual task definitions.

Output only valid YAML for a single task. Do not include the 'tasks:' parent key."#;

    let prompt = format!(
        "Generate a task named '{}' with the following description:\n\n{}",
        task_name, description
    );

    let response = provider
        .generate_with_system(system_prompt, &prompt)
        .await?;

    let yaml = extract_yaml(&response.text);

    Ok(yaml)
}

/// Extract YAML content from response, removing markdown code blocks
fn extract_yaml(text: &str) -> String {
    let text = text.trim();

    // Check if wrapped in markdown code blocks
    if text.starts_with("```yaml") || text.starts_with("```") {
        // Find the actual YAML content between code blocks
        let lines: Vec<&str> = text.lines().collect();

        // Skip first line (opening ```)
        let start = if lines.first().map(|l| l.starts_with("```")).unwrap_or(false) {
            1
        } else {
            0
        };

        // Find closing ``` (if exists)
        let end = lines
            .iter()
            .rposition(|l| l.trim() == "```")
            .unwrap_or(lines.len());

        lines[start..end].join("\n")
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_yaml_with_code_blocks() {
        let input = r#"```yaml
tasks:
  my_task:
    description: "Test"
```"#;

        let result = extract_yaml(input);
        assert!(result.contains("tasks:"));
        assert!(!result.contains("```"));
    }

    #[test]
    fn test_extract_yaml_plain() {
        let input = r#"tasks:
  my_task:
    description: "Test""#;

        let result = extract_yaml(input);
        assert_eq!(result, input);
    }
}

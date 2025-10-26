//! Natural Language to DSL Generator
//!
//! This module provides functionality to convert natural language descriptions
//! into DSL workflows using the agent SDK itself.

use crate::adapters::primary::query_fn::query;
use crate::domain::message::{ContentBlock, Message};
use crate::dsl::parser::parse_workflow;
use crate::dsl::schema::DSLWorkflow;
use crate::dsl::template::generate_nl_to_dsl_prompt;
use crate::dsl::validator::validate_workflow;
use crate::error::{Error, Result};
use crate::AgentOptions;
use futures::StreamExt;

/// Maximum number of retry attempts to fix validation errors
const MAX_VALIDATION_RETRIES: usize = 3;

/// Generate a DSL workflow from a natural language description
///
/// Uses the agent SDK to convert natural language into a structured DSL workflow.
/// The agent is given the complete DSL grammar and examples to ensure valid output.
///
/// # Arguments
///
/// * `description` - Natural language description of the desired workflow
/// * `options` - Optional agent configuration (uses defaults if None)
///
/// # Returns
///
/// Result containing the generated workflow or an error
///
/// # Example
///
/// ```no_run
/// use periplon_sdk::dsl::generate_from_nl;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let description = "Create a workflow to research Rust async programming and write a tutorial";
/// let workflow = generate_from_nl(description, None).await?;
/// println!("Generated workflow: {}", workflow.name);
/// # Ok(())
/// # }
/// ```
pub async fn generate_from_nl(
    description: &str,
    options: Option<AgentOptions>,
) -> Result<DSLWorkflow> {
    let mut retry_count = 0;
    let mut last_yaml = None;
    let mut last_error = None;

    loop {
        // Build the prompt - either initial or fix attempt
        let user_prompt = if retry_count == 0 {
            // Initial generation
            let system_prompt = generate_nl_to_dsl_prompt();
            format!("{}\n\n---\n\nUser Request:\n{}", system_prompt, description)
        } else {
            // Fix attempt - include the failed workflow and error
            let system_prompt = generate_nl_to_dsl_prompt();
            let failed_yaml = last_yaml.as_ref().unwrap();
            let error_msg: &String = last_error.as_ref().unwrap();

            eprintln!(
                "⚠️  Validation failed (attempt {}/{}). Attempting to fix...",
                retry_count, MAX_VALIDATION_RETRIES
            );

            // Add specific hints based on the error
            let hints = if error_msg.contains("missing field `name`") {
                "\n\n⚠️ CRITICAL FIX REQUIRED:\nThe workflow is missing the REQUIRED `name` field.\nYou MUST start the workflow with:\nname: \"Workflow Name\"\nversion: \"1.0.0\"\n"
            } else if error_msg.contains("missing field `version`") {
                "\n\n⚠️ CRITICAL FIX REQUIRED:\nThe workflow is missing the REQUIRED `version` field.\nYou MUST include both `name` and `version` at the top:\nname: \"Workflow Name\"\nversion: \"1.0.0\"\n"
            } else if error_msg.contains("missing field `count`") {
                "\n\n⚠️ CRITICAL FIX REQUIRED:\nA loop with type 'repeat' is missing the REQUIRED `count` field.\nExample:\nloop:\n  type: repeat\n  count: 10  # This field is REQUIRED\n"
            } else {
                ""
            };

            format!(
                "{}\n\n---\n\nUser Request:\n{}\n\n---\n\nPREVIOUS ATTEMPT (FAILED VALIDATION):\n```yaml\n{}\n```\n\n---\n\nVALIDATION ERROR:\n{}{}\n\n---\n\nPlease fix the validation errors in the workflow above. Provide the complete corrected workflow in YAML format.",
                system_prompt, description, failed_yaml, error_msg, hints
            )
        };

        // Configure agent options
        let mut agent_opts = options.clone().unwrap_or_default();

        // Ensure the agent doesn't use tools - we just want text generation
        agent_opts.allowed_tools = vec![];
        agent_opts.permission_mode = Some("plan".to_string());

        // Query the agent
        let mut stream = query(&user_prompt, Some(agent_opts)).await?;

        // Collect the response
        let mut yaml_content = String::new();

        while let Some(msg) = stream.next().await {
            match msg {
                Message::Assistant(assistant_msg) => {
                    for block in assistant_msg.message.content {
                        if let ContentBlock::Text { text } = block {
                            yaml_content.push_str(&text);
                        }
                    }
                }
                Message::Result(_) => {
                    // Final result, processing complete
                    break;
                }
                _ => {
                    // Ignore other message types
                }
            }
        }

        // Extract YAML from code blocks if present
        let yaml = match extract_yaml_from_response(&yaml_content) {
            Ok(y) => y,
            Err(e) => {
                // YAML extraction failed
                if retry_count >= MAX_VALIDATION_RETRIES {
                    return Err(e);
                }

                eprintln!(
                    "⚠️  YAML extraction failed (attempt {}/{}). Attempting to fix...",
                    retry_count + 1,
                    MAX_VALIDATION_RETRIES
                );

                last_yaml = Some(yaml_content.clone());
                last_error = Some(format!("YAML extraction error: {}", e));
                retry_count += 1;
                continue;
            }
        };

        // Parse the generated YAML into a workflow
        let workflow = match parse_workflow(&yaml) {
            Ok(w) => w,
            Err(e) => {
                // Parsing failed
                if retry_count >= MAX_VALIDATION_RETRIES {
                    return Err(Error::InvalidInput(format!(
                        "Workflow parsing failed after {} attempts. Last error:\n{}",
                        MAX_VALIDATION_RETRIES, e
                    )));
                }

                eprintln!(
                    "⚠️  Parsing failed (attempt {}/{}). Attempting to fix...",
                    retry_count + 1,
                    MAX_VALIDATION_RETRIES
                );

                // Store the failed attempt and error for next iteration
                last_yaml = Some(yaml);
                last_error = Some(format!("Parsing error: {}", e));
                retry_count += 1;
                continue;
            }
        };

        // Validate the workflow
        match validate_workflow(&workflow) {
            Ok(_) => {
                // Validation succeeded!
                if retry_count > 0 {
                    eprintln!(
                        "✓ Validation succeeded after {} retry attempt(s)",
                        retry_count
                    );
                }
                return Ok(workflow);
            }
            Err(e) => {
                // Validation failed
                if retry_count >= MAX_VALIDATION_RETRIES {
                    // Max retries reached, return the error
                    return Err(Error::InvalidInput(format!(
                        "Workflow validation failed after {} attempts. Last error:\n{}",
                        MAX_VALIDATION_RETRIES, e
                    )));
                }

                eprintln!(
                    "⚠️  Validation failed (attempt {}/{}). Attempting to fix...",
                    retry_count + 1,
                    MAX_VALIDATION_RETRIES
                );

                // Store the failed attempt and error for next iteration
                last_yaml = Some(yaml);
                last_error = Some(format!("Validation error: {}", e));
                retry_count += 1;
            }
        }
    }
}

/// Extract YAML content from agent response
///
/// Handles responses that may be wrapped in markdown code blocks
fn extract_yaml_from_response(response: &str) -> Result<String> {
    let trimmed = response.trim();

    // Check for YAML code blocks
    if let Some(yaml_start) = trimmed.find("```yaml") {
        let after_start = &trimmed[yaml_start + 7..]; // Skip "```yaml"
        if let Some(yaml_end) = after_start.find("```") {
            return Ok(after_start[..yaml_end].trim().to_string());
        }
    }

    // Check for generic code blocks
    if let Some(code_start) = trimmed.find("```") {
        let after_start = &trimmed[code_start + 3..];
        // Skip language identifier if present
        let content_start = if let Some(newline) = after_start.find('\n') {
            newline + 1
        } else {
            0
        };
        let after_start = &after_start[content_start..];

        if let Some(code_end) = after_start.find("```") {
            return Ok(after_start[..code_end].trim().to_string());
        }
    }

    // If no code blocks, assume the entire response is YAML
    if trimmed.is_empty() {
        return Err(Error::InvalidInput("Agent response was empty".to_string()));
    }

    Ok(trimmed.to_string())
}

/// Generate a DSL workflow from a natural language description and save to file
///
/// Convenience function that generates and saves the workflow in one step.
///
/// # Arguments
///
/// * `description` - Natural language description
/// * `output_path` - Path to save the generated YAML file
/// * `options` - Optional agent configuration
///
/// # Returns
///
/// Result containing the generated workflow or an error
pub async fn generate_and_save(
    description: &str,
    output_path: &str,
    options: Option<AgentOptions>,
) -> Result<DSLWorkflow> {
    let workflow = generate_from_nl(description, options).await?;

    // Save to file
    crate::dsl::parser::write_workflow_file(&workflow, output_path)?;

    Ok(workflow)
}

/// Modify an existing DSL workflow using natural language instructions
///
/// Takes an existing workflow YAML and natural language modification instructions,
/// then generates an updated version of the workflow.
///
/// # Arguments
///
/// * `modification_description` - Natural language describing the modifications to make
/// * `existing_workflow_yaml` - The current workflow YAML content as a string
/// * `options` - Optional agent configuration (uses defaults if None)
///
/// # Returns
///
/// Result containing the modified workflow or an error
///
/// # Example
///
/// ```no_run
/// use periplon_sdk::dsl::nl_generator::modify_workflow_from_nl;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let existing_yaml = std::fs::read_to_string("workflow.yaml")?;
/// let modifications = "Add a new task to validate the output files";
/// let updated_workflow = modify_workflow_from_nl(modifications, &existing_yaml, None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn modify_workflow_from_nl(
    modification_description: &str,
    existing_workflow_yaml: &str,
    options: Option<AgentOptions>,
) -> Result<DSLWorkflow> {
    let mut retry_count = 0;
    let mut last_yaml = None;
    let mut last_error = None;

    loop {
        // Build the prompt - either initial modification or fix attempt
        let user_prompt = if retry_count == 0 {
            // Initial modification
            let system_prompt = generate_nl_to_dsl_prompt();
            format!(
                "{}\n\n---\n\nExisting Workflow (YAML):\n```yaml\n{}\n```\n\n---\n\nModification Request:\n{}\n\nPlease provide the complete modified workflow in YAML format. Make only the requested changes while preserving all other aspects of the workflow.",
                system_prompt, existing_workflow_yaml, modification_description
            )
        } else {
            // Fix attempt - include the failed workflow and error
            let system_prompt = generate_nl_to_dsl_prompt();
            let failed_yaml = last_yaml.as_ref().unwrap();
            let error_msg = last_error.as_ref().unwrap();

            eprintln!(
                "⚠️  Validation failed (attempt {}/{}). Attempting to fix...",
                retry_count, MAX_VALIDATION_RETRIES
            );

            format!(
                "{}\n\n---\n\nOriginal Workflow:\n```yaml\n{}\n```\n\n---\n\nModification Request:\n{}\n\n---\n\nPREVIOUS ATTEMPT (FAILED VALIDATION):\n```yaml\n{}\n```\n\n---\n\nVALIDATION ERROR:\n{}\n\n---\n\nPlease fix the validation errors in the modified workflow above while still applying the requested modifications. Provide the complete corrected workflow in YAML format.",
                system_prompt, existing_workflow_yaml, modification_description, failed_yaml, error_msg
            )
        };

        // Configure agent options
        let mut agent_opts = options.clone().unwrap_or_default();

        // Ensure the agent doesn't use tools - we just want text generation
        agent_opts.allowed_tools = vec![];
        agent_opts.permission_mode = Some("plan".to_string());

        // Query the agent
        let mut stream = query(&user_prompt, Some(agent_opts)).await?;

        // Collect the response
        let mut yaml_content = String::new();

        while let Some(msg) = stream.next().await {
            match msg {
                Message::Assistant(assistant_msg) => {
                    for block in assistant_msg.message.content {
                        if let ContentBlock::Text { text } = block {
                            yaml_content.push_str(&text);
                        }
                    }
                }
                Message::Result(_) => {
                    // Final result, processing complete
                    break;
                }
                _ => {
                    // Ignore other message types
                }
            }
        }

        // Extract YAML from code blocks if present
        let yaml = match extract_yaml_from_response(&yaml_content) {
            Ok(y) => y,
            Err(e) => {
                // YAML extraction failed
                if retry_count >= MAX_VALIDATION_RETRIES {
                    return Err(e);
                }

                eprintln!(
                    "⚠️  YAML extraction failed (attempt {}/{}). Attempting to fix...",
                    retry_count + 1,
                    MAX_VALIDATION_RETRIES
                );

                last_yaml = Some(yaml_content.clone());
                last_error = Some(format!("YAML extraction error: {}", e));
                retry_count += 1;
                continue;
            }
        };

        // Parse the modified YAML into a workflow
        let workflow = match parse_workflow(&yaml) {
            Ok(w) => w,
            Err(e) => {
                // Parsing failed
                if retry_count >= MAX_VALIDATION_RETRIES {
                    return Err(Error::InvalidInput(format!(
                        "Workflow parsing failed after {} attempts. Last error:\n{}",
                        MAX_VALIDATION_RETRIES, e
                    )));
                }

                eprintln!(
                    "⚠️  Parsing failed (attempt {}/{}). Attempting to fix...",
                    retry_count + 1,
                    MAX_VALIDATION_RETRIES
                );

                // Store the failed attempt and error for next iteration
                last_yaml = Some(yaml);
                last_error = Some(format!("Parsing error: {}", e));
                retry_count += 1;
                continue;
            }
        };

        // Validate the workflow
        match validate_workflow(&workflow) {
            Ok(_) => {
                // Validation succeeded!
                if retry_count > 0 {
                    eprintln!(
                        "✓ Validation succeeded after {} retry attempt(s)",
                        retry_count
                    );
                }
                return Ok(workflow);
            }
            Err(e) => {
                // Validation failed
                if retry_count >= MAX_VALIDATION_RETRIES {
                    // Max retries reached, return the error
                    return Err(Error::InvalidInput(format!(
                        "Workflow validation failed after {} attempts. Last error:\n{}",
                        MAX_VALIDATION_RETRIES, e
                    )));
                }

                eprintln!(
                    "⚠️  Validation failed (attempt {}/{}). Attempting to fix...",
                    retry_count + 1,
                    MAX_VALIDATION_RETRIES
                );

                // Store the failed attempt and error for next iteration
                last_yaml = Some(yaml);
                last_error = Some(format!("Validation error: {}", e));
                retry_count += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_yaml_from_yaml_block() {
        let response = r#"
Here's the workflow:

```yaml
name: "Test"
version: "1.0.0"
```

Hope this helps!
        "#;

        let yaml = extract_yaml_from_response(response).unwrap();
        assert!(yaml.contains("name: \"Test\""));
        assert!(yaml.contains("version: \"1.0.0\""));
        assert!(!yaml.contains("Here's"));
        assert!(!yaml.contains("Hope this"));
    }

    #[test]
    fn test_extract_yaml_from_generic_block() {
        let response = r#"
```
name: "Test"
version: "1.0.0"
```
        "#;

        let yaml = extract_yaml_from_response(response).unwrap();
        assert!(yaml.contains("name: \"Test\""));
    }

    #[test]
    fn test_extract_yaml_no_blocks() {
        let response = r#"
name: "Test"
version: "1.0.0"
        "#;

        let yaml = extract_yaml_from_response(response).unwrap();
        assert!(yaml.contains("name: \"Test\""));
        assert!(yaml.contains("version: \"1.0.0\""));
    }

    #[test]
    fn test_extract_yaml_empty_response() {
        let response = "   ";
        let result = extract_yaml_from_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_yaml_with_language_identifier() {
        let response = r#"
```yaml
name: "Test"
version: "1.0.0"
agents:
  test:
    description: "Test agent"
```
        "#;

        let yaml = extract_yaml_from_response(response).unwrap();
        assert!(yaml.contains("agents:"));
        assert!(!yaml.contains("```"));
    }
}

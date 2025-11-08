//! AI-powered debugging suggestions
use super::providers::AiProvider;
use crate::error::Result;

/// Suggest a fix for an error
pub async fn suggest_fix(provider: &dyn AiProvider, error: &str, context: &str) -> Result<String> {
    let system_prompt = r#"You are a Periplon DSL debugging expert. Analyze errors and suggest fixes.

Provide clear, actionable suggestions. Format your response as:
1. Root cause analysis
2. Suggested fix (with code if applicable)
3. Prevention tips"#;

    let prompt = format!(
        "Error:\n{}\n\nContext:\n{}\n\nPlease analyze this error and suggest a fix.",
        error, context
    );

    let response = provider
        .generate_with_system(system_prompt, &prompt)
        .await?;

    Ok(response.text)
}

/// Analyze an error and provide insights
pub async fn analyze_error(provider: &dyn AiProvider, error: &str) -> Result<String> {
    let system_prompt = r#"You are a Periplon DSL expert. Analyze errors and explain them clearly.

Provide:
1. What went wrong
2. Why it happened
3. How to fix it"#;

    let prompt = format!("Analyze this error:\n\n{}", error);

    let response = provider
        .generate_with_system(system_prompt, &prompt)
        .await?;

    Ok(response.text)
}

/// Suggest improvements for a workflow
pub async fn suggest_improvements(
    provider: &dyn AiProvider,
    workflow_yaml: &str,
) -> Result<String> {
    let system_prompt = r#"You are a Periplon DSL expert. Review workflows and suggest improvements.

Focus on:
1. Error handling
2. Task organization
3. Dependency optimization
4. Best practices"#;

    let prompt = format!(
        "Review this workflow and suggest improvements:\n\n{}",
        workflow_yaml
    );

    let response = provider
        .generate_with_system(system_prompt, &prompt)
        .await?;

    Ok(response.text)
}

/// Explain what a workflow does
pub async fn explain_workflow(provider: &dyn AiProvider, workflow_yaml: &str) -> Result<String> {
    let system_prompt = r#"You are a Periplon DSL expert. Explain workflows in clear, simple language.

Provide:
1. High-level purpose
2. Task breakdown
3. Data flow
4. Key features"#;

    let prompt = format!("Explain what this workflow does:\n\n{}", workflow_yaml);

    let response = provider
        .generate_with_system(system_prompt, &prompt)
        .await?;

    Ok(response.text)
}

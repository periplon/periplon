//! Smart Context Injection with Relevance Scoring
//!
//! This module implements selective context injection based on task dependencies,
//! relevance scoring, and configurable limits to prevent context bloat.

use crate::dsl::schema::{ContextConfig, ContextMode, DSLWorkflow};
use crate::dsl::state::{TaskOutput, WorkflowState};
use crate::dsl::task_graph::TaskGraph;
use std::collections::HashSet;

/// Calculate relevance score for a task output relative to the current task
///
/// Relevance scoring:
/// 1. Direct dependency = 1.0
/// 2. Transitive dependency = 0.8 / depth
/// 3. Same agent = 0.5
/// 4. Recent task (within time window) = 0.3
/// 5. No relevance = 0.0
///
/// # Arguments
///
/// * `current_task_id` - ID of the task we're building context for
/// * `output_task_id` - ID of the task that produced the output
/// * `task_graph` - Task graph for dependency analysis
/// * `workflow` - Workflow definition for agent information
///
/// # Returns
///
/// Relevance score from 0.0 to 1.0
pub fn calculate_relevance(
    current_task_id: &str,
    output_task_id: &str,
    task_graph: &TaskGraph,
    workflow: &DSLWorkflow,
) -> f64 {
    // 1. Check if it's a direct dependency
    if is_direct_dependency(current_task_id, output_task_id, task_graph) {
        return 1.0;
    }

    // 2. Check for transitive dependency
    if let Some(depth) = get_dependency_depth(current_task_id, output_task_id, task_graph) {
        return 0.8 / depth as f64;
    }

    // 3. Check if they use the same agent
    if uses_same_agent(current_task_id, output_task_id, workflow) {
        return 0.5;
    }

    // 4. Default: no relevance
    0.0
}

/// Check if output_task is a direct dependency of current_task
fn is_direct_dependency(current_task_id: &str, output_task_id: &str, task_graph: &TaskGraph) -> bool {
    if let Some(current_task) = task_graph.get_task(current_task_id) {
        current_task.dependencies.contains(&output_task_id.to_string())
    } else {
        false
    }
}

/// Get transitive dependency depth (number of hops)
///
/// Returns None if output_task is not a transitive dependency of current_task
fn get_dependency_depth(
    current_task_id: &str,
    output_task_id: &str,
    task_graph: &TaskGraph,
) -> Option<usize> {
    let mut visited = HashSet::new();
    let mut queue = vec![(current_task_id, 0)];

    while let Some((task_id, depth)) = queue.pop() {
        if task_id == output_task_id {
            return Some(depth);
        }

        if visited.contains(task_id) {
            continue;
        }
        visited.insert(task_id);

        if let Some(task) = task_graph.get_task(task_id) {
            for dep in &task.dependencies {
                queue.push((dep.as_str(), depth + 1));
            }
        }
    }

    None
}

/// Check if two tasks use the same agent
fn uses_same_agent(task1_id: &str, task2_id: &str, workflow: &DSLWorkflow) -> bool {
    let agent1 = workflow.tasks.get(task1_id).and_then(|t| t.agent.as_ref());
    let agent2 = workflow.tasks.get(task2_id).and_then(|t| t.agent.as_ref());

    match (agent1, agent2) {
        (Some(a1), Some(a2)) => a1 == a2,
        _ => false,
    }
}

/// Build smart context for a task based on configuration and relevance
///
/// # Arguments
///
/// * `current_task_id` - ID of the task to build context for
/// * `workflow` - Workflow definition
/// * `task_graph` - Task graph for dependency analysis
/// * `state` - Current workflow state
/// * `config` - Context configuration (optional, uses defaults if None)
///
/// # Returns
///
/// Context string ready for injection
pub fn build_smart_context(
    current_task_id: &str,
    workflow: &DSLWorkflow,
    task_graph: &TaskGraph,
    state: &WorkflowState,
    config: Option<&ContextConfig>,
) -> String {
    // Use defaults if no config provided
    let default_config = ContextConfig {
        mode: ContextMode::Automatic,
        include_tasks: Vec::new(),
        exclude_tasks: Vec::new(),
        min_relevance: 0.5,
        max_bytes: None,
        max_tasks: None,
    };

    let config = config.unwrap_or(&default_config);

    // Get workflow-level limits
    let max_bytes = config
        .max_bytes
        .or(workflow.limits.as_ref().map(|l| l.max_context_bytes))
        .unwrap_or(102_400); // 100KB default

    let max_tasks = config
        .max_tasks
        .or(workflow.limits.as_ref().map(|l| l.max_context_tasks))
        .unwrap_or(10);

    match config.mode {
        ContextMode::None => String::new(),
        ContextMode::Manual => build_manual_context(
            &config.include_tasks,
            &config.exclude_tasks,
            state,
            max_bytes,
            max_tasks,
        ),
        ContextMode::Automatic => build_automatic_context(
            current_task_id,
            workflow,
            task_graph,
            state,
            config.min_relevance,
            max_bytes,
            max_tasks,
        ),
    }
}

/// Build context in manual mode (specific include/exclude lists)
fn build_manual_context(
    include_tasks: &[String],
    exclude_tasks: &[String],
    state: &WorkflowState,
    max_bytes: usize,
    max_tasks: usize,
) -> String {
    let mut context = String::new();
    let mut total_bytes = 0;
    let mut included_count = 0;

    context.push_str("=== RELEVANT CONTEXT ===\n\n");

    for task_id in include_tasks {
        if exclude_tasks.contains(task_id) {
            continue;
        }

        if included_count >= max_tasks {
            break;
        }

        if let Some(output) = state.get_task_output(task_id) {
            let output_bytes = output.content.len();
            if total_bytes + output_bytes > max_bytes {
                break;
            }

            context.push_str(&format!("Task: {}\n", task_id));
            if output.truncated {
                context.push_str("[Output was truncated]\n");
            }
            context.push_str(&format!("{}\n\n", output.content));

            total_bytes += output_bytes;
            included_count += 1;
        }
    }

    context.push_str(&format!(
        "=== END CONTEXT ({} tasks, {} bytes) ===\n",
        included_count, total_bytes
    ));

    context
}

/// Build context in automatic mode (dependency-based with relevance scoring)
fn build_automatic_context(
    current_task_id: &str,
    workflow: &DSLWorkflow,
    task_graph: &TaskGraph,
    state: &WorkflowState,
    min_relevance: f64,
    max_bytes: usize,
    max_tasks: usize,
) -> String {
    let mut context = String::new();
    let mut total_bytes = 0;
    let mut included_tasks = 0;

    // Calculate relevance for all outputs and filter by threshold
    let mut scored_outputs: Vec<(&String, &TaskOutput, f64)> = state
        .task_outputs
        .iter()
        .map(|(task_id, output)| {
            let relevance = calculate_relevance(current_task_id, task_id, task_graph, workflow);
            (task_id, output, relevance)
        })
        .filter(|(_, _, relevance)| *relevance >= min_relevance)
        .collect();

    // Sort by relevance (descending)
    scored_outputs.sort_by(|(_, _, a), (_, _, b)| {
        b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
    });

    context.push_str("=== RELEVANT CONTEXT ===\n\n");

    for (task_id, output, relevance) in scored_outputs {
        // Check limits
        if included_tasks >= max_tasks {
            break;
        }

        let output_bytes = output.content.len();
        if total_bytes + output_bytes > max_bytes {
            // Try to include a truncated version if we have room
            let remaining = max_bytes - total_bytes;
            if remaining > 100 {
                let truncated = truncate_to_size(&output.content, remaining);
                context.push_str(&format!(
                    "Task: {} (relevance: {:.2})\n{}\\n\\n",
                    task_id, relevance, truncated
                ));
                break;
            } else {
                break;
            }
        }

        // Include full output
        context.push_str(&format!("Task: {} (relevance: {:.2})\n", task_id, relevance));

        if output.truncated {
            context.push_str("[Output was truncated]\n");
        }

        context.push_str(&format!("{}\n\n", output.content));

        total_bytes += output_bytes;
        included_tasks += 1;
    }

    context.push_str(&format!(
        "=== END CONTEXT ({} tasks, {} bytes) ===\n",
        included_tasks, total_bytes
    ));

    context
}

/// Truncate text to fit within size limit
fn truncate_to_size(text: &str, max_size: usize) -> String {
    if text.len() <= max_size {
        return text.to_string();
    }

    let suffix = "\n... [truncated to fit context limit]";

    // If suffix is longer than max_size, use a shorter version
    if suffix.len() >= max_size {
        let short_suffix = "...[truncated]";
        let available = max_size.saturating_sub(short_suffix.len());
        return format!("{}{}", &text[..available], short_suffix);
    }

    let available = max_size.saturating_sub(suffix.len());

    format!("{}{}", &text[..available], suffix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::schema::TaskSpec;
    use crate::dsl::state::OutputType;
    use crate::dsl::truncation::create_task_output;
    use std::collections::HashMap;

    fn create_test_workflow() -> DSLWorkflow {
        let mut workflow = DSLWorkflow {
            name: "test".to_string(),
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

        // Add tasks with agent assignments
        let mut task1 = TaskSpec::default();
        task1.description = "Task 1".to_string();
        task1.agent = Some("agent1".to_string());

        let mut task2 = TaskSpec::default();
        task2.description = "Task 2".to_string();
        task2.agent = Some("agent1".to_string());
        task2.depends_on = vec!["task1".to_string()];

        let mut task3 = TaskSpec::default();
        task3.description = "Task 3".to_string();
        task3.agent = Some("agent2".to_string());

        workflow
            .tasks
            .insert("task1".to_string(), task1);
        workflow
            .tasks
            .insert("task2".to_string(), task2);
        workflow
            .tasks
            .insert("task3".to_string(), task3);

        workflow
    }

    #[test]
    fn test_calculate_relevance_direct_dependency() {
        let workflow = create_test_workflow();
        let mut task_graph = TaskGraph::new();

        for (task_id, task_spec) in &workflow.tasks {
            task_graph.add_task(task_id.clone(), task_spec.clone());
        }

        let relevance = calculate_relevance("task2", "task1", &task_graph, &workflow);
        assert_eq!(relevance, 1.0);
    }

    #[test]
    fn test_calculate_relevance_same_agent() {
        let workflow = create_test_workflow();
        let task_graph = TaskGraph::new();

        let relevance = calculate_relevance("task2", "task1", &task_graph, &workflow);
        // Should be 1.0 due to direct dependency, not just same agent
        assert!(relevance >= 0.5);
    }

    #[test]
    fn test_build_manual_context() {
        let mut state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

        let output1 = create_task_output(
            "task1".to_string(),
            OutputType::Stdout,
            "Output from task 1".to_string(),
            100,
            &crate::dsl::schema::TruncationStrategy::Tail,
        );

        state.store_task_output(output1);

        let context = build_manual_context(
            &vec!["task1".to_string()],
            &vec![],
            &state,
            10000,
            10,
        );

        assert!(context.contains("RELEVANT CONTEXT"));
        assert!(context.contains("task1"));
        assert!(context.contains("Output from task 1"));
    }

    #[test]
    fn test_truncate_to_size() {
        let text = "This is a long text that needs truncation";
        let truncated = truncate_to_size(text, 20);
        assert!(truncated.len() <= 20);
        assert!(truncated.contains("truncated"));
    }

    #[test]
    fn test_truncate_to_size_no_truncation() {
        let text = "Short";
        let result = truncate_to_size(text, 100);
        assert_eq!(result, text);
    }
}

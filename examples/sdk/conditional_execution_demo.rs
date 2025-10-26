//! Conditional Task Execution Demo
//!
//! This example demonstrates how to use conditional task execution in the DSL.
//! It shows tasks that run based on the status of other tasks and workflow state.

use periplon_sdk::dsl::{
    parse_workflow, Condition, ConditionSpec, TaskSpec, TaskStatusCondition,
};
use periplon_sdk::error::Result;

fn main() -> Result<()> {
    println!("=== Conditional Task Execution Demo ===\n");

    // Example 1: Simple task status condition
    println!("Example 1: Task runs only if previous task completed");
    let task_with_condition = create_conditional_task(
        "deploy",
        "Deploy to production",
        ConditionSpec::Single(Condition::TaskStatus {
            task: "tests".to_string(),
            status: TaskStatusCondition::Completed,
        }),
    );
    println!("  Task: {}", task_with_condition.description);
    println!("  Condition: Run if 'tests' completed\n");

    // Example 2: State-based condition
    println!("Example 2: Task runs based on workflow state");
    let state_condition_task = create_conditional_task(
        "prod_deploy",
        "Deploy to production environment",
        ConditionSpec::Single(Condition::StateEquals {
            key: "environment".to_string(),
            value: serde_json::json!("production"),
        }),
    );
    println!("  Task: {}", state_condition_task.description);
    println!("  Condition: Run if environment == 'production'\n");

    // Example 3: AND condition
    println!("Example 3: Task with AND condition");
    let and_condition_task = create_conditional_task(
        "secure_deploy",
        "Deploy with security checks",
        ConditionSpec::And {
            and: vec![
                ConditionSpec::Single(Condition::TaskStatus {
                    task: "tests".to_string(),
                    status: TaskStatusCondition::Completed,
                }),
                ConditionSpec::Single(Condition::TaskStatus {
                    task: "security_scan".to_string(),
                    status: TaskStatusCondition::Completed,
                }),
            ],
        },
    );
    println!("  Task: {}", and_condition_task.description);
    println!("  Condition: Run if tests AND security_scan completed\n");

    // Example 4: OR condition for failure handling
    println!("Example 4: Task with OR condition for failure handling");
    let or_condition_task = create_conditional_task(
        "notify_failure",
        "Send failure notification",
        ConditionSpec::Or {
            or: vec![
                ConditionSpec::Single(Condition::TaskStatus {
                    task: "build".to_string(),
                    status: TaskStatusCondition::Failed,
                }),
                ConditionSpec::Single(Condition::TaskStatus {
                    task: "tests".to_string(),
                    status: TaskStatusCondition::Failed,
                }),
            ],
        },
    );
    println!("  Task: {}", or_condition_task.description);
    println!("  Condition: Run if build OR tests failed\n");

    // Example 5: NOT condition
    println!("Example 5: Task with NOT condition");
    let not_condition_task = create_conditional_task(
        "staging_deploy",
        "Deploy to staging (non-production)",
        ConditionSpec::Not {
            not: Box::new(ConditionSpec::Single(Condition::StateEquals {
                key: "environment".to_string(),
                value: serde_json::json!("production"),
            })),
        },
    );
    println!("  Task: {}", not_condition_task.description);
    println!("  Condition: Run if NOT production environment\n");

    // Example 6: Complex nested condition
    println!("Example 6: Complex nested condition");
    let complex_condition_task = create_conditional_task(
        "conditional_rollback",
        "Rollback if deployment failed or tests failed in production",
        ConditionSpec::And {
            and: vec![
                ConditionSpec::Or {
                    or: vec![
                        ConditionSpec::Single(Condition::TaskStatus {
                            task: "deploy".to_string(),
                            status: TaskStatusCondition::Failed,
                        }),
                        ConditionSpec::Single(Condition::TaskStatus {
                            task: "smoke_tests".to_string(),
                            status: TaskStatusCondition::Failed,
                        }),
                    ],
                },
                ConditionSpec::Single(Condition::StateEquals {
                    key: "environment".to_string(),
                    value: serde_json::json!("production"),
                }),
            ],
        },
    );
    println!("  Task: {}", complex_condition_task.description);
    println!("  Condition: (deploy failed OR smoke_tests failed) AND production environment\n");

    // Parse the example workflow file
    println!("=== Loading Example Workflow ===");
    match parse_workflow(include_str!("workflows/conditional_tasks.yaml")) {
        Ok(workflow) => {
            println!("✓ Successfully loaded workflow: {}", workflow.name);
            println!("  Version: {}", workflow.version);
            println!("  Total tasks: {}", workflow.tasks.len());

            // Count conditional tasks
            let conditional_count = workflow
                .tasks
                .values()
                .filter(|t| t.condition.is_some())
                .count();
            println!("  Conditional tasks: {}", conditional_count);
        }
        Err(e) => {
            eprintln!("✗ Failed to load workflow: {}", e);
        }
    }

    println!("\n=== Key Features ===");
    println!("✓ Task status conditions (completed, failed, skipped, running, pending)");
    println!("✓ State-based conditions (equals, exists)");
    println!("✓ Logical operators (AND, OR, NOT)");
    println!("✓ Complex nested conditions");
    println!("✓ Skipped tasks don't block dependent tasks");
    println!("\nSee examples/workflows/conditional_tasks.yaml for a complete CI/CD example");
    println!("See docs/conditional-tasks.md for detailed documentation");

    Ok(())
}

fn create_conditional_task(_name: &str, description: &str, condition: ConditionSpec) -> TaskSpec {
    TaskSpec {
        description: description.to_string(),
        agent: Some("example_agent".to_string()),
        condition: Some(condition),
        ..Default::default()
    }
}

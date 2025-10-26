//! Test to verify that parent tasks with subtasks don't require any execution attributes
//!
//! This test ensures that when a task has subtasks, it doesn't need to specify
//! an agent or any other execution type - it's purely a grouping mechanism.

use periplon_sdk::dsl::{parse_workflow, validate_workflow, DSLExecutor};

#[test]
fn test_parent_task_without_agent_is_valid() {
    let yaml = r#"
name: "Parent Without Agent Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools:
      - Read
    permissions:
      mode: "default"

tasks:
  # Parent task with NO agent - should be valid because it has subtasks
  parent_no_agent:
    description: "Parent task without agent"
    subtasks:
      - child1:
          description: "Child task 1"
          agent: "test_agent"
      - child2:
          description: "Child task 2"
          agent: "test_agent"
"#;

    let workflow = parse_workflow(yaml).unwrap();

    // Validation should pass - parent doesn't need an agent if it has subtasks
    let result = validate_workflow(&workflow);
    assert!(
        result.is_ok(),
        "Parent task without agent should be valid when it has subtasks: {:?}",
        result
    );
}

#[tokio::test]
async fn test_parent_task_minimal_attributes() {
    let yaml = r#"
name: "Minimal Parent Test"
version: "1.0.0"

agents:
  worker:
    description: "Worker agent"
    tools:
      - Read
    permissions:
      mode: "default"

tasks:
  # Parent task with ONLY description and subtasks - nothing else!
  minimal_parent:
    description: "Minimal parent"
    subtasks:
      - do_work:
          description: "Actually do the work"
          agent: "worker"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    validate_workflow(&workflow).unwrap();

    let mut executor = DSLExecutor::new(workflow).unwrap();
    executor.initialize().await.unwrap();

    let task_graph = executor.task_graph();
    let all_tasks = task_graph.get_all_tasks();

    println!("Tasks in execution graph: {:?}", all_tasks);

    // Should have only 1 task (the child)
    assert_eq!(all_tasks.len(), 1);

    // Parent should NOT be in the graph
    assert!(
        !all_tasks.contains(&"minimal_parent".to_string()),
        "Parent task should NOT be in execution graph"
    );

    // Child should be in the graph
    assert!(
        all_tasks.contains(&"minimal_parent.do_work".to_string()),
        "Child task should be in execution graph"
    );
}

#[test]
fn test_parent_task_only_description_required() {
    let yaml = r#"
name: "Description Only Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools:
      - Read
    permissions:
      mode: "default"

tasks:
  # Parent with only description - no agent, no priority, no other attributes
  parent:
    description: "Just a description"
    subtasks:
      - child:
          description: "Child task"
          agent: "test_agent"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    let result = validate_workflow(&workflow);

    assert!(
        result.is_ok(),
        "Parent task with only description should be valid: {:?}",
        result
    );
}

#[test]
fn test_nested_parents_no_agents() {
    let yaml = r#"
name: "Nested Parents Test"
version: "1.0.0"

agents:
  worker:
    description: "Worker agent"
    tools:
      - Read
    permissions:
      mode: "default"

tasks:
  # Grandparent - no agent
  grandparent:
    description: "Grandparent level"
    subtasks:
      # Parent - no agent
      - parent:
          description: "Parent level"
          subtasks:
            # Only the leaf has an agent
            - leaf:
                description: "Leaf task"
                agent: "worker"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    let result = validate_workflow(&workflow);

    assert!(
        result.is_ok(),
        "Nested parent tasks without agents should be valid: {:?}",
        result
    );
}

#[test]
fn test_leaf_task_without_agent_is_invalid() {
    let yaml = r#"
name: "Leaf Without Agent Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools:
      - Read
    permissions:
      mode: "default"

tasks:
  # This leaf task has NO agent and NO subtasks - should be INVALID
  leaf_no_agent:
    description: "Leaf task without agent or execution type"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    let result = validate_workflow(&workflow);

    assert!(
        result.is_err(),
        "Leaf task without agent or execution type should be invalid"
    );

    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("must specify an execution type"),
        "Error should mention execution type requirement: {}",
        error_msg
    );
}

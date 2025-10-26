//! Test to verify that parent tasks with subtasks NEVER execute
//!
//! This test ensures that when a task has subtasks, it acts only as a
//! grouping/template mechanism and is NOT added to the execution graph,
//! regardless of whether it has an agent assigned.

use periplon_sdk::dsl::{parse_workflow, DSLExecutor};

#[tokio::test]
async fn test_parent_task_with_subtasks_does_not_execute() {
    let yaml = r#"
name: "Parent Task Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools:
      - Read
    permissions:
      mode: "default"

tasks:
  # This parent task has BOTH an agent AND subtasks
  # It should NOT be added to the execution graph
  parent_with_agent:
    description: "Parent task with agent"
    agent: "test_agent"
    subtasks:
      - child1:
          description: "Child task 1"
          agent: "test_agent"
      - child2:
          description: "Child task 2"
          agent: "test_agent"

  # This is a leaf task (no subtasks) so it SHOULD execute
  standalone_task:
    description: "Standalone task"
    agent: "test_agent"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    let mut executor = DSLExecutor::new(workflow).unwrap();
    executor.initialize().await.unwrap();

    // Access the task graph
    let task_graph = executor.task_graph();

    // The task graph should contain:
    // - parent_with_agent.child1 (executable)
    // - parent_with_agent.child2 (executable)
    // - standalone_task (executable)
    // Total: 3 tasks
    //
    // It should NOT contain:
    // - parent_with_agent (not executable because it has subtasks)

    let all_tasks = task_graph.get_all_tasks();

    println!("Tasks in execution graph: {:?}", all_tasks);

    // Should have exactly 3 tasks
    assert_eq!(
        all_tasks.len(),
        3,
        "Expected 3 executable tasks (2 children + 1 standalone), got {}",
        all_tasks.len()
    );

    // Parent task should NOT be in the graph
    assert!(
        !all_tasks.contains(&"parent_with_agent".to_string()),
        "Parent task 'parent_with_agent' should NOT be in execution graph because it has subtasks"
    );

    // Child tasks should be in the graph
    assert!(
        all_tasks.contains(&"parent_with_agent.child1".to_string()),
        "Child task 'parent_with_agent.child1' should be in execution graph"
    );
    assert!(
        all_tasks.contains(&"parent_with_agent.child2".to_string()),
        "Child task 'parent_with_agent.child2' should be in execution graph"
    );

    // Standalone task should be in the graph
    assert!(
        all_tasks.contains(&"standalone_task".to_string()),
        "Standalone task should be in execution graph"
    );
}

#[tokio::test]
async fn test_nested_parent_tasks_do_not_execute() {
    let yaml = r#"
name: "Nested Parent Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools:
      - Read
    permissions:
      mode: "default"

tasks:
  # Grandparent task with agent and subtasks
  grandparent:
    description: "Grandparent task"
    agent: "test_agent"
    subtasks:
      # Parent task with agent and subtasks
      - parent:
          description: "Parent task"
          agent: "test_agent"
          subtasks:
            # Leaf task - this should execute
            - child:
                description: "Child task"
                agent: "test_agent"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    let mut executor = DSLExecutor::new(workflow).unwrap();
    executor.initialize().await.unwrap();

    let task_graph = executor.task_graph();
    let all_tasks = task_graph.get_all_tasks();

    println!("Tasks in execution graph: {:?}", all_tasks);

    // Should have exactly 1 task (only the leaf child task)
    assert_eq!(
        all_tasks.len(),
        1,
        "Expected 1 executable task (only the leaf child), got {}",
        all_tasks.len()
    );

    // Neither grandparent nor parent should be in the graph
    assert!(
        !all_tasks.contains(&"grandparent".to_string()),
        "Grandparent task should NOT be in execution graph"
    );
    assert!(
        !all_tasks.contains(&"grandparent.parent".to_string()),
        "Parent task should NOT be in execution graph"
    );

    // Only the leaf child should be in the graph
    assert!(
        all_tasks.contains(&"grandparent.parent.child".to_string()),
        "Leaf child task should be in execution graph"
    );
}

#[tokio::test]
async fn test_parent_task_attributes_inherited_by_children() {
    let yaml = r#"
name: "Inheritance Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools:
      - Read
    permissions:
      mode: "default"

  fallback_agent:
    description: "Fallback agent"
    tools:
      - Read
    permissions:
      mode: "default"

tasks:
  parent:
    description: "Parent task"
    agent: "test_agent"
    priority: 10
    inject_context: true
    on_error:
      retry: 3
      fallback_agent: "fallback_agent"
    subtasks:
      # This child should inherit agent, priority, inject_context, and on_error
      - child_inherits:
          description: "Child that inherits"

      # This child overrides the agent
      - child_overrides:
          description: "Child that overrides"
          agent: "fallback_agent"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    let mut executor = DSLExecutor::new(workflow).unwrap();
    executor.initialize().await.unwrap();

    let task_graph = executor.task_graph();
    let all_tasks = task_graph.get_all_tasks();

    println!("Tasks in execution graph: {:?}", all_tasks);

    // Should have exactly 2 tasks (the 2 children)
    assert_eq!(
        all_tasks.len(),
        2,
        "Expected 2 executable tasks (2 children), got {}",
        all_tasks.len()
    );

    // Parent should NOT be in the graph
    assert!(
        !all_tasks.contains(&"parent".to_string()),
        "Parent task should NOT be in execution graph"
    );

    // Both children should be in the graph
    assert!(
        all_tasks.contains(&"parent.child_inherits".to_string()),
        "Child 'child_inherits' should be in execution graph"
    );
    assert!(
        all_tasks.contains(&"parent.child_overrides".to_string()),
        "Child 'child_overrides' should be in execution graph"
    );

    // Verify inheritance by checking the task specs in the graph
    let child_inherits_node = task_graph.get_task("parent.child_inherits").unwrap();
    assert_eq!(
        child_inherits_node.spec.agent,
        Some("test_agent".to_string()),
        "Child should inherit agent from parent"
    );
    assert_eq!(
        child_inherits_node.spec.priority, 10,
        "Child should inherit priority from parent"
    );
    assert!(
        child_inherits_node.spec.inject_context,
        "Child should inherit inject_context from parent"
    );
    assert!(
        child_inherits_node.spec.on_error.is_some(),
        "Child should inherit on_error from parent"
    );

    // Verify override
    let child_overrides_node = task_graph.get_task("parent.child_overrides").unwrap();
    assert_eq!(
        child_overrides_node.spec.agent,
        Some("fallback_agent".to_string()),
        "Child should override agent from parent"
    );
}

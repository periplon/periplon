//! Tests for hierarchical task execution

use periplon_sdk::dsl::{parse_workflow, validate_workflow};

/// Test that hierarchical tasks are properly represented in the workflow structure
#[test]
fn test_hierarchical_task_structure() {
    let yaml = r#"
name: "Hierarchical Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools:
      - Read
    permissions:
      mode: "default"

tasks:
  parent_task:
    description: "Parent task"
    agent: "test_agent"
    subtasks:
      - child_task1:
          description: "First child"
          agent: "test_agent"

      - child_task2:
          description: "Second child"
          agent: "test_agent"
          subtasks:
            - grandchild_task:
                description: "Grandchild task"
                agent: "test_agent"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    validate_workflow(&workflow).unwrap();

    // Check parent task exists
    let parent = workflow.tasks.get("parent_task").unwrap();
    assert_eq!(parent.subtasks.len(), 2);

    // Check child tasks
    let child2 = parent.subtasks[1].get("child_task2").unwrap();
    assert_eq!(child2.subtasks.len(), 1);

    // Check grandchild
    let grandchild = child2.subtasks[0].get("grandchild_task").unwrap();
    assert_eq!(grandchild.description, "Grandchild task");
}

#[test]
fn test_hierarchical_with_dependencies() {
    let yaml = r#"
name: "Dependency Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools:
      - Read
    permissions:
      mode: "default"

tasks:
  task1:
    description: "Task 1"
    agent: "test_agent"
    subtasks:
      - subtask1:
          description: "Subtask 1"
          agent: "test_agent"

  task2:
    description: "Task 2"
    agent: "test_agent"
    depends_on:
      - task1
"#;

    let workflow = parse_workflow(yaml).unwrap();
    validate_workflow(&workflow).unwrap();

    // Task 1 should have 1 subtask
    let task1 = workflow.tasks.get("task1").unwrap();
    assert_eq!(task1.subtasks.len(), 1);

    // Task 2 should depend on task1
    let task2 = workflow.tasks.get("task2").unwrap();
    assert_eq!(task2.depends_on.len(), 1);
    assert_eq!(task2.depends_on[0], "task1");
}

#[test]
fn test_parse_file_organizer_with_subtasks() {
    let yaml = std::fs::read_to_string("examples/dsl/simple_file_organizer.yaml").unwrap();
    let workflow = parse_workflow(&yaml).unwrap();

    // The file organizer has 1 parent task with 3 subtasks
    assert_eq!(workflow.tasks.len(), 1);

    let parent_task = workflow.tasks.get("organize_downloads").unwrap();
    assert_eq!(parent_task.subtasks.len(), 3);
}

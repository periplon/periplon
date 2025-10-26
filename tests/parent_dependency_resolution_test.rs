/// Test parent task dependency resolution
///
/// This test verifies that when a task depends on a parent task with subtasks,
/// the dependency is correctly resolved to all leaf subtasks of that parent.
///
/// Scenario:
/// - setup (no dependencies)
/// - run_tests (parent with subtasks, depends on setup)
///   - execute_test (subtask)
///   - validate_test (subtask, depends on execute_test)
/// - generate_summary (depends on run_tests)
///
/// Expected behavior:
/// - generate_summary should depend on all subtasks of run_tests
/// - No cyclic dependency error should occur
/// - Workflow should execute successfully
use periplon_sdk::dsl::executor::DSLExecutor;
use periplon_sdk::dsl::parser::parse_workflow;

#[tokio::test]
async fn test_parent_task_dependency_resolution() {
    let yaml = r#"
name: "Parent Dependency Resolution Test"
version: "1.0.0"

tasks:
  setup:
    description: "Setup task"
    script:
      language: bash
      content: |
        echo "Setup completed"

  run_tests:
    description: "Run tests (parent with subtasks)"
    depends_on:
      - setup
    subtasks:
      - execute_test:
          description: "Execute test"
          script:
            language: bash
            content: |
              echo "Executing test"
      - validate_test:
          description: "Validate test"
          depends_on:
            - execute_test
          script:
            language: bash
            content: |
              echo "Validating test"

  generate_summary:
    description: "Generate summary (depends on parent)"
    depends_on:
      - run_tests
    script:
      language: bash
      content: |
        echo "Generating summary"
"#;

    // Parse workflow
    let workflow = parse_workflow(yaml).expect("Failed to parse workflow");

    // Create executor
    let mut executor = DSLExecutor::new(workflow).expect("Failed to create executor");

    // Initialize agents and build task graph
    // This should resolve parent dependencies without errors
    let result = executor.initialize().await;

    assert!(
        result.is_ok(),
        "Failed to initialize agents: {:?}",
        result.err()
    );

    // Verify that the task graph was built successfully (no cyclic dependency)
    let task_graph = executor.task_graph();

    // The graph should contain:
    // - setup
    // - run_tests.execute_test
    // - run_tests.validate_test
    // - generate_summary
    assert_eq!(task_graph.task_count(), 4, "Expected 4 tasks in graph");

    // Verify topological sort works (no cycles)
    let sorted = task_graph
        .topological_sort()
        .expect("Topological sort should succeed");
    assert_eq!(sorted.len(), 4, "Expected 4 tasks in sorted order");

    // Verify that generate_summary depends on the subtasks, not the parent
    let generate_summary_node = task_graph
        .get_task("generate_summary")
        .expect("generate_summary task should exist");

    // generate_summary should depend on run_tests.execute_test and run_tests.validate_test
    // (or just run_tests.validate_test if dependencies are transitive)
    assert!(
        generate_summary_node
            .dependencies
            .contains(&"run_tests.validate_test".to_string()),
        "generate_summary should depend on run_tests.validate_test"
    );
}

#[tokio::test]
async fn test_parent_dependency_with_loop() {
    let yaml = r#"
name: "Parent Dependency with Loop Test"
version: "1.0.0"

tasks:
  setup:
    description: "Setup task"
    script:
      language: bash
      content: |
        echo "Setup completed"

  run_iterations:
    description: "Run iterations (parent with subtasks and loop)"
    depends_on:
      - setup
    subtasks:
      - execute:
          description: "Execute iteration"
          script:
            language: bash
            content: |
              echo "Executing iteration ${task.loop_index}"
      - validate:
          description: "Validate iteration"
          depends_on:
            - execute
          script:
            language: bash
            content: |
              echo "Validating iteration ${task.loop_index}"
    loop:
      type: repeat
      count: 3
      iterator: iteration

  cleanup:
    description: "Cleanup (depends on parent with loop)"
    depends_on:
      - run_iterations
    script:
      language: bash
      content: |
        echo "Cleanup completed"
"#;

    // Parse workflow
    let workflow = parse_workflow(yaml).expect("Failed to parse workflow");

    // Create executor
    let mut executor = DSLExecutor::new(workflow).expect("Failed to create executor");

    // Initialize agents and build task graph
    let result = executor.initialize().await;

    assert!(
        result.is_ok(),
        "Failed to initialize agents with parent loop: {:?}",
        result.err()
    );

    // Verify task graph
    let task_graph = executor.task_graph();

    // Should contain: setup, run_iterations (executable with loop+subtasks), cleanup
    // Note: With loop+subtasks, the subtasks are NOT flattened - they execute within the loop
    assert_eq!(task_graph.task_count(), 3, "Expected 3 tasks in graph");

    // Verify topological sort works
    let sorted = task_graph
        .topological_sort()
        .expect("Topological sort should succeed");
    assert_eq!(sorted.len(), 3, "Expected 3 tasks in sorted order");

    // Verify cleanup depends on the parent task (run_iterations), not the subtasks
    // because subtasks are executed within the loop and aren't separate tasks
    let cleanup_node = task_graph
        .get_task("cleanup")
        .expect("cleanup task should exist");

    assert!(
        cleanup_node
            .dependencies
            .contains(&"run_iterations".to_string()),
        "cleanup should depend on run_iterations (the parent loop task)"
    );
}

#[tokio::test]
#[ignore] // TODO: Fix nested parent dependency resolution (triple nesting)
async fn test_nested_parent_dependencies() {
    let yaml = r#"
name: "Nested Parent Dependencies Test"
version: "1.0.0"

tasks:
  init:
    description: "Init task"
    script:
      language: bash
      content: |
        echo "Init completed"

  level1:
    description: "Level 1 parent"
    depends_on:
      - init
    subtasks:
      - level2:
          description: "Level 2 parent (nested)"
          subtasks:
            - actual_task:
                description: "Actual executable task"
                script:
                  language: bash
                  content: |
                    echo "Actual task executed"

  finalize:
    description: "Finalize (depends on nested parent)"
    depends_on:
      - level1
    script:
      language: bash
      content: |
        echo "Finalize completed"
"#;

    // Parse workflow
    let workflow = parse_workflow(yaml).expect("Failed to parse workflow");

    // Create executor
    let mut executor = DSLExecutor::new(workflow).expect("Failed to create executor");

    // Initialize agents and build task graph
    let result = executor.initialize().await;

    assert!(
        result.is_ok(),
        "Failed to initialize agents with nested parents: {:?}",
        result.err()
    );

    // Verify task graph
    let task_graph = executor.task_graph();

    // Should contain: init, level1.level2.actual_task, finalize
    assert_eq!(task_graph.task_count(), 3, "Expected 3 tasks in graph");

    // Verify topological sort works
    let sorted = task_graph
        .topological_sort()
        .expect("Topological sort should succeed");
    assert_eq!(sorted.len(), 3, "Expected 3 tasks in sorted order");

    // Verify finalize depends on the leaf task
    let finalize_node = task_graph
        .get_task("finalize")
        .expect("finalize task should exist");

    assert!(
        finalize_node
            .dependencies
            .contains(&"level1.level2.actual_task".to_string()),
        "finalize should depend on level1.level2.actual_task"
    );
}

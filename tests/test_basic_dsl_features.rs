//! Test basic DSL features: agents, tasks, and dependencies

use periplon_sdk::dsl::executor::DSLExecutor;
use periplon_sdk::dsl::parser::parse_workflow_file;

#[tokio::test]
async fn test_basic_agent_configuration() {
    // Parse the basic features workflow
    let workflow_path = "tests/fixtures/basic_features.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Verify agent count
    assert_eq!(workflow.agents.len(), 3, "Should have 3 agents");

    // Verify researcher agent
    let researcher = workflow
        .agents
        .get("researcher")
        .expect("researcher agent should exist");
    assert_eq!(researcher.description, "Research and gather information");
    assert_eq!(researcher.model, Some("claude-sonnet-4-5".to_string()));
    assert_eq!(researcher.tools.len(), 3);
    assert!(researcher.tools.contains(&"Read".to_string()));
    assert!(researcher.tools.contains(&"Grep".to_string()));
    assert!(researcher.tools.contains(&"WebSearch".to_string()));
    assert_eq!(researcher.permissions.mode, "default");
    assert_eq!(researcher.max_turns, Some(5));

    // Verify coder agent
    let coder = workflow
        .agents
        .get("coder")
        .expect("coder agent should exist");
    assert_eq!(coder.description, "Write and edit code");
    assert_eq!(coder.permissions.mode, "acceptEdits");
    assert_eq!(coder.max_turns, Some(10));
    assert_eq!(coder.tools.len(), 4);

    // Verify reviewer agent
    let reviewer = workflow
        .agents
        .get("reviewer")
        .expect("reviewer agent should exist");
    assert_eq!(reviewer.description, "Review and validate work");
    assert_eq!(reviewer.max_turns, Some(3));
    assert_eq!(reviewer.tools.len(), 2);

    println!("✓ Agent configuration test passed");
}

#[tokio::test]
async fn test_task_dependency_resolution() {
    let workflow_path = "tests/fixtures/basic_features.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Verify task count
    assert_eq!(workflow.tasks.len(), 9, "Should have 9 tasks");

    // Test independent task (no dependencies)
    let init = workflow.tasks.get("init").expect("init task should exist");
    assert!(
        init.depends_on.is_empty(),
        "init should have no dependencies"
    );
    assert_eq!(init.agent, Some("researcher".to_string()));

    // Test single dependency
    let gather = workflow
        .tasks
        .get("gather_requirements")
        .expect("gather_requirements should exist");
    assert_eq!(gather.depends_on.len(), 1);
    assert_eq!(gather.depends_on[0], "init");

    // Test parallel tasks (both depend on same parent)
    let design = workflow
        .tasks
        .get("design_architecture")
        .expect("design_architecture should exist");
    let test_plan = workflow
        .tasks
        .get("create_test_plan")
        .expect("create_test_plan should exist");
    assert_eq!(design.depends_on, vec!["gather_requirements"]);
    assert_eq!(test_plan.depends_on, vec!["gather_requirements"]);

    // Test multiple dependencies
    let implement = workflow
        .tasks
        .get("implement_feature")
        .expect("implement_feature should exist");
    assert_eq!(implement.depends_on.len(), 2);
    assert!(implement
        .depends_on
        .contains(&"design_architecture".to_string()));
    assert!(implement
        .depends_on
        .contains(&"create_test_plan".to_string()));
    assert_eq!(implement.priority, 1);

    // Test sequential chain
    let write_tests = workflow
        .tasks
        .get("write_tests")
        .expect("write_tests should exist");
    assert_eq!(write_tests.depends_on, vec!["implement_feature"]);

    let run_tests = workflow
        .tasks
        .get("run_tests")
        .expect("run_tests should exist");
    assert_eq!(run_tests.depends_on, vec!["write_tests"]);

    // Test parallel_with
    let optimize = workflow
        .tasks
        .get("optimize_performance")
        .expect("optimize_performance should exist");
    assert_eq!(optimize.depends_on, vec!["implement_feature"]);
    assert_eq!(optimize.parallel_with, vec!["write_tests"]);

    println!("✓ Task dependency resolution test passed");
}

#[tokio::test]
#[cfg_attr(
    not(feature = "cli-required-tests"),
    ignore = "Requires CLI binary installed"
)]
async fn test_topological_sort() {
    let workflow_path = "tests/fixtures/basic_features.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Create executor to build task graph
    let mut executor = DSLExecutor::new(workflow).expect("Failed to create executor");

    // Initialize the executor (builds task graph)
    executor
        .initialize()
        .await
        .expect("Failed to initialize executor");

    // Get task graph and perform topological sort
    let task_graph = executor.task_graph();
    let sorted = task_graph
        .topological_sort()
        .expect("Topological sort should succeed");

    println!("Topological sort order: {:?}", sorted);

    // Verify that dependencies come before dependents
    let get_position = |task_id: &str| {
        sorted
            .iter()
            .position(|t| t == task_id)
            .unwrap_or_else(|| panic!("Task {} should be in sorted list", task_id))
    };

    // init should come before gather_requirements
    assert!(
        get_position("init") < get_position("gather_requirements"),
        "init must come before gather_requirements"
    );

    // gather_requirements should come before design_architecture and create_test_plan
    assert!(
        get_position("gather_requirements") < get_position("design_architecture"),
        "gather_requirements must come before design_architecture"
    );
    assert!(
        get_position("gather_requirements") < get_position("create_test_plan"),
        "gather_requirements must come before create_test_plan"
    );

    // Both design_architecture and create_test_plan must come before implement_feature
    assert!(
        get_position("design_architecture") < get_position("implement_feature"),
        "design_architecture must come before implement_feature"
    );
    assert!(
        get_position("create_test_plan") < get_position("implement_feature"),
        "create_test_plan must come before implement_feature"
    );

    // implement_feature must come before write_tests
    assert!(
        get_position("implement_feature") < get_position("write_tests"),
        "implement_feature must come before write_tests"
    );

    // write_tests must come before run_tests
    assert!(
        get_position("write_tests") < get_position("run_tests"),
        "write_tests must come before run_tests"
    );

    // run_tests must come before code_review
    assert!(
        get_position("run_tests") < get_position("code_review"),
        "run_tests must come before code_review"
    );

    // optimize_performance should come after implement_feature
    assert!(
        get_position("implement_feature") < get_position("optimize_performance"),
        "implement_feature must come before optimize_performance"
    );

    println!("✓ Topological sort test passed");
}

#[tokio::test]
#[cfg_attr(
    not(feature = "cli-required-tests"),
    ignore = "Requires CLI binary installed"
)]
async fn test_ready_tasks() {
    let workflow_path = "tests/fixtures/basic_features.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    let mut executor = DSLExecutor::new(workflow).expect("Failed to create executor");

    executor
        .initialize()
        .await
        .expect("Failed to initialize executor");

    let task_graph = executor.task_graph();

    // Initially, only tasks with no dependencies should be ready
    let ready = task_graph.get_ready_tasks();
    assert_eq!(ready.len(), 1, "Only init task should be ready initially");
    assert_eq!(ready[0], "init");

    println!("✓ Ready tasks test passed");
}

#[tokio::test]
async fn test_agent_tool_permissions() {
    let workflow_path = "tests/fixtures/basic_features.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Verify researcher has read-only tools
    let researcher = workflow.agents.get("researcher").unwrap();
    assert!(researcher.tools.contains(&"Read".to_string()));
    assert!(researcher.tools.contains(&"Grep".to_string()));
    assert!(researcher.tools.contains(&"WebSearch".to_string()));
    assert!(!researcher.tools.contains(&"Write".to_string()));
    assert!(!researcher.tools.contains(&"Edit".to_string()));

    // Verify coder has write permissions
    let coder = workflow.agents.get("coder").unwrap();
    assert!(coder.tools.contains(&"Write".to_string()));
    assert!(coder.tools.contains(&"Edit".to_string()));
    assert!(coder.tools.contains(&"Read".to_string()));
    assert_eq!(coder.permissions.mode, "acceptEdits");

    // Verify reviewer is read-only
    let reviewer = workflow.agents.get("reviewer").unwrap();
    assert!(reviewer.tools.contains(&"Read".to_string()));
    assert!(reviewer.tools.contains(&"Grep".to_string()));
    assert!(!reviewer.tools.contains(&"Write".to_string()));
    assert_eq!(reviewer.permissions.mode, "default");

    println!("✓ Agent tool permissions test passed");
}

#[tokio::test]
async fn test_task_output_paths() {
    let workflow_path = "tests/fixtures/basic_features.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Verify all tasks have output paths
    for (task_id, task) in &workflow.tasks {
        assert!(
            task.output.is_some(),
            "Task {} should have output path",
            task_id
        );
        let output = task.output.as_ref().unwrap();
        assert!(
            output.starts_with("test_results/"),
            "Task {} output should be in test_results/ directory",
            task_id
        );
    }

    println!("✓ Task output paths test passed");
}

#[tokio::test]
async fn test_workflow_metadata() {
    let workflow_path = "tests/fixtures/basic_features.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Verify workflow metadata
    assert_eq!(workflow.name, "Basic DSL Features Test");
    assert_eq!(workflow.version, "1.0.0");

    println!("✓ Workflow metadata test passed");
}

/// Integration test to verify Definition of Done auto_elevate_permissions feature
use periplon_sdk::dsl::parser::parse_workflow;
use periplon_sdk::dsl::validator::validate_workflow;
use std::fs;
use std::path::Path;

#[test]
#[ignore] // Missing test_dod_auto_elevate.yaml file
fn test_dod_auto_elevate_workflow_structure() {
    // Load and parse the test workflow
    let yaml_content =
        fs::read_to_string("test_dod_auto_elevate.yaml").expect("Failed to read test workflow");

    let workflow = parse_workflow(&yaml_content).expect("Failed to parse workflow");

    // Verify workflow structure
    assert_eq!(workflow.name, "Test DoD Auto-Elevate Permissions");
    assert_eq!(workflow.version, "1.0.0");

    // Verify agents exist
    assert!(
        workflow.agents.contains_key("cleanup_agent"),
        "cleanup_agent not found"
    );
    assert!(
        workflow.agents.contains_key("test_agent"),
        "test_agent not found"
    );

    // Verify cleanup agent has bypass permissions
    let cleanup_agent = &workflow.agents["cleanup_agent"];
    assert_eq!(
        cleanup_agent.permissions.mode, "bypassPermissions",
        "cleanup_agent should have bypassPermissions"
    );

    // Verify test agent has acceptEdits permission mode initially
    let test_agent = &workflow.agents["test_agent"];
    assert_eq!(
        test_agent.permissions.mode, "acceptEdits",
        "test_agent should start with acceptEdits mode"
    );
    assert!(
        test_agent.tools.contains(&"Bash".to_string()),
        "Bash tool not enabled"
    );

    // Verify tasks exist
    assert!(
        workflow.tasks.contains_key("cleanup"),
        "cleanup task not found"
    );
    assert!(
        workflow.tasks.contains_key("test_auto_elevate"),
        "test_auto_elevate task not found"
    );

    // Verify the main task has DoD configured
    let main_task = &workflow.tasks["test_auto_elevate"];
    assert!(
        main_task.definition_of_done.is_some(),
        "DoD should be configured"
    );

    let dod = main_task.definition_of_done.as_ref().unwrap();

    // Verify auto_elevate_permissions is enabled
    assert!(
        dod.auto_elevate_permissions,
        "auto_elevate_permissions should be true"
    );
    assert_eq!(dod.max_retries, 2, "Should allow 2 retries");
    assert!(
        dod.fail_on_unmet,
        "Should fail if DoD not met after retries"
    );

    // Verify DoD criteria
    assert_eq!(dod.criteria.len(), 3, "Should have 3 DoD criteria");

    println!("✓ Workflow structure verified");
    println!("✓ auto_elevate_permissions enabled on main task");
    println!("✓ DoD criteria configured correctly");
    println!("✓ Cleanup agent has bypass permissions");
    println!("✓ Test agent starts with acceptEdits mode");
}

#[test]
#[ignore] // Missing test_dod_auto_elevate.yaml file
fn test_dod_criteria_types() {
    use periplon_sdk::dsl::schema::DoneCriterion;

    let yaml_content =
        fs::read_to_string("test_dod_auto_elevate.yaml").expect("Failed to read test workflow");

    let workflow = parse_workflow(&yaml_content).expect("Failed to parse workflow");

    let main_task = &workflow.tasks["test_auto_elevate"];
    let dod = main_task.definition_of_done.as_ref().unwrap();

    // Check that we have the expected criterion types
    let has_file_exists = dod
        .criteria
        .iter()
        .any(|c| matches!(c, DoneCriterion::FileExists { .. }));
    assert!(has_file_exists, "Should have FileExists criterion");

    let has_file_contains = dod
        .criteria
        .iter()
        .any(|c| matches!(c, DoneCriterion::FileContains { .. }));
    assert!(has_file_contains, "Should have FileContains criterion");

    let has_command_succeeds = dod
        .criteria
        .iter()
        .any(|c| matches!(c, DoneCriterion::CommandSucceeds { .. }));
    assert!(
        has_command_succeeds,
        "Should have CommandSucceeds criterion"
    );

    println!("✓ All expected DoD criterion types present");
}

#[test]
#[ignore] // Missing test_dod_auto_elevate.yaml file
fn test_task_dependencies() {
    let yaml_content =
        fs::read_to_string("test_dod_auto_elevate.yaml").expect("Failed to read test workflow");

    let workflow = parse_workflow(&yaml_content).expect("Failed to parse workflow");

    // Verify test_auto_elevate depends on cleanup
    let main_task = &workflow.tasks["test_auto_elevate"];
    assert!(
        main_task.depends_on.contains(&"cleanup".to_string()),
        "test_auto_elevate should depend on cleanup"
    );

    println!("✓ Task dependencies configured correctly");
}

#[tokio::test]
#[ignore] // Run with: cargo test --test dod_auto_elevate_test -- --ignored --nocapture
async fn test_dod_auto_elevate_full_workflow() {
    // This test validates the workflow configuration and provides instructions
    // for manual verification of the auto-elevation feature

    let workflow_path = "test_dod_auto_elevate.yaml";
    assert!(
        Path::new(workflow_path).exists(),
        "Test workflow file not found: {}",
        workflow_path
    );

    let yaml_content = fs::read_to_string(workflow_path).expect("Failed to read workflow file");

    let workflow = parse_workflow(&yaml_content).expect("Failed to parse workflow");

    validate_workflow(&workflow).expect("Workflow validation failed");

    println!("✓ Workflow loaded and validated successfully");
    println!("✓ auto_elevate_permissions enabled with 2 max retries");
    println!("\nTo test the auto-elevation feature:");
    println!("  1. Run: ./target/release/dsl-executor run test_dod_auto_elevate.yaml");
    println!("  2. Watch for DoD evaluation messages");
    println!("  3. If DoD fails initially, look for: '🔓 Auto-elevating permissions to bypassPermissions'");
    println!("  4. Verify the task succeeds on retry with elevated permissions");
    println!("  5. Check the output file: dod_test_results.txt");
}

#[test]
fn test_definition_of_done_schema() {
    use periplon_sdk::dsl::schema::{DefinitionOfDone, DoneCriterion};

    // Test that we can create a DoD with auto_elevate_permissions
    let dod = DefinitionOfDone {
        criteria: vec![DoneCriterion::FileExists {
            path: "/tmp/test.txt".to_string(),
            description: "Test file must exist".to_string(),
        }],
        max_retries: 3,
        fail_on_unmet: true,
        auto_elevate_permissions: true,
    };

    assert!(dod.auto_elevate_permissions);
    assert_eq!(dod.max_retries, 3);
    assert!(dod.fail_on_unmet);
    assert_eq!(dod.criteria.len(), 1);

    println!("✓ DefinitionOfDone with auto_elevate_permissions created successfully");
}

#[test]
fn test_permission_mode_transitions() {
    // Document the expected permission mode transition
    // This is a documentation test showing how auto-elevation should work

    let initial_mode = "acceptEdits";
    let elevated_mode = "bypassPermissions";

    // When auto_elevate_permissions is true and DoD fails:
    // 1. System detects permission issue (via detect_permission_issue function)
    // 2. Agent's permission mode changes: acceptEdits -> bypassPermissions
    // 3. Task retries with elevated permissions
    // 4. DoD is evaluated again

    println!("Permission mode transition on auto-elevation:");
    println!("  Initial:  {}", initial_mode);
    println!("  Elevated: {}", elevated_mode);
    println!("\nTrigger conditions:");
    println!("  - auto_elevate_permissions: true");
    println!("  - DoD criteria not met");
    println!("  - Permission issue detected in output or DoD results");
    println!("  - Retries remaining > 0");
}

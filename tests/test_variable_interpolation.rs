//! Test variable interpolation across workflow, agent, and task scopes

use periplon_sdk::dsl::parser::parse_workflow_file;
use periplon_sdk::dsl::variables::{Scope, VariableContext};
use serde_json::json;

#[tokio::test]
async fn test_workflow_input_variables() {
    let workflow_path = "tests/fixtures/variable_interpolation.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Verify workflow input definitions
    assert_eq!(workflow.inputs.len(), 4, "Should have 4 workflow inputs");

    // Test project_name input
    let project_name = workflow
        .inputs
        .get("project_name")
        .expect("project_name should exist");
    assert_eq!(project_name.param_type, "string");
    assert!(project_name.required);
    assert_eq!(project_name.default, Some(json!("TestProject")));

    // Test environment input
    let environment = workflow
        .inputs
        .get("environment")
        .expect("environment should exist");
    assert_eq!(environment.param_type, "string");
    assert!(!environment.required);
    assert_eq!(environment.default, Some(json!("development")));

    // Test max_retries input (number type)
    let max_retries = workflow
        .inputs
        .get("max_retries")
        .expect("max_retries should exist");
    assert_eq!(max_retries.param_type, "number");
    assert_eq!(max_retries.default, Some(json!(3)));

    // Test enable_debug input (boolean type)
    let enable_debug = workflow
        .inputs
        .get("enable_debug")
        .expect("enable_debug should exist");
    assert_eq!(enable_debug.param_type, "boolean");
    assert_eq!(enable_debug.default, Some(json!(true)));

    println!("✓ Workflow input variables test passed");
}

#[tokio::test]
async fn test_workflow_output_variables() {
    let workflow_path = "tests/fixtures/variable_interpolation.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Verify workflow output definitions
    assert_eq!(workflow.outputs.len(), 1, "Should have 1 workflow output");

    let final_report = workflow
        .outputs
        .get("final_report")
        .expect("final_report should exist");
    assert!(final_report.description.is_some());

    println!("✓ Workflow output variables test passed");
}

#[tokio::test]
async fn test_agent_input_variables() {
    let workflow_path = "tests/fixtures/variable_interpolation.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Test analyzer agent inputs
    let analyzer = workflow
        .agents
        .get("analyzer")
        .expect("analyzer should exist");
    assert_eq!(analyzer.inputs.len(), 1);

    let analysis_depth = analyzer
        .inputs
        .get("analysis_depth")
        .expect("analysis_depth should exist");
    assert_eq!(analysis_depth.param_type, "string");
    assert_eq!(analysis_depth.default, Some(json!("deep")));

    // Test builder agent inputs
    let builder = workflow
        .agents
        .get("builder")
        .expect("builder should exist");
    assert_eq!(builder.inputs.len(), 2);

    let build_target = builder
        .inputs
        .get("build_target")
        .expect("build_target should exist");
    assert_eq!(build_target.param_type, "string");
    assert_eq!(build_target.default, Some(json!("release")));

    let opt_level = builder
        .inputs
        .get("optimization_level")
        .expect("optimization_level should exist");
    assert_eq!(opt_level.param_type, "number");
    assert_eq!(opt_level.default, Some(json!(2)));

    println!("✓ Agent input variables test passed");
}

#[tokio::test]
async fn test_agent_output_variables() {
    let workflow_path = "tests/fixtures/variable_interpolation.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Test analyzer agent outputs
    let analyzer = workflow
        .agents
        .get("analyzer")
        .expect("analyzer should exist");
    assert_eq!(analyzer.outputs.len(), 1);
    assert!(analyzer.outputs.contains_key("analysis_result"));

    // Test builder agent outputs
    let builder = workflow
        .agents
        .get("builder")
        .expect("builder should exist");
    assert_eq!(builder.outputs.len(), 1);
    assert!(builder.outputs.contains_key("build_log"));

    println!("✓ Agent output variables test passed");
}

#[tokio::test]
async fn test_workflow_variable_interpolation_in_descriptions() {
    let workflow_path = "tests/fixtures/variable_interpolation.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Check that workflow variables are used in agent descriptions
    let analyzer = workflow
        .agents
        .get("analyzer")
        .expect("analyzer should exist");
    assert!(analyzer.description.contains("${workflow.project_name}"));
    assert!(analyzer.description.contains("${workflow.environment}"));

    let builder = workflow
        .agents
        .get("builder")
        .expect("builder should exist");
    assert!(builder.description.contains("${workflow.project_name}"));

    // Check that workflow variables are used in task descriptions
    let init = workflow
        .tasks
        .get("init_project")
        .expect("init_project should exist");
    assert!(init.description.contains("${workflow.project_name}"));
    assert!(init.description.contains("${workflow.environment}"));

    println!("✓ Workflow variable interpolation in descriptions test passed");
}

#[tokio::test]
async fn test_agent_variable_interpolation() {
    let workflow_path = "tests/fixtures/variable_interpolation.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Check agent variable usage in task descriptions
    let analyze = workflow
        .tasks
        .get("analyze_codebase")
        .expect("analyze_codebase should exist");
    assert!(analyze.description.contains("${agent.analysis_depth}"));

    // Check agent variable usage in task outputs
    assert!(analyze.output.is_some());
    let output = analyze.output.as_ref().unwrap();
    assert!(output.contains("${agent.analysis_depth}"));

    println!("✓ Agent variable interpolation test passed");
}

#[tokio::test]
async fn test_task_input_variables() {
    let workflow_path = "tests/fixtures/variable_interpolation.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Test init_project task inputs with workflow variables
    let init = workflow
        .tasks
        .get("init_project")
        .expect("init_project should exist");
    assert_eq!(init.inputs.len(), 2);

    let config_file = init
        .inputs
        .get("config_file")
        .expect("config_file should exist");
    assert_eq!(config_file, &json!("${workflow.project_name}/config.yaml"));

    let debug_mode = init
        .inputs
        .get("debug_mode")
        .expect("debug_mode should exist");
    assert_eq!(debug_mode, &json!("${workflow.enable_debug}"));

    // Test analyze_codebase task inputs with mixed variable scopes
    let analyze = workflow
        .tasks
        .get("analyze_codebase")
        .expect("analyze_codebase should exist");
    assert_eq!(analyze.inputs.len(), 3);

    let project_path = analyze
        .inputs
        .get("project_path")
        .expect("project_path should exist");
    assert_eq!(project_path, &json!("${workflow.project_name}/src"));

    let depth = analyze.inputs.get("depth").expect("depth should exist");
    assert_eq!(depth, &json!("${agent.analysis_depth}"));

    let retries = analyze.inputs.get("retries").expect("retries should exist");
    assert_eq!(retries, &json!("${workflow.max_retries}"));

    println!("✓ Task input variables test passed");
}

#[tokio::test]
async fn test_variable_scope_hierarchy() {
    let workflow_path = "tests/fixtures/variable_interpolation.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Test deploy_service which uses variables from all scopes
    let deploy = workflow
        .tasks
        .get("deploy_service")
        .expect("deploy_service should exist");
    assert_eq!(deploy.inputs.len(), 4);

    // Workflow scope
    let target_env = deploy
        .inputs
        .get("target_env")
        .expect("target_env should exist");
    assert_eq!(target_env, &json!("${workflow.environment}"));

    let debug = deploy.inputs.get("debug").expect("debug should exist");
    assert_eq!(debug, &json!("${workflow.enable_debug}"));

    // Agent scope
    let build_type = deploy
        .inputs
        .get("build_type")
        .expect("build_type should exist");
    assert_eq!(build_type, &json!("${agent.build_target}"));

    // Workflow scope combined with string
    let service_name = deploy
        .inputs
        .get("service_name")
        .expect("service_name should exist");
    assert_eq!(service_name, &json!("${workflow.project_name}-api"));

    println!("✓ Variable scope hierarchy test passed");
}

#[tokio::test]
async fn test_task_output_variables() {
    let workflow_path = "tests/fixtures/variable_interpolation.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Test task outputs with file sources
    let init = workflow
        .tasks
        .get("init_project")
        .expect("init_project should exist");
    assert_eq!(init.outputs.len(), 1);
    assert!(init.outputs.contains_key("init_status"));

    let analyze = workflow
        .tasks
        .get("analyze_codebase")
        .expect("analyze_codebase should exist");
    assert_eq!(analyze.outputs.len(), 1);
    assert!(analyze.outputs.contains_key("code_metrics"));

    let build = workflow
        .tasks
        .get("build_components")
        .expect("build_components should exist");
    assert_eq!(build.outputs.len(), 1);
    assert!(build.outputs.contains_key("build_artifact"));

    // Test task outputs with state source
    let deploy = workflow
        .tasks
        .get("deploy_service")
        .expect("deploy_service should exist");
    assert_eq!(deploy.outputs.len(), 1);
    assert!(deploy.outputs.contains_key("deployment_status"));

    println!("✓ Task output variables test passed");
}

#[tokio::test]
async fn test_variable_types() {
    let workflow_path = "tests/fixtures/variable_interpolation.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Test validate_deployment which uses multiple variable types
    let validate = workflow
        .tasks
        .get("validate_deployment")
        .expect("validate_deployment should exist");
    assert_eq!(validate.inputs.len(), 4);

    // Number type
    let max_attempts = validate
        .inputs
        .get("max_attempts")
        .expect("max_attempts should exist");
    assert_eq!(max_attempts, &json!("${workflow.max_retries}"));

    // Boolean type
    let debug_enabled = validate
        .inputs
        .get("debug_enabled")
        .expect("debug_enabled should exist");
    assert_eq!(debug_enabled, &json!("${workflow.enable_debug}"));

    // String types
    let env_name = validate
        .inputs
        .get("env_name")
        .expect("env_name should exist");
    assert_eq!(env_name, &json!("${workflow.environment}"));

    let project = validate
        .inputs
        .get("project")
        .expect("project should exist");
    assert_eq!(project, &json!("${workflow.project_name}"));

    println!("✓ Variable types test passed");
}

#[tokio::test]
async fn test_variable_interpolation_in_output_paths() {
    let workflow_path = "tests/fixtures/variable_interpolation.yaml";
    let workflow = parse_workflow_file(workflow_path).expect("Failed to parse workflow");

    // Test that workflow variables are used in output paths
    let init = workflow
        .tasks
        .get("init_project")
        .expect("init_project should exist");
    assert!(init
        .output
        .as_ref()
        .unwrap()
        .contains("${workflow.project_name}"));

    let analyze = workflow
        .tasks
        .get("analyze_codebase")
        .expect("analyze_codebase should exist");
    assert!(analyze
        .output
        .as_ref()
        .unwrap()
        .contains("${agent.analysis_depth}"));

    let deploy = workflow
        .tasks
        .get("deploy_service")
        .expect("deploy_service should exist");
    assert!(deploy
        .output
        .as_ref()
        .unwrap()
        .contains("${workflow.project_name}"));

    let generate = workflow
        .tasks
        .get("generate_report")
        .expect("generate_report should exist");
    assert!(generate
        .output
        .as_ref()
        .unwrap()
        .contains("${workflow.project_name}"));

    let validate = workflow
        .tasks
        .get("validate_deployment")
        .expect("validate_deployment should exist");
    assert!(validate
        .output
        .as_ref()
        .unwrap()
        .contains("${workflow.environment}"));

    println!("✓ Variable interpolation in output paths test passed");
}

#[tokio::test]
async fn test_variable_context_api() {
    // Test the VariableContext API directly
    let mut ctx = VariableContext::new();

    // Set workflow variables
    ctx.insert(&Scope::Workflow, "project_name", json!("TestProject"));
    ctx.insert(&Scope::Workflow, "environment", json!("development"));
    ctx.insert(&Scope::Workflow, "max_retries", json!(3));
    ctx.insert(&Scope::Workflow, "enable_debug", json!(true));

    // Set agent variables
    ctx.insert(
        &Scope::Agent("analyzer".into()),
        "analysis_depth",
        json!("deep"),
    );
    ctx.insert(
        &Scope::Agent("builder".into()),
        "build_target",
        json!("release"),
    );

    // Set task variables
    ctx.insert(
        &Scope::Task("init_project".into()),
        "config_file",
        json!("config.yaml"),
    );

    // Test retrieval
    assert_eq!(
        ctx.get(Some(&Scope::Workflow), "project_name"),
        Some(&json!("TestProject"))
    );
    assert_eq!(
        ctx.get(Some(&Scope::Workflow), "environment"),
        Some(&json!("development"))
    );
    assert_eq!(
        ctx.get(Some(&Scope::Agent("analyzer".into())), "analysis_depth"),
        Some(&json!("deep"))
    );

    println!("✓ Variable context API test passed");
}

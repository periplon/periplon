//! Integration tests for subflow functionality

use periplon_sdk::dsl::{
    parse_workflow, validate_workflow, AgentSpec, DSLWorkflow, InputSpec, OutputDataSource,
    PermissionsSpec, SubflowSource, SubflowSpec, TaskSpec,
};
use std::collections::HashMap;

#[test]
fn test_inline_subflow_parsing() {
    let yaml = r#"
name: "Test Workflow"
version: "1.0.0"

subflows:
  validation:
    description: "Validate data"
    agents:
      validator:
        description: "Validation agent"
        tools:
          - Read
        permissions:
          mode: "default"
    tasks:
      check:
        description: "Run checks"
        agent: "validator"

agents:
  main:
    description: "Main agent"

tasks:
  run_validation:
    description: "Execute validation"
    subflow: "validation"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    assert_eq!(workflow.subflows.len(), 1);
    assert!(workflow.subflows.contains_key("validation"));

    let subflow = &workflow.subflows["validation"];
    assert_eq!(subflow.agents.len(), 1);
    assert_eq!(subflow.tasks.len(), 1);
}

#[test]
fn test_subflow_with_inputs() {
    let yaml = r#"
name: "Test Workflow"
version: "1.0.0"

subflows:
  process:
    description: "Process data"
    agents:
      processor:
        description: "Processor agent"
    tasks:
      process_task:
        description: "Process"
        agent: "processor"
    inputs:
      data_file:
        type: "string"
        required: true
        description: "Input file"
      mode:
        type: "string"
        required: false
        default: "normal"

agents:
  main:
    description: "Main agent"

tasks:
  run_process:
    description: "Run processing"
    subflow: "process"
    inputs:
      data_file: "data.json"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    let subflow = &workflow.subflows["process"];

    assert_eq!(subflow.inputs.len(), 2);
    assert!(subflow.inputs["data_file"].required);
    assert!(!subflow.inputs["mode"].required);
}

#[test]
fn test_subflow_validation_missing_reference() {
    let mut workflow = DSLWorkflow {
        name: "Test".to_string(),
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
        notifications: None,
        limits: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
    };

    let task = TaskSpec {
        description: "Test task".to_string(),
        subflow: Some("non_existent".to_string()),
        ..Default::default()
    };

    workflow.tasks.insert("task1".to_string(), task);

    let result = validate_workflow(&workflow);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("non-existent subflow"));
}

#[test]
fn test_subflow_validation_agent_and_subflow_mutually_exclusive() {
    let mut workflow = DSLWorkflow {
        name: "Test".to_string(),
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
        notifications: None,
        limits: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
    };

    workflow.agents.insert(
        "test_agent".to_string(),
        AgentSpec {
            description: "Test".to_string(),
            model: None,
            system_prompt: None,
            cwd: None,
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: vec![],
            permissions: PermissionsSpec::default(),
            max_turns: None,
        },
    );

    workflow.subflows.insert(
        "test_subflow".to_string(),
        SubflowSpec {
            description: None,
            source: None,
            agents: HashMap::new(),
            tasks: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        },
    );

    let task = TaskSpec {
        description: "Test task".to_string(),
        agent: Some("test_agent".to_string()),
        subflow: Some("test_subflow".to_string()),
        ..Default::default()
    };

    workflow.tasks.insert("task1".to_string(), task);

    let result = validate_workflow(&workflow);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Only one can be specified"));
}

#[test]
fn test_subflow_validation_missing_required_input() {
    let mut workflow = DSLWorkflow {
        name: "Test".to_string(),
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
        notifications: None,
        limits: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
    };

    let mut inputs = HashMap::new();
    inputs.insert(
        "required_input".to_string(),
        InputSpec {
            param_type: "string".to_string(),
            required: true,
            default: None,
            description: None,
        },
    );

    workflow.subflows.insert(
        "test_subflow".to_string(),
        SubflowSpec {
            description: None,
            source: None,
            agents: HashMap::new(),
            tasks: HashMap::new(),
            inputs,
            outputs: HashMap::new(),
        },
    );

    let task = TaskSpec {
        description: "Test task".to_string(),
        subflow: Some("test_subflow".to_string()),
        inputs: HashMap::new(), // Missing required input
        ..Default::default()
    };

    workflow.tasks.insert("task1".to_string(), task);

    let result = validate_workflow(&workflow);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("missing required input"));
}

#[test]
fn test_subflow_validation_valid_inline() {
    let mut workflow = DSLWorkflow {
        name: "Test".to_string(),
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
        notifications: None,
        limits: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
    };

    let mut subflow_agents = HashMap::new();
    subflow_agents.insert(
        "validator".to_string(),
        AgentSpec {
            description: "Validator".to_string(),
            model: None,
            system_prompt: None,
            cwd: None,
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: vec!["Read".to_string()],
            permissions: PermissionsSpec::default(),
            max_turns: None,
        },
    );

    let mut subflow_tasks = HashMap::new();
    subflow_tasks.insert(
        "validate".to_string(),
        TaskSpec {
            description: "Validate".to_string(),
            agent: Some("validator".to_string()),
            ..Default::default()
        },
    );

    workflow.subflows.insert(
        "validation".to_string(),
        SubflowSpec {
            description: Some("Validation subflow".to_string()),
            source: None,
            agents: subflow_agents,
            tasks: subflow_tasks,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        },
    );

    workflow.agents.insert(
        "main".to_string(),
        AgentSpec {
            description: "Main agent".to_string(),
            model: None,
            system_prompt: None,
            cwd: None,
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: vec![],
            permissions: PermissionsSpec::default(),
            max_turns: None,
        },
    );

    let task = TaskSpec {
        description: "Run validation".to_string(),
        subflow: Some("validation".to_string()),
        ..Default::default()
    };

    workflow.tasks.insert("task1".to_string(), task);

    let result = validate_workflow(&workflow);
    assert!(result.is_ok());
}

#[test]
fn test_external_subflow_source_parsing() {
    let yaml = r#"
name: "Test Workflow"
version: "1.0.0"

subflows:
  local_subflow:
    source:
      type: file
      path: "./subflow.yaml"

  git_subflow:
    source:
      type: git
      url: "https://github.com/org/repo.git"
      path: "workflows/subflow.yaml"
      reference: "v1.0.0"

  http_subflow:
    source:
      type: http
      url: "https://example.com/subflow.yaml"
      checksum: "sha256:abc123"

agents:
  main:
    description: "Main agent"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    assert_eq!(workflow.subflows.len(), 3);

    // Check file source
    let local = &workflow.subflows["local_subflow"];
    if let Some(SubflowSource::File { path }) = &local.source {
        assert_eq!(path, "./subflow.yaml");
    } else {
        panic!("Expected file source");
    }

    // Check git source
    let git = &workflow.subflows["git_subflow"];
    if let Some(SubflowSource::Git {
        url,
        path,
        reference,
    }) = &git.source
    {
        assert_eq!(url, "https://github.com/org/repo.git");
        assert_eq!(path, "workflows/subflow.yaml");
        assert_eq!(reference.as_ref().unwrap(), "v1.0.0");
    } else {
        panic!("Expected git source");
    }

    // Check HTTP source
    let http = &workflow.subflows["http_subflow"];
    if let Some(SubflowSource::Http { url, checksum }) = &http.source {
        assert_eq!(url, "https://example.com/subflow.yaml");
        assert_eq!(checksum.as_ref().unwrap(), "sha256:abc123");
    } else {
        panic!("Expected http source");
    }
}

#[test]
fn test_subflow_outputs_parsing() {
    let yaml = r#"
name: "Test Workflow"
version: "1.0.0"

subflows:
  process:
    description: "Processing subflow"
    agents:
      processor:
        description: "Processor"
    tasks:
      process:
        description: "Process"
        agent: "processor"
    outputs:
      result_file:
        source:
          type: file
          path: "result.json"
        description: "Processing result"
      state_value:
        source:
          type: state
          key: "final_state"
      task_output:
        source:
          type: task_output
          task: "process"

agents:
  main:
    description: "Main agent"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    let subflow = &workflow.subflows["process"];

    assert_eq!(subflow.outputs.len(), 3);

    if let OutputDataSource::File { path } = &subflow.outputs["result_file"].source {
        assert_eq!(path, "result.json");
    } else {
        panic!("Expected file output source");
    }

    if let OutputDataSource::State { key } = &subflow.outputs["state_value"].source {
        assert_eq!(key, "final_state");
    } else {
        panic!("Expected state output source");
    }

    if let OutputDataSource::TaskOutput { task } = &subflow.outputs["task_output"].source {
        assert_eq!(task, "process");
    } else {
        panic!("Expected task_output source");
    }
}

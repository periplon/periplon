//! Integration tests for loop functionality
//!
//! These tests verify ForEach and Repeat loop patterns work correctly
//! with variable substitution, state tracking, and result collection.

use periplon_sdk::dsl::{parse_workflow, CollectionSource, DSLWorkflow, LoopSpec, TaskSpec};
use periplon_sdk::error::Result;

#[test]
fn test_parse_foreach_loop_inline() -> Result<()> {
    let yaml = r#"
name: "ForEach Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []
    permissions:
      mode: "default"

tasks:
  test_task:
    description: "Process {{item}}"
    agent: "test_agent"
    loop:
      type: for_each
      collection:
        source: inline
        items: ["a", "b", "c"]
      iterator: "item"
    loop_control:
      collect_results: true
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    assert_eq!(workflow.name, "ForEach Test");
    assert_eq!(workflow.tasks.len(), 1);

    let task = workflow.tasks.get("test_task").unwrap();
    assert!(task.loop_spec.is_some());

    if let Some(LoopSpec::ForEach {
        collection,
        iterator,
        ..
    }) = &task.loop_spec
    {
        assert_eq!(iterator, "item");

        if let CollectionSource::Inline { items } = collection {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], serde_json::Value::String("a".to_string()));
        } else {
            panic!("Expected inline collection");
        }
    } else {
        panic!("Expected ForEach loop spec");
    }

    assert!(task.loop_control.is_some());
    assert!(task.loop_control.as_ref().unwrap().collect_results);

    Ok(())
}

#[test]
fn test_parse_foreach_loop_range() -> Result<()> {
    let yaml = r#"
name: "ForEach Range Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Process number {{num}}"
    agent: "test_agent"
    loop:
      type: for_each
      collection:
        source: range
        start: 1
        end: 10
        step: 2
      iterator: "num"
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::ForEach { collection, .. }) = &task.loop_spec {
        if let CollectionSource::Range { start, end, step } = collection {
            assert_eq!(*start, 1);
            assert_eq!(*end, 10);
            assert_eq!(*step, Some(2));
        } else {
            panic!("Expected range collection");
        }
    } else {
        panic!("Expected ForEach loop spec");
    }

    Ok(())
}

#[test]
fn test_parse_repeat_loop() -> Result<()> {
    let yaml = r#"
name: "Repeat Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Iteration {{iteration}}"
    agent: "test_agent"
    loop:
      type: repeat
      count: 5
      iterator: "index"
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::Repeat {
        count, iterator, ..
    }) = &task.loop_spec
    {
        assert_eq!(*count, 5);
        assert_eq!(iterator.as_ref().unwrap(), "index");
    } else {
        panic!("Expected Repeat loop spec");
    }

    Ok(())
}

#[test]
fn test_parse_foreach_loop_with_file() -> Result<()> {
    let yaml = r#"
name: "ForEach File Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Process {{entry}}"
    agent: "test_agent"
    loop:
      type: for_each
      collection:
        source: file
        path: "/tmp/data.json"
        format: json
      iterator: "entry"
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::ForEach { collection, .. }) = &task.loop_spec {
        if let CollectionSource::File { path, format } = collection {
            assert_eq!(path, "/tmp/data.json");
            assert!(matches!(format, periplon_sdk::dsl::FileFormat::Json));
        } else {
            panic!("Expected file collection");
        }
    } else {
        panic!("Expected ForEach loop spec");
    }

    Ok(())
}

#[test]
fn test_parse_loop_control_with_break_condition() -> Result<()> {
    let yaml = r#"
name: "Loop Control Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Process {{item}}"
    agent: "test_agent"
    loop:
      type: for_each
      collection:
        source: inline
        items: [1, 2, 3]
      iterator: "item"
    loop_control:
      break_condition:
        type: state_equals
        key: "stop"
        value: true
      collect_results: true
      result_key: "results"
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    assert!(task.loop_control.is_some());

    let loop_control = task.loop_control.as_ref().unwrap();
    assert!(loop_control.break_condition.is_some());
    assert!(loop_control.collect_results);
    assert_eq!(loop_control.result_key.as_ref().unwrap(), "results");

    Ok(())
}

#[test]
fn test_parse_parallel_foreach() -> Result<()> {
    let yaml = r#"
name: "Parallel ForEach Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Process {{item}}"
    agent: "test_agent"
    loop:
      type: for_each
      collection:
        source: inline
        items: [1, 2, 3, 4, 5]
      iterator: "item"
      parallel: true
      max_parallel: 3
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::ForEach {
        parallel,
        max_parallel,
        ..
    }) = &task.loop_spec
    {
        assert!(*parallel);
        assert_eq!(*max_parallel, Some(3));
    } else {
        panic!("Expected ForEach loop spec");
    }

    Ok(())
}

#[test]
fn test_variable_substitution_in_description() {
    use periplon_sdk::dsl::{substitute_task_variables, LoopContext};

    let task = TaskSpec {
        description: "Process file {{filename}} on iteration {{iteration}}".to_string(),
        ..Default::default()
    };

    let mut context = LoopContext::new(3);
    context.set_variable(
        "filename".to_string(),
        serde_json::Value::String("data.txt".to_string()),
    );

    let substituted = substitute_task_variables(&task, &context);

    assert_eq!(
        substituted.description,
        "Process file data.txt on iteration 3"
    );
}

#[test]
fn test_variable_substitution_in_output_path() {
    use periplon_sdk::dsl::{substitute_task_variables, LoopContext};

    let task = TaskSpec {
        description: "Task".to_string(),
        output: Some("output_{{index}}.txt".to_string()),
        ..Default::default()
    };

    let mut context = LoopContext::new(5);
    context.set_variable("index".to_string(), serde_json::Value::Number(42.into()));

    let substituted = substitute_task_variables(&task, &context);

    assert_eq!(substituted.output, Some("output_42.txt".to_string()));
}

#[test]
fn test_loop_state_tracking() {
    use periplon_sdk::dsl::{TaskStatus, WorkflowState};

    let mut state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

    // Initialize loop
    state.init_loop("task1", Some(5));

    assert!(state.has_loop("task1"));
    assert_eq!(state.get_loop_progress("task1"), Some(0.0));

    // Update iterations
    state.update_loop_iteration(
        "task1",
        0,
        TaskStatus::Completed,
        Some(serde_json::Value::Number(0.into())),
    );

    state.update_loop_iteration(
        "task1",
        1,
        TaskStatus::Completed,
        Some(serde_json::Value::Number(1.into())),
    );

    // Check progress - current_iteration is 1, so progress is based on that
    // But get_loop_progress uses current_iteration / total, not completed count
    let progress = state.get_loop_progress("task1").unwrap();
    assert!((progress - 20.0).abs() < 0.1); // 1/5 = 20% (current_iteration is 1, 0-indexed)

    // Check loop state
    let loop_state = state.get_loop_state("task1").unwrap();
    assert_eq!(loop_state.current_iteration, 1);
    assert_eq!(loop_state.total_iterations, Some(5));
    assert_eq!(loop_state.iteration_statuses.len(), 2);
}

#[test]
fn test_loop_result_collection() {
    use periplon_sdk::dsl::WorkflowState;

    let mut state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

    state.init_loop("task1", Some(3));

    // Store results
    state.store_loop_result("task1", serde_json::Value::String("result1".to_string()));
    state.store_loop_result("task1", serde_json::Value::String("result2".to_string()));
    state.store_loop_result("task1", serde_json::Value::String("result3".to_string()));

    // Retrieve results
    let results = state.get_loop_results("task1").unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], serde_json::Value::String("result1".to_string()));
    assert_eq!(results[1], serde_json::Value::String("result2".to_string()));
    assert_eq!(results[2], serde_json::Value::String("result3".to_string()));
}

#[test]
fn test_parse_while_loop() -> Result<()> {
    let yaml = r#"
name: "While Loop Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Check condition {{iteration}}"
    agent: "test_agent"
    loop:
      type: while
      condition:
        type: state_equals
        key: "ready"
        value: true
      max_iterations: 10
      iteration_variable: "count"
      delay_between_secs: 2
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::While {
        condition,
        max_iterations,
        iteration_variable,
        delay_between_secs,
    }) = &task.loop_spec
    {
        assert_eq!(*max_iterations, 10);
        assert_eq!(iteration_variable.as_ref().unwrap(), "count");
        assert_eq!(*delay_between_secs, Some(2));

        // Check condition structure
        match condition.as_ref() {
            periplon_sdk::dsl::ConditionSpec::Single(cond) => {
                if let periplon_sdk::dsl::Condition::StateEquals { key, value } = cond {
                    assert_eq!(key, "ready");
                    assert_eq!(*value, serde_json::Value::Bool(true));
                } else {
                    panic!("Expected StateEquals condition");
                }
            }
            _ => panic!("Expected single condition"),
        }
    } else {
        panic!("Expected While loop spec");
    }

    Ok(())
}

#[test]
fn test_parse_repeat_until_loop() -> Result<()> {
    let yaml = r#"
name: "RepeatUntil Loop Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Retry {{iteration}}"
    agent: "test_agent"
    loop:
      type: repeat_until
      condition:
        type: state_equals
        key: "success"
        value: true
      min_iterations: 2
      max_iterations: 5
      iteration_variable: "attempt"
      delay_between_secs: 3
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::RepeatUntil {
        condition,
        min_iterations,
        max_iterations,
        iteration_variable,
        delay_between_secs,
    }) = &task.loop_spec
    {
        assert_eq!(*min_iterations, Some(2));
        assert_eq!(*max_iterations, 5);
        assert_eq!(iteration_variable.as_ref().unwrap(), "attempt");
        assert_eq!(*delay_between_secs, Some(3));

        // Check condition structure
        match condition.as_ref() {
            periplon_sdk::dsl::ConditionSpec::Single(cond) => {
                if let periplon_sdk::dsl::Condition::StateEquals { key, value } = cond {
                    assert_eq!(key, "success");
                    assert_eq!(*value, serde_json::Value::Bool(true));
                } else {
                    panic!("Expected StateEquals condition");
                }
            }
            _ => panic!("Expected single condition"),
        }
    } else {
        panic!("Expected RepeatUntil loop spec");
    }

    Ok(())
}

#[test]
fn test_parse_while_loop_with_defaults() -> Result<()> {
    let yaml = r#"
name: "While Loop Defaults Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Task"
    agent: "test_agent"
    loop:
      type: while
      condition:
        type: state_equals
        key: "active"
        value: true
      max_iterations: 100
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::While {
        iteration_variable,
        delay_between_secs,
        ..
    }) = &task.loop_spec
    {
        // Should have default None values
        assert!(iteration_variable.is_none());
        assert!(delay_between_secs.is_none());
    } else {
        panic!("Expected While loop spec");
    }

    Ok(())
}

#[test]
fn test_parse_repeat_until_with_min_default() -> Result<()> {
    let yaml = r#"
name: "RepeatUntil Min Default Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Task"
    agent: "test_agent"
    loop:
      type: repeat_until
      condition:
        type: state_equals
        key: "done"
        value: true
      max_iterations: 10
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::RepeatUntil { min_iterations, .. }) = &task.loop_spec {
        // min_iterations should be None (defaults to 1 in execution)
        assert!(min_iterations.is_none());
    } else {
        panic!("Expected RepeatUntil loop spec");
    }

    Ok(())
}

#[test]
fn test_conditional_loop_with_complex_condition() -> Result<()> {
    let yaml = r#"
name: "Complex Condition Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Task"
    agent: "test_agent"
    loop:
      type: while
      condition:
        and:
          - type: state_equals
            key: "ready"
            value: true
          - type: state_equals
            key: "count"
            value: 0
      max_iterations: 5
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::While { condition, .. }) = &task.loop_spec {
        // Check it's an And condition with 2 sub-conditions
        match condition.as_ref() {
            periplon_sdk::dsl::ConditionSpec::And { and } => {
                assert_eq!(and.len(), 2);
            }
            _ => panic!("Expected And condition"),
        }
    } else {
        panic!("Expected While loop spec");
    }

    Ok(())
}

#[test]
fn test_parse_parallel_foreach_with_max_parallel() -> Result<()> {
    let yaml = r#"
name: "Parallel ForEach Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Task"
    agent: "test_agent"
    loop:
      type: for_each
      collection:
        source: inline
        items: [1, 2, 3, 4, 5]
      iterator: "item"
      parallel: true
      max_parallel: 3
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::ForEach {
        parallel,
        max_parallel,
        ..
    }) = &task.loop_spec
    {
        assert!(*parallel);
        assert_eq!(*max_parallel, Some(3));
    } else {
        panic!("Expected ForEach loop spec");
    }

    Ok(())
}

#[test]
fn test_parse_parallel_repeat_with_max_parallel() -> Result<()> {
    let yaml = r#"
name: "Parallel Repeat Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Task {{index}}"
    agent: "test_agent"
    loop:
      type: repeat
      count: 10
      iterator: "index"
      parallel: true
      max_parallel: 5
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::Repeat {
        count,
        parallel,
        max_parallel,
        ..
    }) = &task.loop_spec
    {
        assert_eq!(*count, 10);
        assert!(*parallel);
        assert_eq!(*max_parallel, Some(5));
    } else {
        panic!("Expected Repeat loop spec");
    }

    Ok(())
}

#[test]
fn test_loop_spec_max_parallel_helper() {
    use periplon_sdk::dsl::schema::{CollectionSource, LoopSpec};

    // ForEach with max_parallel
    let foreach_loop = LoopSpec::ForEach {
        collection: CollectionSource::Inline {
            items: vec![serde_json::Value::Number(1.into())],
        },
        iterator: "item".to_string(),
        parallel: true,
        max_parallel: Some(3),
    };
    assert_eq!(foreach_loop.max_parallel(), Some(3));

    // Repeat with max_parallel
    let repeat_loop = LoopSpec::Repeat {
        count: 5,
        iterator: Some("i".to_string()),
        parallel: true,
        max_parallel: Some(2),
    };
    assert_eq!(repeat_loop.max_parallel(), Some(2));

    // While loop (no max_parallel)
    let while_loop = LoopSpec::While {
        condition: Box::new(periplon_sdk::dsl::ConditionSpec::Single(
            periplon_sdk::dsl::Condition::StateEquals {
                key: "ready".to_string(),
                value: serde_json::Value::Bool(true),
            },
        )),
        max_iterations: 10,
        iteration_variable: None,
        delay_between_secs: None,
    };
    assert_eq!(while_loop.max_parallel(), None);
}

#[test]
fn test_parallel_foreach_without_max_parallel() -> Result<()> {
    let yaml = r#"
name: "Parallel ForEach Default Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Task"
    agent: "test_agent"
    loop:
      type: for_each
      collection:
        source: inline
        items: [1, 2, 3]
      iterator: "item"
      parallel: true
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    if let Some(LoopSpec::ForEach {
        parallel,
        max_parallel,
        ..
    }) = &task.loop_spec
    {
        assert!(*parallel);
        // max_parallel should be None, will default at runtime
        assert_eq!(*max_parallel, None);
    } else {
        panic!("Expected ForEach loop spec");
    }

    Ok(())
}

#[test]
fn test_parse_loop_with_break_and_continue_conditions() -> Result<()> {
    let yaml = r#"
name: "Loop Control Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Task"
    agent: "test_agent"
    loop:
      type: for_each
      collection:
        source: inline
        items: [1, 2, 3]
      iterator: "item"
    loop_control:
      break_condition:
        type: state_equals
        key: "should_break"
        value: true
      continue_condition:
        type: state_equals
        key: "should_skip"
        value: true
      collect_results: true
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    assert!(task.loop_control.is_some());
    let loop_control = task.loop_control.as_ref().unwrap();
    assert!(loop_control.break_condition.is_some());
    assert!(loop_control.continue_condition.is_some());
    assert!(loop_control.collect_results);

    Ok(())
}

#[test]
fn test_parse_loop_with_timeout() -> Result<()> {
    let yaml = r#"
name: "Loop Timeout Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Task"
    agent: "test_agent"
    loop:
      type: repeat
      count: 100
      iterator: "i"
    loop_control:
      timeout_secs: 30
      collect_results: true
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    assert!(task.loop_control.is_some());
    let loop_control = task.loop_control.as_ref().unwrap();
    assert_eq!(loop_control.timeout_secs, Some(30));
    assert!(loop_control.collect_results);

    Ok(())
}

#[test]
fn test_parse_combined_advanced_features() -> Result<()> {
    let yaml = r#"
name: "Combined Features Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Task {{item}}"
    agent: "test_agent"
    loop:
      type: for_each
      collection:
        source: inline
        items: ["a", "b", "c"]
      iterator: "item"
      parallel: true
      max_parallel: 2
    loop_control:
      break_condition:
        type: state_equals
        key: "error"
        value: true
      continue_condition:
        type: state_equals
        key: "skip"
        value: true
      timeout_secs: 60
      collect_results: true
      result_key: "results"
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    // Check loop spec
    if let Some(LoopSpec::ForEach {
        parallel,
        max_parallel,
        ..
    }) = &task.loop_spec
    {
        assert!(*parallel);
        assert_eq!(*max_parallel, Some(2));
    } else {
        panic!("Expected ForEach loop spec");
    }

    // Check loop control
    assert!(task.loop_control.is_some());
    let loop_control = task.loop_control.as_ref().unwrap();
    assert!(loop_control.break_condition.is_some());
    assert!(loop_control.continue_condition.is_some());
    assert_eq!(loop_control.timeout_secs, Some(60));
    assert!(loop_control.collect_results);
    assert_eq!(loop_control.result_key.as_ref().unwrap(), "results");

    Ok(())
}

#[test]
fn test_parse_loop_with_checkpoint_interval() -> Result<()> {
    let yaml = r#"
name: "Checkpoint Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Task"
    agent: "test_agent"
    loop:
      type: for_each
      collection:
        source: inline
        items: [1, 2, 3, 4, 5]
      iterator: "item"
    loop_control:
      checkpoint_interval: 2
      collect_results: true
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    assert!(task.loop_control.is_some());
    let loop_control = task.loop_control.as_ref().unwrap();
    assert_eq!(loop_control.checkpoint_interval, Some(2));
    assert!(loop_control.collect_results);

    Ok(())
}

#[test]
fn test_loop_state_resume_methods() {
    use periplon_sdk::dsl::{TaskStatus, WorkflowState};

    let mut state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

    // Initialize loop
    state.init_loop("task1", Some(10));

    // Mark some iterations as completed
    state.update_loop_iteration("task1", 0, TaskStatus::Completed, None);
    state.update_loop_iteration("task1", 1, TaskStatus::Completed, None);
    state.update_loop_iteration("task1", 2, TaskStatus::Completed, None);
    state.update_loop_iteration("task1", 3, TaskStatus::Running, None);

    // Test is_iteration_completed
    assert!(state.is_iteration_completed("task1", 0));
    assert!(state.is_iteration_completed("task1", 1));
    assert!(state.is_iteration_completed("task1", 2));
    assert!(!state.is_iteration_completed("task1", 3)); // Running, not completed
    assert!(!state.is_iteration_completed("task1", 4)); // Not started

    // Test get_last_completed_iteration
    assert_eq!(state.get_last_completed_iteration("task1"), Some(2));

    // Test with non-existent task
    assert_eq!(state.get_last_completed_iteration("nonexistent"), None);
    assert!(!state.is_iteration_completed("nonexistent", 0));
}

#[test]
fn test_checkpoint_all_features_combined() -> Result<()> {
    let yaml = r#"
name: "All Features Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: []

tasks:
  test_task:
    description: "Task {{i}}"
    agent: "test_agent"
    loop:
      type: repeat
      count: 50
      iterator: "i"
    loop_control:
      checkpoint_interval: 10
      break_condition:
        type: state_equals
        key: "stop"
        value: true
      continue_condition:
        type: state_equals
        key: "skip"
        value: true
      timeout_secs: 60
      collect_results: true
      result_key: "all_results"
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("test_task").unwrap();

    // Check loop spec
    assert!(task.loop_spec.is_some());

    // Check loop control has all advanced features
    assert!(task.loop_control.is_some());
    let loop_control = task.loop_control.as_ref().unwrap();
    assert_eq!(loop_control.checkpoint_interval, Some(10));
    assert!(loop_control.break_condition.is_some());
    assert!(loop_control.continue_condition.is_some());
    assert_eq!(loop_control.timeout_secs, Some(60));
    assert!(loop_control.collect_results);
    assert_eq!(loop_control.result_key.as_ref().unwrap(), "all_results");

    Ok(())
}

// ============================================================================
// Phase 7: HTTP Collection Source Tests
// ============================================================================

#[test]
fn test_parse_http_collection_basic() -> Result<()> {
    let yaml = r#"
name: "HTTP Collection Test"
version: "1.0.0"

agents:
  api_agent:
    description: "API consumer"
    tools: []
    permissions:
      mode: "default"

tasks:
  fetch_data:
    description: "Process {{item}}"
    agent: "api_agent"
    loop:
      type: for_each
      collection:
        source: http
        url: "https://api.example.com/data"
        method: "GET"
        format: json
      iterator: "item"
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("fetch_data").unwrap();
    assert!(task.loop_spec.is_some());

    if let Some(LoopSpec::ForEach {
        collection,
        iterator,
        ..
    }) = &task.loop_spec
    {
        assert_eq!(iterator, "item");

        if let CollectionSource::Http {
            url,
            method,
            headers,
            body,
            format,
            json_path,
        } = collection
        {
            assert_eq!(url, "https://api.example.com/data");
            assert_eq!(method, "GET");
            assert!(headers.is_none());
            assert!(body.is_none());
            assert_eq!(format, &periplon_sdk::dsl::FileFormat::Json);
            assert!(json_path.is_none());
        } else {
            panic!("Expected HTTP collection source");
        }
    } else {
        panic!("Expected ForEach loop");
    }

    Ok(())
}

#[test]
fn test_parse_http_collection_with_headers() -> Result<()> {
    let yaml = r#"
name: "HTTP Collection with Headers"
version: "1.0.0"

agents:
  api_agent:
    description: "API consumer"
    tools: []
    permissions:
      mode: "default"

tasks:
  fetch_data:
    description: "Process {{item}}"
    agent: "api_agent"
    loop:
      type: for_each
      collection:
        source: http
        url: "https://api.example.com/data"
        method: "POST"
        headers:
          Authorization: "Bearer token123"
          Content-Type: "application/json"
        body: '{"query": "test"}'
        format: json
      iterator: "item"
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("fetch_data").unwrap();

    if let Some(LoopSpec::ForEach { collection, .. }) = &task.loop_spec {
        if let CollectionSource::Http {
            method,
            headers,
            body,
            ..
        } = collection
        {
            assert_eq!(method, "POST");
            assert!(headers.is_some());

            let headers_map = headers.as_ref().unwrap();
            assert_eq!(headers_map.get("Authorization").unwrap(), "Bearer token123");
            assert_eq!(headers_map.get("Content-Type").unwrap(), "application/json");

            assert!(body.is_some());
            assert_eq!(body.as_ref().unwrap(), r#"{"query": "test"}"#);
        } else {
            panic!("Expected HTTP collection source");
        }
    }

    Ok(())
}

#[test]
fn test_parse_http_collection_with_json_path() -> Result<()> {
    let yaml = r#"
name: "HTTP Collection with JSON Path"
version: "1.0.0"

agents:
  api_agent:
    description: "API consumer"
    tools: []
    permissions:
      mode: "default"

tasks:
  fetch_data:
    description: "Process {{item}}"
    agent: "api_agent"
    loop:
      type: for_each
      collection:
        source: http
        url: "https://api.example.com/data"
        method: "GET"
        format: json
        json_path: "data.items"
      iterator: "item"
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;

    let task = workflow.tasks.get("fetch_data").unwrap();

    if let Some(LoopSpec::ForEach { collection, .. }) = &task.loop_spec {
        if let CollectionSource::Http { json_path, .. } = collection {
            assert!(json_path.is_some());
            assert_eq!(json_path.as_ref().unwrap(), "data.items");
        } else {
            panic!("Expected HTTP collection source");
        }
    }

    Ok(())
}

#[test]
fn test_http_collection_validation_invalid_url() -> Result<()> {
    let yaml = r#"
name: "Invalid URL Test"
version: "1.0.0"

agents:
  api_agent:
    description: "API consumer"
    tools: []
    permissions:
      mode: "default"

tasks:
  fetch_data:
    description: "Process {{item}}"
    agent: "api_agent"
    loop:
      type: for_each
      collection:
        source: http
        url: "ftp://invalid.com/data"
        method: "GET"
        format: json
      iterator: "item"
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;
    let result = periplon_sdk::dsl::validate_workflow(&workflow);

    // Should fail validation because URL doesn't start with http:// or https://
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_http_collection_validation_invalid_method() -> Result<()> {
    let yaml = r#"
name: "Invalid Method Test"
version: "1.0.0"

agents:
  api_agent:
    description: "API consumer"
    tools: []
    permissions:
      mode: "default"

tasks:
  fetch_data:
    description: "Process {{item}}"
    agent: "api_agent"
    loop:
      type: for_each
      collection:
        source: http
        url: "https://api.example.com/data"
        method: "INVALID"
        format: json
      iterator: "item"
"#;

    let workflow: DSLWorkflow = parse_workflow(yaml)?;
    let result = periplon_sdk::dsl::validate_workflow(&workflow);

    // Should fail validation because method is invalid
    assert!(result.is_err());

    Ok(())
}

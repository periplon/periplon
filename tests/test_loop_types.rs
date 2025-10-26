//! Test all loop types: for_each, repeat, while, repeat_until

use periplon_sdk::dsl::parser::parse_workflow_file;
use periplon_sdk::dsl::schema::{LoopSpec, CollectionSource, ConditionSpec};
use serde_json::json;

#[tokio::test]
async fn test_foreach_inline_collection() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_foreach_inline")
        .expect("test_foreach_inline should exist");

    // Verify loop specification
    assert!(task.loop_spec.is_some(), "Task should have loop spec");

    match task.loop_spec.as_ref().unwrap() {
        LoopSpec::ForEach { collection, iterator, parallel, max_parallel } => {
            // Verify iterator name
            assert_eq!(iterator, "fruit");

            // Verify not parallel
            assert_eq!(*parallel, false);
            assert!(max_parallel.is_none());

            // Verify inline collection
            match collection {
                CollectionSource::Inline { items } => {
                    assert_eq!(items.len(), 4);
                    assert_eq!(items[0], json!("apple"));
                    assert_eq!(items[1], json!("banana"));
                    assert_eq!(items[2], json!("cherry"));
                    assert_eq!(items[3], json!("date"));
                }
                _ => panic!("Expected inline collection"),
            }
        }
        _ => panic!("Expected ForEach loop"),
    }

    // Verify subtasks exist
    assert_eq!(task.subtasks.len(), 1);
    assert!(task.subtasks[0].contains_key("process_item"));

    println!("✓ for_each with inline collection test passed");
}

#[tokio::test]
async fn test_foreach_range_collection() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_foreach_range")
        .expect("test_foreach_range should exist");

    match task.loop_spec.as_ref().unwrap() {
        LoopSpec::ForEach { collection, iterator, parallel, .. } => {
            assert_eq!(iterator, "number");
            assert_eq!(*parallel, false);

            match collection {
                CollectionSource::Range { start, end, step } => {
                    assert_eq!(*start, 1);
                    assert_eq!(*end, 5);
                    assert_eq!(*step, Some(1));
                }
                _ => panic!("Expected range collection"),
            }
        }
        _ => panic!("Expected ForEach loop"),
    }

    println!("✓ for_each with range collection test passed");
}

#[tokio::test]
async fn test_foreach_parallel() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_foreach_parallel")
        .expect("test_foreach_parallel should exist");

    match task.loop_spec.as_ref().unwrap() {
        LoopSpec::ForEach { collection, iterator, parallel, max_parallel } => {
            assert_eq!(iterator, "task_name");
            assert_eq!(*parallel, true, "Should be parallel");
            assert_eq!(*max_parallel, Some(2), "Should have max_parallel=2");

            match collection {
                CollectionSource::Inline { items } => {
                    assert_eq!(items.len(), 4);
                }
                _ => panic!("Expected inline collection"),
            }
        }
        _ => panic!("Expected ForEach loop"),
    }

    println!("✓ for_each with parallel execution test passed");
}

#[tokio::test]
async fn test_repeat_loop() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_repeat")
        .expect("test_repeat should exist");

    assert!(task.loop_spec.is_some());

    match task.loop_spec.as_ref().unwrap() {
        LoopSpec::Repeat { count, iterator, parallel, max_parallel } => {
            assert_eq!(*count, 3, "Should repeat 3 times");
            assert_eq!(iterator, &Some("index".to_string()));
            assert_eq!(*parallel, false);
            assert!(max_parallel.is_none());
        }
        _ => panic!("Expected Repeat loop"),
    }

    println!("✓ repeat loop test passed");
}

#[tokio::test]
async fn test_repeat_parallel() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_repeat_parallel")
        .expect("test_repeat_parallel should exist");

    match task.loop_spec.as_ref().unwrap() {
        LoopSpec::Repeat { count, iterator, parallel, max_parallel } => {
            assert_eq!(*count, 4);
            assert_eq!(iterator, &Some("batch".to_string()));
            assert_eq!(*parallel, true);
            assert_eq!(*max_parallel, Some(2));
        }
        _ => panic!("Expected Repeat loop"),
    }

    println!("✓ repeat with parallel execution test passed");
}

#[tokio::test]
async fn test_while_loop() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_while")
        .expect("test_while should exist");

    assert!(task.loop_spec.is_some());

    match task.loop_spec.as_ref().unwrap() {
        LoopSpec::While { condition, max_iterations, iteration_variable, delay_between_secs } => {
            assert_eq!(*max_iterations, 5, "Should have max 5 iterations");
            assert_eq!(iteration_variable, &Some("iteration".to_string()));
            assert_eq!(*delay_between_secs, Some(1));

            // Verify condition
            assert!(matches!(condition.as_ref(), ConditionSpec::Single(_)));
        }
        _ => panic!("Expected While loop"),
    }

    println!("✓ while loop test passed");
}

#[tokio::test]
async fn test_repeat_until_loop() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_repeat_until")
        .expect("test_repeat_until should exist");

    assert!(task.loop_spec.is_some());

    match task.loop_spec.as_ref().unwrap() {
        LoopSpec::RepeatUntil { condition, min_iterations, max_iterations, iteration_variable, delay_between_secs } => {
            assert_eq!(*min_iterations, Some(1));
            assert_eq!(*max_iterations, 3);
            assert_eq!(iteration_variable, &Some("attempt".to_string()));
            assert_eq!(*delay_between_secs, Some(1));

            // Verify condition
            assert!(matches!(condition.as_ref(), ConditionSpec::Single(_)));
        }
        _ => panic!("Expected RepeatUntil loop"),
    }

    println!("✓ repeat_until loop test passed");
}

#[tokio::test]
async fn test_loop_with_break_condition() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_loop_break")
        .expect("test_loop_break should exist");

    // Verify loop control
    assert!(task.loop_control.is_some());
    let control = task.loop_control.as_ref().unwrap();

    assert!(control.break_condition.is_some(), "Should have break condition");
    assert_eq!(control.collect_results, true);
    assert_eq!(control.result_key, Some("break_results".to_string()));

    println!("✓ loop with break condition test passed");
}

#[tokio::test]
async fn test_loop_with_continue_condition() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_loop_continue")
        .expect("test_loop_continue should exist");

    // Verify loop control
    assert!(task.loop_control.is_some());
    let control = task.loop_control.as_ref().unwrap();

    assert!(control.continue_condition.is_some(), "Should have continue condition");
    assert_eq!(control.collect_results, true);
    assert_eq!(control.result_key, Some("continue_results".to_string()));

    println!("✓ loop with continue condition test passed");
}

#[tokio::test]
async fn test_loop_with_result_collection() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_loop_results")
        .expect("test_loop_results should exist");

    // Verify loop control
    assert!(task.loop_control.is_some());
    let control = task.loop_control.as_ref().unwrap();

    assert_eq!(control.collect_results, true);
    assert_eq!(control.result_key, Some("collected_items".to_string()));

    // Verify subtask has outputs
    let subtask = task.subtasks[0].get("collect_task").unwrap();
    assert_eq!(subtask.outputs.len(), 1);
    assert!(subtask.outputs.contains_key("item_result"));

    println!("✓ loop with result collection test passed");
}

#[tokio::test]
async fn test_nested_loops() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let outer_task = workflow.tasks.get("test_nested_loops")
        .expect("test_nested_loops should exist");

    // Verify outer loop
    assert!(outer_task.loop_spec.is_some());
    match outer_task.loop_spec.as_ref().unwrap() {
        LoopSpec::Repeat { count, iterator, .. } => {
            assert_eq!(*count, 2);
            assert_eq!(iterator, &Some("outer".to_string()));
        }
        _ => panic!("Expected Repeat loop for outer"),
    }

    // Verify inner loop subtask
    assert_eq!(outer_task.subtasks.len(), 1);
    let inner_task = outer_task.subtasks[0].get("inner_loop").unwrap();

    assert!(inner_task.loop_spec.is_some());
    match inner_task.loop_spec.as_ref().unwrap() {
        LoopSpec::ForEach { iterator, .. } => {
            assert_eq!(iterator, "inner");
        }
        _ => panic!("Expected ForEach loop for inner"),
    }

    // Verify innermost subtask
    assert_eq!(inner_task.subtasks.len(), 1);
    assert!(inner_task.subtasks[0].contains_key("nested_task"));

    println!("✓ nested loops test passed");
}

#[tokio::test]
async fn test_loop_with_timeout() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_loop_timeout")
        .expect("test_loop_timeout should exist");

    // Verify loop control with timeout
    assert!(task.loop_control.is_some());
    let control = task.loop_control.as_ref().unwrap();

    assert_eq!(control.timeout_secs, Some(30));
    assert_eq!(control.checkpoint_interval, Some(10));

    println!("✓ loop with timeout test passed");
}

#[tokio::test]
async fn test_loop_with_complex_condition() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    let task = workflow.tasks.get("test_complex_condition")
        .expect("test_complex_condition should exist");

    // Verify loop control with complex condition
    assert!(task.loop_control.is_some());
    let control = task.loop_control.as_ref().unwrap();

    assert!(control.break_condition.is_some());

    // Verify it's an AND condition
    match control.break_condition.as_ref().unwrap() {
        ConditionSpec::And { and } => {
            assert_eq!(and.len(), 2, "Should have 2 conditions in AND");
        }
        _ => panic!("Expected AND condition"),
    }

    println!("✓ loop with complex condition test passed");
}

#[tokio::test]
async fn test_loop_variable_substitution() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    // Test variable substitution in for_each
    let foreach_task = workflow.tasks.get("test_foreach_inline").unwrap();
    let subtask = foreach_task.subtasks[0].get("process_item").unwrap();
    assert!(subtask.description.contains("${loop.fruit}"));
    assert!(subtask.output.as_ref().unwrap().contains("${loop.fruit}"));

    // Test variable substitution in repeat
    let repeat_task = workflow.tasks.get("test_repeat").unwrap();
    let subtask = repeat_task.subtasks[0].get("repeat_task").unwrap();
    assert!(subtask.description.contains("${loop.index}"));
    assert!(subtask.output.as_ref().unwrap().contains("${loop.index}"));

    // Test variable substitution in while
    let while_task = workflow.tasks.get("test_while").unwrap();
    let subtask = while_task.subtasks[0].get("while_task").unwrap();
    assert!(subtask.description.contains("${loop.iteration}"));

    // Test variable substitution in nested loops
    let nested_task = workflow.tasks.get("test_nested_loops").unwrap();
    let inner_loop = nested_task.subtasks[0].get("inner_loop").unwrap();
    let innermost = inner_loop.subtasks[0].get("nested_task").unwrap();
    assert!(innermost.description.contains("${loop.outer}"));
    assert!(innermost.description.contains("${loop.inner}"));
    assert!(innermost.output.as_ref().unwrap().contains("${loop.outer}"));
    assert!(innermost.output.as_ref().unwrap().contains("${loop.inner}"));

    println!("✓ loop variable substitution test passed");
}

#[tokio::test]
async fn test_all_loop_types_present() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    // Count each loop type
    let mut foreach_count = 0;
    let mut repeat_count = 0;
    let mut while_count = 0;
    let mut repeat_until_count = 0;

    for (_task_id, task) in &workflow.tasks {
        if let Some(loop_spec) = &task.loop_spec {
            match loop_spec {
                LoopSpec::ForEach { .. } => foreach_count += 1,
                LoopSpec::Repeat { .. } => repeat_count += 1,
                LoopSpec::While { .. } => while_count += 1,
                LoopSpec::RepeatUntil { .. } => repeat_until_count += 1,
            }
        }
    }

    assert!(foreach_count >= 3, "Should have at least 3 for_each loops");
    assert!(repeat_count >= 2, "Should have at least 2 repeat loops");
    assert_eq!(while_count, 1, "Should have 1 while loop");
    assert_eq!(repeat_until_count, 1, "Should have 1 repeat_until loop");

    println!("✓ All loop types present test passed");
    println!("  - for_each loops: {}", foreach_count);
    println!("  - repeat loops: {}", repeat_count);
    println!("  - while loops: {}", while_count);
    println!("  - repeat_until loops: {}", repeat_until_count);
}

#[tokio::test]
async fn test_loop_max_parallel() {
    let workflow_path = "tests/fixtures/loop_types.yaml";
    let workflow = parse_workflow_file(workflow_path)
        .expect("Failed to parse workflow");

    // Test for_each with max_parallel
    let foreach_parallel = workflow.tasks.get("test_foreach_parallel").unwrap();
    if let Some(LoopSpec::ForEach { .. }) = &foreach_parallel.loop_spec {
        let max_par = foreach_parallel.loop_spec.as_ref().unwrap().max_parallel();
        assert_eq!(max_par, Some(2), "for_each should have max_parallel=2");
    }

    // Test repeat with max_parallel
    let repeat_parallel = workflow.tasks.get("test_repeat_parallel").unwrap();
    if let Some(LoopSpec::Repeat { .. }) = &repeat_parallel.loop_spec {
        let max_par = repeat_parallel.loop_spec.as_ref().unwrap().max_parallel();
        assert_eq!(max_par, Some(2), "repeat should have max_parallel=2");
    }

    println!("✓ loop max_parallel test passed");
}

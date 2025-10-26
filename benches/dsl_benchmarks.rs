//! Performance benchmarks for DSL components
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use periplon_sdk::dsl::{
    parse_workflow, validate_workflow, DSLWorkflow, StatePersistence, TaskGraph, TaskSpec,
    TaskStatus, WorkflowState, WorkflowStatus,
};
use std::collections::HashMap;
use std::time::SystemTime;
use tempfile::TempDir;

// ============================================================================
// Workflow Parsing Benchmarks
// ============================================================================

fn benchmark_parse_simple_workflow(c: &mut Criterion) {
    let yaml = r#"
name: "Simple Workflow"
version: "1.0.0"

agents:
  worker:
    description: "Worker agent"
    tools: [Read, Write]
    permissions:
      mode: "default"

tasks:
  task1:
    description: "First task"
    agent: "worker"
"#;

    c.bench_function("parse_simple_workflow", |b| {
        b.iter(|| {
            let result = parse_workflow(black_box(yaml));
            assert!(result.is_ok());
            result
        })
    });
}

fn benchmark_parse_complex_workflow(c: &mut Criterion) {
    // Generate a workflow with many tasks
    let num_tasks_list = vec![10, 50, 100];

    let mut group = c.benchmark_group("parse_complex_workflow");

    for num_tasks in num_tasks_list {
        let yaml = generate_workflow_yaml(num_tasks);

        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            &yaml,
            |b, yaml_str| {
                b.iter(|| {
                    let result = parse_workflow(black_box(yaml_str));
                    assert!(result.is_ok());
                    result
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Workflow Validation Benchmarks
// ============================================================================

fn benchmark_validate_workflow(c: &mut Criterion) {
    let workflow = create_test_workflow(10);

    c.bench_function("validate_workflow_10_tasks", |b| {
        b.iter(|| {
            let result = validate_workflow(black_box(&workflow));
            assert!(result.is_ok());
            result
        })
    });
}

fn benchmark_validate_complex_workflow(c: &mut Criterion) {
    let task_counts = vec![10, 50, 100];

    let mut group = c.benchmark_group("validate_complex_workflow");

    for count in task_counts {
        let workflow = create_test_workflow(count);

        group.bench_with_input(BenchmarkId::from_parameter(count), &workflow, |b, wf| {
            b.iter(|| {
                let result = validate_workflow(black_box(wf));
                assert!(result.is_ok());
                result
            })
        });
    }

    group.finish();
}

// ============================================================================
// Task Graph Benchmarks
// ============================================================================

fn benchmark_task_graph_build(c: &mut Criterion) {
    let task_counts = vec![10, 50, 100];

    let mut group = c.benchmark_group("task_graph_build");

    for count in task_counts {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |b, &num_tasks| {
                b.iter(|| {
                    let mut graph = TaskGraph::new();
                    for i in 0..num_tasks {
                        let task_id = format!("task{}", i);
                        let deps = if i > 0 {
                            vec![format!("task{}", i - 1)]
                        } else {
                            vec![]
                        };
                        let spec = TaskSpec {
                            description: format!("Task {}", i),
                            agent: Some("worker".to_string()),
                            depends_on: deps,
                            ..Default::default()
                        };
                        graph.add_task(task_id, spec);
                    }
                    graph
                })
            },
        );
    }

    group.finish();
}

fn benchmark_task_graph_topological_sort(c: &mut Criterion) {
    let task_counts = vec![10, 50, 100];

    let mut group = c.benchmark_group("task_graph_topological_sort");

    for count in task_counts {
        let mut graph = TaskGraph::new();
        for i in 0..count {
            let task_id = format!("task{}", i);
            let deps = if i > 0 {
                vec![format!("task{}", i - 1)]
            } else {
                vec![]
            };
            let spec = TaskSpec {
                description: format!("Task {}", i),
                agent: Some("worker".to_string()),
                depends_on: deps,
                ..Default::default()
            };
            graph.add_task(task_id, spec);
        }

        group.bench_with_input(BenchmarkId::from_parameter(count), &graph, |b, g| {
            b.iter(|| {
                let result = g.topological_sort();
                assert!(result.is_ok());
                result
            })
        });
    }

    group.finish();
}

fn benchmark_task_graph_get_ready_tasks(c: &mut Criterion) {
    let mut graph = TaskGraph::new();
    for i in 0..100 {
        let task_id = format!("task{}", i);
        let deps = if i > 0 && i % 10 != 0 {
            vec![format!("task{}", i - 1)]
        } else {
            vec![]
        };
        let spec = TaskSpec {
            description: format!("Task {}", i),
            agent: Some("worker".to_string()),
            depends_on: deps,
            ..Default::default()
        };
        graph.add_task(task_id, spec);
    }

    c.bench_function("task_graph_get_ready_tasks", |b| {
        b.iter(|| black_box(graph.get_ready_tasks()))
    });
}

// ============================================================================
// State Persistence Benchmarks
// ============================================================================

fn benchmark_state_serialization(c: &mut Criterion) {
    let task_counts = vec![10, 50, 100];

    let mut group = c.benchmark_group("state_serialization");

    for count in task_counts {
        let state = create_workflow_state(count);

        group.bench_with_input(BenchmarkId::from_parameter(count), &state, |b, s| {
            b.iter(|| {
                let json = serde_json::to_string(black_box(s));
                assert!(json.is_ok());
                json
            })
        });
    }

    group.finish();
}

fn benchmark_state_deserialization(c: &mut Criterion) {
    let task_counts = vec![10, 50, 100];

    let mut group = c.benchmark_group("state_deserialization");

    for count in task_counts {
        let state = create_workflow_state(count);
        let json = serde_json::to_string(&state).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(count), &json, |b, j| {
            b.iter(|| {
                let result: Result<WorkflowState, _> = serde_json::from_str(black_box(j));
                assert!(result.is_ok());
                result
            })
        });
    }

    group.finish();
}

fn benchmark_state_save_load(c: &mut Criterion) {
    let task_counts = vec![10, 50, 100];

    let mut group = c.benchmark_group("state_save_load");

    for count in task_counts {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |b, &num_tasks| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let persistence =
                        StatePersistence::new(temp_dir.path().to_str().unwrap()).unwrap();
                    let state = create_workflow_state(num_tasks);

                    // Save
                    let save_result = persistence.save_state(&state);
                    assert!(save_result.is_ok());

                    // Load
                    let load_result = persistence.load_state(&state.workflow_name);
                    assert!(load_result.is_ok());

                    temp_dir
                })
            },
        );
    }

    group.finish();
}

fn benchmark_state_update_task_status(c: &mut Criterion) {
    let mut state = create_workflow_state(100);

    c.bench_function("state_update_task_status", |b| {
        let mut counter = 0;
        b.iter(|| {
            let task_id = format!("task{}", counter % 100);
            state.update_task_status(&task_id, black_box(TaskStatus::Completed));
            counter += 1;
        })
    });
}

fn benchmark_state_get_progress(c: &mut Criterion) {
    let mut state = create_workflow_state(100);

    // Mark half as completed
    for i in 0..50 {
        state.update_task_status(&format!("task{}", i), TaskStatus::Completed);
    }

    c.bench_function("state_get_progress", |b| {
        b.iter(|| black_box(state.get_progress()))
    });
}

// ============================================================================
// Helper Functions
// ============================================================================

fn generate_workflow_yaml(num_tasks: usize) -> String {
    let mut yaml = String::from(
        r#"name: "Benchmark Workflow"
version: "1.0.0"

agents:
  worker:
    description: "Worker agent"
    tools: [Read, Write]
    permissions:
      mode: "default"

tasks:
"#,
    );

    for i in 0..num_tasks {
        yaml.push_str(&format!(
            r#"  task{}:
    description: "Task {}"
    agent: "worker"
"#,
            i, i
        ));

        if i > 0 && i % 10 != 0 {
            yaml.push_str(&format!("    depends_on: [\"task{}\"]\n", i - 1));
        }
    }

    yaml
}

fn create_test_workflow(num_tasks: usize) -> DSLWorkflow {
    let yaml = generate_workflow_yaml(num_tasks);
    parse_workflow(&yaml).unwrap()
}

fn create_workflow_state(num_tasks: usize) -> WorkflowState {
    let mut task_statuses = HashMap::new();
    let mut task_start_times = HashMap::new();
    let mut task_end_times = HashMap::new();
    let mut task_attempts = HashMap::new();

    let now = SystemTime::now();

    for i in 0..num_tasks {
        let task_id = format!("task{}", i);

        if i < num_tasks / 2 {
            task_statuses.insert(task_id.clone(), TaskStatus::Completed);
            task_start_times.insert(task_id.clone(), now);
            task_end_times.insert(task_id.clone(), now);
            task_attempts.insert(task_id.clone(), 1);
        } else {
            task_statuses.insert(task_id.clone(), TaskStatus::Pending);
        }
    }

    WorkflowState {
        workflow_name: "Benchmark Workflow".to_string(),
        workflow_version: "1.0.0".to_string(),
        task_statuses,
        task_start_times,
        task_end_times,
        task_attempts,
        task_errors: HashMap::new(),
        task_results: HashMap::new(),
        task_outputs: HashMap::new(),
        status: WorkflowStatus::Running,
        started_at: now,
        ended_at: None,
        checkpoint_at: now,
        metadata: HashMap::new(),
        loop_states: HashMap::new(),
        loop_results: HashMap::new(),
    }
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    parsing_benches,
    benchmark_parse_simple_workflow,
    benchmark_parse_complex_workflow,
);

criterion_group!(
    validation_benches,
    benchmark_validate_workflow,
    benchmark_validate_complex_workflow,
);

criterion_group!(
    task_graph_benches,
    benchmark_task_graph_build,
    benchmark_task_graph_topological_sort,
    benchmark_task_graph_get_ready_tasks,
);

criterion_group!(
    state_benches,
    benchmark_state_serialization,
    benchmark_state_deserialization,
    benchmark_state_save_load,
    benchmark_state_update_task_status,
    benchmark_state_get_progress,
);

criterion_main!(
    parsing_benches,
    validation_benches,
    task_graph_benches,
    state_benches,
);

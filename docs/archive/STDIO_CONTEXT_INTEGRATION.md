# Stdio and Context Management - Integration Guide

This document provides practical guidance for integrating the stdio and context management system into the DSL executor.

## Integration Points

The system integrates at three main points in the executor:

1. **Output Capture** - When tasks produce stdout/stderr
2. **Context Injection** - Before executing agent-based tasks
3. **State Management** - Periodic cleanup and metrics

## 1. Output Capture Integration

### Location: `src/dsl/executor.rs`

Find where script and command outputs are captured and add truncation:

```rust
// In execute_script_task() or similar
async fn execute_script_task(
    &mut self,
    task_id: &str,
    script_spec: &ScriptSpec,
) -> Result<String> {
    // ... existing execution code ...

    // NEW: Get limits configuration
    let limits = self.get_task_limits(task_id);

    // Capture stdout/stderr
    let output = process.wait_with_output().await?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // NEW: Apply truncation
    use crate::dsl::truncation::create_task_output;
    use crate::dsl::state::OutputType;

    let stdout_output = create_task_output(
        format!("{}_stdout", task_id),
        OutputType::Stdout,
        stdout.clone(),
        limits.max_stdout_bytes,
        &limits.truncation_strategy,
    );

    let stderr_output = create_task_output(
        format!("{}_stderr", task_id),
        OutputType::Stderr,
        stderr.clone(),
        limits.max_stderr_bytes,
        &limits.truncation_strategy,
    );

    // NEW: Store outputs in state
    if let Some(state) = &mut self.state {
        state.store_task_output(stdout_output);
        state.store_task_output(stderr_output);
    }

    // Return truncated stdout for compatibility
    Ok(stdout)
}
```

### Helper Method

Add a helper to get effective limits for a task:

```rust
impl DSLExecutor {
    /// Get effective limits for a task (task-level overrides workflow-level)
    fn get_task_limits(&self, task_id: &str) -> LimitsConfig {
        // Try task-level limits first
        if let Some(task_spec) = self.workflow.tasks.get(task_id) {
            if let Some(task_limits) = &task_spec.limits {
                return task_limits.clone();
            }
        }

        // Fall back to workflow-level limits
        if let Some(workflow_limits) = &self.workflow.limits {
            return workflow_limits.clone();
        }

        // Use defaults
        LimitsConfig::default()
    }
}
```

## 2. Context Injection Integration

### Location: `src/dsl/executor.rs`

Before executing agent-based tasks, build and inject context:

```rust
// In execute_agent_task() or similar
async fn execute_agent_task(
    &mut self,
    task_id: &str,
    task_spec: &TaskSpec,
) -> Result<String> {
    // NEW: Build smart context if inject_context is enabled
    let context = if task_spec.inject_context {
        use crate::dsl::context_injection::build_smart_context;

        if let Some(state) = &self.state {
            build_smart_context(
                task_id,
                &self.workflow,
                &self.task_graph,
                state,
                task_spec.context.as_ref(),
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Build full prompt with context
    let full_description = if !context.is_empty() {
        format!("{}\n\n{}", context, &task_spec.description)
    } else {
        task_spec.description.clone()
    };

    // Execute agent with context-enhanced description
    let agent = self.agents.get(task_spec.agent.as_ref().unwrap())
        .ok_or_else(|| Error::InvalidInput("Agent not found".into()))?;

    let mut query_stream = agent.query(&full_description).await?;

    // ... rest of existing execution code ...

    Ok(result)
}
```

## 3. Periodic Cleanup Integration

### Location: `src/dsl/executor.rs`

Add cleanup after every N tasks:

```rust
impl DSLExecutor {
    /// Execute a single task and perform cleanup if needed
    async fn execute_task_with_cleanup(
        &mut self,
        task_id: &str,
    ) -> Result<()> {
        // Execute the task
        self.execute_task(task_id).await?;

        // Get cleanup configuration
        let cleanup_interval = 10; // Every 10 tasks
        let completed_count = self.state
            .as_ref()
            .map(|s| s.get_completed_task_count())
            .unwrap_or(0);

        // Perform periodic cleanup
        if completed_count % cleanup_interval == 0 {
            if let Some(state) = &mut self.state {
                // Get cleanup strategy
                let strategy = self.workflow.limits
                    .as_ref()
                    .map(|l| &l.cleanup_strategy)
                    .unwrap_or(&CleanupStrategy::MostRecent { keep_count: 20 });

                // Prune old outputs
                state.prune_outputs(strategy);

                // Log metrics (optional)
                state.log_metrics();
            }
        }

        Ok(())
    }
}
```

## 4. Relevance Score Updates

After task completion, update relevance scores for dependent tasks:

```rust
async fn post_task_completion(
    &mut self,
    task_id: &str,
) -> Result<()> {
    use crate::dsl::context_injection::calculate_relevance;

    // Get all tasks that depend on this one
    let dependent_tasks = self.task_graph.get_dependent_tasks(task_id);

    // Update relevance scores for outputs
    if let Some(state) = &mut self.state {
        for dependent_id in dependent_tasks {
            if let Some(output) = state.get_task_output_mut(task_id) {
                let relevance = calculate_relevance(
                    &dependent_id,
                    task_id,
                    &self.task_graph,
                    &self.workflow,
                );

                output.set_relevance(relevance);
                output.add_dependent(dependent_id.clone());
            }
        }
    }

    Ok(())
}
```

## 5. External Storage Integration (Future)

When implementing external storage:

```rust
async fn store_large_output(
    &mut self,
    task_id: &str,
    content: &str,
) -> Result<Option<PathBuf>> {
    let limits = self.get_task_limits(task_id);

    // Check if external storage threshold is exceeded
    if let Some(threshold) = limits.external_storage_threshold {
        if content.len() > threshold {
            // Store externally
            let storage_dir = PathBuf::from(&limits.external_storage_dir);
            fs::create_dir_all(&storage_dir)?;

            let file_path = storage_dir.join(format!("{}.log", task_id));

            if limits.compress_external {
                // Store compressed
                use flate2::write::GzEncoder;
                use flate2::Compression;

                let file = File::create(&file_path)?;
                let mut encoder = GzEncoder::new(file, Compression::default());
                encoder.write_all(content.as_bytes())?;
                encoder.finish()?;
            } else {
                // Store uncompressed
                fs::write(&file_path, content)?;
            }

            return Ok(Some(file_path));
        }
    }

    Ok(None)
}
```

## Complete Example: Modified execute_task()

Here's how a complete task execution might look with all integrations:

```rust
async fn execute_task(&mut self, task_id: &str) -> Result<()> {
    let task_spec = self.workflow.tasks.get(task_id)
        .ok_or_else(|| Error::InvalidInput(format!("Task not found: {}", task_id)))?
        .clone();

    // 1. Build context (if needed)
    let context = if task_spec.inject_context && task_spec.agent.is_some() {
        use crate::dsl::context_injection::build_smart_context;

        self.state.as_ref()
            .map(|state| build_smart_context(
                task_id,
                &self.workflow,
                &self.task_graph,
                state,
                task_spec.context.as_ref(),
            ))
            .unwrap_or_default()
    } else {
        String::new()
    };

    // 2. Execute task (script, command, or agent)
    let result = if let Some(script_spec) = &task_spec.script {
        self.execute_script_with_limits(task_id, script_spec).await?
    } else if let Some(agent_id) = &task_spec.agent {
        self.execute_agent_with_context(task_id, agent_id, &task_spec, &context).await?
    } else {
        // Other execution types...
        String::new()
    };

    // 3. Store result with truncation
    if let Some(state) = &mut self.state {
        use crate::dsl::truncation::create_task_output;
        use crate::dsl::state::OutputType;

        let limits = self.get_task_limits(task_id);

        let output = create_task_output(
            task_id.to_string(),
            OutputType::Combined,
            result.clone(),
            limits.max_combined_bytes,
            &limits.truncation_strategy,
        );

        state.store_task_output(output);
    }

    // 4. Update relevance scores
    self.update_relevance_scores(task_id).await?;

    // 5. Periodic cleanup
    self.maybe_cleanup().await?;

    Ok(())
}
```

## Testing Integration

Create integration tests to verify the system works end-to-end:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_output_truncation_integration() {
        let workflow = create_test_workflow_with_limits();
        let mut executor = DSLExecutor::new(workflow).unwrap();
        executor.enable_state_persistence(None).unwrap();

        // Execute task that produces large output
        executor.execute_task("large_output_task").await.unwrap();

        // Verify output was truncated
        let state = executor.state.as_ref().unwrap();
        let output = state.get_task_output("large_output_task").unwrap();

        assert!(output.truncated);
        assert!(output.content.len() <= 1_048_576); // 1MB limit
    }

    #[tokio::test]
    async fn test_context_injection_integration() {
        let workflow = create_test_workflow_with_dependencies();
        let mut executor = DSLExecutor::new(workflow).unwrap();
        executor.enable_state_persistence(None).unwrap();

        // Execute tasks in order
        executor.execute_task("task1").await.unwrap();
        executor.execute_task("task2").await.unwrap(); // Depends on task1

        // Verify task2 received context from task1
        // (Would need to capture agent prompts to verify)
    }

    #[tokio::test]
    async fn test_cleanup_integration() {
        let workflow = create_test_workflow_with_many_tasks();
        let mut executor = DSLExecutor::new(workflow).unwrap();
        executor.enable_state_persistence(None).unwrap();

        // Execute 20 tasks
        for i in 0..20 {
            executor.execute_task(&format!("task{}", i)).await.unwrap();
        }

        // Verify cleanup happened
        let state = executor.state.as_ref().unwrap();
        let metrics = state.get_context_metrics();

        // Should have pruned to keep_count
        assert!(metrics.task_count <= 20);
    }
}
```

## Gradual Rollout Strategy

1. **Phase 1**: Add output capture and truncation (no behavior changes without config)
2. **Phase 2**: Add context injection (disabled by default)
3. **Phase 3**: Enable cleanup (conservative defaults)
4. **Phase 4**: Add external storage
5. **Phase 5**: Add AI summarization

Each phase can be deployed independently.

## Monitoring and Debugging

Add logging at key points:

```rust
// When truncating
if truncated {
    log::debug!(
        "Truncated output for task {}: {} -> {} bytes (strategy: {:?})",
        task_id,
        original_size,
        truncated_size,
        strategy
    );
}

// When building context
log::debug!(
    "Built context for task {}: {} bytes from {} tasks",
    task_id,
    context.len(),
    included_count
);

// When pruning
log::info!(
    "Pruned outputs: {} -> {} tasks (strategy: {:?})",
    before_count,
    after_count,
    strategy
);
```

## Performance Optimization

For high-throughput workflows:

1. **Lazy Context Building**: Only build context when actually needed
2. **Cache Relevance Scores**: Don't recalculate on every access
3. **Batch Cleanup**: Don't prune after every task
4. **Async Storage**: Write external storage in background

## Conclusion

The integration is straightforward with well-defined touchpoints. The system is designed to be non-invasive and can be adopted incrementally.

Key integration points:
1. Output capture → Truncation
2. Agent execution → Context injection
3. Task completion → Cleanup
4. State persistence → Save/restore

Start with output capture and truncation, then add context injection and cleanup as needed.

# Stdio and Context Management - Design Document

## Problem Statement

### Current Issues

1. **Unbounded stdout/stderr**: Script tasks capture ALL output with no limits
2. **Context accumulation**: WorkflowState accumulates results from ALL tasks
3. **Indiscriminate injection**: All completed tasks injected into context
4. **Memory growth**: Long-running workflows can exhaust memory
5. **No cleanup**: No mechanism to prune old/irrelevant context

### Example Scenario

```yaml
tasks:
  generate_100_reports:
    loop:
      type: repeat
      count: 100
    script:
      content: "generate_huge_report.sh"  # Outputs 10MB per iteration
```

**Problem**: 100 iterations × 10MB = 1GB of output stored in memory!

## Solution Overview

### Three-Tier Strategy

1. **Tier 1: Capture Limits** - Prevent unbounded capture at source
2. **Tier 2: Storage Management** - Smart truncation and summarization
3. **Tier 3: Injection Control** - Selective context for each task

## Detailed Design

### 1. Capture Limits (Tier 1)

#### Configuration Schema

Add to DSL workflow and task levels:

```yaml
# Workflow-level defaults
name: "My Workflow"
version: "1.0.0"

limits:
  # Maximum stdout bytes per task (default: 1MB)
  max_stdout_bytes: 1048576

  # Maximum stderr bytes per task (default: 256KB)
  max_stderr_bytes: 262144

  # Maximum combined stdout+stderr (default: 1.5MB)
  max_combined_bytes: 1572864

  # Truncation strategy: head, tail, both, summary
  truncation_strategy: tail

  # Maximum context size for injection (default: 100KB)
  max_context_bytes: 102400

  # Maximum tasks in context (default: 10)
  max_context_tasks: 10

tasks:
  verbose_task:
    description: "Task with custom limits"
    agent: "worker"

    # Task-level limits override workflow defaults
    limits:
      max_stdout_bytes: 2097152  # 2MB for this specific task
      truncation_strategy: summary  # Use AI summarization
```

#### Truncation Strategies

**1. Head (First N bytes)**
```
--- Output (showing first 1000 bytes of 50000) ---
[first 1000 bytes]
--- [49000 bytes truncated] ---
```

**2. Tail (Last N bytes)**
```
--- [49000 bytes truncated] ---
[last 1000 bytes]
--- Output (showing last 1000 bytes of 50000) ---
```

**3. Both (First N/2 + Last N/2)**
```
--- Output (showing first/last 500 bytes of 50000) ---
[first 500 bytes]
--- [49000 bytes truncated] ---
[last 500 bytes]
```

**4. Summary (AI-generated summary)**
```
--- Output Summary (50000 bytes) ---
The script processed 1000 files successfully.
3 files had warnings: file1.txt, file2.txt, file3.txt
All tests passed. Total runtime: 45 seconds.
--- [Original output stored in: .workflow_state/task_outputs/task_123.log] ---
```

### 2. Storage Management (Tier 2)

#### Enhanced WorkflowState Schema

```rust
pub struct WorkflowState {
    // ... existing fields ...

    /// Task outputs with metadata
    pub task_outputs: HashMap<String, TaskOutput>,

    /// Context pruning settings
    pub context_config: ContextConfig,
}

pub struct TaskOutput {
    /// Task ID
    pub task_id: String,

    /// Output type
    pub output_type: OutputType,

    /// Actual content (truncated or summarized)
    pub content: String,

    /// Original size in bytes
    pub original_size: usize,

    /// Was truncated/summarized?
    pub truncated: bool,

    /// Truncation strategy used
    pub strategy: TruncationStrategy,

    /// File path if stored externally
    pub file_path: Option<PathBuf>,

    /// Relevance score (0.0-1.0) for context injection
    pub relevance_score: f64,

    /// Last accessed timestamp (for LRU cleanup)
    pub last_accessed: SystemTime,

    /// Dependencies (tasks that depend on this output)
    pub depended_by: Vec<String>,
}

pub enum OutputType {
    Stdout,
    Stderr,
    Combined,
    File,
    Summary,
}

pub struct ContextConfig {
    /// Maximum bytes of context to inject
    pub max_bytes: usize,

    /// Maximum number of tasks in context
    pub max_tasks: usize,

    /// Relevance threshold (0.0-1.0)
    pub relevance_threshold: f64,

    /// Time window (only include tasks from last N seconds)
    pub time_window_secs: Option<u64>,

    /// Cleanup strategy
    pub cleanup_strategy: CleanupStrategy,
}

pub enum CleanupStrategy {
    /// Keep most recent N tasks
    MostRecent(usize),

    /// Keep highest relevance scores
    HighestRelevance(usize),

    /// LRU (Least Recently Used)
    LRU(usize),

    /// Keep only direct dependencies
    DirectDependencies,

    /// Custom retention policy
    Custom(Box<dyn Fn(&TaskOutput) -> bool>),
}
```

#### Truncation Implementation

```rust
/// Truncate output based on strategy
fn truncate_output(
    content: &str,
    max_bytes: usize,
    strategy: TruncationStrategy,
) -> (String, bool) {
    if content.len() <= max_bytes {
        return (content.to_string(), false);
    }

    let truncated = match strategy {
        TruncationStrategy::Head => {
            format!(
                "--- Output (showing first {} bytes of {}) ---\n{}\n--- [{} bytes truncated] ---",
                max_bytes,
                content.len(),
                &content[..max_bytes],
                content.len() - max_bytes
            )
        }

        TruncationStrategy::Tail => {
            let start = content.len() - max_bytes;
            format!(
                "--- [{} bytes truncated] ---\n{}\n--- Output (showing last {} bytes of {}) ---",
                start,
                &content[start..],
                max_bytes,
                content.len()
            )
        }

        TruncationStrategy::Both => {
            let half = max_bytes / 2;
            let end_start = content.len() - half;
            format!(
                "--- Output (showing first/last {} bytes of {}) ---\n{}\n--- [{} bytes truncated] ---\n{}",
                half,
                content.len(),
                &content[..half],
                content.len() - max_bytes,
                &content[end_start..]
            )
        }

        TruncationStrategy::Summary => {
            // Use AI to generate summary (requires agent call)
            generate_ai_summary(content, max_bytes)
        }
    };

    (truncated, true)
}
```

#### External Storage for Large Outputs

```yaml
# Store large outputs externally
limits:
  external_storage_threshold: 1048576  # 1MB
  external_storage_dir: ".workflow_state/task_outputs"
  compress_external: true  # gzip compression
```

```rust
/// Store large output externally
fn store_output_externally(
    task_id: &str,
    content: &str,
    config: &ExternalStorageConfig,
) -> Result<PathBuf> {
    let file_path = config.dir.join(format!("{}.log", task_id));

    if config.compress {
        // Store as gzipped file
        let file = File::create(&file_path)?;
        let encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(content.as_bytes())?;
    } else {
        fs::write(&file_path, content)?;
    }

    Ok(file_path)
}
```

### 3. Selective Context Injection (Tier 3)

#### Dependency-Based Relevance

Only inject context from tasks that the current task depends on (directly or transitively).

```rust
/// Calculate relevance score for a task output
fn calculate_relevance(
    current_task_id: &str,
    output_task_id: &str,
    workflow: &DSLWorkflow,
    state: &WorkflowState,
) -> f64 {
    // 1. Direct dependency = 1.0
    if is_direct_dependency(current_task_id, output_task_id, workflow) {
        return 1.0;
    }

    // 2. Transitive dependency = 0.8 / depth
    if let Some(depth) = get_dependency_depth(current_task_id, output_task_id, workflow) {
        return 0.8 / depth as f64;
    }

    // 3. Same agent = 0.5
    if uses_same_agent(current_task_id, output_task_id, workflow) {
        return 0.5;
    }

    // 4. Recent task = 0.3
    if is_recent(output_task_id, state, Duration::from_secs(300)) {
        return 0.3;
    }

    // 5. No relevance
    0.0
}
```

#### Smart Context Builder

```rust
/// Build context with smart selection
fn build_smart_context(
    current_task_id: &str,
    workflow: &DSLWorkflow,
    state: &WorkflowState,
    config: &ContextConfig,
) -> String {
    let mut context = String::new();
    let mut total_bytes = 0;
    let mut included_tasks = 0;

    // Get all task outputs sorted by relevance
    let mut outputs: Vec<(&String, &TaskOutput)> = state
        .task_outputs
        .iter()
        .filter(|(task_id, output)| {
            let relevance = calculate_relevance(
                current_task_id,
                task_id,
                workflow,
                state,
            );
            relevance >= config.relevance_threshold
        })
        .collect();

    // Sort by relevance (descending)
    outputs.sort_by(|(_, a), (_, b)| {
        b.relevance_score.partial_cmp(&a.relevance_score).unwrap()
    });

    context.push_str("=== RELEVANT CONTEXT ===\n\n");

    for (task_id, output) in outputs {
        // Check limits
        if included_tasks >= config.max_tasks {
            break;
        }

        let output_bytes = output.content.len();
        if total_bytes + output_bytes > config.max_bytes {
            // Try to include truncated version
            let remaining = config.max_bytes - total_bytes;
            if remaining > 100 {
                let truncated = truncate_to_size(&output.content, remaining);
                context.push_str(&format!(
                    "Task: {} (relevance: {:.2})\n{}\n\n",
                    task_id,
                    output.relevance_score,
                    truncated
                ));
                break;
            } else {
                break;
            }
        }

        // Include full output
        context.push_str(&format!(
            "Task: {} (relevance: {:.2})\n",
            task_id,
            output.relevance_score
        ));

        if output.truncated {
            context.push_str("[Output was truncated]\n");
        }

        context.push_str(&format!("{}\n\n", output.content));

        total_bytes += output_bytes;
        included_tasks += 1;
    }

    context.push_str(&format!(
        "=== END CONTEXT ({} tasks, {} bytes) ===\n",
        included_tasks,
        total_bytes
    ));

    context
}
```

#### Task-Level Context Control

```yaml
tasks:
  task1:
    description: "Fetch data"
    agent: "fetcher"
    output: "data.json"

  task2:
    description: "Process data"
    agent: "processor"
    depends_on: [task1]

    # Fine-grained context control
    context:
      # Include only these specific tasks
      include_tasks: [task1]

      # Or exclude specific tasks
      exclude_tasks: [noisy_task]

      # Or use automatic (dependency-based)
      mode: automatic  # automatic, manual, none

      # Relevance threshold
      min_relevance: 0.5

      # Maximum context size
      max_bytes: 50000

  task3:
    description: "Generate report (no context needed)"
    agent: "reporter"
    inject_context: false  # Disable entirely
```

### 4. Cleanup and Pruning

#### Automatic Pruning

```rust
impl WorkflowState {
    /// Prune old/irrelevant outputs
    pub fn prune_outputs(&mut self, strategy: &CleanupStrategy) {
        match strategy {
            CleanupStrategy::MostRecent(n) => {
                self.prune_most_recent(*n);
            }

            CleanupStrategy::HighestRelevance(n) => {
                self.prune_by_relevance(*n);
            }

            CleanupStrategy::LRU(n) => {
                self.prune_lru(*n);
            }

            CleanupStrategy::DirectDependencies => {
                self.prune_non_dependencies();
            }

            CleanupStrategy::Custom(predicate) => {
                self.task_outputs.retain(|_, output| predicate(output));
            }
        }
    }

    fn prune_most_recent(&mut self, keep_count: usize) {
        // Sort by end time, keep most recent N
        let mut outputs: Vec<_> = self.task_outputs.iter().collect();
        outputs.sort_by_key(|(_, output)| output.last_accessed);
        outputs.reverse();

        let keep: HashSet<_> = outputs
            .iter()
            .take(keep_count)
            .map(|(id, _)| (*id).clone())
            .collect();

        self.task_outputs.retain(|id, _| keep.contains(id));
    }

    fn prune_by_relevance(&mut self, keep_count: usize) {
        let mut outputs: Vec<_> = self.task_outputs.iter().collect();
        outputs.sort_by(|(_, a), (_, b)| {
            b.relevance_score.partial_cmp(&a.relevance_score).unwrap()
        });

        let keep: HashSet<_> = outputs
            .iter()
            .take(keep_count)
            .map(|(id, _)| (*id).clone())
            .collect();

        self.task_outputs.retain(|id, _| keep.contains(id));
    }
}
```

#### Checkpoint Pruning

```yaml
workflows:
  main:
    hooks:
      # Prune context after every 10 tasks
      post_task:
        - checkpoint_and_prune:
            every: 10
            strategy: highest_relevance
            keep_count: 20
```

### 5. Monitoring and Observability

#### Context Metrics

```rust
pub struct ContextMetrics {
    /// Total bytes stored
    pub total_bytes: usize,

    /// Number of task outputs
    pub task_count: usize,

    /// Number of truncated outputs
    pub truncated_count: usize,

    /// Number of externally stored outputs
    pub external_count: usize,

    /// Average relevance score
    pub avg_relevance: f64,

    /// Last pruning time
    pub last_pruned_at: Option<SystemTime>,
}

impl WorkflowState {
    pub fn get_context_metrics(&self) -> ContextMetrics {
        // Calculate and return metrics
    }

    pub fn log_metrics(&self) {
        let metrics = self.get_context_metrics();
        println!("Context Metrics:");
        println!("  Total bytes: {}", metrics.total_bytes);
        println!("  Task outputs: {}", metrics.task_count);
        println!("  Truncated: {}", metrics.truncated_count);
        println!("  Avg relevance: {:.2}", metrics.avg_relevance);
    }
}
```

## Implementation Plan

### Phase 1: Basic Limits (Week 1)
- [ ] Add `limits` configuration to schema
- [ ] Implement stdout/stderr truncation in executor
- [ ] Add head/tail/both truncation strategies
- [ ] Update WorkflowState to store TaskOutput structs

### Phase 2: Smart Storage (Week 2)
- [ ] Implement external storage for large outputs
- [ ] Add compression support
- [ ] Implement relevance calculation
- [ ] Add context size limits

### Phase 3: Selective Injection (Week 3)
- [ ] Implement dependency-based relevance
- [ ] Build smart context builder
- [ ] Add task-level context controls
- [ ] Update context injection in executor

### Phase 4: Cleanup & Pruning (Week 4)
- [ ] Implement pruning strategies
- [ ] Add automatic pruning triggers
- [ ] Add checkpoint pruning hooks
- [ ] Implement LRU cache

### Phase 5: Observability (Week 5)
- [ ] Add context metrics
- [ ] Add monitoring endpoints
- [ ] Create diagnostic tools
- [ ] Write documentation

## Configuration Examples

### Minimal (Use Defaults)
```yaml
name: "Simple Workflow"
version: "1.0.0"

# No limits specified - use defaults:
# - max_stdout: 1MB
# - max_stderr: 256KB
# - truncation: tail
# - max_context: 100KB
```

### Conservative (Low Memory)
```yaml
name: "Memory-Constrained Workflow"
version: "1.0.0"

limits:
  max_stdout_bytes: 102400  # 100KB
  max_stderr_bytes: 51200   # 50KB
  max_context_bytes: 51200  # 50KB
  max_context_tasks: 5
  truncation_strategy: summary
  cleanup_strategy:
    type: lru
    keep_count: 10
```

### Generous (High Memory)
```yaml
name: "High-Memory Workflow"
version: "1.0.0"

limits:
  max_stdout_bytes: 10485760  # 10MB
  max_stderr_bytes: 2097152   # 2MB
  max_context_bytes: 1048576  # 1MB
  max_context_tasks: 50
  external_storage_threshold: 5242880  # 5MB
  truncation_strategy: both
```

### Dependency-Focused
```yaml
name: "Dependency-Aware Workflow"
version: "1.0.0"

limits:
  max_context_bytes: 204800  # 200KB
  cleanup_strategy:
    type: direct_dependencies  # Only keep dependency outputs

tasks:
  fetch:
    output: "data.json"

  process:
    depends_on: [fetch]
    context:
      mode: automatic  # Only include 'fetch' output
      min_relevance: 0.8
```

## Migration Path

### Backwards Compatibility

All limits are **optional** with sensible defaults:

```rust
impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_stdout_bytes: 1_048_576,    // 1MB
            max_stderr_bytes: 262_144,      // 256KB
            max_combined_bytes: 1_572_864,  // 1.5MB
            max_context_bytes: 102_400,     // 100KB
            max_context_tasks: 10,
            truncation_strategy: TruncationStrategy::Tail,
            cleanup_strategy: CleanupStrategy::MostRecent(20),
        }
    }
}
```

Existing workflows continue to work without changes.

### Gradual Adoption

```yaml
# Step 1: Just add limits (no other changes)
limits:
  max_stdout_bytes: 524288  # 512KB

# Step 2: Add external storage
limits:
  max_stdout_bytes: 524288
  external_storage_threshold: 1048576
  external_storage_dir: ".outputs"

# Step 3: Enable smart context
limits:
  max_stdout_bytes: 524288
  max_context_bytes: 102400
  cleanup_strategy:
    type: highest_relevance
    keep_count: 15

# Step 4: Fine-tune per task
tasks:
  verbose_task:
    limits:
      max_stdout_bytes: 2097152  # Override for this task
```

## Benefits

1. **Bounded Memory**: Workflows can't exhaust memory
2. **Faster Execution**: Less data to serialize/deserialize
3. **Better Context**: Only relevant information injected
4. **Scalability**: Can handle long-running workflows
5. **Flexibility**: Configure per workflow or per task
6. **Observability**: Track context usage
7. **Backwards Compatible**: Existing workflows work unchanged

## Trade-offs

### Pros
✅ Prevents memory exhaustion
✅ Improves agent focus (less noise)
✅ Enables long-running workflows
✅ Configurable per use case

### Cons
⚠️ May lose some output (by design)
⚠️ Adds complexity to configuration
⚠️ External storage requires disk space
⚠️ AI summarization requires extra agent calls

## Conclusion

This design provides a comprehensive solution for managing stdio and context growth while maintaining backwards compatibility and allowing flexible configuration based on workflow needs.

The three-tier approach (Capture → Storage → Injection) ensures bounded memory usage while preserving the most relevant information for downstream tasks.

# Stdio & Context Management - Quick Start Guide

## Problem: Unbounded Memory Growth

**Before**: Workflows could exhaust memory with large outputs

```yaml
# ❌ PROBLEM: This will consume 1GB+ memory!
tasks:
  process_logs:
    loop:
      type: repeat
      count: 1000
    script:
      content: |
        # Each iteration outputs 10MB
        generate_huge_report.sh
```

**After**: Configurable limits prevent memory issues

```yaml
# ✅ SOLUTION: Limits protect memory
name: "Safe Workflow"
version: "1.0.0"

limits:
  max_stdout_bytes: 524288  # 512KB max per task
  truncation_strategy: tail

tasks:
  process_logs:
    # same as before, but output is now bounded
```

## Quick Examples

### 1. Basic Limits (Recommended Start)

Add workflow-level limits:

```yaml
name: "My Workflow"
version: "1.0.0"

# Set sensible defaults for all tasks
limits:
  max_stdout_bytes: 1048576    # 1MB stdout
  max_stderr_bytes: 262144     # 256KB stderr
  max_context_bytes: 102400    # 100KB context
  truncation_strategy: tail    # Keep last N bytes

tasks:
  # ... your tasks ...
```

### 2. Task-Specific Limits

Override limits for specific tasks:

```yaml
tasks:
  verbose_task:
    description: "Task that produces lots of output"
    script:
      content: "verbose_script.sh"

    # Task-level override
    limits:
      max_stdout_bytes: 2097152  # 2MB for this task only
      truncation_strategy: both   # Keep first + last
```

### 3. Dependency-Based Context

Only inject relevant context:

```yaml
tasks:
  fetch_data:
    description: "Fetch data from API"
    agent: "fetcher"
    output: "data.json"

  process_data:
    description: "Process fetched data"
    agent: "processor"
    depends_on: [fetch_data]

    # Automatic: only includes 'fetch_data' output
    context:
      mode: automatic
      min_relevance: 0.8

  generate_report:
    description: "Generate report (doesn't need previous context)"
    agent: "reporter"
    inject_context: false  # No context at all
```

### 4. External Storage for Large Outputs

Store big outputs on disk:

```yaml
limits:
  # Store outputs >5MB externally
  external_storage_threshold: 5242880
  external_storage_dir: ".workflow_state/outputs"
  compress_external: true  # gzip compression

tasks:
  download_logs:
    script:
      content: "download_huge_logs.sh"  # May be 50MB
    # Automatically stored externally if >5MB
```

### 5. Memory-Constrained Workflows

Optimize for low memory:

```yaml
name: "Low-Memory Workflow"
version: "1.0.0"

limits:
  # Conservative limits
  max_stdout_bytes: 102400      # 100KB
  max_stderr_bytes: 51200       # 50KB
  max_context_bytes: 51200      # 50KB
  max_context_tasks: 5          # Only 5 tasks in context

  # Aggressive cleanup
  cleanup_strategy:
    type: lru                   # Least Recently Used
    keep_count: 10              # Keep only 10 outputs

tasks:
  # ... tasks run with minimal memory footprint ...
```

### 6. Truncation Strategies

Choose how output is truncated:

```yaml
tasks:
  # Keep the beginning (good for error messages at start)
  task1:
    limits:
      truncation_strategy: head

  # Keep the end (good for final results)
  task2:
    limits:
      truncation_strategy: tail

  # Keep both ends (good for overview + results)
  task3:
    limits:
      truncation_strategy: both

  # AI summary (requires extra processing)
  task4:
    limits:
      truncation_strategy: summary

  # Line-based truncation (preserve complete lines)
  task5:
    limits:
      truncation_strategy: tail_lines
```

## Common Patterns

### Pattern 1: Data Pipeline

```yaml
name: "ETL Pipeline"
version: "1.0.0"

# Moderate limits
limits:
  max_stdout_bytes: 524288
  max_context_bytes: 204800
  cleanup_strategy:
    type: direct_dependencies  # Only keep what's needed

tasks:
  extract:
    description: "Extract raw data"
    agent: "extractor"
    output: "raw_data.csv"

  transform:
    description: "Transform data"
    agent: "transformer"
    depends_on: [extract]
    # Automatically gets extract output in context
    output: "clean_data.csv"

  load:
    description: "Load to database"
    agent: "loader"
    depends_on: [transform]
    # Gets transform output, extract output pruned
```

### Pattern 2: Parallel Processing

```yaml
name: "Parallel Analysis"
version: "1.0.0"

limits:
  # Higher limits for parallel tasks
  max_stdout_bytes: 1048576
  max_context_bytes: 512000

  # Keep recent results
  cleanup_strategy:
    type: most_recent
    keep_count: 50

tasks:
  analyze_batch:
    loop:
      type: for_each
      collection:
        source: file
        path: "batches.json"
      iterator: "batch"
      parallel: true
      max_parallel: 10

    script:
      content: |
        analyze_batch.sh ${loop.batch}
    # Each iteration limited to 1MB
    # Recent 50 kept in memory
```

### Pattern 3: Long-Running Workflow

```yaml
name: "Long-Running Process"
version: "1.0.0"

limits:
  # Conservative for long runs
  max_stdout_bytes: 262144      # 256KB
  max_context_bytes: 102400     # 100KB
  external_storage_threshold: 524288  # >512KB goes to disk

  # Aggressive cleanup
  cleanup_strategy:
    type: highest_relevance
    keep_count: 20

  # Only recent context
  time_window_secs: 3600  # Last hour only

tasks:
  # 1000 iterations, but memory stays bounded
  process_items:
    loop:
      type: repeat
      count: 1000
    script:
      content: "process_item.sh"
```

### Pattern 4: Manual Context Control

```yaml
tasks:
  task1:
    output: "data1.json"

  task2:
    output: "data2.json"

  task3:
    output: "data3.json"

  final_task:
    description: "Combine results"
    depends_on: [task1, task2, task3]

    # Manually control what's included
    context:
      mode: manual
      include_tasks: [task1, task3]  # Only task1 and task3
      exclude_tasks: [task2]          # Explicit exclude
      max_bytes: 50000                # 50KB max
```

## Monitoring

### Check Context Size

Enable metrics logging:

```yaml
limits:
  # ... your limits ...

workflows:
  main:
    hooks:
      post_task:
        - log_context_metrics  # Log after each task
```

Output:
```
Context Metrics:
  Total bytes: 45234
  Task outputs: 7
  Truncated: 2
  External: 1
  Avg relevance: 0.75
```

### View Truncated Output

When output is truncated, you'll see:

```
--- [49000 bytes truncated] ---
[last 1000 bytes of output]
--- Output (showing last 1000 bytes of 50000) ---
```

Access full output from external storage:

```bash
cat .workflow_state/outputs/task_name.log
```

## Migration Guide

### Step 1: Add Basic Limits

Start with defaults:

```yaml
name: "My Existing Workflow"
version: "1.0.0"

# Add this - no other changes needed
limits:
  max_stdout_bytes: 1048576

# ... existing tasks work as-is ...
```

### Step 2: Tune Per Task

Identify verbose tasks:

```yaml
tasks:
  normal_task:
    # Uses workflow default (1MB)

  verbose_task:
    # Override for this task
    limits:
      max_stdout_bytes: 5242880  # 5MB
```

### Step 3: Optimize Context

Add context control:

```yaml
limits:
  max_context_bytes: 102400
  cleanup_strategy:
    type: highest_relevance
    keep_count: 15
```

### Step 4: External Storage

For very large outputs:

```yaml
limits:
  external_storage_threshold: 2097152  # 2MB
  external_storage_dir: ".outputs"
```

## Troubleshooting

### Problem: Important Output Truncated

**Solution**: Increase limits for that task

```yaml
tasks:
  important_task:
    limits:
      max_stdout_bytes: 10485760  # 10MB
      truncation_strategy: none    # Future: disable truncation
```

### Problem: Memory Still Growing

**Solution**: More aggressive cleanup

```yaml
limits:
  cleanup_strategy:
    type: most_recent
    keep_count: 5  # Very aggressive

  external_storage_threshold: 524288  # Store more externally
```

### Problem: Context Missing Needed Info

**Solution**: Adjust relevance or manually include

```yaml
tasks:
  my_task:
    context:
      min_relevance: 0.3  # Lower threshold
      include_tasks: [critical_task]  # Force include
```

### Problem: Want Full Output

**Access external storage**:

```bash
# Find the output file
ls .workflow_state/outputs/

# View it
cat .workflow_state/outputs/task_name.log

# Or if compressed
zcat .workflow_state/outputs/task_name.log.gz
```

## Best Practices

### ✅ DO

- Set workflow-level defaults
- Override per task as needed
- Use `tail` or `both` for truncation
- Enable external storage for large outputs
- Use dependency-based context
- Monitor metrics in long-running workflows

### ❌ DON'T

- Set limits too low (breaks functionality)
- Keep all context (defeats the purpose)
- Ignore truncation warnings
- Store everything in memory
- Use `head` truncation for results

## Configuration Presets

### Preset: Default (Balanced)

```yaml
limits:
  max_stdout_bytes: 1048576      # 1MB
  max_stderr_bytes: 262144       # 256KB
  max_context_bytes: 102400      # 100KB
  truncation_strategy: tail
  cleanup_strategy:
    type: most_recent
    keep_count: 20
```

### Preset: Conservative (Low Memory)

```yaml
limits:
  max_stdout_bytes: 102400       # 100KB
  max_stderr_bytes: 51200        # 50KB
  max_context_bytes: 51200       # 50KB
  truncation_strategy: summary
  external_storage_threshold: 102400
  cleanup_strategy:
    type: lru
    keep_count: 10
```

### Preset: Generous (High Memory)

```yaml
limits:
  max_stdout_bytes: 10485760     # 10MB
  max_stderr_bytes: 2097152      # 2MB
  max_context_bytes: 1048576     # 1MB
  truncation_strategy: both
  external_storage_threshold: 20971520  # 20MB
  cleanup_strategy:
    type: highest_relevance
    keep_count: 50
```

### Preset: CI/CD

```yaml
limits:
  max_stdout_bytes: 524288       # 512KB (logs)
  max_stderr_bytes: 262144       # 256KB (errors)
  max_context_bytes: 204800      # 200KB
  truncation_strategy: tail_lines  # Preserve line structure
  cleanup_strategy:
    type: direct_dependencies  # Only what's needed
```

## Summary

**Three simple steps to manage stdio and context:**

1. **Add limits** to your workflow
2. **Override** for verbose tasks
3. **Monitor** and tune

**Default limits work for most cases - just add:**

```yaml
limits: {}  # Uses sensible defaults
```

**For fine control:**

```yaml
limits:
  max_stdout_bytes: 1048576
  truncation_strategy: tail
  cleanup_strategy:
    type: most_recent
    keep_count: 20
```

Done! Your workflows now have bounded memory usage. 🎉

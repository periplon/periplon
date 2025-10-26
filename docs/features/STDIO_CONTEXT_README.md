# Stdio and Context Management

**Prevent memory exhaustion and improve context quality in DSL workflows**

## Overview

The stdio and context management system provides bounded memory management for DSL workflows through:

- 📏 **Output Limits** - Cap stdout/stderr to prevent unbounded growth
- ✂️ **Smart Truncation** - Multiple strategies preserving important information
- 🎯 **Intelligent Context** - Inject only relevant outputs into downstream tasks
- 🧹 **Automatic Cleanup** - Prune old outputs to maintain bounded memory
- 📊 **Observability** - Track metrics and monitor context usage

## Quick Start

### 1. Add Basic Limits

```yaml
name: "My Workflow"
version: "1.0.0"

limits:
  max_stdout_bytes: 1048576  # 1MB
  truncation_strategy: tail

tasks:
  my_task:
    agent: "worker"
    # ... task definition ...
```

### 2. Enable Smart Context

```yaml
tasks:
  analyze:
    agent: "analyzer"
    depends_on: [fetch_data]
    inject_context: true  # Include fetch_data output
    context:
      mode: automatic     # Dependency-based
      min_relevance: 0.7
```

### 3. Add Cleanup

```yaml
limits:
  cleanup_strategy:
    type: highest_relevance
    keep_count: 15
```

Done! Your workflow now has bounded memory.

## Features

### Output Truncation

Prevent unbounded stdout/stderr from consuming memory:

```yaml
limits:
  max_stdout_bytes: 1048576   # 1MB per task
  max_stderr_bytes: 262144    # 256KB per task
  truncation_strategy: tail    # head|tail|both|summary
```

**Truncation Strategies**:
- `tail` - Keep last N bytes (most recent output)
- `head` - Keep first N bytes (startup messages)
- `both` - Keep first N/2 and last N/2 bytes
- `summary` - AI-generated summary (coming soon)

**Output**:
```
--- [524288 bytes truncated] ---
[last 1048576 bytes of output]
--- Output (showing last 1048576 bytes of 5242880) ---
```

### Context Injection

Inject relevant task outputs into downstream tasks:

```yaml
tasks:
  task2:
    depends_on: [task1]
    inject_context: true
    context:
      mode: automatic      # automatic|manual|none
      min_relevance: 0.5   # 0.0-1.0
      max_bytes: 100000    # 100KB
      max_tasks: 5
```

**Context Modes**:

**Automatic** (dependency-based):
- Direct dependency: relevance = 1.0
- Transitive dependency: relevance = 0.8 / depth
- Same agent: relevance = 0.5
- Filtered by `min_relevance` threshold

**Manual** (explicit control):
```yaml
context:
  mode: manual
  include_tasks: [task1, task3]
  exclude_tasks: [noisy_task]
```

**None** (disable):
```yaml
context:
  mode: none
```

### Cleanup Strategies

Automatically prune old outputs to maintain bounded memory:

```yaml
limits:
  cleanup_strategy:
    type: highest_relevance  # most_recent|highest_relevance|lru|direct_dependencies
    keep_count: 15
```

**Strategies**:
- `most_recent` - Keep N most recent tasks
- `highest_relevance` - Keep N highest relevance scores
- `lru` - Least Recently Used
- `direct_dependencies` - Keep only tasks with dependents

### Task-Level Overrides

Override workflow defaults for specific tasks:

```yaml
tasks:
  verbose_task:
    limits:
      max_stdout_bytes: 10485760  # 10MB override
      truncation_strategy: both
    context:
      mode: manual
      include_tasks: [critical_task]
```

## Configuration Reference

### Workflow-Level Limits

```yaml
limits:
  # Output capture
  max_stdout_bytes: 1048576        # 1MB (default)
  max_stderr_bytes: 262144         # 256KB (default)
  max_combined_bytes: 1572864      # 1.5MB (default)
  truncation_strategy: tail        # head|tail|both|summary (default: tail)

  # Context injection
  max_context_bytes: 102400        # 100KB (default)
  max_context_tasks: 10            # (default)

  # External storage (coming soon)
  external_storage_threshold: 5242880  # 5MB (default)
  external_storage_dir: ".workflow_state/task_outputs"
  compress_external: true

  # Cleanup
  cleanup_strategy:
    type: highest_relevance        # (default: most_recent)
    keep_count: 20                 # (default)
```

### Task-Level Context

```yaml
tasks:
  my_task:
    inject_context: true  # Enable context injection
    context:
      mode: automatic             # automatic|manual|none (default: automatic)
      min_relevance: 0.5          # 0.0-1.0 (default: 0.5)
      max_bytes: 100000           # Override workflow limit
      max_tasks: 5                # Override workflow limit
      include_tasks: [task1]      # Manual mode only
      exclude_tasks: [task2]      # Manual mode only
```

## Use Cases

### Long-Running Workflows

Prevent memory exhaustion in loops or infinite workflows:

```yaml
limits:
  max_stdout_bytes: 262144   # 256KB
  cleanup_strategy:
    type: lru
    keep_count: 10

tasks:
  process_forever:
    loop:
      type: while
      condition: "true"
    agent: "processor"
```

### Data Pipelines

Smart context in processing pipelines:

```yaml
tasks:
  fetch:
    agent: "fetcher"
    output: "data.json"

  process:
    agent: "processor"
    depends_on: [fetch]
    inject_context: true  # Gets fetch output
    context:
      min_relevance: 0.9  # Only direct dependencies

  analyze:
    agent: "analyzer"
    depends_on: [process]
    inject_context: true  # Gets process output (1.0) and fetch (0.4)
```

### Resource-Constrained Environments

Minimal memory footprint:

```yaml
limits:
  max_stdout_bytes: 102400   # 100KB
  max_context_bytes: 51200   # 50KB
  max_context_tasks: 3
  cleanup_strategy:
    type: lru
    keep_count: 5
```

## Examples

See complete examples in:
- `examples/stdio-context-example.yaml` - Comprehensive demonstration
- `examples/low-memory-workflow.yaml` - Resource-constrained settings

## Documentation

- `STDIO_CONTEXT_IMPLEMENTATION.md` - Detailed implementation guide
- `STDIO_CONTEXT_INTEGRATION.md` - Integration with executor
- `STDIO_CONTEXT_MIGRATION.md` - Migration guide for existing workflows
- `STDIO_CONTEXT_STATUS.md` - Implementation status
- `STDIO_CONTEXT_SUMMARY.md` - Quick reference

## API

### Rust API

```rust
use claude_agent_sdk::dsl::{
    build_smart_context,
    calculate_relevance,
    create_task_output,
    truncate_output,
    LimitsConfig,
    TruncationStrategy,
    ContextConfig,
    ContextMode,
};

// Truncate output
let (truncated, was_truncated) = truncate_output(
    &large_output,
    1_048_576,  // 1MB
    &TruncationStrategy::Tail,
);

// Create TaskOutput
let output = create_task_output(
    "task1".to_string(),
    OutputType::Stdout,
    raw_output,
    1_048_576,
    &TruncationStrategy::Tail,
);

// Build context
let context = build_smart_context(
    "current_task",
    &workflow,
    &task_graph,
    &state,
    Some(&context_config),
);

// Calculate relevance
let score = calculate_relevance(
    "task2",
    "task1",
    &task_graph,
    &workflow,
);
```

## Performance

- **Truncation**: O(1) for tail/head, O(n) for copying
- **Relevance**: O(d) where d = dependency depth
- **Context Building**: O(n log n) where n = task count
- **Cleanup**: O(n log n) for sorting-based strategies

Efficient even with 100+ tasks.

## Monitoring

### Metrics

```rust
let metrics = state.get_context_metrics();
println!("Total bytes: {}", metrics.total_bytes);
println!("Task outputs: {}", metrics.task_count);
println!("Truncated: {}", metrics.truncated_count);
println!("Avg relevance: {:.2}", metrics.avg_relevance);
```

### Logging

```yaml
# Enable debug logging to see:
# - Truncation notices
# - Context building
# - Cleanup events
```

## Backwards Compatibility

✅ **All features are optional**
✅ **Existing workflows work unchanged**
✅ **Defaults are sensible**
✅ **Easy rollback**

If you don't add `limits`, workflows behave exactly as before.

## Roadmap

### ✅ Implemented
- Output truncation (head/tail/both)
- Smart context injection
- Multiple cleanup strategies
- Task-level overrides
- Observability metrics

### ⏳ Coming Soon
- External storage for large outputs
- AI-powered summarization
- Streaming truncation
- Custom relevance functions

## Contributing

The implementation is in:
- `src/dsl/truncation.rs` - Truncation logic
- `src/dsl/context_injection.rs` - Context building
- `src/dsl/schema.rs` - Configuration types
- `src/dsl/state.rs` - State management

## License

Same as claude-agent-sdk

## Support

For issues or questions, see:
- Implementation docs in `STDIO_CONTEXT_IMPLEMENTATION.md`
- Migration guide in `STDIO_CONTEXT_MIGRATION.md`
- Examples in `examples/`

---

**Status**: ✅ Core implementation complete, ready for integration
**Version**: 1.0.0
**Date**: 2025-10-24

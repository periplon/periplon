# Migration Guide: Stdio and Context Management

This guide helps you adopt the new stdio and context management features in your existing workflows.

## Compatibility

**Good News**: Your existing workflows work unchanged! All new features are optional.

## Migration Strategy

We recommend a **gradual adoption** approach:

### Phase 1: Add Basic Output Limits (Week 1)

Start by adding simple output limits to prevent runaway stdout/stderr:

```yaml
# Before (no limits)
name: "My Workflow"
version: "1.0.0"

tasks:
  # ... your existing tasks ...

# After (basic limits)
name: "My Workflow"
version: "1.0.0"

limits:
  max_stdout_bytes: 1048576  # 1MB - catches most issues
  truncation_strategy: tail   # Keep recent output

tasks:
  # ... your existing tasks unchanged ...
```

**Impact**:
- ✅ No breaking changes
- ✅ Prevents memory exhaustion from verbose tasks
- ✅ You can see truncation notices in logs

### Phase 2: Add Context Limits (Week 2)

Once comfortable with output limits, add context limits:

```yaml
limits:
  max_stdout_bytes: 1048576
  truncation_strategy: tail

  # NEW: Context limits
  max_context_bytes: 102400   # 100KB
  max_context_tasks: 10       # Max 10 tasks in context
```

**Impact**:
- ✅ Limits context size for agent tasks
- ✅ Prevents context bloat in long workflows
- ✅ Still no changes to task definitions

### Phase 3: Enable Smart Context (Week 3)

Enable intelligent context injection on specific tasks:

```yaml
tasks:
  analyze_data:
    description: "Analyze the processed data"
    agent: "analyzer"
    depends_on: [process_data]

    # NEW: Enable automatic context injection
    inject_context: true
    context:
      mode: automatic
      min_relevance: 0.7  # Only include relevant outputs
```

**Impact**:
- ✅ Tasks get relevant context from dependencies
- ✅ Better task execution with proper context
- ✅ Still opt-in per task

### Phase 4: Add Cleanup Strategy (Week 4)

Add automatic cleanup for long-running workflows:

```yaml
limits:
  max_stdout_bytes: 1048576
  max_context_bytes: 102400

  # NEW: Automatic cleanup
  cleanup_strategy:
    type: highest_relevance
    keep_count: 15
```

**Impact**:
- ✅ Automatic pruning of old outputs
- ✅ Bounded memory for infinite workflows
- ✅ Configurable retention policy

### Phase 5: Fine-Tune Per Task (Ongoing)

Override defaults for specific needs:

```yaml
tasks:
  verbose_task:
    description: "Task that produces lots of output"
    agent: "worker"

    # Task-specific overrides
    limits:
      max_stdout_bytes: 10485760  # 10MB for this task
      truncation_strategy: both    # Keep first and last

  quiet_task:
    description: "Task that needs no context"
    agent: "worker"

    # Disable context injection
    context:
      mode: none
```

## Common Migration Patterns

### Pattern 1: Data Processing Pipeline

**Before**:
```yaml
tasks:
  fetch:
    agent: "fetcher"
    output: "data.json"

  process:
    agent: "processor"
    depends_on: [fetch]
    output: "processed.json"

  analyze:
    agent: "analyzer"
    depends_on: [process]
```

**After** (with smart context):
```yaml
limits:
  max_stdout_bytes: 2097152  # 2MB
  max_context_bytes: 204800  # 200KB
  cleanup_strategy:
    type: direct_dependencies

tasks:
  fetch:
    agent: "fetcher"
    output: "data.json"
    limits:
      max_stdout_bytes: 5242880  # 5MB - large dataset

  process:
    agent: "processor"
    depends_on: [fetch]
    output: "processed.json"
    inject_context: true  # Gets fetch output
    context:
      mode: automatic
      min_relevance: 0.9  # Only direct deps

  analyze:
    agent: "analyzer"
    depends_on: [process]
    inject_context: true  # Gets process output
    context:
      mode: automatic
      max_tasks: 5
```

### Pattern 2: Long-Running Workflow

**Before** (risk of memory exhaustion):
```yaml
tasks:
  generate_reports:
    loop:
      type: repeat
      count: 100
    agent: "reporter"
```

**After** (bounded memory):
```yaml
limits:
  max_stdout_bytes: 524288  # 512KB
  max_context_bytes: 102400  # 100KB
  cleanup_strategy:
    type: lru
    keep_count: 10  # Only keep 10 most recent

tasks:
  generate_reports:
    loop:
      type: repeat
      count: 100
    agent: "reporter"
    limits:
      max_stdout_bytes: 1048576  # 1MB per iteration
```

### Pattern 3: Parallel Tasks with Convergence

**Before**:
```yaml
tasks:
  task1:
    agent: "worker1"

  task2:
    agent: "worker2"

  task3:
    agent: "worker3"

  merge:
    agent: "merger"
    depends_on: [task1, task2, task3]
```

**After** (with manual context control):
```yaml
limits:
  max_context_bytes: 153600  # 150KB

tasks:
  task1:
    agent: "worker1"
    output: "result1.json"

  task2:
    agent: "worker2"
    output: "result2.json"

  task3:
    agent: "worker3"
    output: "result3.json"

  merge:
    agent: "merger"
    depends_on: [task1, task2, task3]
    inject_context: true
    context:
      mode: manual
      include_tasks: [task1, task2, task3]
      max_bytes: 100000  # 100KB total
```

## Workflow-by-Workflow Migration

### Small Workflows (< 10 tasks)

**Recommendation**: Start with defaults, add limits only if needed

```yaml
# Minimal approach
limits:
  max_stdout_bytes: 1048576  # Just prevent runaway output
```

### Medium Workflows (10-50 tasks)

**Recommendation**: Add context limits and enable smart injection

```yaml
limits:
  max_stdout_bytes: 1048576
  max_context_bytes: 204800
  max_context_tasks: 10
  cleanup_strategy:
    type: most_recent
    keep_count: 20
```

### Large Workflows (50+ tasks)

**Recommendation**: Full configuration with aggressive cleanup

```yaml
limits:
  max_stdout_bytes: 524288
  max_context_bytes: 102400
  max_context_tasks: 5
  cleanup_strategy:
    type: highest_relevance
    keep_count: 15
```

### Long-Running Workflows (loops, infinite)

**Recommendation**: Very conservative limits with LRU cleanup

```yaml
limits:
  max_stdout_bytes: 262144   # 256KB
  max_context_bytes: 51200   # 50KB
  max_context_tasks: 3
  cleanup_strategy:
    type: lru
    keep_count: 10
```

## Testing Your Migration

After adding limits, verify:

### 1. Check Truncation

Look for truncation notices in logs:
```
--- Output (showing last 1048576 bytes of 5242880) ---
```

### 2. Verify Context Injection

Enable debug logging and check context size:
```
Built context for task analyze: 45678 bytes from 3 tasks
```

### 3. Monitor Cleanup

Watch for pruning events:
```
Pruned outputs: 25 -> 15 tasks (strategy: highest_relevance)
```

### 4. Measure Memory

Compare before/after memory usage:
```bash
# Before
ps aux | grep dsl-executor
# 2.5GB RSS

# After (with limits)
ps aux | grep dsl-executor
# 512MB RSS
```

## Rollback Plan

If issues occur, rollback is simple:

### Option 1: Remove Limits
```yaml
# Just delete the limits section
# limits:
#   max_stdout_bytes: 1048576
```

### Option 2: Increase Limits
```yaml
limits:
  max_stdout_bytes: 10485760  # 10MB - very permissive
  max_context_bytes: 1048576  # 1MB - large context
  max_context_tasks: 50       # Many tasks
  cleanup_strategy:
    type: most_recent
    keep_count: 100  # Keep almost everything
```

### Option 3: Disable Per Task
```yaml
tasks:
  problematic_task:
    # Disable all limits for this task
    limits:
      max_stdout_bytes: 104857600  # 100MB
    context:
      mode: none  # No context injection
```

## Common Issues and Solutions

### Issue 1: Important Output Truncated

**Symptom**: Critical information lost in truncation

**Solution**: Increase limits or change strategy
```yaml
tasks:
  important_task:
    limits:
      max_stdout_bytes: 5242880  # 5MB
      truncation_strategy: both  # Keep first and last
```

### Issue 2: Too Much Context

**Symptom**: Agent gets confused with too much information

**Solution**: Reduce context or use manual mode
```yaml
tasks:
  focused_task:
    context:
      mode: manual
      include_tasks: [essential_task]  # Only critical context
      max_bytes: 50000
```

### Issue 3: Aggressive Cleanup

**Symptom**: Needed outputs pruned too soon

**Solution**: Adjust cleanup strategy
```yaml
limits:
  cleanup_strategy:
    type: direct_dependencies  # Only prune if no dependents
```

### Issue 4: Performance Overhead

**Symptom**: Workflow slower after adding limits

**Solution**: Optimize cleanup interval
```yaml
# In executor code
if task_count % 20 == 0 {  // Less frequent cleanup
    state.prune_outputs(&strategy);
}
```

## Best Practices

1. **Start Conservative**: Use defaults, increase only if needed
2. **Monitor First**: Add limits, observe behavior, adjust
3. **Test Locally**: Try on dev workflows before production
4. **Document Overrides**: Comment why specific tasks have custom limits
5. **Review Periodically**: Check if limits still make sense

## FAQ

**Q: Will this break my existing workflows?**
A: No. All features are optional and backwards compatible.

**Q: Do I need to update all workflows at once?**
A: No. Migrate gradually, one workflow at a time.

**Q: What if I don't add limits?**
A: Workflows work as before, but without memory protection.

**Q: Can I disable limits for specific tasks?**
A: Yes. Set very high limits or use `context: { mode: none }`.

**Q: How do I know what limits to use?**
A: Start with defaults, monitor metrics, adjust based on actual usage.

**Q: Will truncation lose important data?**
A: Truncation notices show what was removed. Increase limits if needed.

**Q: Does context injection change task behavior?**
A: Only if `inject_context: true`. It provides additional context to agents.

**Q: How often does cleanup run?**
A: Configurable. Default is every 10 tasks.

## Support

For issues or questions:
1. Check logs for truncation/cleanup notices
2. Use `state.log_metrics()` to see current usage
3. Review examples in `examples/stdio-context-example.yaml`
4. Consult `STDIO_CONTEXT_IMPLEMENTATION.md` for details

## Summary

Migration is **safe and gradual**:
- ✅ No breaking changes
- ✅ All features optional
- ✅ Easy rollback
- ✅ Incremental adoption
- ✅ Backwards compatible

Start with basic limits, monitor, and expand as needed.

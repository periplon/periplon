# Stdio & Context Management DSL Integration

## Summary

Successfully integrated comprehensive stdio and context management documentation into the DSL template generator and natural language generator systems.

## Changes Made

### 1. Template Generator (`src/dsl/template.rs`)

#### A. Updated `generate_template()` Function

**Workflow-Level Limits Section** (lines 213-235):
- Added comprehensive documentation for output truncation
- Documented context injection limits
- Included cleanup strategy configuration
- Provides default values and strategy options

**Task-Level Configuration** (lines 1222-1255):
- Added context configuration documentation with all modes (automatic, manual, none)
- Documented relevance scoring and filtering
- Included task-level limit overrides
- Provided clear examples for each option

#### B. Updated `generate_nl_to_dsl_prompt()` Function

**Root Fields Documentation** (line 1887):
- Added `limits` to the list of optional root fields
- References the new "Stdio & Context Management" section

**New Section: "Stdio & Context Management"** (lines 1921-1986):
- Complete overview of the feature
- Workflow-level limits with examples
- Truncation strategies explained
- Task-level context configuration
- Context modes (automatic, manual, none) with relevance scoring
- Task-level limit overrides
- Use cases for different scenarios

**Task Schema Documentation** (lines 2410-2453):
- Enhanced `inject_context` documentation
- Added `context` configuration block with all options
- Added `limits` configuration block for task-level overrides
- Fixed stray placeholder text

### 2. Natural Language Generator (`src/dsl/nl_generator.rs`)

No changes required - the NL generator uses `generate_nl_to_dsl_prompt()` from template.rs, so it automatically includes all the stdio/context documentation.

## Features Documented

### Output Truncation
- `max_stdout_bytes`: Default 1MB per task
- `max_stderr_bytes`: Default 256KB per task
- `truncation_strategy`: head | tail | both | summary

### Context Injection
- `inject_context`: Enable/disable
- `context.mode`: automatic | manual | none
- `context.min_relevance`: 0.0-1.0 threshold
- `context.max_bytes`: Size limit
- `context.max_tasks`: Task count limit
- `context.include_tasks`: Manual mode whitelist
- `context.exclude_tasks`: Manual mode blacklist

### Cleanup Strategies
- `most_recent`: Keep N most recent tasks
- `highest_relevance`: Keep N highest scored
- `lru`: Least Recently Used
- `direct_dependencies`: Keep only tasks with dependents

### Context Modes Explained
- **Automatic**: Dependency-based relevance scoring
  - Direct dependency: 1.0
  - Transitive: 0.8 / depth
  - Same agent: 0.5
- **Manual**: Explicit include/exclude lists
- **None**: Disable context injection

## Verification

### Tests Passing
```bash
cargo test --lib dsl::template
# All 3 tests pass:
# - test_dsl_grammar_version
# - test_generate_template
# - test_generate_nl_to_dsl_prompt
```

### Template Output Verified
```bash
./target/release/dsl-executor template
# Generates complete template with:
# - Workflow-level limits (commented examples)
# - Task-level context configuration
# - Task-level limit overrides
# - Proper documentation for all options
```

### Documentation Counts
- `max_stdout_bytes`: 5 occurrences (workflow + task levels)
- `mode: automatic`: 3 occurrences
- `cleanup_strategy`: 2 occurrences
- `inject_context`: 3 occurrences

## Examples Generated

### Workflow-Level Example
```yaml
limits:
  # Output Truncation
  max_stdout_bytes: 1048576       # 1MB per task (default)
  max_stderr_bytes: 262144        # 256KB per task (default)
  truncation_strategy: tail        # head|tail|both|summary (default: tail)
  # Context Injection
  max_context_bytes: 102400        # 100KB total context (default)
  max_context_tasks: 10            # Max tasks to include (default)
  # Cleanup Strategy
  cleanup_strategy:
    type: highest_relevance        # most_recent|highest_relevance|lru|direct_dependencies
    keep_count: 20                 # Number of task outputs to retain
```

### Task-Level Example
```yaml
tasks:
  my_task:
    inject_context: true
    context:
      mode: automatic              # automatic|manual|none (default: automatic)
      min_relevance: 0.5           # 0.0-1.0 filter threshold (default: 0.5)
      max_bytes: 100000            # Override workflow limit
      max_tasks: 5                 # Override workflow limit
      include_tasks: [task1, task2] # Manual mode only
      exclude_tasks: [noisy_task]   # Manual mode only
    limits:
      max_stdout_bytes: 10485760   # 10MB override
      truncation_strategy: both    # head|tail|both|summary
```

## Use Cases Documented

1. **Long-running workflows**: Prevent memory exhaustion with cleanup strategies
2. **Data pipelines**: Smart context injection for processing chains
3. **Resource-constrained environments**: Minimal memory footprint

## Integration Points

The stdio/context documentation is now available to:

1. **Template Generator**: Users running `dsl-executor template` see full documentation
2. **Natural Language Generator**: AI has complete schema when generating workflows
3. **Documentation**: Comprehensive inline examples and explanations

## Backwards Compatibility

✅ All features are optional
✅ Existing workflows work unchanged  
✅ Defaults are sensible
✅ Easy to adopt incrementally

## Next Steps

Users can now:
1. Generate templates with `dsl-executor template` to see examples
2. Use natural language descriptions that reference memory management
3. Create workflows with bounded memory and smart context injection
4. Override limits at task level for specific needs

## Files Modified

- `src/dsl/template.rs`: Added comprehensive stdio/context documentation to both template generation and NL prompt generation functions

## Build Status

✅ Compiles successfully
✅ All tests pass
✅ Template generation works
✅ Natural language prompt includes full documentation

---

**Date**: 2025-10-24
**Status**: ✅ Complete
**Version**: Integrated with DSL v2.14.0

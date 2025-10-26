# Stdio and Context Management - Implementation Plan

## Executive Summary

**Goal**: Prevent unbounded memory growth in DSL workflows by implementing configurable limits, smart truncation, and selective context injection.

**Timeline**: 5 weeks
**Priority**: High (solves critical memory exhaustion issues)
**Status**: Design complete, ready for implementation

## Problem Statement

Current DSL implementation has no limits on:
- stdout/stderr capture from script tasks
- Task result accumulation in WorkflowState
- Context injection into agent prompts

This causes:
- Memory exhaustion in long-running workflows
- OOM crashes with loops producing large outputs
- Performance degradation over time

## Solution Architecture

### Three-Tier Approach

```
┌─────────────────────────────────────────────┐
│ Tier 1: Capture Limits                     │
│ • Truncate stdout/stderr at source         │
│ • Configurable per workflow/task           │
│ • Multiple truncation strategies           │
└─────────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────────┐
│ Tier 2: Storage Management                 │
│ • TaskOutput struct with metadata          │
│ • External storage for large outputs       │
│ • Compression support                      │
└─────────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────────┐
│ Tier 3: Injection Control                  │
│ • Dependency-based relevance               │
│ • Smart context builder                    │
│ • Configurable thresholds                  │
└─────────────────────────────────────────────┘
```

## Implementation Phases

### Phase 1: Schema and Configuration (Week 1)

**Goal**: Add configuration schema for limits

#### Tasks

**1.1 Update `src/dsl/schema.rs`**
- [ ] Add `LimitsConfig` struct
  ```rust
  pub struct LimitsConfig {
      pub max_stdout_bytes: usize,
      pub max_stderr_bytes: usize,
      pub max_combined_bytes: usize,
      pub truncation_strategy: TruncationStrategy,
      pub max_context_bytes: usize,
      pub max_context_tasks: usize,
      // ... more fields
  }
  ```
- [ ] Add `TruncationStrategy` enum (Head, Tail, Both, HeadLines, TailLines, Summary)
- [ ] Add `CleanupStrategyConfig` enum
- [ ] Add `TaskContextConfig` struct
- [ ] Add `ContextMode` enum
- [ ] Update `DSLWorkflow` - add `limits: LimitsConfig` field
- [ ] Update `TaskSpec` - add optional `limits` and `context` fields
- [ ] Implement `Default` trait for all new types

**1.2 Update Parser**
- [ ] Update `src/dsl/parser.rs` to parse new fields
- [ ] Add validation for limit values (no negative, reasonable max)
- [ ] Test parsing with sample YAML

**1.3 Update Validator**
- [ ] Update `src/dsl/validator.rs` to validate limits config
- [ ] Warn if limits are too restrictive (< 1KB)
- [ ] Warn if limits are unreasonably high (> 100MB)

**Testing**
- [ ] Unit tests for schema deserialization
- [ ] Test default values are applied
- [ ] Test task-level overrides workflow-level
- [ ] Test invalid configurations are rejected

**Deliverables**
- Updated schema with all new types
- Parsing support for limits configuration
- Validation rules
- Unit tests passing

---

### Phase 2: Basic Truncation (Week 2)

**Goal**: Implement stdout/stderr truncation in executor

#### Tasks

**2.1 Create Truncation Utilities**
- [ ] Create `src/dsl/truncation.rs` module
- [ ] Implement `truncate_output()` function
  - Head strategy
  - Tail strategy
  - Both strategy
  - HeadLines strategy
  - TailLines strategy
- [ ] Implement `truncate_to_size()` helper
- [ ] Add metadata annotations (show bytes truncated)

**2.2 Update Script Executor**
- [ ] Modify `execute_script_task()` in `src/dsl/executor.rs`
- [ ] Get limits config from task or workflow
- [ ] Apply truncation to stdout before storing
- [ ] Apply truncation to stderr before storing
- [ ] Handle combined output limits
- [ ] Preserve original error handling

**2.3 Update Command Executor**
- [ ] Modify `execute_command_task()` similarly
- [ ] Apply same truncation logic
- [ ] Ensure exit codes still checked correctly

**2.4 Update Output Handling**
- [ ] Modify safe_print/safe_eprint to handle truncated output
- [ ] Add visual indicators for truncated output
- [ ] Ensure truncation messages are clear

**Testing**
- [ ] Test each truncation strategy
- [ ] Test with outputs smaller than limit (no truncation)
- [ ] Test with outputs larger than limit (truncation applied)
- [ ] Test with very large outputs (MB+)
- [ ] Test combined stdout+stderr limits
- [ ] Verify exit codes still work after truncation

**Deliverables**
- Truncation module with all strategies
- Script/command executors apply limits
- Tests for all truncation strategies
- Visual indicators for truncated output

---

### Phase 3: Enhanced State Management (Week 3)

**Goal**: Replace simple task_results with rich TaskOutput tracking

#### Tasks

**3.1 Update State Schema**
- [ ] Create `src/dsl/task_output.rs` module
- [ ] Define `TaskOutput` struct
  ```rust
  pub struct TaskOutput {
      pub task_id: String,
      pub output_type: OutputType,
      pub content: String,
      pub original_size: usize,
      pub truncated: bool,
      pub truncation_strategy: Option<TruncationStrategy>,
      pub external_path: Option<PathBuf>,
      pub relevance_score: f64,
      pub last_accessed: SystemTime,
      pub depended_by: Vec<String>,
      pub completed_at: SystemTime,
  }
  ```
- [ ] Define `OutputType` enum
- [ ] Define `ContextMetrics` struct

**3.2 Update WorkflowState**
- [ ] In `src/dsl/state.rs`, replace `task_results: HashMap<String, String>`
  with `task_outputs: HashMap<String, TaskOutput>`
- [ ] Add `context_metrics: ContextMetrics` field
- [ ] Implement `record_task_output()` method
- [ ] Update `build_context_summary()` to use TaskOutput
- [ ] Add `update_metrics()` method
- [ ] Implement `get_context_metrics()` method

**3.3 Update Executor**
- [ ] Modify `execute_task_static()` to record TaskOutput
- [ ] Pass limits config to recording function
- [ ] Store truncation metadata
- [ ] Update relevance scores (initially set to 1.0)

**3.4 State Persistence**
- [ ] Update state serialization to handle new format
- [ ] Add migration path from old task_results format
- [ ] Test state save/load with new schema

**Testing**
- [ ] Test TaskOutput creation and storage
- [ ] Test metrics calculation
- [ ] Test state persistence with new format
- [ ] Test migration from old format
- [ ] Verify backwards compatibility

**Deliverables**
- TaskOutput module with full metadata
- WorkflowState using TaskOutput
- Context metrics tracking
- State persistence working
- Migration path for old state files

---

### Phase 4: External Storage (Week 4)

**Goal**: Implement external storage for large outputs

#### Tasks

**4.1 External Storage Module**
- [ ] Create `src/dsl/external_storage.rs`
- [ ] Implement `store_output_externally()` function
- [ ] Support plain text storage
- [ ] Support gzip compression
- [ ] Implement `load_external_output()` function
- [ ] Create directory structure on demand

**4.2 Update WorkflowState**
- [ ] Add `store_externally()` method
- [ ] Check `external_storage_threshold` before storing
- [ ] Compress if `compress_external` is true
- [ ] Store only metadata in memory, full content on disk
- [ ] Update metrics to track external storage

**4.3 Update Output Recording**
- [ ] In `record_task_output()`, check size threshold
- [ ] Store externally if over threshold
- [ ] Keep truncated summary in memory
- [ ] Set `external_path` in TaskOutput

**4.4 Cleanup**
- [ ] Implement cleanup of external files when pruning
- [ ] Delete external files when tasks removed from state
- [ ] Handle missing external files gracefully

**Testing**
- [ ] Test external storage creation
- [ ] Test compression works correctly
- [ ] Test loading from external storage
- [ ] Test cleanup of external files
- [ ] Test with missing external files
- [ ] Verify compressed vs uncompressed
- [ ] Test directory creation

**Deliverables**
- External storage module
- Compression support
- WorkflowState integration
- Cleanup handling
- Tests for all storage scenarios

---

### Phase 5: Relevance and Context Selection (Week 5)

**Goal**: Implement dependency-based context injection

#### Tasks

**5.1 Relevance Calculation**
- [ ] Create `src/dsl/relevance.rs` module
- [ ] Implement `calculate_relevance()` function
  - Direct dependency = 1.0
  - Transitive dependency = 0.8 / depth
  - Same agent = 0.5
  - Recent task = 0.3
  - No relation = 0.0
- [ ] Implement `get_dependency_depth()` helper
- [ ] Implement `is_direct_dependency()` helper
- [ ] Implement `uses_same_agent()` helper
- [ ] Implement `is_recent()` helper

**5.2 Smart Context Builder**
- [ ] Create `build_smart_context()` function
- [ ] Filter by relevance threshold
- [ ] Sort by relevance score
- [ ] Apply max_bytes limit
- [ ] Apply max_tasks limit
- [ ] Handle time window filtering
- [ ] Respect task-level context config

**5.3 Update Executor**
- [ ] Replace `build_context_summary()` with `build_smart_context()`
- [ ] Calculate relevance scores when recording outputs
- [ ] Update relevance on new task completion
- [ ] Update last_accessed timestamp when context used
- [ ] Handle `inject_context` flag
- [ ] Handle `context.mode` settings

**5.4 Context Modes**
- [ ] Implement automatic mode (dependency-based)
- [ ] Implement manual mode (include/exclude lists)
- [ ] Implement none mode (disable injection)
- [ ] Test mode switching

**Testing**
- [ ] Test relevance calculation for all scenarios
- [ ] Test dependency depth calculation
- [ ] Test smart context builder limits
- [ ] Test context modes
- [ ] Test with complex dependency graphs
- [ ] Verify performance with many tasks

**Deliverables**
- Relevance calculation module
- Smart context builder
- Context mode implementations
- Updated executor using smart context
- Tests for all relevance scenarios

---

### Phase 6: Cleanup and Pruning (Week 6)

**Goal**: Implement automatic output pruning

#### Tasks

**6.1 Pruning Strategies**
- [ ] In `WorkflowState`, implement `prune_outputs()` method
- [ ] Implement `prune_most_recent()` strategy
- [ ] Implement `prune_by_relevance()` strategy
- [ ] Implement `prune_lru()` strategy
- [ ] Implement `prune_non_dependencies()` strategy
- [ ] Update metrics after pruning

**6.2 Automatic Pruning**
- [ ] Add pruning triggers to executor
- [ ] Prune after N tasks (configurable)
- [ ] Prune on memory threshold
- [ ] Prune before checkpointing
- [ ] Log pruning events

**6.3 Hook Integration**
- [ ] Support `post_task` hook for pruning
- [ ] Support `checkpoint_and_prune` command
- [ ] Allow manual pruning trigger

**6.4 External File Cleanup**
- [ ] Delete external files when pruning
- [ ] Keep external files if still referenced
- [ ] Handle cleanup errors gracefully

**Testing**
- [ ] Test each pruning strategy
- [ ] Test automatic pruning triggers
- [ ] Test external file cleanup
- [ ] Test with various keep_count values
- [ ] Verify metrics updated after pruning

**Deliverables**
- All pruning strategies implemented
- Automatic pruning working
- Hook integration
- External file cleanup
- Tests for all strategies

---

### Phase 7: Monitoring and Observability (Week 7)

**Goal**: Add metrics and diagnostic tools

#### Tasks

**7.1 Metrics Collection**
- [ ] Enhance `ContextMetrics` with more fields
- [ ] Track total memory usage
- [ ] Track external storage usage
- [ ] Track truncation frequency
- [ ] Track pruning frequency
- [ ] Track average relevance

**7.2 Metrics Logging**
- [ ] Implement `log_context_metrics()` method
- [ ] Add periodic logging (every N tasks)
- [ ] Add summary at workflow end
- [ ] Format metrics for readability

**7.3 Diagnostic Commands**
- [ ] Add `--show-context-metrics` flag to executor
- [ ] Add `--show-task-outputs` to list outputs
- [ ] Add `--show-external-storage` to show disk usage
- [ ] Add `--validate-limits` to check configuration

**7.4 Warnings**
- [ ] Warn if approaching memory limits
- [ ] Warn if external storage filling up
- [ ] Warn if too many outputs being pruned
- [ ] Suggest configuration adjustments

**Testing**
- [ ] Test metrics collection accuracy
- [ ] Test logging output
- [ ] Test diagnostic commands
- [ ] Test warnings trigger correctly

**Deliverables**
- Complete metrics collection
- Logging and reporting
- Diagnostic commands
- Warning system
- User-friendly output

---

### Phase 8: Documentation and Examples (Week 8)

**Goal**: Complete documentation and create examples

#### Tasks

**8.1 User Documentation**
- [ ] Update main README with limits section
- [ ] Create configuration guide
- [ ] Create troubleshooting guide
- [ ] Add migration guide for existing workflows

**8.2 API Documentation**
- [ ] Document all new types in rustdoc
- [ ] Add examples to rustdoc
- [ ] Document configuration options
- [ ] Document best practices

**8.3 Example Workflows**
- [ ] Create example: basic limits
- [ ] Create example: external storage
- [ ] Create example: smart context
- [ ] Create example: cleanup strategies
- [ ] Create example: long-running workflow
- [ ] Create example: memory-constrained

**8.4 Integration with Existing Docs**
- [ ] Update DSL guide with limits
- [ ] Update template generation
- [ ] Update validation documentation
- [ ] Add to CLAUDE.md project guide

**Testing**
- [ ] Verify all examples work
- [ ] Test documentation accuracy
- [ ] Get feedback from users

**Deliverables**
- Complete user documentation
- API documentation
- Working examples
- Updated project docs

---

## Testing Strategy

### Unit Tests (Each Phase)
- Test individual functions in isolation
- Test edge cases and error conditions
- Test default values
- Test configuration validation

### Integration Tests
- [ ] Test full workflow with limits
- [ ] Test truncation end-to-end
- [ ] Test external storage end-to-end
- [ ] Test context injection end-to-end
- [ ] Test pruning end-to-end
- [ ] Test with various configurations

### Performance Tests
- [ ] Memory usage over time
- [ ] CPU overhead of truncation
- [ ] Disk I/O for external storage
- [ ] Context building performance
- [ ] Pruning performance

### Stress Tests
- [ ] Workflow with 1000+ tasks
- [ ] Loop with 10,000 iterations
- [ ] Very large outputs (100MB+)
- [ ] Long-running workflow (24+ hours)
- [ ] Memory-constrained environment

### Regression Tests
- [ ] Existing workflows work unchanged
- [ ] Backwards compatibility with old state files
- [ ] No performance regression for small workflows

## Success Criteria

### Functional Requirements
✅ Workflows respect configured limits
✅ Truncation strategies work correctly
✅ External storage works with compression
✅ Context injection is selective
✅ Pruning keeps memory bounded
✅ Metrics are accurate

### Non-Functional Requirements
✅ Memory usage bounded (< 100MB for typical workflow)
✅ Performance overhead < 5%
✅ Configuration is intuitive
✅ Error messages are clear
✅ Backwards compatible

### User Acceptance
✅ Users can run long workflows without OOM
✅ Configuration is straightforward
✅ Documentation is comprehensive
✅ Migration is smooth

## Dependencies

### External Crates
- `flate2` - For gzip compression
- `chrono` - For time-based filtering (already used)
- `serde` - For serialization (already used)

### Internal Dependencies
- Phase 2 depends on Phase 1 (schema)
- Phase 3 depends on Phase 2 (truncation)
- Phase 4 depends on Phase 3 (TaskOutput)
- Phase 5 depends on Phase 3 (TaskOutput)
- Phase 6 depends on Phase 5 (relevance)
- Phase 7 depends on all phases (metrics)
- Phase 8 depends on all phases (docs)

## Risk Management

### Risks

1. **Breaking Changes**
   - Mitigation: Maintain backwards compatibility
   - Default values for all new fields
   - Migration path for old state files

2. **Performance Degradation**
   - Mitigation: Benchmark each phase
   - Optimize hot paths
   - Make features optional

3. **Complexity**
   - Mitigation: Clear documentation
   - Sensible defaults
   - Progressive configuration

4. **External Storage Failures**
   - Mitigation: Graceful degradation
   - Fallback to in-memory
   - Clear error messages

## Rollout Plan

### Phase 1: Internal Testing (Week 9)
- Deploy to internal test environments
- Run existing workflows with defaults
- Monitor for issues
- Gather feedback

### Phase 2: Beta Release (Week 10)
- Release as beta feature
- Document as experimental
- Gather user feedback
- Fix issues

### Phase 3: General Availability (Week 11)
- Make feature generally available
- Update all documentation
- Announce in release notes
- Provide migration guide

## Metrics and KPIs

### During Development
- Code coverage > 80%
- All tests passing
- No performance regressions
- Documentation completeness

### Post-Release
- User adoption rate
- Bug reports
- Performance improvements
- Memory usage reduction

## Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| 1. Schema | 1 week | Configuration types |
| 2. Truncation | 1 week | Basic limits working |
| 3. State | 1 week | TaskOutput tracking |
| 4. Storage | 1 week | External storage |
| 5. Context | 1 week | Smart injection |
| 6. Cleanup | 1 week | Pruning strategies |
| 7. Observability | 1 week | Metrics and logging |
| 8. Documentation | 1 week | Complete docs |
| **Total** | **8 weeks** | **Production ready** |

## Resources Required

### Development
- 1 Senior Rust Developer (full-time, 8 weeks)
- 1 Reviewer (part-time, 2 hours/week)

### Testing
- QA Engineer (part-time, last 2 weeks)
- Beta testers (5-10 users)

### Documentation
- Technical Writer (part-time, week 8)

## Communication Plan

### Weekly Updates
- Progress report every Friday
- Blockers and risks
- Next week's plan

### Milestones
- Phase completion announcements
- Beta release announcement
- GA release announcement

### Documentation
- Keep design docs updated
- Update implementation notes
- Maintain CHANGELOG

## Appendix

### Reference Documentation
- `STDIO_CONTEXT_MANAGEMENT_DESIGN.md` - Full design
- `STDIO_CONTEXT_SCHEMA.rs` - Code schemas
- `STDIO_CONTEXT_QUICKSTART.md` - User guide
- `STDIO_CONTEXT_SUMMARY.md` - Executive summary

### Configuration Examples

**Minimal**
```yaml
limits: {}  # Use all defaults
```

**Recommended**
```yaml
limits:
  max_stdout_bytes: 1048576
  truncation_strategy: tail
```

**Full**
```yaml
limits:
  max_stdout_bytes: 524288
  max_stderr_bytes: 262144
  max_context_bytes: 102400
  truncation_strategy: both
  external_storage_threshold: 2097152
  cleanup_strategy:
    type: highest_relevance
    keep_count: 20
```

---

**Plan Version**: 1.0
**Last Updated**: 2025-10-24
**Status**: Ready for implementation
**Approver**: _______________
**Start Date**: _______________

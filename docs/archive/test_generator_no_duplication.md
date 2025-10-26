# Test: DSL Generator No Longer Duplicates Tasks

## Issue
The DSL generator was creating workflows with duplicate task definitions:
1. Tasks in top-level `tasks:` map (executed)
2. Same tasks in `workflows.main.steps[].tasks` (ignored, but confusing)

## Fix Applied
Updated `/src/dsl/template.rs` to clearly instruct the AI:
- Define tasks ONLY in top-level `tasks:` section
- Use `workflows:` ONLY for lifecycle hooks (pre_workflow, post_workflow, on_error)
- DO NOT duplicate tasks in both locations

## Testing

### Test 1: Simple Workflow Generation
```bash
echo "Create a workflow to process CSV files and generate a report" > /tmp/test_prompt.txt

./target/release/dsl-executor generate \
  -f /tmp/test_prompt.txt \
  -o /tmp/generated_workflow.yaml

# Check the generated file
cat /tmp/generated_workflow.yaml
```

**Expected Result:**
- ✅ Tasks defined in `tasks:` section
- ✅ NO `workflows.main.steps` section (or only hooks if present)
- ✅ Tasks use `depends_on` for execution order

### Test 2: Multi-Stage Workflow
```bash
echo "Create a workflow with data fetching, processing, and analysis stages" > /tmp/test_prompt2.txt

./target/release/dsl-executor generate \
  -f /tmp/test_prompt2.txt \
  -o /tmp/generated_workflow2.yaml

cat /tmp/generated_workflow2.yaml
```

**Expected Result:**
- ✅ All tasks in `tasks:` section only
- ✅ Tasks use `depends_on` to create stage ordering
- ✅ Optional `workflows:` section for hooks only (no task definitions)

### Test 3: Validation
```bash
# Validate the generated workflow
./target/release/dsl-executor validate /tmp/generated_workflow.yaml
./target/release/dsl-executor validate /tmp/generated_workflow2.yaml
```

**Expected Result:**
- ✅ Both workflows pass validation
- ✅ No warnings about duplicate tasks

### Test 4: Execution
```bash
# Try running a generated workflow
./target/release/dsl-executor run /tmp/generated_workflow.yaml
```

**Expected Result:**
- ✅ Workflow executes normally
- ✅ All tasks from `tasks:` section are executed
- ✅ No confusion about which tasks to run

## What Changed in the Template

### Before (Caused Duplication)
```yaml
# Old template showed BOTH:
tasks:
  process_data:
    description: "Process data"
    agent: "processor"

workflows:  # This was confusing - showed task definitions here too
  main:
    steps:
      - stage: "processing"
        tasks:  # DUPLICATED the tasks above
          - process_data:
              description: "Process data"
```

### After (No Duplication)
```yaml
# New template shows ONLY:
agents:
  processor:
    description: "Data processor"
    tools: [Read, Write]

tasks:  # ALL tasks defined here ONLY
  fetch_data:
    description: "Fetch raw data"
    agent: "processor"
  process_data:
    description: "Process data"
    agent: "processor"
    depends_on: [fetch_data]  # Use depends_on for ordering

# workflows: section is OPTIONAL - only for hooks
workflows:
  main:
    description: "Main workflow"
    hooks:  # Only hooks here, NO task definitions
      pre_workflow:
        - "echo 'Starting workflow'"
      post_workflow:
        - "echo 'Workflow complete'"
```

## Verification Checklist

After running the tests above, verify:
- [ ] Generated workflows have tasks in `tasks:` section only
- [ ] No duplicate task definitions in `workflows.main.steps`
- [ ] Tasks use `depends_on` for execution order
- [ ] `workflows:` section (if present) only contains hooks
- [ ] Generated workflows validate successfully
- [ ] Generated workflows execute correctly

## Impact

### For Users
- ✅ Cleaner, more maintainable workflows
- ✅ No confusion about which tasks will execute
- ✅ Single source of truth for task definitions
- ✅ Easier to modify workflows (change in one place)

### For Developers
- ✅ Simpler mental model
- ✅ Workflows align with actual execution behavior
- ✅ No misleading documentation

## Rollout

1. **Rebuild the generator:**
   ```bash
   cargo build --release --bin dsl-executor
   ```

2. **Test with sample prompts:**
   - Run the tests above
   - Verify no duplication

3. **Regenerate existing workflows** (optional):
   - Workflows with duplication still work (duplicates are ignored)
   - But new generation will be cleaner

4. **Update documentation:**
   - Examples now show single task definition pattern
   - Clear guidance on when to use `workflows:` section

## Notes

- Old workflows with duplication will still work (backwards compatible)
- The executor only reads from `tasks:` anyway
- The `workflows.steps.tasks` was always ignored in current implementation
- This change just prevents future confusion by not generating unused code

## Summary

The DSL generator now produces clean, non-redundant workflows:
- Tasks defined once in `tasks:` section
- `depends_on` for execution order
- `workflows:` only for lifecycle hooks

This aligns with how the executor actually works and provides a better user experience.

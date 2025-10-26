# DSL Template and Generator Structure Fixes

## Summary

Updated the DSL template and natural language generator to clearly document the required structure for outputs and notifications, making it easier for users to write valid DSL workflows.

## Issues Fixed

### 1. Outputs Structure

**Problem**: The `outputs` field requires a nested object with a `type` field, but this wasn't clear in the template or generator prompts.

**Incorrect Structure** (what users might write):
```yaml
outputs:
  final_result:
    source: file  # ❌ Wrong - flat string
    path: ./result.txt
```

**Correct Structure**:
```yaml
outputs:
  final_result:
    source:
      type: file  # ✅ Correct - object with 'type' field
      path: ./result.txt
    description: "Final workflow result"
```

### 2. Notifications Structure

**Problem**: The schema doesn't support named channel definitions. Channels must be defined inline.

**Incorrect Structure** (what users might write):
```yaml
notifications:
  channels:
    console_log:  # ❌ Wrong - named channel definition
      type: console
      config:
        use_colors: true
  defaults:
    default_channels:
      - console_log  # ❌ Wrong - reference to named channel
```

**Correct Structure**:
```yaml
notifications:
  default_channels:
    - type: console  # ✅ Correct - inline channel definition
      colored: true
      timestamp: true
  notify_on_start: true
  notify_on_completion: true
  notify_on_failure: true
```

## Changes Made

### 1. Template Generator (`src/dsl/template.rs`)

#### Updated Workflow Outputs Section (lines 212-259)
- Added `# IMPORTANT: source must be an object with 'type' field` comment
- Changed single example to three examples showing all output types:
  - `type: file` with `path` field
  - `type: state` with `key` field
  - `type: task_output` with `task` field
- Marked `type` field as `# REQUIRED` in comments

#### Updated Agent Outputs Section (lines 341-349)
- Added `# IMPORTANT: source must be an object with 'type' field` comment
- Marked `type` field as `# REQUIRED`
- Added `description` field example

#### Updated Task Outputs Section (lines 498-511)
- Added `# IMPORTANT: source must be an object with 'type' field` comment
- Marked `type` field as `# REQUIRED`
- Added `description` field example

#### Added Notifications Documentation Section (lines 1487-1537)
- Added comprehensive notification system documentation
- Clearly stated: `# IMPORTANT: Channels must be defined inline (not as named references)`
- Showed workflow-level defaults structure
- Showed task-level notification examples
- Listed all supported notification channel types
- Provided examples of inline channel definitions

### 2. Natural Language Generator Prompt (`src/dsl/template.rs`)

#### Updated Optional Root Fields (lines 1629-1630)
- Changed outputs description to: `"Map of workflow output variables (IMPORTANT: source must be object with 'type' field)"`
- Added notifications field: `"Notification defaults (IMPORTANT: channels must be inline, not named references)"`

#### Added Output Schema Section (lines 1699-1712)
- New `## Output Schema (IMPORTANT)` section
- Shows exact required structure with examples
- Emphasizes: `"CRITICAL: 'source' is an OBJECT with a 'type' field, NOT a flat string."`

#### Added Notification Schema Section (lines 1979-2009)
- New `## Notification Schema (IMPORTANT)` section
- Shows workflow-level and task-level notification structures
- Emphasizes: `"CRITICAL: Channels are NOT named references. Each channel must be fully defined inline."`
- Lists all available channel types

## Testing

1. **Template Generation**: Verified that `./target/release/dsl-executor template` generates the correct structure
2. **Workflow Validation**: Confirmed that the corrected test workflow validates successfully
3. **Unit Tests**: All template tests pass (3/3 tests passed)

## Example Corrected Workflow

See `test_workflow_validation.yaml` for a complete working example that demonstrates:
- Correct workflow-level outputs structure
- Correct notifications structure with inline channels
- Task-level outputs with proper structure
- Task-level notifications with inline channel definitions

## Files Modified

1. `src/dsl/template.rs` - Template generator and NL-to-DSL prompt
2. `test_workflow_validation.yaml` - Example corrected workflow

## Benefits

1. **Clearer Documentation**: Users can see the exact required structure
2. **Better Error Prevention**: IMPORTANT comments highlight critical requirements
3. **Multiple Examples**: Shows all variants of outputs (file, state, task_output)
4. **AI Generator Accuracy**: Natural language generator now has clear instructions
5. **Reduced Validation Errors**: Users less likely to make structural mistakes

## Migration Guide for Existing Workflows

### Updating Outputs

**Before**:
```yaml
outputs:
  result:
    source: file
    path: ./output.txt
```

**After**:
```yaml
outputs:
  result:
    source:
      type: file
      path: ./output.txt
    description: "Output description"
```

### Updating Notifications

**Before**:
```yaml
notifications:
  channels:
    my_console:
      type: console
      config:
        use_colors: true
  defaults:
    default_channels:
      - my_console
```

**After**:
```yaml
notifications:
  default_channels:
    - type: console
      colored: true
      timestamp: true
  notify_on_completion: true
```

### Updating Task Notifications

**Before**:
```yaml
on_complete:
  notify:
    channels:
      - console_log  # Named reference
    message: "Done"
```

**After**:
```yaml
on_complete:
  notify:
    message: "Done"
    channels:
      - type: console
        colored: true
        timestamp: true
```

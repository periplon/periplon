# Definition of Done (DoD)

The Definition of Done feature provides automatic quality validation for task completion with intelligent retry logic and detailed feedback to agents.

## Overview

When a task completes execution, the Definition of Done criteria are automatically checked. If any criteria are not met, the task is retried with specific feedback about what needs to be fixed. This creates a self-correcting quality loop where agents can improve their work based on concrete, actionable feedback.

## Key Features

- **Automatic Validation**: Check files, commands, tests, and outputs after task execution
- **Intelligent Retries**: Configurable retry count (default: 3) with feedback to agents
- **Detailed Feedback**: Agents receive specific information about unmet criteria
- **Flexible Failure**: Choose whether to fail tasks or continue when criteria aren't met
- **Multiple Criterion Types**: File checks, command execution, test validation, pattern matching

## Basic Usage

```yaml
tasks:
  implement_feature:
    description: "Implement user authentication"
    agent: "coder"
    definition_of_done:
      max_retries: 3  # Retry up to 3 times if criteria not met
      fail_on_unmet: true  # Fail task if still unmet after retries
      criteria:
        - type: "file_exists"
          path: "src/auth.rs"
          description: "Authentication module must exist"
        - type: "file_contains"
          path: "src/auth.rs"
          pattern: "pub fn authenticate"
          description: "Must have public authenticate function"
```

## Configuration

### Definition of Done Structure

```yaml
definition_of_done:
  max_retries: <number>       # Maximum retry attempts (default: 3)
  fail_on_unmet: <boolean>    # Fail task if unmet (default: true)
  criteria: [...]             # List of validation criteria
```

**Parameters:**
- `max_retries` (optional): Number of times to retry if criteria aren't met. Default: 3
- `fail_on_unmet` (optional): Whether to mark task as failed if criteria still unmet after all retries. Default: true
- `criteria` (required): List of validation criteria to check

## Criterion Types

### 1. File Exists

Check if a file exists in the filesystem.

```yaml
- type: "file_exists"
  path: "output/result.json"
  description: "Output file must be created"
```

**Use cases:**
- Verify output files were generated
- Check that required configuration files exist
- Ensure artifacts were created

### 2. File Contains

Check if a file contains a specific pattern or text.

```yaml
- type: "file_contains"
  path: "src/module.rs"
  pattern: "pub struct User"
  description: "Must define User struct"
```

**Use cases:**
- Verify code contains required functions or types
- Check documentation has specific sections
- Ensure configuration has required settings

### 3. File Not Contains

Check that a file does NOT contain a specific pattern.

```yaml
- type: "file_not_contains"
  path: "src/lib.rs"
  pattern: "TODO"
  description: "No TODO comments should remain"
```

**Use cases:**
- Ensure no debug code remains
- Verify TODO comments are resolved
- Check sensitive information was removed

### 4. Directory Exists

Check if a directory exists.

```yaml
- type: "directory_exists"
  path: "build/output"
  description: "Build output directory must exist"
```

**Use cases:**
- Verify build directories were created
- Check output folders exist
- Ensure directory structure is set up

### 5. Command Succeeds

Run a command and verify it exits successfully (exit code 0).

```yaml
- type: "command_succeeds"
  command: "cargo"
  args: ["build", "--release"]
  description: "Release build must succeed"
  working_dir: "."  # Optional
```

**Use cases:**
- Verify builds succeed
- Check linting passes
- Ensure compilation works

### 6. Tests Passed

Run tests and verify they all pass.

```yaml
- type: "tests_passed"
  command: "cargo"
  args: ["test", "--lib"]
  description: "All unit tests must pass"
```

**Use cases:**
- Ensure test suite passes
- Verify integration tests succeed
- Check end-to-end tests work

**Note:** This is similar to `command_succeeds` but semantically indicates test execution.

### 7. Output Matches

Check if task output or a file matches a pattern.

```yaml
- type: "output_matches"
  source:
    task_output: true  # Or: file: {path: "log.txt"}
  pattern: "Success"
  description: "Output must indicate success"
```

**Source options:**
```yaml
# Check task's output
source:
  task_output: true

# Check a file's content
source:
  file:
    path: "output.log"
```

**Use cases:**
- Verify task output contains expected messages
- Check log files for success indicators
- Ensure outputs match expected patterns

## Retry Behavior

When criteria are not met, the following happens:

1. **Check Criteria**: After task execution, all DoD criteria are evaluated
2. **Generate Feedback**: Unmet criteria are formatted into actionable feedback
3. **Retry with Feedback**: Task is re-executed with feedback appended to description
4. **Repeat**: Steps 1-3 repeat up to `max_retries` times
5. **Final Decision**:
   - If `fail_on_unmet: true`, task fails
   - If `fail_on_unmet: false`, task completes (with warning)

### Feedback Format

When criteria aren't met, agents receive detailed feedback:

```
=== DEFINITION OF DONE - UNMET CRITERIA ===

The following criteria were not met:

1. Authentication module must exist
   Status: ✗ FAILED
   Details: File 'src/auth.rs' does not exist

2. No TODO comments should remain
   Status: ✗ FAILED
   Details: File 'src/lib.rs' contains pattern 'TODO' (should not)

Please address these issues and retry the task.
```

This feedback is appended to the task description on retry, so the agent knows exactly what to fix.

## Examples

### Example 1: Code Quality Gates

Ensure code meets quality standards before marking task complete:

```yaml
tasks:
  implement_module:
    description: "Implement the payment processing module"
    agent: "developer"
    definition_of_done:
      max_retries: 3
      fail_on_unmet: true
      criteria:
        # File must exist
        - type: "file_exists"
          path: "src/payment.rs"
          description: "Payment module file must exist"

        # Must have core functions
        - type: "file_contains"
          path: "src/payment.rs"
          pattern: "pub fn process_payment"
          description: "Must have process_payment function"

        # No debug code
        - type: "file_not_contains"
          path: "src/payment.rs"
          pattern: "println!"
          description: "No debug print statements"

        # No TODOs
        - type: "file_not_contains"
          path: "src/payment.rs"
          pattern: "TODO"
          description: "All TODOs must be resolved"
```

### Example 2: Test-Driven Development

Ensure tests are written and passing:

```yaml
tasks:
  add_feature:
    description: "Add new feature with tests"
    agent: "developer"
    definition_of_done:
      max_retries: 2
      fail_on_unmet: true
      criteria:
        # Test file must exist
        - type: "file_exists"
          path: "tests/feature_tests.rs"
          description: "Test file must be created"

        # Tests must pass
        - type: "tests_passed"
          command: "cargo"
          args: ["test", "feature_tests"]
          description: "Feature tests must pass"

        # Must test success case
        - type: "file_contains"
          path: "tests/feature_tests.rs"
          pattern: "test_success"
          description: "Must include success test case"

        # Must test error case
        - type: "file_contains"
          path: "tests/feature_tests.rs"
          pattern: "test_error"
          description: "Must include error test case"
```

### Example 3: Build Verification

Ensure build succeeds and produces expected artifacts:

```yaml
tasks:
  build_release:
    description: "Build release version"
    agent: "builder"
    definition_of_done:
      max_retries: 3
      fail_on_unmet: true
      criteria:
        # Build must succeed
        - type: "command_succeeds"
          command: "cargo"
          args: ["build", "--release"]
          description: "Release build must succeed"

        # Binary must exist
        - type: "file_exists"
          path: "target/release/myapp"
          description: "Release binary must be created"

        # Output directory exists
        - type: "directory_exists"
          path: "target/release"
          description: "Release directory must exist"
```

### Example 4: Documentation Requirements

Ensure documentation is complete:

```yaml
tasks:
  write_docs:
    description: "Write API documentation"
    agent: "documenter"
    definition_of_done:
      max_retries: 2
      fail_on_unmet: false  # Optional - won't fail workflow
      criteria:
        # Docs must generate
        - type: "command_succeeds"
          command: "cargo"
          args: ["doc", "--no-deps"]
          description: "Documentation generation must succeed"

        # Main sections must exist
        - type: "file_contains"
          path: "README.md"
          pattern: "## Installation"
          description: "README must have Installation section"

        - type: "file_contains"
          path: "README.md"
          pattern: "## Usage"
          description: "README must have Usage section"

        - type: "file_contains"
          path: "README.md"
          pattern: "## Examples"
          description: "README must have Examples section"
```

### Example 5: Multi-Stage Quality Pipeline

Combine with conditional execution for sophisticated pipelines:

```yaml
tasks:
  code_implementation:
    description: "Implement feature"
    agent: "coder"
    definition_of_done:
      max_retries: 3
      criteria:
        - type: "file_exists"
          path: "src/feature.rs"
          description: "Feature file must exist"

  run_tests:
    description: "Run test suite"
    agent: "tester"
    depends_on: ["code_implementation"]
    condition:
      type: "task_status"
      task: "code_implementation"
      status: "completed"
    definition_of_done:
      max_retries: 2
      criteria:
        - type: "tests_passed"
          command: "cargo"
          args: ["test"]
          description: "All tests must pass"

  deploy:
    description: "Deploy to production"
    agent: "deployer"
    depends_on: ["run_tests"]
    condition:
      and:
        - type: "task_status"
          task: "run_tests"
          status: "completed"
        - type: "state_equals"
          key: "environment"
          value: "production"
    definition_of_done:
      max_retries: 1
      fail_on_unmet: true
      criteria:
        - type: "command_succeeds"
          command: "./deploy.sh"
          description: "Deployment script must succeed"
```

## Best Practices

### 1. Use Descriptive Messages

Make criterion descriptions actionable:

```yaml
# Good
- type: "file_contains"
  path: "src/main.rs"
  pattern: "fn main"
  description: "Must define main function entry point"

# Less helpful
- type: "file_contains"
  path: "src/main.rs"
  pattern: "fn main"
  description: "Check main"
```

### 2. Set Appropriate Retry Counts

- **Simple tasks**: 1-2 retries
- **Complex tasks**: 3-5 retries
- **Critical tasks**: Higher retries, but set realistic limits

```yaml
# Simple file creation
max_retries: 1

# Complex code generation
max_retries: 3

# Critical production deployment
max_retries: 2  # Fewer retries for safety
```

### 3. Use fail_on_unmet Wisely

```yaml
# Critical quality gates - must pass
fail_on_unmet: true

# Optional improvements - nice to have
fail_on_unmet: false
```

### 4. Combine Multiple Checks

Group related criteria together:

```yaml
criteria:
  # Existence checks
  - type: "file_exists"
    path: "src/module.rs"
    description: "Module file exists"

  # Content checks
  - type: "file_contains"
    path: "src/module.rs"
    pattern: "pub struct"
    description: "Has public types"

  # Quality checks
  - type: "file_not_contains"
    path: "src/module.rs"
    pattern: "unwrap()"
    description: "No unsafe unwraps"
```

### 5. Order Criteria Logically

Check prerequisites first:

```yaml
criteria:
  # 1. File must exist first
  - type: "file_exists"
    path: "output.txt"
    description: "Output file created"

  # 2. Then check its contents
  - type: "file_contains"
    path: "output.txt"
    pattern: "SUCCESS"
    description: "Output indicates success"
```

## Integration with Other Features

### With Conditional Execution

```yaml
tasks:
  tests:
    description: "Run tests"
    definition_of_done:
      criteria:
        - type: "tests_passed"
          command: "cargo"
          args: ["test"]
          description: "Tests pass"

  deploy:
    description: "Deploy if tests pass"
    depends_on: ["tests"]
    condition:
      type: "task_status"
      task: "tests"
      status: "completed"  # Only runs if tests completed (DoD met)
```

### With Error Handling

```yaml
tasks:
  risky_operation:
    description: "Perform risky operation"
    on_error:
      retry: 2
      fallback_agent: "backup_agent"
    definition_of_done:
      max_retries: 3
      criteria:
        - type: "file_exists"
          path: "result.txt"
          description: "Result file created"
```

## Troubleshooting

### DoD Keeps Failing

1. **Check criterion is achievable**: Verify the task can actually meet the criteria
2. **Review agent capabilities**: Ensure agent has tools to satisfy requirements
3. **Check file paths**: Verify paths are correct relative to working directory
4. **Increase retries**: Some tasks need more attempts
5. **Review feedback**: Check what the agent is being told

### Criteria Pass But Task Still Retries

- Ensure all criteria return `met: true`
- Check for typos in pattern matching
- Verify file paths are absolute or relative to correct directory

### Want More Detailed Feedback

Modify `format_unmet_criteria()` in executor to include:
- More context from criterion details
- Suggestions for fixing
- Examples of correct output

## Performance Considerations

### Minimize Command Execution

Commands run synchronously and can be slow:

```yaml
# Avoid if possible
criteria:
  - type: "command_succeeds"
    command: "heavy_build_script.sh"  # Slow!

# Better: Check outputs instead
criteria:
  - type: "file_exists"
    path: "build/output.bin"  # Fast!
```

### Use Caching When Possible

For expensive checks, consider:
- Checking file timestamps
- Using file hashes
- Implementing custom criterion types with caching

## Future Enhancements

Potential additions to the DoD system:

1. **Custom Validators**: User-defined criterion types
2. **Async Checking**: Parallel criterion evaluation
3. **Smart Retries**: Exponential backoff for retries
4. **Criterion Dependencies**: Skip checks if prerequisites fail
5. **Metric Collection**: Track DoD success rates and retry counts
6. **AI-Powered Suggestions**: LLM-generated fix suggestions

## See Also

- [Conditional Tasks](conditional-tasks.md) - Combine DoD with conditional execution
- [Error Handling](../README.md#error-handling) - Error recovery and fallbacks
- [Example Workflow](../examples/workflows/definition_of_done.yaml) - Complete example

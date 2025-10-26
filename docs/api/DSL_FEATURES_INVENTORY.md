# DSL Features Inventory

Comprehensive documentation of all features available in the Workflow DSL system.

**Version:** 1.0.0
**Last Updated:** 2025-10-19

---

## Table of Contents

1. [Workflow Root Structure](#workflow-root-structure)
2. [Agents](#agents)
3. [Tasks](#tasks)
4. [Workflows (Orchestration)](#workflows-orchestration)
5. [Tools](#tools)
6. [Permissions](#permissions)
7. [Loops](#loops)
8. [Conditionals](#conditionals)
9. [Definition of Done](#definition-of-done)
10. [Error Handling](#error-handling)
11. [Hooks](#hooks)
12. [Communication Channels](#communication-channels)
13. [MCP Servers](#mcp-servers)
14. [State Management](#state-management)
15. [Safety Limits](#safety-limits)

---

## Workflow Root Structure

Top-level workflow configuration.

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | ✅ | Workflow name |
| `version` | String | ✅ | Workflow version (semver) |
| `dsl_version` | String | ⬜ | DSL grammar version (default: "1.0.0") |
| `description` | String | ⬜ | Optional workflow description |
| `cwd` | String | ⬜ | Working directory for all agents |
| `create_cwd` | Boolean | ⬜ | Create working directory if missing (default: false) |
| `agents` | Map | ⬜ | Agent definitions (keyed by ID) |
| `tasks` | Map | ⬜ | Task definitions (keyed by ID) |
| `workflows` | Map | ⬜ | Multi-stage workflow orchestrations |
| `tools` | Object | ⬜ | Global tool configuration |
| `communication` | Object | ⬜ | Inter-agent communication channels |
| `mcp_servers` | Map | ⬜ | MCP server configurations |

### Example

```yaml
name: "my_workflow"
version: "1.0.0"
dsl_version: "1.0.0"
cwd: "/tmp/workspace"
create_cwd: true
```

---

## Agents

Autonomous actors that execute tasks using AI models.

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `description` | String | ✅ | Agent's role and capabilities |
| `model` | String | ⬜ | AI model (default: "claude-sonnet-4-5") |
| `system_prompt` | String | ⬜ | Custom system prompt override |
| `cwd` | String | ⬜ | Agent-specific working directory |
| `create_cwd` | Boolean | ⬜ | Create agent's working directory |
| `tools` | Array[String] | ⬜ | Allowed tools for this agent |
| `permissions` | Object | ⬜ | Permission configuration |
| `max_turns` | Integer | ⬜ | Maximum conversation turns |

### Available Models

- `claude-sonnet-4-5` (default, balanced)
- `claude-opus-4` (advanced reasoning)

### Example

```yaml
agents:
  researcher:
    description: "Research and gather information"
    model: "claude-sonnet-4-5"
    cwd: "./research"
    create_cwd: true
    tools:
      - Read
      - WebSearch
      - Write
    permissions:
      mode: "acceptEdits"
      allowed_directories: ["./data", "/tmp"]
    max_turns: 15
```

---

## Tasks

Executable work units with dependencies and conditions.

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `description` | String | ✅ | Task description (supports {{variable}} substitution) |
| `agent` | String | ⬜ | Agent ID to execute this task |
| `priority` | Integer | ⬜ | Priority (lower = higher, default: 0) |
| `subtasks` | Array[Map] | ⬜ | Nested child tasks |
| `depends_on` | Array[String] | ⬜ | Task IDs this depends on |
| `parallel_with` | Array[String] | ⬜ | Tasks that can run concurrently |
| `output` | String | ⬜ | Output file path |
| `condition` | Object | ⬜ | Conditional execution logic |
| `definition_of_done` | Object | ⬜ | Completion criteria |
| `loop` | Object | ⬜ | Loop specification |
| `loop_control` | Object | ⬜ | Loop control flow settings |
| `on_complete` | Object | ⬜ | Completion actions |
| `on_error` | Object | ⬜ | Error handling configuration |

### Variable Substitution

Tasks support template variables in descriptions:

- `{{item}}` - Current loop item
- `{{num}}` - Numeric iterator
- `{{iteration}}` - Iteration count (0-based)
- `{{<state_key>}}` - Any workflow state variable

### Example

```yaml
tasks:
  analyze_code:
    description: "Analyze code structure"
    agent: "analyzer"
    priority: 1
    output: "analysis.md"
    depends_on: ["fetch_code"]

  build_project:
    description: "Build the project"
    agent: "builder"
    depends_on: ["analyze_code"]
    condition:
      type: task_status
      task: "analyze_code"
      status: "completed"
```

### Subtasks (Hierarchical Decomposition)

```yaml
tasks:
  research_project:
    description: "Complete research project"
    subtasks:
      - collect_data:
          description: "Collect data"
          agent: "researcher"
      - analyze_data:
          description: "Analyze data"
          agent: "analyst"
          depends_on: ["collect_data"]
```

---

## Workflows (Orchestration)

Multi-stage workflow definitions with dependencies.

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `description` | String | ✅ | Workflow description |
| `steps` | Array[Stage] | ✅ | Workflow stages |
| `hooks` | Object | ⬜ | Lifecycle hooks |

### Stage Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `stage` | String | ✅ | Stage name |
| `agents` | Array[String] | ✅ | Agents involved in this stage |
| `tasks` | Array[Map] | ✅ | Tasks in this stage |
| `depends_on` | Array[String] | ⬜ | Stage dependencies |
| `mode` | Enum | ⬜ | Execution mode (default: "sequential") |

### Execution Modes

- `sequential` - Tasks run one after another
- `parallel` - Tasks run concurrently

### Example

```yaml
workflows:
  ci_cd_pipeline:
    description: "Complete CI/CD pipeline"
    steps:
      - stage: "build"
        agents: ["builder"]
        mode: sequential
        tasks:
          - compile:
              description: "Compile code"
              agent: "builder"

      - stage: "test"
        agents: ["tester"]
        mode: parallel
        depends_on: ["build"]
        tasks:
          - unit_tests:
              description: "Run unit tests"
              agent: "tester"
          - integration_tests:
              description: "Run integration tests"
              agent: "tester"
```

---

## Tools

Available tools and their constraints.

### Valid Tools

| Tool | Description |
|------|-------------|
| `Read` | Read files from filesystem |
| `Write` | Write/create files |
| `Edit` | Edit existing files |
| `Bash` | Execute shell commands |
| `Grep` | Search file contents |
| `Glob` | Find files by pattern |
| `WebSearch` | Search the web |
| `WebFetch` | Fetch web content |
| `Task` | Spawn sub-agents |
| `TodoWrite` | Manage task lists |
| `Skill` | Execute skills |
| `SlashCommand` | Execute slash commands |

### Global Tool Configuration

```yaml
tools:
  allowed: ["Bash", "Read", "Write"]
  disallowed: ["WebSearch"]
  constraints:
    Bash:
      timeout: 300000  # milliseconds
      allowed_commands: ["git", "npm", "cargo", "docker"]
    Write:
      max_file_size: 1048576  # bytes
      allowed_extensions: [".rs", ".yaml", ".md"]
    WebSearch:
      rate_limit: 10  # requests per minute
```

### Tool Constraint Fields

| Tool | Constraint Fields |
|------|------------------|
| `Bash` | `timeout`, `allowed_commands` |
| `Write` | `max_file_size`, `allowed_extensions` |
| `WebSearch` | `rate_limit` |
| All tools | `timeout` |

---

## Permissions

Control agent access and auto-approval behavior.

### Permission Modes

| Mode | Description |
|------|-------------|
| `default` | Prompt for dangerous operations |
| `acceptEdits` | Auto-approve file edits |
| `plan` | Planning mode, no execution |
| `bypassPermissions` | Skip all permission checks (dangerous) |

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `mode` | String | Permission mode (default: "default") |
| `allowed_directories` | Array[String] | Directory allowlist |

### Example

```yaml
permissions:
  mode: "acceptEdits"
  allowed_directories:
    - "./src"
    - "/tmp"
    - "../shared"
```

---

## Loops

Iterative task execution patterns.

### Loop Types

#### 1. ForEach Loop

Iterate over collections.

**Fields:**
- `type: for_each`
- `collection` - Collection source (required)
- `iterator` - Variable name for current item (required)
- `parallel` - Execute in parallel (default: false)
- `max_parallel` - Concurrency limit for parallel execution

**Collection Sources:**

**Inline:**
```yaml
loop:
  type: for_each
  collection:
    source: inline
    items: ["apple", "banana", "cherry"]
  iterator: "item"
```

**Range:**
```yaml
loop:
  type: for_each
  collection:
    source: range
    start: 0
    end: 10
    step: 1  # optional, default: 1
  iterator: "num"
```

**State:**
```yaml
loop:
  type: for_each
  collection:
    source: state
    key: "items_to_process"
  iterator: "item"
```

**File:**
```yaml
loop:
  type: for_each
  collection:
    source: file
    path: "/data/items.json"
    format: json  # json | jsonlines | csv | lines
  iterator: "entry"
```

**Parallel ForEach:**
```yaml
loop:
  type: for_each
  collection:
    source: inline
    items: ["task1", "task2", "task3"]
  iterator: "task"
  parallel: true
  max_parallel: 3  # Process 3 at a time
```

#### 2. While Loop

Execute while condition is true.

**Fields:**
- `type: while`
- `condition` - Condition to check (required)
- `max_iterations` - Safety limit (required)
- `iteration_variable` - Variable name for iteration count
- `delay_between_secs` - Delay between iterations

```yaml
loop:
  type: while
  condition:
    type: state_equals
    key: "processing"
    value: true
  max_iterations: 100
  iteration_variable: "attempt"
  delay_between_secs: 2
```

#### 3. RepeatUntil Loop

Execute until condition becomes true (do-while pattern).

**Fields:**
- `type: repeat_until`
- `condition` - Exit condition (required)
- `max_iterations` - Safety limit (required)
- `min_iterations` - Minimum iterations (default: 1)
- `iteration_variable` - Variable name for iteration count
- `delay_between_secs` - Delay between iterations

```yaml
loop:
  type: repeat_until
  condition:
    type: state_equals
    key: "task_complete"
    value: true
  min_iterations: 1
  max_iterations: 10
  iteration_variable: "attempt"
  delay_between_secs: 3
```

**Use Case:** Polling for task completion with backoff.

#### 4. Repeat Loop

Execute fixed number of times.

**Fields:**
- `type: repeat`
- `count` - Number of iterations (required)
- `iterator` - Variable name for index (0-based)
- `parallel` - Execute in parallel (default: false)
- `max_parallel` - Concurrency limit

```yaml
loop:
  type: repeat
  count: 5
  iterator: "index"
  parallel: false
```

**Parallel Repeat:**
```yaml
loop:
  type: repeat
  count: 10
  iterator: "batch"
  parallel: true
  max_parallel: 4
```

### Loop Control

Advanced loop control flow.

**Fields:**
- `break_condition` - Break out of loop early
- `continue_condition` - Skip current iteration
- `collect_results` - Collect iteration results (default: false)
- `result_key` - State key for collected results

```yaml
loop_control:
  break_condition:
    type: state_equals
    key: "fatal_error"
    value: true
  continue_condition:
    type: state_equals
    key: "skip_item"
    value: true
  collect_results: true
  result_key: "processing_results"
```

---

## Conditionals

Conditional task execution based on state and task status.

### Condition Types

#### 1. TaskStatus

Check if a task has a specific status.

```yaml
condition:
  type: task_status
  task: "build_project"
  status: "completed"  # completed | failed | running | pending | skipped
```

#### 2. StateEquals

Check if state variable equals a value.

```yaml
condition:
  type: state_equals
  key: "environment"
  value: "production"
```

#### 3. StateExists

Check if state variable exists.

```yaml
condition:
  type: state_exists
  key: "deployment_config"
```

#### 4. Always

Always execute (useful for testing).

```yaml
condition:
  type: always
```

#### 5. Never

Never execute (disable task).

```yaml
condition:
  type: never
```

### Logical Operators

#### AND

All conditions must be true.

```yaml
condition:
  and:
    - type: task_status
      task: "tests"
      status: "completed"
    - type: state_equals
      key: "environment"
      value: "production"
```

#### OR

At least one condition must be true.

```yaml
condition:
  or:
    - type: task_status
      task: "build"
      status: "failed"
    - type: task_status
      task: "tests"
      status: "failed"
```

#### NOT

Negate a condition.

```yaml
condition:
  not:
    type: state_equals
    key: "environment"
    value: "production"
```

### Complex Nested Conditions

```yaml
condition:
  and:
    - or:
        - type: task_status
          task: "deploy"
          status: "failed"
        - type: task_status
          task: "smoke_tests"
          status: "failed"
    - type: state_equals
      key: "environment"
      value: "production"
```

---

## Definition of Done

Quality gates with automatic verification and retries.

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `criteria` | Array[Criterion] | ✅ | List of criteria to check |
| `max_retries` | Integer | ⬜ | Max retry attempts (default: 3) |
| `fail_on_unmet` | Boolean | ⬜ | Fail task if criteria unmet (default: true) |

### Criterion Types

#### 1. FileExists

Check if a file exists.

```yaml
criteria:
  - type: file_exists
    path: "src/auth.rs"
    description: "Authentication module must exist"
```

#### 2. FileContains

Check if file contains a pattern.

```yaml
criteria:
  - type: file_contains
    path: "src/auth.rs"
    pattern: "pub fn authenticate"
    description: "Must contain public authenticate function"
```

#### 3. FileNotContains

Check if file does NOT contain a pattern.

```yaml
criteria:
  - type: file_not_contains
    path: "src/auth.rs"
    pattern: "TODO"
    description: "No TODO comments should remain"
```

#### 4. DirectoryExists

Check if directory exists.

```yaml
criteria:
  - type: directory_exists
    path: "target/release"
    description: "Release directory must exist"
```

#### 5. CommandSucceeds

Check if command exits successfully.

```yaml
criteria:
  - type: command_succeeds
    command: "cargo"
    args: ["build", "--release"]
    description: "Release build must succeed"
    working_dir: "/project"  # optional
```

#### 6. TestsPassed

Run test command and verify success.

```yaml
criteria:
  - type: tests_passed
    command: "cargo"
    args: ["test", "--lib"]
    description: "All tests must pass"
```

#### 7. OutputMatches

Check if output matches a pattern.

**From File:**
```yaml
criteria:
  - type: output_matches
    source:
      file:
        path: "output.log"
    pattern: "Build successful"
    description: "Output must indicate success"
```

**From Task Output:**
```yaml
criteria:
  - type: output_matches
    source: task_output
    pattern: "All checks passed"
    description: "Task output must confirm success"
```

### Example

```yaml
definition_of_done:
  max_retries: 3
  fail_on_unmet: true
  criteria:
    - type: file_exists
      path: "target/release/myapp"
      description: "Binary must be created"
    - type: tests_passed
      command: "cargo"
      args: ["test"]
      description: "All tests must pass"
    - type: file_not_contains
      path: "src/**/*.rs"
      pattern: "TODO|FIXME"
      description: "No TODO markers allowed"
```

---

## Error Handling

Retry logic and fallback strategies.

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `retry` | Integer | Number of retry attempts (default: 0) |
| `retry_delay_secs` | Integer | Delay between retries in seconds (default: 1) |
| `exponential_backoff` | Boolean | Use exponential backoff (default: false) |
| `fallback_agent` | String | Agent ID to use as fallback |

### Example

```yaml
on_error:
  retry: 3
  retry_delay_secs: 10
  exponential_backoff: true
  fallback_agent: "backup_agent"
```

**Retry Pattern with Exponential Backoff:**
- Attempt 1: Immediate
- Attempt 2: 10s delay
- Attempt 3: 20s delay
- Attempt 4: 40s delay

### Fallback Agent Pattern

```yaml
tasks:
  analyze_data:
    agent: "primary_analyst"
    on_error:
      retry: 2
      fallback_agent: "backup_analyst"
```

---

## Hooks

Lifecycle event handlers.

### Workflow Hooks

Execute commands at workflow lifecycle events.

**Available Hooks:**
- `pre_workflow` - Before workflow starts
- `post_workflow` - After workflow completes
- `on_stage_complete` - After each stage completes
- `on_error` - When an error occurs

### Hook Formats

**Simple String:**
```yaml
hooks:
  pre_workflow:
    - "echo 'Starting workflow'"
```

**Command with Metadata:**
```yaml
hooks:
  pre_workflow:
    - command: "mkdir -p /tmp/output"
      description: "Create output directory"
```

### Environment Variables

Hooks have access to special variables:
- `$WORKFLOW_STAGE` - Current stage name
- `$WORKFLOW_ERROR` - Error message (in on_error)

### Example

```yaml
workflows:
  my_workflow:
    hooks:
      pre_workflow:
        - "echo 'Starting workflow'"
        - command: "mkdir -p /tmp/data"
          description: "Setup directories"

      post_workflow:
        - "echo 'Workflow complete'"

      on_stage_complete:
        - "echo 'Completed: $WORKFLOW_STAGE'"

      on_error:
        - "echo 'Error in $WORKFLOW_STAGE: $WORKFLOW_ERROR'"
```

### Task Hooks

**On Complete:**
```yaml
on_complete:
  notify: "Task completed successfully"
```

---

## Communication Channels

Inter-agent messaging for collaboration.

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `description` | String | Channel purpose |
| `participants` | Array[String] | Agent IDs that can use this channel |
| `message_format` | String | Message format (e.g., "markdown", "json") |

### Example

```yaml
communication:
  channels:
    research_findings:
      description: "Share research findings and insights"
      participants:
        - data_researcher
        - ml_specialist
        - report_writer
      message_format: "markdown"

    ml_insights:
      description: "ML analysis results"
      participants:
        - ml_specialist
        - report_writer
      message_format: "json"
```

### Message Type Schemas

Define structured message types with JSON schemas:

```yaml
communication:
  message_types:
    analysis_result:
      schema:
        type: "object"
        properties:
          confidence:
            type: "number"
          findings:
            type: "array"
```

---

## MCP Servers

Model Context Protocol server integrations.

### Server Types

- `stdio` - Standard I/O communication
- `http` - HTTP-based communication

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `type` | String | Server type ("stdio" or "http") |
| `command` | String | Command to run (stdio only) |
| `args` | Array[String] | Command arguments (stdio) |
| `env` | Map[String, String] | Environment variables |
| `url` | String | Server URL (http only) |
| `headers` | Map[String, String] | HTTP headers (http only) |

### STDIO Example

```yaml
mcp_servers:
  database_server:
    type: "stdio"
    command: "npx"
    args: ["-y", "@modelcontextprotocol/server-postgres"]
    env:
      POSTGRES_URL: "postgresql://localhost/mydb"
```

### HTTP Example

```yaml
mcp_servers:
  api_server:
    type: "http"
    url: "https://api.example.com/mcp"
    headers:
      Authorization: "Bearer ${API_TOKEN}"
      Content-Type: "application/json"
```

---

## State Management

Workflow state persistence and variable access.

### State Variables

Tasks can read and write workflow state:

```yaml
# Set state
state:
  environment: "production"
  deployment_id: "abc-123"

# Read state in conditions
condition:
  type: state_equals
  key: "environment"
  value: "production"

# Use in loop collections
loop:
  type: for_each
  collection:
    source: state
    key: "items_to_process"
  iterator: "item"
```

### Loop Results

Collect results from loop iterations:

```yaml
loop_control:
  collect_results: true
  result_key: "processing_results"
```

Access in Rust:
```rust
if let Some(state) = executor.get_state() {
    let results = state.get_loop_results("task_id");
}
```

### Task Outputs

Tasks can write to files:

```yaml
tasks:
  analyze:
    output: "analysis.md"
```

### State Persistence

State includes:
- Task statuses (pending, running, completed, failed, skipped)
- Task outputs
- Loop states (current iteration, results)
- Custom state variables
- Workflow status

---

## Safety Limits

Hard limits enforced by the validator.

### Loop Safety

| Constant | Value | Description |
|----------|-------|-------------|
| `MAX_LOOP_ITERATIONS` | 10,000 | Maximum iterations for while/repeat_until/repeat loops |
| `MAX_COLLECTION_SIZE` | 100,000 | Maximum items in a collection |
| `MAX_PARALLEL_ITERATIONS` | 100 | Maximum parallel task limit |
| `MAX_NESTED_DEPTH` | 5 | Maximum loop nesting depth |

### Validation Checks

The validator enforces:
- ✅ Agent references exist
- ✅ Task dependencies exist
- ✅ No circular dependencies
- ✅ Valid tool names
- ✅ Valid permission modes
- ✅ Loop iteration limits
- ✅ Collection size limits
- ✅ Parallel execution limits
- ✅ Range validity (start < end, step > 0)

### Error Messages

```
Task 'my_task': max_iterations (20000) exceeds safety limit (10000)
Task 'my_task': range would generate 200000 items, exceeding safety limit (100000)
Task 'my_task': max_parallel (150) exceeds safety limit (100)
```

---

## Feature Matrix

### By Category

| Category | Features |
|----------|----------|
| **Agents** | Model selection, tool filtering, permissions, turn limits, working directories |
| **Tasks** | Dependencies, priorities, subtasks, outputs, variable substitution |
| **Orchestration** | Multi-stage workflows, sequential/parallel execution, stage dependencies |
| **Loops** | ForEach (4 sources), While, RepeatUntil, Repeat, parallel execution |
| **Conditionals** | Task status, state checks, logical operators (AND/OR/NOT) |
| **Quality Gates** | 7 criterion types, automatic retries, flexible failure handling |
| **Error Handling** | Retries, exponential backoff, fallback agents |
| **Communication** | Channels, message types, JSON schemas |
| **Hooks** | 4 workflow hooks, task completion actions |
| **Tools** | 12 built-in tools, constraints, allowlists |
| **Permissions** | 4 modes, directory scoping |
| **State** | Persistence, variable access, loop results |
| **Safety** | Validation, limits, cycle detection |

---

## Quick Reference

### Minimal Workflow

```yaml
name: "minimal"
version: "1.0.0"

agents:
  worker:
    description: "Does work"
    tools: [Read, Write]

tasks:
  do_work:
    description: "Do the work"
    agent: "worker"
```

### Common Patterns

**Sequential Pipeline:**
```yaml
tasks:
  task1:
    agent: "agent1"
  task2:
    agent: "agent2"
    depends_on: ["task1"]
  task3:
    agent: "agent3"
    depends_on: ["task2"]
```

**Parallel Fan-Out:**
```yaml
tasks:
  task1:
    parallel_with: ["task2", "task3"]
  task2:
    parallel_with: ["task1", "task3"]
  task3:
    parallel_with: ["task1", "task2"]
```

**Conditional Deployment:**
```yaml
tasks:
  deploy_prod:
    condition:
      and:
        - type: task_status
          task: "tests"
          status: "completed"
        - type: state_equals
          key: "environment"
          value: "production"
```

**Batch Processing with Quality Gates:**
```yaml
tasks:
  process_batch:
    loop:
      type: for_each
      collection:
        source: file
        path: "data.json"
        format: json
      iterator: "item"
      parallel: true
      max_parallel: 5
    definition_of_done:
      criteria:
        - type: file_exists
          path: "output.csv"
          description: "Output must be generated"
```

---

## Version History

- **1.0.0** (2025-10-19) - Initial comprehensive inventory

---

## See Also

- [CLAUDE.md](CLAUDE.md) - Development guide
- [examples/](examples/) - Example workflows
- [examples/workflows/](examples/workflows/) - Loop and conditional examples
- [src/dsl/schema.rs](src/dsl/schema.rs) - Type definitions
- [src/dsl/validator.rs](src/dsl/validator.rs) - Validation logic

# Task Groups Tutorial

Step-by-step guide to mastering task groups.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Your First Task Group](#your-first-task-group)
3. [Sequential vs Parallel](#sequential-vs-parallel)
4. [Adding Dependencies](#adding-dependencies)
5. [Working with Variables](#working-with-variables)
6. [Hierarchical Groups](#hierarchical-groups)
7. [Error Handling](#error-handling)
8. [Conditional Execution](#conditional-execution)
9. [Real-World Example](#real-world-example)
10. [Next Steps](#next-steps)

## Getting Started

### Prerequisites

- Rust installed (1.70+)
- Basic understanding of YAML
- Agent SDK set up

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/periplon
cd periplon

# Build the DSL executor
cargo build --release --bin periplon-executor
```

### Verify Installation

```bash
./target/release/periplon-executor --version
```

## Your First Task Group

Let's create a simple workflow with a task group.

### Step 1: Create Workflow File

Create `my-first-workflow.yaml`:

```yaml
name: "My First Workflow"
version: "1.0.0"
description: "A simple workflow to learn task groups"

agents:
  worker:
    description: "A general-purpose worker agent"
    tools: [Read, Write, Bash]
    permissions:
      mode: "default"

task_groups:
  my_first_group:
    description: "My first task group"
    execution_mode: "sequential"
    tasks:
      - hello
      - world

tasks:
  hello:
    description: "Print hello"
    agent: "worker"
    group: "my_first_group"

  world:
    description: "Print world"
    agent: "worker"
    group: "my_first_group"
```

### Step 2: Validate

```bash
./target/release/periplon-executor validate my-first-workflow.yaml
```

**Expected Output:**
```
✓ Workflow is valid
```

### Step 3: Run

```bash
./target/release/periplon-executor run my-first-workflow.yaml
```

**What Happened:**
1. Group `my_first_group` started
2. Task `hello` executed
3. Task `world` executed
4. Group completed

🎉 Congratulations! You've created your first task group!

## Sequential vs Parallel

Let's explore the difference between execution modes.

### Sequential Example

Tasks run one after another:

```yaml
task_groups:
  sequential_example:
    description: "Sequential task execution"
    execution_mode: "sequential"
    tasks:
      - step1
      - step2
      - step3
```

**Timeline:**
```
0s    2s    4s    6s
│─────│─────│─────│
step1 step2 step3
```

### Parallel Example

Tasks run concurrently:

```yaml
task_groups:
  parallel_example:
    description: "Parallel task execution"
    execution_mode: "parallel"
    tasks:
      - test1
      - test2
      - test3
```

**Timeline:**
```
0s         2s
│──────────│
test1
test2
test3
```

### Exercise: Compare Performance

Create `compare-modes.yaml`:

```yaml
name: "Compare Execution Modes"
version: "1.0.0"

agents:
  worker:
    description: "Worker agent"
    tools: [Bash]
    permissions:
      mode: "acceptEdits"

task_groups:
  sequential_group:
    description: "Sequential execution"
    execution_mode: "sequential"
    tasks:
      - seq_task1
      - seq_task2
      - seq_task3

  parallel_group:
    description: "Parallel execution"
    execution_mode: "parallel"
    tasks:
      - par_task1
      - par_task2
      - par_task3

tasks:
  seq_task1:
    description: "Sleep 2 seconds"
    agent: "worker"
    group: "sequential_group"

  seq_task2:
    description: "Sleep 2 seconds"
    agent: "worker"
    group: "sequential_group"

  seq_task3:
    description: "Sleep 2 seconds"
    agent: "worker"
    group: "sequential_group"

  par_task1:
    description: "Sleep 2 seconds"
    agent: "worker"
    group: "parallel_group"

  par_task2:
    description: "Sleep 2 seconds"
    agent: "worker"
    group: "parallel_group"

  par_task3:
    description: "Sleep 2 seconds"
    agent: "worker"
    group: "parallel_group"
```

**Run and time:**
```bash
time ./target/release/periplon-executor run compare-modes.yaml
```

**Expected:**
- Sequential: ~6 seconds (2s × 3)
- Parallel: ~2 seconds (all at once)

## Adding Dependencies

Groups can depend on other groups.

### Basic Dependency

```yaml
task_groups:
  prepare:
    description: "Prepare environment"
    execution_mode: "sequential"
    tasks:
      - setup_env
      - fetch_data

  process:
    description: "Process data"
    execution_mode: "parallel"
    depends_on: [prepare]  # Waits for prepare
    tasks:
      - process_data
      - analyze_data
```

**Execution Flow:**
```
prepare (sequential)
  ├── setup_env
  └── fetch_data
         ↓
process (parallel)
  ├── process_data
  └── analyze_data
```

### Multiple Dependencies

```yaml
task_groups:
  fetch:
    tasks: [fetch_data]

  validate:
    tasks: [validate_data]

  process:
    depends_on: [fetch, validate]  # Both must complete
    tasks: [process_data]
```

**Execution Flow:**
```
fetch ──┐
        ├─→ process
validate─┘
```

### Exercise: Build Pipeline

Create `build-pipeline.yaml`:

```yaml
name: "Build Pipeline"
version: "1.0.0"

agents:
  builder:
    description: "Build agent"
    tools: [Bash, Read, Write]
    permissions:
      mode: "acceptEdits"

task_groups:
  checkout:
    description: "Checkout code"
    execution_mode: "sequential"
    tasks:
      - git_clone

  build:
    description: "Build application"
    execution_mode: "sequential"
    depends_on: [checkout]
    tasks:
      - compile

  test:
    description: "Run tests"
    execution_mode: "parallel"
    depends_on: [build]
    tasks:
      - unit_tests
      - integration_tests

tasks:
  git_clone:
    description: "Clone repository"
    agent: "builder"
    group: "checkout"

  compile:
    description: "Compile code"
    agent: "builder"
    group: "build"

  unit_tests:
    description: "Run unit tests"
    agent: "builder"
    group: "test"

  integration_tests:
    description: "Run integration tests"
    agent: "builder"
    group: "test"
```

**Run:**
```bash
./target/release/periplon-executor run build-pipeline.yaml
```

## Working with Variables

Groups can pass data through variables.

### Group Outputs

```yaml
task_groups:
  analysis:
    description: "Analyze code"
    execution_mode: "sequential"
    tasks:
      - analyze

    outputs:
      score:
        source:
          type: state
          key: "analysis.score"
```

### Group Inputs

```yaml
task_groups:
  reporting:
    description: "Generate report"
    execution_mode: "sequential"
    depends_on: [analysis]
    tasks:
      - report

    inputs:
      analysis_score: "${group.analysis.score}"
```

### Complete Example

Create `variables-example.yaml`:

```yaml
name: "Variables Example"
version: "1.0.0"

inputs:
  project_name:
    type: string
    required: true

agents:
  analyzer:
    description: "Analyzer agent"
    tools: [Read, Grep, Glob]
    permissions:
      mode: "default"

task_groups:
  scan:
    description: "Scan ${workflow.project_name}"
    execution_mode: "sequential"
    tasks:
      - count_files

    outputs:
      file_count:
        source:
          type: state
          key: "scan.file_count"

  report:
    description: "Generate report"
    execution_mode: "sequential"
    depends_on: [scan]
    tasks:
      - create_report

    inputs:
      files: "${group.scan.file_count}"

tasks:
  count_files:
    description: "Count files in ${workflow.project_name}"
    agent: "analyzer"
    group: "scan"

  create_report:
    description: "Report on ${group.scan.file_count} files"
    agent: "analyzer"
    group: "report"
```

**Run:**
```bash
./target/release/periplon-executor run variables-example.yaml \
  --input project_name=my-project
```

## Hierarchical Groups

Groups can contain other groups.

### Basic Hierarchy

```yaml
task_groups:
  # Parent group
  deployment:
    description: "Complete deployment"
    groups:
      - build_stage
      - deploy_stage

  # Child groups
  build_stage:
    parent: "deployment"
    tasks: [build]

  deploy_stage:
    parent: "deployment"
    depends_on: [build_stage]
    tasks: [deploy]
```

**Structure:**
```
deployment
├── build_stage
│   └── build
└── deploy_stage
    └── deploy
```

### Multi-Level Hierarchy

Create `hierarchy-example.yaml`:

```yaml
name: "Hierarchical Groups"
version: "1.0.0"

agents:
  worker:
    description: "Worker agent"
    tools: [Bash, Read, Write]
    permissions:
      mode: "acceptEdits"

task_groups:
  # Level 0: Root
  ci_cd:
    description: "CI/CD Pipeline"
    execution_mode: "sequential"
    groups:
      - build_stage
      - test_stage

  # Level 1: Stages
  build_stage:
    description: "Build Stage"
    parent: "ci_cd"
    execution_mode: "sequential"
    groups:
      - compile
      - package

  test_stage:
    description: "Test Stage"
    parent: "ci_cd"
    execution_mode: "parallel"
    depends_on: [build_stage]
    groups:
      - unit_tests
      - integration_tests

  # Level 2: Sub-stages
  compile:
    description: "Compilation"
    parent: "build_stage"
    execution_mode: "parallel"
    tasks:
      - compile_backend
      - compile_frontend

  package:
    description: "Packaging"
    parent: "build_stage"
    execution_mode: "sequential"
    depends_on: [compile]
    tasks:
      - create_package

  unit_tests:
    description: "Unit Tests"
    parent: "test_stage"
    execution_mode: "parallel"
    tasks:
      - test_backend
      - test_frontend

  integration_tests:
    description: "Integration Tests"
    parent: "test_stage"
    execution_mode: "sequential"
    tasks:
      - test_api

tasks:
  compile_backend:
    description: "Compile backend"
    agent: "worker"
    group: "compile"

  compile_frontend:
    description: "Compile frontend"
    agent: "worker"
    group: "compile"

  create_package:
    description: "Create deployment package"
    agent: "worker"
    group: "package"

  test_backend:
    description: "Test backend"
    agent: "worker"
    group: "unit_tests"

  test_frontend:
    description: "Test frontend"
    agent: "worker"
    group: "unit_tests"

  test_api:
    description: "Test API"
    agent: "worker"
    group: "integration_tests"
```

**Structure:**
```
ci_cd
├── build_stage
│   ├── compile
│   │   ├── compile_backend
│   │   └── compile_frontend
│   └── package
│       └── create_package
└── test_stage
    ├── unit_tests
    │   ├── test_backend
    │   └── test_frontend
    └── integration_tests
        └── test_api
```

**Run:**
```bash
./target/release/periplon-executor run hierarchy-example.yaml
```

## Error Handling

Control how groups handle failures.

### Stop on Error

Default behavior - stops immediately:

```yaml
task_groups:
  critical_tasks:
    on_error: "stop"
    tasks:
      - task1
      - task2
      - task3
```

**Behavior:** If task2 fails, task3 is cancelled

### Continue on Error

Collects all failures:

```yaml
task_groups:
  test_suite:
    on_error: "continue"
    tasks:
      - test1
      - test2
      - test3
```

**Behavior:** All tests run, even if some fail

### Exercise: Error Handling

Create `error-handling.yaml`:

```yaml
name: "Error Handling Demo"
version: "1.0.0"

agents:
  worker:
    description: "Worker"
    tools: [Bash]
    permissions:
      mode: "acceptEdits"

task_groups:
  stop_on_error:
    description: "Stop on first error"
    execution_mode: "sequential"
    on_error: "stop"
    tasks:
      - task1
      - failing_task
      - task3

  continue_on_error:
    description: "Continue despite errors"
    execution_mode: "sequential"
    on_error: "continue"
    tasks:
      - task4
      - failing_task2
      - task5

tasks:
  task1:
    description: "Success"
    agent: "worker"
    group: "stop_on_error"

  failing_task:
    description: "This will fail"
    agent: "worker"
    group: "stop_on_error"

  task3:
    description: "Won't execute"
    agent: "worker"
    group: "stop_on_error"

  task4:
    description: "Success"
    agent: "worker"
    group: "continue_on_error"

  failing_task2:
    description: "This will fail"
    agent: "worker"
    group: "continue_on_error"

  task5:
    description: "Will execute anyway"
    agent: "worker"
    group: "continue_on_error"
```

## Conditional Execution

Groups can execute conditionally.

### Basic Condition

```yaml
task_groups:
  quality_check:
    tasks: [analyze]
    outputs:
      score:
        source:
          type: state
          key: "quality.score"

  deployment:
    depends_on: [quality_check]
    condition: "${group.quality_check.score} >= 0.8"
    tasks: [deploy]
```

**Behavior:** `deployment` only runs if score ≥ 0.8

### Exercise: Quality Gate

Create `quality-gate.yaml`:

```yaml
name: "Quality Gate"
version: "1.0.0"

inputs:
  min_coverage:
    type: number
    required: false
    default: 0.8

agents:
  tester:
    description: "Test agent"
    tools: [Bash, Read]
    permissions:
      mode: "default"

task_groups:
  run_tests:
    description: "Run test suite"
    execution_mode: "parallel"
    tasks:
      - unit_tests
      - integration_tests

    outputs:
      coverage:
        source:
          type: state
          key: "test.coverage"

  deploy:
    description: "Deploy to production"
    depends_on: [run_tests]
    condition: "${group.run_tests.coverage} >= ${workflow.min_coverage}"
    tasks:
      - deploy_prod

tasks:
  unit_tests:
    description: "Run unit tests"
    agent: "tester"
    group: "run_tests"

  integration_tests:
    description: "Run integration tests"
    agent: "tester"
    group: "run_tests"

  deploy_prod:
    description: "Deploy to production"
    agent: "tester"
    group: "deploy"
```

**Run with different thresholds:**
```bash
# Strict quality gate
./target/release/periplon-executor run quality-gate.yaml \
  --input min_coverage=0.95

# Relaxed quality gate
./target/release/periplon-executor run quality-gate.yaml \
  --input min_coverage=0.7
```

## Real-World Example

Let's build a complete microservice deployment pipeline.

Create `microservice-deploy.yaml`:

```yaml
name: "Microservice Deployment Pipeline"
version: "1.0.0"
description: "Complete CI/CD pipeline for microservice deployment"

inputs:
  service_name:
    type: string
    required: true
    description: "Name of the microservice"

  environment:
    type: string
    required: true
    description: "Target environment (dev, staging, prod)"

  repo_url:
    type: string
    required: true
    description: "Git repository URL"

  min_test_coverage:
    type: number
    required: false
    default: 0.8
    description: "Minimum test coverage required"

agents:
  ci_agent:
    description: "CI/CD automation agent"
    tools: [Bash, Read, Write, Grep]
    permissions:
      mode: "acceptEdits"

  test_agent:
    description: "Testing agent"
    tools: [Bash, Read]
    permissions:
      mode: "default"

  deploy_agent:
    description: "Deployment agent"
    tools: [Bash, Read, Write]
    permissions:
      mode: "acceptEdits"

task_groups:
  # Root pipeline
  pipeline:
    description: "Complete deployment pipeline for ${workflow.service_name}"
    execution_mode: "sequential"
    groups:
      - source_preparation
      - build_and_test
      - deployment

    timeout: 1800  # 30 minutes

  # Stage 1: Source preparation
  source_preparation:
    description: "Prepare source code"
    parent: "pipeline"
    execution_mode: "sequential"
    tasks:
      - checkout_code
      - fetch_dependencies

    timeout: 300

    outputs:
      commit_hash:
        source:
          type: state
          key: "git.commit_hash"

  # Stage 2: Build and test
  build_and_test:
    description: "Build and test ${workflow.service_name}"
    parent: "pipeline"
    execution_mode: "sequential"
    groups:
      - build
      - test

    depends_on: [source_preparation]
    timeout: 900

  # Build phase
  build:
    description: "Build application"
    parent: "build_and_test"
    execution_mode: "parallel"
    tasks:
      - compile_code
      - build_docker_image

    max_concurrency: 2

    outputs:
      docker_image:
        source:
          type: state
          key: "docker.image_tag"

  # Test phase
  test:
    description: "Run test suite"
    parent: "build_and_test"
    execution_mode: "parallel"
    depends_on: [build]
    tasks:
      - run_unit_tests
      - run_integration_tests
      - run_security_scan

    on_error: "continue"  # Collect all test results
    max_concurrency: 3

    outputs:
      coverage:
        source:
          type: state
          key: "test.coverage"
      tests_passed:
        source:
          type: state
          key: "test.passed"

  # Stage 3: Deployment
  deployment:
    description: "Deploy to ${workflow.environment}"
    parent: "pipeline"
    execution_mode: "sequential"
    depends_on: [build_and_test]

    # Quality gate: only deploy if tests passed and coverage is good
    condition: |
      ${group.test.tests_passed} == true &&
      ${group.test.coverage} >= ${workflow.min_test_coverage}

    tasks:
      - backup_current
      - deploy_new_version
      - verify_deployment
      - update_monitoring

    timeout: 600
    on_error: "rollback"

    inputs:
      image: "${group.build.docker_image}"

    outputs:
      deployment_url:
        source:
          type: state
          key: "deployment.url"

tasks:
  # Source preparation tasks
  checkout_code:
    description: "Checkout code from ${workflow.repo_url}"
    agent: "ci_agent"
    group: "source_preparation"

  fetch_dependencies:
    description: "Fetch dependencies"
    agent: "ci_agent"
    group: "source_preparation"
    depends_on: [checkout_code]

  # Build tasks
  compile_code:
    description: "Compile ${workflow.service_name}"
    agent: "ci_agent"
    group: "build"

  build_docker_image:
    description: "Build Docker image for ${workflow.service_name}"
    agent: "ci_agent"
    group: "build"
    depends_on: [compile_code]

  # Test tasks
  run_unit_tests:
    description: "Run unit tests"
    agent: "test_agent"
    group: "test"

  run_integration_tests:
    description: "Run integration tests"
    agent: "test_agent"
    group: "test"

  run_security_scan:
    description: "Run security scan"
    agent: "test_agent"
    group: "test"

  # Deployment tasks
  backup_current:
    description: "Backup current ${workflow.environment} deployment"
    agent: "deploy_agent"
    group: "deployment"

  deploy_new_version:
    description: "Deploy new version to ${workflow.environment}"
    agent: "deploy_agent"
    group: "deployment"
    depends_on: [backup_current]

  verify_deployment:
    description: "Verify deployment"
    agent: "deploy_agent"
    group: "deployment"
    depends_on: [deploy_new_version]

  update_monitoring:
    description: "Update monitoring dashboards"
    agent: "deploy_agent"
    group: "deployment"
    depends_on: [verify_deployment]
```

**Run:**
```bash
./target/release/periplon-executor run microservice-deploy.yaml \
  --input service_name=user-api \
  --input environment=staging \
  --input repo_url=https://github.com/myorg/user-api \
  --input min_test_coverage=0.85
```

**Pipeline Flow:**
```
pipeline
├── source_preparation (sequential)
│   ├── checkout_code
│   └── fetch_dependencies
│
├── build_and_test (sequential)
│   ├── build (parallel)
│   │   ├── compile_code
│   │   └── build_docker_image
│   └── test (parallel, continue on error)
│       ├── run_unit_tests
│       ├── run_integration_tests
│       └── run_security_scan
│
└── deployment (sequential, conditional, rollback on error)
    ├── backup_current
    ├── deploy_new_version
    ├── verify_deployment
    └── update_monitoring
```

## Next Steps

### Practice Exercises

1. **Modify the Pipeline**: Add a performance testing phase
2. **Add More Environments**: Support dev, staging, and prod
3. **Implement Canary Deployment**: Deploy to subset of servers first
4. **Add Notifications**: Send notifications on success/failure
5. **Create Rollback Group**: Implement automatic rollback

### Advanced Topics

- [Hierarchical Groups](./README.md#hierarchical-groups) - Deep nesting patterns
- [Variable System](./README.md#variable-system) - Advanced variable usage
- [Error Handling](./README.md#error-handling) - Complex error strategies
- [Architecture](./architecture.md) - Internal implementation

### Resources

- [API Reference](./api-reference.md) - Complete schema documentation
- [Examples](../../examples/task-groups/) - More working examples
- [Main DSL Docs](../../README.md) - Overall DSL documentation

### Getting Help

- GitHub Issues: Report bugs or request features
- Discussions: Ask questions and share workflows
- Examples: Learn from community workflows

---

**Congratulations!** You've completed the task groups tutorial. You now know how to:

✓ Create basic task groups
✓ Use sequential and parallel execution
✓ Add dependencies between groups
✓ Pass data with variables
✓ Build hierarchical structures
✓ Handle errors appropriately
✓ Use conditional execution
✓ Build real-world pipelines

Keep experimenting and building amazing workflows! 🚀

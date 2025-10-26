# Task Groups Guide

Complete guide to organizing and orchestrating tasks using task groups.

## Table of Contents

1. [Introduction](#introduction)
2. [Basic Concepts](#basic-concepts)
3. [Hierarchical Organization](#hierarchical-organization)
4. [Shared Configuration](#shared-configuration)
5. [Execution Modes](#execution-modes)
6. [Dependencies](#dependencies)
7. [Variables and Data Flow](#variables-and-data-flow)
8. [Error Handling](#error-handling)
9. [Examples](#examples)
10. [Best Practices](#best-practices)

## Introduction

Task groups are a powerful organizational feature that allows you to:

- **Group related tasks** together into logical units
- **Control execution flow** with sequential, parallel, or auto modes
- **Manage dependencies** between groups of tasks
- **Share configuration** across multiple tasks
- **Create hierarchies** by nesting groups within groups
- **Pass data** between groups using variables
- **Handle errors** consistently across related tasks

### Why Use Task Groups?

Without task groups, complex workflows can become difficult to manage:

```yaml
# Without groups - flat structure, hard to understand
tasks:
  - checkout_code
  - fetch_deps
  - compile_backend
  - compile_frontend
  - test_unit
  - test_integration
  - deploy_staging
  - verify_deploy
```

With task groups, the structure becomes clear:

```yaml
# With groups - organized, clear intent
task_groups:
  preparation:
    tasks: [checkout_code, fetch_deps]

  build:
    depends_on: [preparation]
    tasks: [compile_backend, compile_frontend]

  test:
    depends_on: [build]
    tasks: [test_unit, test_integration]

  deploy:
    depends_on: [test]
    tasks: [deploy_staging, verify_deploy]
```

## Basic Concepts

### What is a Task Group?

A task group is a collection of tasks that:
- Execute together as a coordinated unit
- Share common configuration (timeouts, error handling)
- Can run sequentially or in parallel
- Can depend on other groups

### Minimal Example

```yaml
task_groups:
  my_group:
    description: "What this group does"
    execution_mode: "sequential"
    tasks:
      - task1
      - task2
```

### Complete Example

```yaml
task_groups:
  build_group:
    description: "Build the application"
    execution_mode: "parallel"
    tasks:
      - compile_backend
      - compile_frontend

    # Optional configuration
    depends_on: [preparation]
    on_error: "stop"
    timeout: 600
    max_concurrency: 2

    # Variables
    inputs:
      source_dir: "${workflow.project_path}"
    outputs:
      build_artifact:
        source:
          type: state
          key: "build.artifact_path"
```

## Hierarchical Organization

Task groups can be nested to create multi-level hierarchies, making complex workflows easier to understand and manage.

### Parent-Child Relationships

Groups can contain other groups:

```yaml
task_groups:
  # Parent group
  ci_cd_pipeline:
    description: "Complete CI/CD pipeline"
    groups:
      - build_stage
      - test_stage
      - deploy_stage

  # Child groups
  build_stage:
    parent: "ci_cd_pipeline"
    tasks: [compile, package]

  test_stage:
    parent: "ci_cd_pipeline"
    depends_on: [build_stage]
    tasks: [unit_tests, integration_tests]

  deploy_stage:
    parent: "ci_cd_pipeline"
    depends_on: [test_stage]
    tasks: [deploy, verify]
```

**Structure:**
```
ci_cd_pipeline
├── build_stage
│   ├── compile
│   └── package
├── test_stage
│   ├── unit_tests
│   └── integration_tests
└── deploy_stage
    ├── deploy
    └── verify
```

### Multi-Level Nesting

You can nest groups multiple levels deep:

```yaml
task_groups:
  # Level 0: Root
  deployment:
    groups: [infrastructure, application]

  # Level 1: Major phases
  infrastructure:
    parent: "deployment"
    groups: [database, cache, compute]

  application:
    parent: "deployment"
    depends_on: [infrastructure]
    groups: [backend, frontend]

  # Level 2: Specific components
  database:
    parent: "infrastructure"
    tasks: [setup_db, run_migrations]

  cache:
    parent: "infrastructure"
    tasks: [setup_redis, configure_cache]

  compute:
    parent: "infrastructure"
    depends_on: [database, cache]
    tasks: [provision_servers, configure_lb]

  backend:
    parent: "application"
    tasks: [deploy_api, start_services]

  frontend:
    parent: "application"
    tasks: [deploy_ui, configure_cdn]
```

**Structure:**
```
deployment
├── infrastructure
│   ├── database
│   │   ├── setup_db
│   │   └── run_migrations
│   ├── cache
│   │   ├── setup_redis
│   │   └── configure_cache
│   └── compute
│       ├── provision_servers
│       └── configure_lb
└── application
    ├── backend
    │   ├── deploy_api
    │   └── start_services
    └── frontend
        ├── deploy_ui
        └── configure_cdn
```

### Benefits of Hierarchical Organization

1. **Clarity**: Complex workflows are broken into understandable stages
2. **Reusability**: Child groups can be reused in different contexts
3. **Scoped Configuration**: Settings apply to specific subtrees
4. **Progress Tracking**: Monitor completion at each level
5. **Debugging**: Isolate issues to specific branches

### When to Use Hierarchies

**Use hierarchies when:**
- Workflow has distinct stages (CI/CD: build → test → deploy)
- Tasks naturally group into phases (infrastructure → application)
- You need different configurations for different parts
- You want to track progress at multiple granularities

**Keep flat when:**
- Workflow is simple with few tasks
- All tasks are similar in nature
- No clear stages or phases exist

## Shared Configuration

Task groups allow you to define shared configuration once and apply it to all tasks in the group. This shared configuration includes timeouts, error handling strategies, concurrency limits, and variables that all tasks in the group can access.

### Timeout

Set execution timeout for entire group:

```yaml
task_groups:
  long_running_tests:
    description: "Tests that may take a while"
    execution_mode: "parallel"
    timeout: 1800  # 30 minutes for entire group
    tasks:
      - stress_test
      - load_test
      - performance_test
```

**Behavior:**
- Timer starts when group begins execution
- Applies to all tasks in group (including child groups)
- On timeout, running tasks are cancelled
- Group marked as failed

### Error Handling Strategy

Apply consistent error handling to all tasks:

```yaml
task_groups:
  critical_deployment:
    description: "Production deployment - no failures allowed"
    execution_mode: "sequential"
    on_error: "stop"  # Stop immediately on any failure
    tasks:
      - backup_database
      - deploy_new_version
      - verify_health

  test_suite:
    description: "Run all tests - collect all failures"
    execution_mode: "parallel"
    on_error: "continue"  # Run all tests regardless of failures
    tasks:
      - test_backend
      - test_frontend
      - test_api
      - test_mobile
```

**Error Strategies:**
- `stop`: Stop group on first failure (default)
- `continue`: Run all tasks, collect failures
- `rollback`: Execute rollback logic on failure

### Concurrency Limits

Control resource usage across tasks:

```yaml
task_groups:
  heavy_processing:
    description: "Resource-intensive tasks"
    execution_mode: "parallel"
    max_concurrency: 2  # Only 2 tasks run concurrently
    tasks:
      - process_large_dataset_1
      - process_large_dataset_2
      - process_large_dataset_3
      - process_large_dataset_4
      - process_large_dataset_5
```

**Use cases:**
- Prevent memory exhaustion
- Rate limit API calls
- Control CPU/GPU usage
- Respect external service limits

### Configuration Inheritance

Child groups inherit configuration from parents:

```yaml
task_groups:
  parent:
    timeout: 3600      # 1 hour
    on_error: "stop"
    max_concurrency: 5
    groups: [child1, child2]

  child1:
    parent: "parent"
    # Inherits: timeout=3600, on_error=stop, max_concurrency=5
    tasks: [task1]

  child2:
    parent: "parent"
    timeout: 1800      # Override: 30 minutes
    # Inherits: on_error=stop, max_concurrency=5
    tasks: [task2]
```

**Inheritance rules:**
- Child inherits: `timeout`, `on_error`, `max_concurrency`
- Child can override any inherited value
- Explicit child value takes precedence
- Variables do NOT inherit (must be explicitly passed)

### Shared Variables

Define variables at group level for all tasks:

```yaml
task_groups:
  deployment:
    description: "Deploy to environment"
    execution_mode: "sequential"
    tasks:
      - backup
      - deploy
      - verify

    # All tasks can access these inputs
    inputs:
      environment:
        type: string
        required: true
      version:
        type: string
        required: true

    # All tasks can contribute to outputs
    outputs:
      deployment_url:
        source:
          type: state
          key: "deploy.url"

tasks:
  deploy:
    description: "Deploy version ${group.deployment.version} to ${group.deployment.environment}"
    agent: "deployer"
    group: "deployment"
```

## Execution Modes

Task groups support three execution modes that control how tasks run.

### Sequential Mode

Tasks execute one after another in order:

```yaml
task_groups:
  database_migration:
    execution_mode: "sequential"
    tasks:
      - backup_db
      - run_migrations
      - verify_schema
      - update_indexes
```

**Timeline:**
```
backup_db → run_migrations → verify_schema → update_indexes
   2s           5s               1s              3s
Total: 11 seconds
```

**Characteristics:**
- Predictable order
- Each task waits for previous
- Lower resource usage
- Longer total time

**When to use:**
- Order matters (migrations, deployments)
- Tasks depend on previous outputs
- Resource constraints require serial execution
- Debugging (easier to trace)

### Parallel Mode

Tasks execute concurrently:

```yaml
task_groups:
  independent_tests:
    execution_mode: "parallel"
    max_concurrency: 4  # Run up to 4 at once
    tasks:
      - test_auth
      - test_payments
      - test_analytics
      - test_notifications
```

**Timeline:**
```
test_auth          ─────────┐
test_payments      ─────────┤
test_analytics     ─────────┼─→ All complete
test_notifications ─────────┘
           3s (all run together)
Total: 3 seconds
```

**Characteristics:**
- Maximum throughput
- Tasks run independently
- Higher resource usage
- Shorter total time
- Non-deterministic completion order

**When to use:**
- Tasks are independent
- No shared state
- Have available resources
- Want faster completion

### Auto Mode

Automatically determines execution strategy based on dependencies:

```yaml
task_groups:
  smart_build:
    execution_mode: "auto"
    tasks:
      - fetch_deps     # No dependencies
      - compile_lib    # depends_on: [fetch_deps]
      - compile_app    # depends_on: [fetch_deps]
      - run_tests      # depends_on: [compile_lib, compile_app]
```

**Timeline:**
```
fetch_deps ─────┬──→ compile_lib ──┐
                └──→ compile_app ──┴──→ run_tests
```

**Behavior:**
- Analyzes task dependencies
- Parallelizes when safe
- Respects ordering when needed
- Optimizes execution plan

**When to use:**
- Mixed dependencies
- Want optimal performance
- Don't want to manually optimize
- Workflow changes frequently

### Comparison

| Feature | Sequential | Parallel | Auto |
|---------|-----------|----------|------|
| Execution | One at a time | All together | Optimized |
| Resource usage | Low | High | Medium |
| Total time | Longest | Shortest | Optimized |
| Order guarantee | Yes | No | Partial |
| Setup complexity | Simple | Medium | Simple |
| Best for | Ordered steps | Independent tasks | Mixed workflows |

## Dependencies

Task groups can depend on other groups, creating execution graphs.

### Basic Dependencies

One group waits for another:

```yaml
task_groups:
  prepare:
    tasks: [setup_env, fetch_data]

  process:
    depends_on: [prepare]  # Waits for prepare
    tasks: [transform_data, analyze_data]
```

**Execution:**
```
prepare → process
```

### Multiple Dependencies

Group waits for multiple groups:

```yaml
task_groups:
  fetch_code:
    tasks: [git_clone]

  fetch_data:
    tasks: [download_datasets]

  build:
    depends_on: [fetch_code, fetch_data]  # Waits for both
    tasks: [compile, package]
```

**Execution:**
```
fetch_code ──┐
             ├─→ build
fetch_data ──┘
```

### Dependency Chains

Create multi-stage pipelines:

```yaml
task_groups:
  stage1:
    tasks: [task1]

  stage2:
    depends_on: [stage1]
    tasks: [task2]

  stage3:
    depends_on: [stage2]
    tasks: [task3]

  stage4:
    depends_on: [stage3]
    tasks: [task4]
```

**Execution:**
```
stage1 → stage2 → stage3 → stage4
```

### Diamond Dependencies

Multiple paths converge:

```yaml
task_groups:
  source:
    tasks: [fetch]

  path_a:
    depends_on: [source]
    tasks: [process_a]

  path_b:
    depends_on: [source]
    tasks: [process_b]

  merge:
    depends_on: [path_a, path_b]
    tasks: [combine_results]
```

**Execution:**
```
      source
      ↙    ↘
  path_a  path_b
      ↘    ↙
       merge
```

### Conditional Dependencies

Groups execute conditionally:

```yaml
task_groups:
  quality_check:
    tasks: [run_checks]
    outputs:
      quality_score:
        source:
          type: state
          key: "quality.score"

  deploy_staging:
    depends_on: [quality_check]
    condition: "${group.quality_check.quality_score} >= 0.7"
    tasks: [deploy_to_staging]

  deploy_prod:
    depends_on: [quality_check]
    condition: "${group.quality_check.quality_score} >= 0.9"
    tasks: [deploy_to_production]
```

**Execution:**
```
quality_check
     ├─→ deploy_staging (if score >= 0.7)
     └─→ deploy_prod (if score >= 0.9)
```

### Dependency Resolution

The system automatically:
1. Builds dependency graph
2. Detects circular dependencies (errors if found)
3. Performs topological sort
4. Determines execution order

**Circular dependency example (ERROR):**
```yaml
task_groups:
  A:
    depends_on: [C]  # A depends on C
  B:
    depends_on: [A]  # B depends on A
  C:
    depends_on: [B]  # C depends on B (creates cycle!)
```

**Error:** `Circular dependency detected: A → B → C → A`

## Variables and Data Flow

Groups can pass data through variables.

### Group Outputs

Produce variables for downstream groups:

```yaml
task_groups:
  build:
    description: "Build application"
    tasks: [compile, package]

    outputs:
      artifact_path:
        source:
          type: state
          key: "build.artifact"
      version:
        source:
          type: file
          path: "./VERSION"
```

### Group Inputs

Consume variables from other groups:

```yaml
task_groups:
  deploy:
    description: "Deploy application"
    depends_on: [build]
    tasks: [upload, start]

    inputs:
      artifact: "${group.build.artifact_path}"
      version: "${group.build.version}"
```

### Variable Scopes

Variables have hierarchical scoping:

```yaml
# Workflow-level variables
inputs:
  environment:
    type: string
    required: true

task_groups:
  deploy:
    # Group-level variables
    outputs:
      url:
        source:
          type: state
          key: "deploy.url"

    tasks:
      - deploy_app

tasks:
  deploy_app:
    # Task-level variables
    description: "Deploy to ${workflow.environment}"
    outputs:
      status:
        source:
          type: state
          key: "task.status"
```

**Access patterns:**
- `${workflow.environment}` - Workflow variable
- `${group.deploy.url}` - Group variable
- `${task.deploy_app.status}` - Task variable
- `${environment}` - Implicit search (task → group → workflow)

### Data Flow Example

Complete example showing data flow:

```yaml
task_groups:
  # Stage 1: Produce data
  analysis:
    execution_mode: "parallel"
    tasks: [analyze_code, analyze_deps]
    outputs:
      code_quality:
        source:
          type: state
          key: "analysis.code_quality"
      dep_count:
        source:
          type: state
          key: "analysis.dependencies"

  # Stage 2: Consume and transform
  processing:
    depends_on: [analysis]
    tasks: [process_results]
    inputs:
      quality: "${group.analysis.code_quality}"
      deps: "${group.analysis.dep_count}"
    outputs:
      report:
        source:
          type: file
          path: "./report.json"

  # Stage 3: Final consumption
  notification:
    depends_on: [processing]
    tasks: [send_notification]
    inputs:
      report_path: "${group.processing.report}"
```

**Data flow:**
```
analysis [produces: code_quality, dep_count]
    ↓
processing [consumes: quality, deps] [produces: report]
    ↓
notification [consumes: report_path]
```

## Error Handling

Groups provide multiple strategies for handling failures.

### Stop on Error

Stop immediately when a task fails (default):

```yaml
task_groups:
  critical_deployment:
    on_error: "stop"
    tasks:
      - backup_db
      - deploy_code
      - run_migrations
      - verify_health
```

**Behavior:**
If `deploy_code` fails:
1. Execution stops immediately
2. `run_migrations` and `verify_health` are cancelled
3. Group marked as failed
4. Error propagates to parent

**Use when:**
- Failures are critical
- No point continuing if one fails
- Fast failure detection needed
- Resources should not be wasted

### Continue on Error

Execute all tasks regardless of failures:

```yaml
task_groups:
  test_suite:
    on_error: "continue"
    tasks:
      - test_backend
      - test_frontend
      - test_api
      - test_mobile
```

**Behavior:**
If `test_frontend` fails:
1. Error is recorded
2. Remaining tests continue
3. All tests execute
4. Group marked failed if any task failed
5. Full error report available

**Use when:**
- Want complete error report
- Tasks are independent
- Collecting metrics
- Test suites

### Rollback on Error

Execute rollback logic on failure:

```yaml
task_groups:
  deployment:
    on_error: "rollback"
    tasks:
      - backup_current
      - deploy_new
      - verify_deployment
```

**Behavior:**
If `verify_deployment` fails:
1. Rollback tasks execute
2. Attempt to restore previous state
3. Group marked failed with rollback attempted

**Use when:**
- Changes must be reversible
- System state must be consistent
- Deployments, migrations, config changes

### Timeout Handling

Set maximum execution time:

```yaml
task_groups:
  long_running:
    timeout: 3600  # 1 hour
    tasks:
      - process_large_dataset
      - train_ml_model
      - generate_reports
```

**Behavior:**
If group exceeds 1 hour:
1. Running tasks are cancelled
2. Group marked as timeout
3. Error propagates to parent

### Error Propagation

Errors propagate up hierarchy:

```yaml
task_groups:
  parent:
    on_error: "stop"
    groups: [child]

  child:
    parent: "parent"
    on_error: "continue"
    tasks: [task1, task2]
```

**Scenario:** `task1` fails
1. `child` continues (on_error: continue)
2. `task2` executes
3. `child` completes with failed status
4. `parent` sees child failed
5. `parent` stops (on_error: stop)

## Examples

### Example 1: Simple CI/CD Pipeline

```yaml
name: "Simple CI/CD"
version: "1.0.0"

agents:
  builder:
    description: "Build agent"
    tools: [Bash, Read, Write]
    permissions:
      mode: "acceptEdits"

task_groups:
  build:
    description: "Build application"
    execution_mode: "sequential"
    tasks:
      - compile
      - test
    timeout: 600
    on_error: "stop"

  deploy:
    description: "Deploy to staging"
    execution_mode: "sequential"
    depends_on: [build]
    tasks:
      - deploy_staging
      - verify
    timeout: 300
    on_error: "rollback"

tasks:
  compile:
    description: "Compile source code"
    agent: "builder"
    group: "build"

  test:
    description: "Run tests"
    agent: "builder"
    group: "build"

  deploy_staging:
    description: "Deploy to staging"
    agent: "builder"
    group: "deploy"

  verify:
    description: "Verify deployment"
    agent: "builder"
    group: "deploy"
```

### Example 2: Multi-Stage with Parallel Tests

```yaml
name: "Multi-Stage Pipeline"
version: "1.0.0"

agents:
  ci_agent:
    description: "CI agent"
    tools: [Bash, Read, Write]
    permissions:
      mode: "acceptEdits"

task_groups:
  preparation:
    description: "Prepare environment"
    execution_mode: "sequential"
    tasks:
      - checkout
      - fetch_deps

  build:
    description: "Build components"
    execution_mode: "parallel"
    depends_on: [preparation]
    max_concurrency: 2
    tasks:
      - build_backend
      - build_frontend

  test:
    description: "Run test suites"
    execution_mode: "parallel"
    depends_on: [build]
    on_error: "continue"  # Run all tests
    tasks:
      - test_unit
      - test_integration
      - test_e2e

  deploy:
    description: "Deploy application"
    execution_mode: "sequential"
    depends_on: [test]
    tasks:
      - deploy_app
      - health_check

tasks:
  checkout:
    description: "Checkout code"
    agent: "ci_agent"
    group: "preparation"

  fetch_deps:
    description: "Fetch dependencies"
    agent: "ci_agent"
    group: "preparation"

  build_backend:
    description: "Build backend"
    agent: "ci_agent"
    group: "build"

  build_frontend:
    description: "Build frontend"
    agent: "ci_agent"
    group: "build"

  test_unit:
    description: "Unit tests"
    agent: "ci_agent"
    group: "test"

  test_integration:
    description: "Integration tests"
    agent: "ci_agent"
    group: "test"

  test_e2e:
    description: "E2E tests"
    agent: "ci_agent"
    group: "test"

  deploy_app:
    description: "Deploy application"
    agent: "ci_agent"
    group: "deploy"

  health_check:
    description: "Health check"
    agent: "ci_agent"
    group: "deploy"
```

### Example 3: Hierarchical Deployment

```yaml
name: "Hierarchical Deployment"
version: "1.0.0"

inputs:
  environment:
    type: string
    required: true

agents:
  deployer:
    description: "Deployment agent"
    tools: [Bash, Read, Write]
    permissions:
      mode: "acceptEdits"

task_groups:
  # Root group
  full_deployment:
    description: "Complete deployment pipeline"
    groups:
      - infrastructure
      - application
    timeout: 1800

  # Level 1: Infrastructure
  infrastructure:
    description: "Deploy infrastructure"
    parent: "full_deployment"
    execution_mode: "sequential"
    groups:
      - database
      - cache

  # Level 2: Database
  database:
    description: "Setup database"
    parent: "infrastructure"
    execution_mode: "sequential"
    tasks:
      - provision_db
      - run_migrations

  # Level 2: Cache
  cache:
    description: "Setup cache"
    parent: "infrastructure"
    depends_on: [database]
    tasks:
      - setup_redis

  # Level 1: Application
  application:
    description: "Deploy application"
    parent: "full_deployment"
    depends_on: [infrastructure]
    execution_mode: "sequential"
    tasks:
      - deploy_backend
      - deploy_frontend
      - configure_routing

tasks:
  provision_db:
    description: "Provision database"
    agent: "deployer"
    group: "database"

  run_migrations:
    description: "Run migrations"
    agent: "deployer"
    group: "database"

  setup_redis:
    description: "Setup Redis"
    agent: "deployer"
    group: "cache"

  deploy_backend:
    description: "Deploy backend to ${workflow.environment}"
    agent: "deployer"
    group: "application"

  deploy_frontend:
    description: "Deploy frontend to ${workflow.environment}"
    agent: "deployer"
    group: "application"

  configure_routing:
    description: "Configure routing"
    agent: "deployer"
    group: "application"
```

### Example 4: Quality Gate with Variables

```yaml
name: "Quality Gate Pipeline"
version: "1.0.0"

inputs:
  min_coverage:
    type: number
    required: false
    default: 0.8

agents:
  tester:
    description: "Test agent"
    tools: [Bash, Read, Write]
    permissions:
      mode: "default"

task_groups:
  test:
    description: "Run tests and calculate coverage"
    execution_mode: "parallel"
    tasks:
      - run_tests
      - calculate_coverage

    outputs:
      coverage:
        source:
          type: state
          key: "test.coverage"
      tests_passed:
        source:
          type: state
          key: "test.passed"

  deploy:
    description: "Deploy if quality gate passes"
    depends_on: [test]
    condition: |
      ${group.test.coverage} >= ${workflow.min_coverage} &&
      ${group.test.tests_passed} == true
    tasks:
      - deploy_production

    inputs:
      coverage_value: "${group.test.coverage}"

tasks:
  run_tests:
    description: "Run test suite"
    agent: "tester"
    group: "test"

  calculate_coverage:
    description: "Calculate coverage"
    agent: "tester"
    group: "test"

  deploy_production:
    description: "Deploy to production (coverage: ${group.deploy.coverage_value})"
    agent: "tester"
    group: "deploy"
```

## Best Practices

### 1. Use Meaningful Names

**Good:**
```yaml
task_groups:
  database_migration:
    description: "Migrate database schema to v2.0"

  security_scanning:
    description: "Run security vulnerability scans"
```

**Bad:**
```yaml
task_groups:
  group1:
    description: "Do stuff"

  grp2:
    description: "More things"
```

### 2. Keep Hierarchies Shallow

**Good:** 2-3 levels
```yaml
pipeline
├── build
│   ├── compile
│   └── package
└── deploy
    └── verify
```

**Avoid:** >4 levels (too complex)
```yaml
root
└── level1
    └── level2
        └── level3
            └── level4
                └── level5  # Too deep!
```

### 3. Use Parallel Mode Wisely

**Good:** Independent tasks
```yaml
task_groups:
  tests:
    execution_mode: "parallel"
    max_concurrency: 5  # Limit resource usage
    tasks:
      - test_auth
      - test_payments
      - test_analytics
```

**Bad:** Dependent tasks in parallel
```yaml
task_groups:
  build:
    execution_mode: "parallel"  # Wrong! These have dependencies
    tasks:
      - fetch_deps
      - compile  # Needs deps!
      - package  # Needs compile!
```

### 4. Set Appropriate Timeouts

**Good:**
```yaml
task_groups:
  quick_checks:
    timeout: 60  # 1 minute
    tasks: [lint, format_check]

  long_build:
    timeout: 1800  # 30 minutes
    tasks: [compile_all, run_all_tests]
```

**Bad:**
```yaml
task_groups:
  everything:
    timeout: 10  # Too short! Will timeout
    tasks: [build, test, deploy]
```

### 5. Choose Right Error Strategy

**Stop:** Critical operations
```yaml
deployment:
  on_error: "stop"  # Can't continue if backup fails
  tasks: [backup, deploy, verify]
```

**Continue:** Test suites
```yaml
tests:
  on_error: "continue"  # Want all test results
  tasks: [test1, test2, test3]
```

**Rollback:** Reversible changes
```yaml
migration:
  on_error: "rollback"  # Can undo changes
  tasks: [backup, migrate, verify]
```

### 6. Document Group Purpose

```yaml
task_groups:
  infrastructure_setup:
    description: |
      Sets up complete infrastructure including:
      - Database cluster with replication
      - Redis cache with persistence
      - Load balancers with health checks
      Estimated time: 10-15 minutes
    groups: [database, cache, load_balancers]
```

### 7. Use Variables for Flexibility

```yaml
task_groups:
  deploy:
    description: "Deploy to ${workflow.environment}"
    inputs:
      environment:
        type: string
        required: true
      replicas:
        type: number
        required: false
        default: 3
    tasks: [deploy_app]
```

### 8. Validate Before Running

```bash
# Always validate first!
./target/release/periplon-executor validate workflow.yaml

# Then run
./target/release/periplon-executor run workflow.yaml
```

### 9. Start Simple, Add Complexity

**Phase 1:** Basic groups
```yaml
task_groups:
  build:
    tasks: [compile, test]
```

**Phase 2:** Add dependencies
```yaml
task_groups:
  build:
    tasks: [compile, test]

  deploy:
    depends_on: [build]
    tasks: [deploy]
```

**Phase 3:** Add hierarchy
```yaml
task_groups:
  pipeline:
    groups: [build, deploy]

  build:
    parent: "pipeline"
    tasks: [compile, test]

  deploy:
    parent: "pipeline"
    depends_on: [build]
    tasks: [deploy]
```

### 10. Monitor and Iterate

- Start with simple structure
- Monitor execution times
- Identify bottlenecks
- Add parallelism where safe
- Refine based on real usage

---

## Additional Resources

- [API Reference](./task-groups/api-reference.md) - Complete schema documentation
- [Architecture](./task-groups/architecture.md) - Internal implementation
- [Tutorial](./task-groups/tutorial.md) - Step-by-step guide
- [Examples](../examples/task-groups/) - Working examples

## Summary

Task groups provide:
- ✓ **Organization**: Logical grouping of related tasks
- ✓ **Control**: Sequential, parallel, or auto execution
- ✓ **Hierarchy**: Multi-level nesting for complex workflows
- ✓ **Configuration**: Shared settings across tasks
- ✓ **Dependencies**: Clear execution order
- ✓ **Variables**: Data flow between groups
- ✓ **Error Handling**: Flexible failure strategies

Use task groups to build maintainable, scalable, and efficient workflows!

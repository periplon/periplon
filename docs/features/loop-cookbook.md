# DSL Loop Pattern Cookbook

**Version:** 1.0
**Date:** 2025-10-19

Real-world examples and patterns for DSL loops. Copy, adapt, and use these proven patterns in your workflows.

---

## Table of Contents

1. [File Processing](#file-processing)
2. [API Integration](#api-integration)
3. [Monitoring & Polling](#monitoring--polling)
4. [Batch Processing](#batch-processing)
5. [Error Handling & Retry](#error-handling--retry)
6. [Data Transformation](#data-transformation)
7. [Parallel Processing](#parallel-processing)
8. [Checkpointing & Resume](#checkpointing--resume)
9. [Database Operations](#database-operations)
10. [Complex Workflows](#complex-workflows)

---

## File Processing

### Pattern 1: Process Files in Directory

**Use Case:** Scan directory, process each file.

```yaml
name: "File Processor"
version: "1.0.0"

agents:
  file_processor:
    description: "Process files"
    tools: [Read, Write, Bash]
    permissions:
      mode: "acceptEdits"

tasks:
  list_files:
    description: "List all .txt files in directory"
    agent: "file_processor"
    # Stores file list in state["txt_files"]

  process_files:
    description: "Processing {{file}}"
    agent: "file_processor"
    depends_on: [list_files]
    loop:
      type: for_each
      collection:
        source: state
        key: "txt_files"
      iterator: "file"
    loop_control:
      collect_results: true
      result_key: "processed_files"
      checkpoint_interval: 10
```

---

### Pattern 2: Batch Rename Files

**Use Case:** Rename files with pattern.

```yaml
name: "Batch File Renamer"
version: "1.0.0"

agents:
  renamer:
    description: "Rename files"
    tools: [Bash]
    permissions:
      mode: "acceptEdits"

tasks:
  rename_files:
    description: "Renaming {{file}} to {{file}}.backup"
    agent: "renamer"
    loop:
      type: for_each
      collection:
        source: file
        path: "files_to_rename.txt"
        format: lines
      iterator: "file"
```

---

### Pattern 3: Convert File Formats

**Use Case:** Convert multiple files from one format to another.

```yaml
name: "Format Converter"
version: "1.0.0"

agents:
  converter:
    description: "Convert file formats"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  convert_csv_to_json:
    description: "Converting {{csv_file}}"
    agent: "converter"
    output: "output/{{csv_file}}.json"
    loop:
      type: for_each
      collection:
        source: file
        path: "csv_files.txt"
        format: lines
      iterator: "csv_file"
      parallel: true
      max_parallel: 3
    loop_control:
      collect_results: true
      result_key: "converted_files"
```

---

## API Integration

### Pattern 4: Fetch and Process API Data

**Use Case:** Get data from REST API, process each item.

```yaml
name: "API Data Processor"
version: "1.0.0"

agents:
  api_processor:
    description: "Process API data"
    tools: [WebFetch]
    permissions:
      mode: "default"

tasks:
  fetch_and_process:
    description: "Processing user {{user.name}} (ID: {{user.id}})"
    agent: "api_processor"
    loop:
      type: for_each
      collection:
        source: http
        url: "https://jsonplaceholder.typicode.com/users"
        method: "GET"
        format: json
      iterator: "user"
    loop_control:
      collect_results: true
      result_key: "processed_users"
      timeout_secs: 300
```

---

### Pattern 5: Paginate Through API Results

**Use Case:** Fetch all pages from paginated API.

```yaml
name: "API Paginator"
version: "1.0.0"

agents:
  paginator:
    description: "Fetch paginated data"
    tools: [WebFetch]
    permissions:
      mode: "default"

tasks:
  fetch_all_pages:
    description: "Fetching page {{page}}"
    agent: "paginator"
    loop:
      type: for_each
      collection:
        source: range
        start: 1
        end: 11  # Pages 1-10
        step: 1
      iterator: "page"
    loop_control:
      collect_results: true
      result_key: "all_results"
```

---

### Pattern 6: POST Data to API

**Use Case:** Submit multiple items to API endpoint.

```yaml
name: "API Submitter"
version: "1.0.0"

agents:
  submitter:
    description: "Submit data to API"
    tools: [WebFetch]
    permissions:
      mode: "default"

tasks:
  submit_items:
    description: "Submitting {{item.name}}"
    agent: "submitter"
    loop:
      type: for_each
      collection:
        source: file
        path: "items_to_submit.json"
        format: json
      iterator: "item"
      parallel: true
      max_parallel: 5  # Respect API rate limits
    loop_control:
      collect_results: true
      result_key: "submission_results"
      timeout_secs: 600
```

---

## Monitoring & Polling

### Pattern 7: Poll for Job Completion

**Use Case:** Wait for background job to complete.

```yaml
name: "Job Poller"
version: "1.0.0"

agents:
  poller:
    description: "Poll job status"
    tools: [WebFetch]
    permissions:
      mode: "default"

tasks:
  wait_for_job:
    description: "Checking job status (attempt {{iteration}})"
    agent: "poller"
    loop:
      type: while
      condition:
        type: state_equals
        key: "job_complete"
        value: false
      max_iterations: 60
      iteration_variable: "iteration"
      delay_between_secs: 10  # Check every 10 seconds
    loop_control:
      timeout_secs: 600  # 10 minute max
```

---

### Pattern 8: Health Check Monitoring

**Use Case:** Monitor service health until ready.

```yaml
name: "Health Monitor"
version: "1.0.0"

agents:
  monitor:
    description: "Monitor service health"
    tools: [WebFetch]
    permissions:
      mode: "default"

tasks:
  wait_for_healthy:
    description: "Health check {{iteration}}"
    agent: "monitor"
    loop:
      type: while
      condition:
        type: state_equals
        key: "service_healthy"
        value: false
      max_iterations: 30
      iteration_variable: "iteration"
      delay_between_secs: 5
    loop_control:
      timeout_secs: 180
      break_condition:
        type: state_equals
        key: "service_error"
        value: true
```

---

### Pattern 9: Continuous Monitoring

**Use Case:** Monitor metrics repeatedly.

```yaml
name: "Metrics Monitor"
version: "1.0.0"

agents:
  monitor:
    description: "Collect metrics"
    tools: [WebFetch]
    permissions:
      mode: "default"

tasks:
  collect_metrics:
    description: "Collecting metrics (sample {{sample}})"
    agent: "monitor"
    loop:
      type: repeat
      count: 100
      iterator: "sample"
    loop_control:
      collect_results: true
      result_key: "metric_samples"
      checkpoint_interval: 10
```

---

## Batch Processing

### Pattern 10: Process Large Dataset in Batches

**Use Case:** Split large job into manageable batches.

```yaml
name: "Batch Processor"
version: "1.0.0"

agents:
  processor:
    description: "Process data batches"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  process_batches:
    description: "Processing batch {{batch}}/100"
    agent: "processor"
    loop:
      type: repeat
      count: 100
      iterator: "batch"
      parallel: true
      max_parallel: 10  # Process 10 batches concurrently
    loop_control:
      collect_results: true
      result_key: "batch_results"
      checkpoint_interval: 10
```

---

### Pattern 11: Chunked File Processing

**Use Case:** Process file in chunks for memory efficiency.

```yaml
name: "Chunked Processor"
version: "1.0.0"

agents:
  processor:
    description: "Process file chunks"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  process_chunks:
    description: "Processing chunk {{chunk}}"
    agent: "processor"
    loop:
      type: for_each
      collection:
        source: range
        start: 0
        end: 1000  # 1000 chunks
        step: 1
      iterator: "chunk"
    loop_control:
      checkpoint_interval: 50
      collect_results: true
      result_key: "chunk_results"
```

---

## Error Handling & Retry

### Pattern 12: Retry with Exponential Backoff

**Use Case:** Retry operation with increasing delays.

```yaml
name: "Retry with Backoff"
version: "1.0.0"

agents:
  retrier:
    description: "Retry operations"
    tools: [WebFetch]
    permissions:
      mode: "default"

tasks:
  fetch_with_retry:
    description: "Attempt {{iteration}}/5"
    agent: "retrier"
    loop:
      type: repeat_until
      condition:
        type: state_equals
        key: "fetch_success"
        value: true
      min_iterations: 1
      max_iterations: 5
      iteration_variable: "iteration"
      delay_between_secs: 2  # 2, 4, 8, 16 seconds
    loop_control:
      timeout_secs: 60
```

---

### Pattern 13: Graceful Failure Handling

**Use Case:** Continue processing despite individual failures.

```yaml
name: "Fault Tolerant Processor"
version: "1.0.0"

agents:
  processor:
    description: "Process with error handling"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  process_with_error_handling:
    description: "Processing {{item}}"
    agent: "processor"
    loop:
      type: for_each
      collection:
        source: file
        path: "items.json"
        format: json
      iterator: "item"
    loop_control:
      collect_results: true
      result_key: "results"
      continue_condition:
        type: state_equals
        key: "skip_failed_item"
        value: true
```

---

### Pattern 14: Circuit Breaker

**Use Case:** Stop processing after too many failures.

```yaml
name: "Circuit Breaker"
version: "1.0.0"

agents:
  processor:
    description: "Process with circuit breaker"
    tools: [WebFetch]
    permissions:
      mode: "default"

tasks:
  process_with_breaker:
    description: "Processing {{item}}"
    agent: "processor"
    loop:
      type: for_each
      collection:
        source: http
        url: "https://api.example.com/items"
        format: json
      iterator: "item"
    loop_control:
      break_condition:
        type: state_equals
        key: "failure_count_exceeded"
        value: true
      collect_results: true
      result_key: "successful_results"
```

---

## Data Transformation

### Pattern 15: Map-Reduce Pattern

**Use Case:** Transform and aggregate data.

```yaml
name: "Map-Reduce"
version: "1.0.0"

agents:
  mapper:
    description: "Transform data"
    tools: [Read, Write]
    permissions:
      mode: "default"

  reducer:
    description: "Aggregate results"
    tools: [Read, Write]
    permissions:
      mode: "default"

tasks:
  map_phase:
    description: "Mapping {{item}}"
    agent: "mapper"
    loop:
      type: for_each
      collection:
        source: file
        path: "input_data.json"
        format: json
      iterator: "item"
      parallel: true
      max_parallel: 10
    loop_control:
      collect_results: true
      result_key: "mapped_results"

  reduce_phase:
    description: "Reducing results"
    agent: "reducer"
    depends_on: [map_phase]
```

---

### Pattern 16: Filter and Transform

**Use Case:** Filter items and transform remaining.

```yaml
name: "Filter and Transform"
version: "1.0.0"

agents:
  transformer:
    description: "Filter and transform data"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  filter_and_transform:
    description: "Processing {{item.id}}"
    agent: "transformer"
    loop:
      type: for_each
      collection:
        source: http
        url: "https://api.example.com/data"
        format: json
      iterator: "item"
    loop_control:
      continue_condition:
        type: state_equals
        key: "skip_item"
        value: true
      collect_results: true
      result_key: "transformed_results"
```

---

## Parallel Processing

### Pattern 17: Parallel API Calls

**Use Case:** Fetch from multiple APIs concurrently.

```yaml
name: "Parallel API Fetcher"
version: "1.0.0"

agents:
  fetcher:
    description: "Fetch from APIs"
    tools: [WebFetch]
    permissions:
      mode: "default"

tasks:
  fetch_parallel:
    description: "Fetching from {{api.name}}"
    agent: "fetcher"
    loop:
      type: for_each
      collection:
        source: inline
        items:
          - {name: "users", url: "https://api.example.com/users"}
          - {name: "posts", url: "https://api.example.com/posts"}
          - {name: "comments", url: "https://api.example.com/comments"}
      iterator: "api"
      parallel: true
      max_parallel: 3
    loop_control:
      collect_results: true
      result_key: "api_results"
      timeout_secs: 300
```

---

### Pattern 18: Parallel File Processing

**Use Case:** Process multiple files concurrently.

```yaml
name: "Parallel File Processor"
version: "1.0.0"

agents:
  processor:
    description: "Process files in parallel"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  process_files_parallel:
    description: "Processing {{file}}"
    agent: "processor"
    loop:
      type: for_each
      collection:
        source: file
        path: "file_list.txt"
        format: lines
      iterator: "file"
      parallel: true
      max_parallel: 5
    loop_control:
      collect_results: true
      result_key: "processed_files"
      checkpoint_interval: 10
```

---

## Checkpointing & Resume

### Pattern 19: Resumable Long Job

**Use Case:** Long-running job with resume capability.

```yaml
name: "Resumable Job"
version: "1.0.0"

agents:
  worker:
    description: "Long-running worker"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  long_job:
    description: "Processing item {{item}}/10000"
    agent: "worker"
    loop:
      type: for_each
      collection:
        source: range
        start: 0
        end: 10000
        step: 1
      iterator: "item"
    loop_control:
      checkpoint_interval: 100  # Checkpoint every 100 items
      collect_results: true
      result_key: "all_results"
      timeout_secs: 7200  # 2 hour max
```

**Resume:**
```bash
# First run (may be interrupted)
cargo run --example my_workflow

# Resume from checkpoint
cargo run --example my_workflow
# Automatically skips completed iterations
```

---

### Pattern 20: Multi-Stage Processing with Checkpoints

**Use Case:** Multiple stages with checkpointing.

```yaml
name: "Multi-Stage Pipeline"
version: "1.0.0"

agents:
  stage1:
    description: "First stage processor"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

  stage2:
    description: "Second stage processor"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  stage1_process:
    description: "Stage 1: Processing {{item}}"
    agent: "stage1"
    loop:
      type: for_each
      collection:
        source: file
        path: "input.json"
        format: json
      iterator: "item"
    loop_control:
      checkpoint_interval: 50
      collect_results: true
      result_key: "stage1_results"

  stage2_process:
    description: "Stage 2: Processing {{item}}"
    agent: "stage2"
    depends_on: [stage1_process]
    loop:
      type: for_each
      collection:
        source: state
        key: "stage1_results"
      iterator: "item"
    loop_control:
      checkpoint_interval: 50
      collect_results: true
      result_key: "stage2_results"
```

---

## Database Operations

### Pattern 21: Batch Database Updates

**Use Case:** Update database records in batches.

```yaml
name: "Batch DB Updater"
version: "1.0.0"

agents:
  updater:
    description: "Update database records"
    tools: [Bash]
    permissions:
      mode: "default"

tasks:
  update_records:
    description: "Updating batch {{batch}}/100"
    agent: "updater"
    loop:
      type: repeat
      count: 100
      iterator: "batch"
    loop_control:
      checkpoint_interval: 10
      collect_results: true
      result_key: "update_results"
```

---

### Pattern 22: Database Migration

**Use Case:** Migrate data from old to new schema.

```yaml
name: "Data Migration"
version: "1.0.0"

agents:
  migrator:
    description: "Migrate database records"
    tools: [Bash, Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  migrate_records:
    description: "Migrating record {{record.id}}"
    agent: "migrator"
    loop:
      type: for_each
      collection:
        source: file
        path: "records_to_migrate.jsonl"
        format: json_lines
      iterator: "record"
      parallel: true
      max_parallel: 5
    loop_control:
      checkpoint_interval: 100
      collect_results: true
      result_key: "migration_results"
      timeout_secs: 3600
```

---

## Complex Workflows

### Pattern 23: ETL Pipeline

**Use Case:** Extract, Transform, Load data pipeline.

```yaml
name: "ETL Pipeline"
version: "1.0.0"

agents:
  extractor:
    description: "Extract data"
    tools: [WebFetch]
    permissions:
      mode: "default"

  transformer:
    description: "Transform data"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

  loader:
    description: "Load data"
    tools: [Bash]
    permissions:
      mode: "default"

tasks:
  extract:
    description: "Extracting data from source {{source}}"
    agent: "extractor"
    loop:
      type: for_each
      collection:
        source: inline
        items:
          - {name: "api1", url: "https://api1.example.com/data"}
          - {name: "api2", url: "https://api2.example.com/data"}
      iterator: "source"
      parallel: true
      max_parallel: 2
    loop_control:
      collect_results: true
      result_key: "extracted_data"

  transform:
    description: "Transforming record {{record.id}}"
    agent: "transformer"
    depends_on: [extract]
    loop:
      type: for_each
      collection:
        source: state
        key: "extracted_data"
      iterator: "record"
      parallel: true
      max_parallel: 10
    loop_control:
      collect_results: true
      result_key: "transformed_data"
      checkpoint_interval: 100

  load:
    description: "Loading batch {{batch}}"
    agent: "loader"
    depends_on: [transform]
    loop:
      type: for_each
      collection:
        source: state
        key: "transformed_data"
      iterator: "batch"
    loop_control:
      checkpoint_interval: 10
```

---

### Pattern 24: Multi-Source Aggregation

**Use Case:** Aggregate data from multiple sources.

```yaml
name: "Multi-Source Aggregator"
version: "1.0.0"

agents:
  fetcher:
    description: "Fetch from sources"
    tools: [WebFetch]
    permissions:
      mode: "default"

  aggregator:
    description: "Aggregate results"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  fetch_from_sources:
    description: "Fetching from {{source.name}}"
    agent: "fetcher"
    loop:
      type: for_each
      collection:
        source: file
        path: "data_sources.json"
        format: json
      iterator: "source"
      parallel: true
      max_parallel: 5
    loop_control:
      collect_results: true
      result_key: "source_results"
      timeout_secs: 600

  aggregate:
    description: "Aggregating results"
    agent: "aggregator"
    depends_on: [fetch_from_sources]
```

---

### Pattern 25: Workflow Orchestration

**Use Case:** Orchestrate complex multi-step workflow.

```yaml
name: "Workflow Orchestrator"
version: "1.0.0"

agents:
  orchestrator:
    description: "Orchestrate workflow steps"
    tools: [Bash, Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  setup:
    description: "Setup environment"
    agent: "orchestrator"

  execute_steps:
    description: "Executing step {{step.name}}"
    agent: "orchestrator"
    depends_on: [setup]
    loop:
      type: for_each
      collection:
        source: file
        path: "workflow_steps.json"
        format: json
      iterator: "step"
    loop_control:
      break_condition:
        type: state_equals
        key: "step_failed"
        value: true
      collect_results: true
      result_key: "step_results"
      checkpoint_interval: 1

  cleanup:
    description: "Cleanup resources"
    agent: "orchestrator"
    depends_on: [execute_steps]
```

---

## Summary

This cookbook provides **25 production-ready patterns** covering:

✅ File processing (3 patterns)
✅ API integration (3 patterns)
✅ Monitoring & polling (3 patterns)
✅ Batch processing (2 patterns)
✅ Error handling & retry (3 patterns)
✅ Data transformation (2 patterns)
✅ Parallel processing (2 patterns)
✅ Checkpointing & resume (2 patterns)
✅ Database operations (2 patterns)
✅ Complex workflows (3 patterns)

**Usage:**
1. Find pattern matching your use case
2. Copy example YAML
3. Adapt to your specific needs
4. Test with small dataset first
5. Scale to production

**Best Practices:**
- Always set safety limits (max_iterations, timeout)
- Use checkpoints for long-running jobs
- Limit parallel concurrency appropriately
- Add delays to polling loops
- Collect results only when needed

---

**Last Updated:** 2025-10-19
**Version:** 1.0
**Status:** ✅ Production Ready

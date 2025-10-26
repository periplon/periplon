# DSL Loop Patterns Guide

**Version:** 1.0
**Date:** 2025-10-19
**Status:** Production Ready

---

## Table of Contents

1. [Overview](#overview)
2. [Loop Pattern Types](#loop-pattern-types)
3. [Collection Sources](#collection-sources)
4. [Loop Control Features](#loop-control-features)
5. [Variable Substitution](#variable-substitution)
6. [Best Practices](#best-practices)
7. [Common Pitfalls](#common-pitfalls)
8. [Performance Considerations](#performance-considerations)
9. [Security Considerations](#security-considerations)

---

## Overview

The DSL loop system provides powerful iteration capabilities for multi-agent workflows. Loops enable processing collections, polling for conditions, retrying operations, and executing repeated tasks.

### Key Capabilities

- **Collection Iteration** - Process arrays, files, ranges, HTTP APIs
- **Conditional Loops** - While/until patterns for dynamic workflows
- **Repeat Patterns** - Count-based iteration for batch operations
- **Parallel Execution** - Concurrent iteration with concurrency limits
- **State Persistence** - Checkpoint and resume interrupted loops
- **Loop Control** - Break/continue, timeouts, result collection
- **Variable Substitution** - Use loop variables in task definitions

### When to Use Loops

**Use loops when you need to:**
- Process multiple items from a collection
- Poll for a condition to become true
- Retry an operation with backoff
- Execute a task multiple times
- Batch process data
- Iterate over API results

**Don't use loops when:**
- Single task execution is sufficient
- Dependencies can be modeled as task graph
- Operation doesn't involve iteration

---

## Loop Pattern Types

### 1. ForEach Loop

**Purpose:** Iterate over a collection of items.

**YAML Syntax:**
```yaml
tasks:
  process_items:
    description: "Process {{item.name}}"
    agent: "processor"
    loop:
      type: for_each
      collection:
        source: inline
        items: ["item1", "item2", "item3"]
      iterator: "item"
      parallel: false  # Optional: enable parallel execution
      max_parallel: 3  # Optional: limit concurrent iterations
    loop_control:
      collect_results: true  # Optional: collect iteration outputs
      result_key: "processed_items"  # Optional: key to store results
```

**When to Use:**
- Processing files in a directory
- Handling multiple entities from a database query
- Batch processing API results
- Data transformation pipelines
- Multi-step workflows per item

**Execution Modes:**

**Sequential (default):**
```yaml
loop:
  type: for_each
  collection:
    source: inline
    items: [1, 2, 3]
  iterator: "num"
  parallel: false  # Execute one at a time
```

**Parallel:**
```yaml
loop:
  type: for_each
  collection:
    source: inline
    items: [1, 2, 3]
  iterator: "num"
  parallel: true
  max_parallel: 2  # At most 2 concurrent iterations
```

**Example: Process GitHub Repositories**
```yaml
tasks:
  analyze_repos:
    description: "Analyzing {{repo.name}}"
    agent: "analyzer"
    loop:
      type: for_each
      collection:
        source: http
        url: "https://api.github.com/users/octocat/repos"
        headers:
          Accept: "application/vnd.github.v3+json"
        format: json
      iterator: "repo"
    loop_control:
      collect_results: true
      result_key: "analysis_results"
```

---

### 2. While Loop

**Purpose:** Execute while a condition is true (condition checked BEFORE each iteration).

**YAML Syntax:**
```yaml
tasks:
  poll_status:
    description: "Checking status (iteration {{iteration}})"
    agent: "checker"
    loop:
      type: while
      condition:
        type: state_equals
        key: "job_complete"
        value: false
      max_iterations: 100  # Safety limit
      iteration_variable: "iteration"  # Optional: track iteration number
      delay_between_secs: 5  # Optional: wait between iterations
    loop_control:
      timeout_secs: 300  # Optional: 5 minute timeout
```

**When to Use:**
- Polling for job completion
- Waiting for external condition
- Event-driven workflows
- Dynamic iteration based on runtime state

**Safety Features:**
- **max_iterations:** Required safety limit
- **timeout_secs:** Optional overall timeout
- **delay_between_secs:** Optional delay to avoid tight loops

**Example: Poll API Until Ready**
```yaml
tasks:
  wait_for_deployment:
    description: "Checking deployment status (attempt {{iteration}})"
    agent: "poller"
    loop:
      type: while
      condition:
        type: state_equals
        key: "deployment_ready"
        value: false
      max_iterations: 60
      iteration_variable: "iteration"
      delay_between_secs: 10  # Check every 10 seconds
    loop_control:
      timeout_secs: 600  # 10 minute max
```

---

### 3. RepeatUntil Loop

**Purpose:** Execute until a condition is true (condition checked AFTER each iteration).

**YAML Syntax:**
```yaml
tasks:
  retry_operation:
    description: "Attempting operation (iteration {{iteration}})"
    agent: "worker"
    loop:
      type: repeat_until
      condition:
        type: state_equals
        key: "operation_success"
        value: true
      min_iterations: 1  # Optional: minimum iterations
      max_iterations: 10  # Required: safety limit
      iteration_variable: "iteration"  # Optional
      delay_between_secs: 2  # Optional: exponential backoff
    loop_control:
      timeout_secs: 60
```

**When to Use:**
- Retry operations with condition check
- Operations that must run at least once
- Exponential backoff retry logic
- Validation loops (do-while pattern)

**Difference from While:**
- **While:** Checks condition BEFORE iteration (may never execute)
- **RepeatUntil:** Checks condition AFTER iteration (executes at least once)

**Example: Retry API Call**
```yaml
tasks:
  fetch_data_with_retry:
    description: "Fetching data (attempt {{iteration}})"
    agent: "fetcher"
    loop:
      type: repeat_until
      condition:
        type: state_equals
        key: "fetch_success"
        value: true
      min_iterations: 1
      max_iterations: 5
      iteration_variable: "iteration"
      delay_between_secs: 2  # 2, 4, 8, 16 seconds (if using exponential)
    loop_control:
      timeout_secs: 60
```

---

### 4. Repeat Loop

**Purpose:** Execute a fixed number of iterations.

**YAML Syntax:**
```yaml
tasks:
  batch_process:
    description: "Processing batch {{batch_num}}"
    agent: "processor"
    loop:
      type: repeat
      count: 10  # Execute 10 times
      iterator: "batch_num"  # Optional: variable for iteration number
      parallel: false  # Optional: enable parallel execution
      max_parallel: 3  # Optional: limit concurrency
```

**When to Use:**
- Fixed number of iterations
- Batch processing with known count
- Stress testing (run N times)
- Parallel task execution

**Execution Modes:**

**Sequential:**
```yaml
loop:
  type: repeat
  count: 5
  iterator: "iteration"
  parallel: false
```

**Parallel:**
```yaml
loop:
  type: repeat
  count: 10
  iterator: "iteration"
  parallel: true
  max_parallel: 5
```

**Example: Parallel Batch Processing**
```yaml
tasks:
  process_batches:
    description: "Processing batch {{batch}}/10"
    agent: "processor"
    loop:
      type: repeat
      count: 10
      iterator: "batch"
      parallel: true
      max_parallel: 3  # Process 3 batches concurrently
    loop_control:
      collect_results: true
      result_key: "batch_results"
```

---

## Collection Sources

### 1. Inline Collections

**Hardcoded arrays in YAML.**

```yaml
collection:
  source: inline
  items: ["file1.txt", "file2.txt", "file3.txt"]
```

**Use When:**
- Small, static lists
- Configuration-driven workflows
- Testing and examples

---

### 2. State Collections

**Arrays stored in workflow state.**

```yaml
collection:
  source: state
  key: "file_list"  # State key containing array
```

**Use When:**
- Dynamic collections from previous tasks
- Results from earlier iterations
- Runtime-determined items

**Example:**
```yaml
tasks:
  fetch_files:
    description: "List files"
    agent: "lister"
    # This task stores results in state["file_list"]

  process_files:
    description: "Process {{file}}"
    agent: "processor"
    depends_on: [fetch_files]
    loop:
      type: for_each
      collection:
        source: state
        key: "file_list"
      iterator: "file"
```

---

### 3. File Collections

**Arrays loaded from files.**

**JSON:**
```yaml
collection:
  source: file
  path: "data/items.json"
  format: json
```

**JSON Lines (one JSON object per line):**
```yaml
collection:
  source: file
  path: "data/items.jsonl"
  format: json_lines
```

**CSV:**
```yaml
collection:
  source: file
  path: "data/items.csv"
  format: csv  # Each row becomes an array
```

**Plain text lines:**
```yaml
collection:
  source: file
  path: "data/urls.txt"
  format: lines  # Each line becomes a string
```

**Use When:**
- Large datasets
- External data sources
- Batch jobs with input files

---

### 4. Range Collections

**Numeric ranges.**

```yaml
collection:
  source: range
  start: 0
  end: 100
  step: 1  # Optional, default: 1
```

**Use When:**
- Numeric iteration (0 to N)
- Batch number generation
- Pagination (process pages 1-10)

**Example: Process Pages**
```yaml
loop:
  type: for_each
  collection:
    source: range
    start: 1
    end: 11  # Pages 1-10
    step: 1
  iterator: "page"
```

---

### 5. HTTP Collections

**Arrays fetched from HTTP APIs.**

**Basic GET:**
```yaml
collection:
  source: http
  url: "https://api.example.com/items"
  method: "GET"
  format: json
```

**With Headers:**
```yaml
collection:
  source: http
  url: "https://api.example.com/items"
  method: "GET"
  headers:
    Authorization: "Bearer token123"
    Accept: "application/json"
  format: json
```

**With JSON Path:**
```yaml
collection:
  source: http
  url: "https://api.example.com/data"
  method: "GET"
  format: json
  json_path: "data.items"  # Extract nested array
```

**POST Request:**
```yaml
collection:
  source: http
  url: "https://api.example.com/search"
  method: "POST"
  headers:
    Content-Type: "application/json"
  body: '{"query": "rust programming"}'
  format: json
```

**Use When:**
- REST API results
- External data sources
- Dynamic collections from APIs
- Paginated API responses

---

## Loop Control Features

### 1. Break Condition

**Exit loop early when condition is met.**

```yaml
loop_control:
  break_condition:
    type: state_equals
    key: "error_found"
    value: true
```

**Evaluated:** AFTER each iteration
**Effect:** Stops loop immediately

**Example:**
```yaml
tasks:
  scan_files:
    description: "Scanning {{file}}"
    agent: "scanner"
    loop:
      type: for_each
      collection:
        source: state
        key: "files"
      iterator: "file"
    loop_control:
      break_condition:
        type: state_equals
        key: "malware_found"
        value: true
```

---

### 2. Continue Condition

**Skip iteration when condition is met.**

```yaml
loop_control:
  continue_condition:
    type: state_equals
    key: "skip_this"
    value: true
```

**Evaluated:** BEFORE each iteration
**Effect:** Skips current iteration, continues to next

**Example:**
```yaml
tasks:
  process_files:
    description: "Processing {{file}}"
    agent: "processor"
    loop:
      type: for_each
      collection:
        source: state
        key: "files"
      iterator: "file"
    loop_control:
      continue_condition:
        type: state_equals
        key: "file_processed"
        value: true  # Skip already processed files
```

---

### 3. Timeout

**Limit total loop execution time.**

```yaml
loop_control:
  timeout_secs: 300  # 5 minutes max
```

**Behavior:**
- Applies to entire loop
- Cancels current iteration on timeout
- Returns timeout error

**Example:**
```yaml
tasks:
  poll_service:
    description: "Polling service"
    agent: "poller"
    loop:
      type: while
      condition:
        type: state_equals
        key: "service_ready"
        value: false
      max_iterations: 100
      delay_between_secs: 3
    loop_control:
      timeout_secs: 300  # Don't wait more than 5 minutes
```

---

### 4. Checkpoint Interval

**Save state periodically for resume capability.**

```yaml
loop_control:
  checkpoint_interval: 10  # Save every 10 iterations
```

**Behavior:**
- Saves state to disk after N iterations
- Enables resume after interruption
- Skips completed iterations on resume

**Example:**
```yaml
tasks:
  process_large_batch:
    description: "Processing item {{item}}"
    agent: "processor"
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
      result_key: "processed_items"
```

---

### 5. Result Collection

**Collect outputs from iterations.**

```yaml
loop_control:
  collect_results: true
  result_key: "my_results"  # Store in state["my_results"]
```

**Behavior:**
- Collects iteration outputs into array
- Stores in workflow state under result_key
- Available to subsequent tasks

**Example:**
```yaml
tasks:
  transform_data:
    description: "Transforming {{item}}"
    agent: "transformer"
    loop:
      type: for_each
      collection:
        source: state
        key: "raw_data"
      iterator: "item"
    loop_control:
      collect_results: true
      result_key: "transformed_data"

  analyze_results:
    description: "Analyze transformed data"
    agent: "analyzer"
    depends_on: [transform_data]
    # Can access state["transformed_data"]
```

---

## Variable Substitution

### Iteration Variables

Loop variables can be used in task fields:

**In Description:**
```yaml
description: "Processing {{item.name}} ({{iteration}}/{{total}})"
```

**In Output Path:**
```yaml
output: "results/{{item.id}}.json"
```

**In Conditions:**
```yaml
condition:
  type: state_equals
  key: "current_{{iterator}}_status"
  value: "ready"
```

### Available Variables

**ForEach Loops:**
- `{{iterator}}` - Current item value
- `{{iteration}}` - Current iteration number (0-based)
- `{{item.field}}` - Object field access (if item is object)

**Repeat Loops:**
- `{{iterator}}` - Current iteration number
- `{{iteration}}` - Same as iterator

**While/RepeatUntil Loops:**
- `{{iteration}}` - Current iteration number (if iteration_variable set)

### Examples

**Process files with numbering:**
```yaml
tasks:
  process_file:
    description: "Processing file {{file}} ({{iteration}}/{{total}})"
    agent: "processor"
    output: "results/file_{{iteration}}.json"
    loop:
      type: for_each
      collection:
        source: inline
        items: ["a.txt", "b.txt", "c.txt"]
      iterator: "file"
```

**Access nested object fields:**
```yaml
tasks:
  process_user:
    description: "Processing user {{user.name}} (ID: {{user.id}})"
    agent: "processor"
    loop:
      type: for_each
      collection:
        source: http
        url: "https://api.example.com/users"
        format: json
      iterator: "user"
```

---

## Best Practices

### 1. Always Set Safety Limits

**DO:**
```yaml
loop:
  type: while
  condition: ...
  max_iterations: 100  # ALWAYS set max
  delay_between_secs: 5
loop_control:
  timeout_secs: 600  # Add timeout for extra safety
```

**DON'T:**
```yaml
loop:
  type: while
  condition: ...
  # Missing max_iterations - UNSAFE!
```

---

### 2. Use Checkpoints for Long Loops

**DO:**
```yaml
loop:
  type: for_each
  collection:
    source: range
    start: 0
    end: 10000
  iterator: "item"
loop_control:
  checkpoint_interval: 100  # Save every 100 iterations
```

**DON'T:**
```yaml
loop:
  type: for_each
  collection:
    source: range
    start: 0
    end: 10000
  iterator: "item"
  # No checkpointing - all progress lost on failure
```

---

### 3. Limit Parallel Concurrency

**DO:**
```yaml
loop:
  type: for_each
  collection: ...
  iterator: "item"
  parallel: true
  max_parallel: 5  # Reasonable limit
```

**DON'T:**
```yaml
loop:
  type: for_each
  collection: ...  # 1000 items
  iterator: "item"
  parallel: true
  # No max_parallel - may spawn 1000 concurrent tasks!
```

---

### 4. Add Delays to Polling Loops

**DO:**
```yaml
loop:
  type: while
  condition: ...
  max_iterations: 60
  delay_between_secs: 5  # Check every 5 seconds
```

**DON'T:**
```yaml
loop:
  type: while
  condition: ...
  max_iterations: 1000
  # No delay - tight loop, wastes resources
```

---

### 5. Collect Results When Needed

**DO:**
```yaml
loop_control:
  collect_results: true  # Need results for next task
  result_key: "outputs"
```

**DON'T:**
```yaml
loop_control:
  collect_results: true  # Results never used
  # Wastes memory collecting unnecessary data
```

---

### 6. Use Appropriate Collection Sources

**DO:**
```yaml
# Use HTTP for dynamic API data
collection:
  source: http
  url: "https://api.example.com/items"

# Use state for previous task results
collection:
  source: state
  key: "previous_results"
```

**DON'T:**
```yaml
# Don't hardcode large collections
collection:
  source: inline
  items: [1, 2, 3, ... 1000]  # Use range or file instead!
```

---

## Common Pitfalls

### 1. Infinite Loops

**Problem:** While loop with condition that never becomes false.

**Example:**
```yaml
loop:
  type: while
  condition:
    type: state_equals
    key: "always_true"
    value: true
  max_iterations: 1000000  # Too high!
```

**Solution:** Set reasonable max_iterations and add timeout.

---

### 2. Resource Exhaustion

**Problem:** Unbounded parallel execution.

**Example:**
```yaml
loop:
  type: for_each
  collection:
    source: range
    start: 0
    end: 100000  # 100k items!
  iterator: "item"
  parallel: true  # All at once!
```

**Solution:** Always set max_parallel.

---

### 3. Missing Error Handling

**Problem:** Loop continues despite errors.

**Example:**
```yaml
loop:
  type: for_each
  collection: ...
  iterator: "item"
  # No break_condition for errors
```

**Solution:** Add break condition for critical errors.

---

### 4. Forgotten Checkpoints

**Problem:** Long loop loses all progress on failure.

**Example:**
```yaml
loop:
  type: for_each
  collection:
    source: range
    start: 0
    end: 10000
  iterator: "item"
  # No checkpoint_interval
```

**Solution:** Add checkpoint_interval for loops > 100 iterations.

---

### 5. Tight Polling Loops

**Problem:** Polling without delay wastes resources.

**Example:**
```yaml
loop:
  type: while
  condition: ...
  max_iterations: 10000
  # No delay_between_secs!
```

**Solution:** Always add delay_between_secs for while/repeat_until.

---

## Performance Considerations

### 1. Parallel vs Sequential

**Sequential (default):**
- One iteration at a time
- Predictable resource usage
- Maintains order

**Parallel:**
- Multiple iterations concurrently
- Faster for I/O-bound tasks
- Higher resource usage
- No guaranteed order

**Choose Parallel When:**
- Iterations are independent
- I/O-bound operations (API calls, file I/O)
- Time is critical
- Resources are available

**Choose Sequential When:**
- Order matters
- Shared state mutations
- Resource-constrained
- Debugging

---

### 2. Collection Size Limits

**Hard Limits:**
- Max collection size: 100,000 items
- Max iterations per loop: 10,000
- Max parallel iterations: 100

**Recommendations:**
- Keep collections < 1,000 items for best performance
- Use pagination for larger datasets
- Split large jobs into multiple workflows

---

### 3. Checkpoint Frequency

**Checkpoint Interval Guidelines:**
- **Fast iterations (<1s):** Every 100-1000 iterations
- **Medium iterations (1-10s):** Every 10-100 iterations
- **Slow iterations (>10s):** Every 1-10 iterations

**Trade-offs:**
- More frequent: Better resume granularity, more I/O overhead
- Less frequent: Lower overhead, more lost work on failure

---

### 4. State Collection Memory

**Memory Usage:**
- Each collected result stored in memory
- Large results accumulate quickly

**Best Practices:**
- Only collect results when needed
- Consider streaming for large datasets
- Clear state after processing

---

## Security Considerations

### 1. Loop Bombs

**Definition:** Malicious loops designed to exhaust resources.

**Protection:**
- Hard-coded MAX_LOOP_ITERATIONS (10,000)
- Hard-coded MAX_COLLECTION_SIZE (100,000)
- Required max_iterations for while/repeat_until
- Timeout enforcement

**Example Blocked Loop Bomb:**
```yaml
loop:
  type: repeat
  count: 999999999  # Rejected - exceeds MAX_LOOP_ITERATIONS
```

---

### 2. API Rate Limiting

**Problem:** HTTP collections can trigger rate limits.

**Solutions:**
- Add delays between iterations
- Use max_parallel to limit concurrency
- Implement backoff strategies
- Cache responses

**Example:**
```yaml
loop:
  type: for_each
  collection:
    source: http
    url: "https://api.example.com/items"
  iterator: "item"
  parallel: true
  max_parallel: 2  # Respect API rate limits
loop_control:
  timeout_secs: 600
```

---

### 3. Input Validation

**Always validate:**
- Collection sources (URLs, file paths)
- Loop parameters (counts, iterations)
- User-provided data in collections

**Validation Checks:**
- URL must start with http:// or https://
- File paths must be safe (no directory traversal)
- Counts must be within limits
- Methods must be whitelisted

---

## Summary

The DSL loop system provides powerful, safe, and flexible iteration capabilities:

✅ **Four loop patterns** - ForEach, While, RepeatUntil, Repeat
✅ **Five collection sources** - Inline, State, File, Range, HTTP
✅ **Advanced control** - Break, continue, timeout, checkpoints
✅ **Parallel execution** - Concurrent iterations with limits
✅ **State persistence** - Resume capability
✅ **Safety features** - Resource limits, timeouts, validation

Follow best practices and avoid common pitfalls for production-ready workflows!

---

**Last Updated:** 2025-10-19
**Version:** 1.0
**Status:** ✅ Production Ready

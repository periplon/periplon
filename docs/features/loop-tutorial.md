# DSL Loop Tutorial

**Version:** 1.0
**Date:** 2025-10-19
**Target Audience:** Beginners to Intermediate

Learn how to use loops in DSL workflows through hands-on examples, progressing from simple to advanced patterns.

---

## Table of Contents

1. [Introduction](#introduction)
2. [Tutorial 1: Your First Loop](#tutorial-1-your-first-loop)
3. [Tutorial 2: Processing Files](#tutorial-2-processing-files)
4. [Tutorial 3: Fetching from APIs](#tutorial-3-fetching-from-apis)
5. [Tutorial 4: Polling and Waiting](#tutorial-4-polling-and-waiting)
6. [Tutorial 5: Parallel Processing](#tutorial-5-parallel-processing)
7. [Tutorial 6: Error Handling](#tutorial-6-error-handling)
8. [Tutorial 7: Checkpointing Long Jobs](#tutorial-7-checkpointing-long-jobs)
9. [Next Steps](#next-steps)

---

## Introduction

### What You'll Learn

By the end of this tutorial, you'll be able to:
- Create basic loops to process collections
- Use different loop types (ForEach, While, Repeat)
- Fetch data from APIs and process results
- Handle errors and retry operations
- Use parallel execution for performance
- Checkpoint long-running jobs

### Prerequisites

- Basic understanding of YAML syntax
- Familiarity with DSL workflow concepts (agents, tasks)
- Rust and cargo installed (for running examples)

### Setup

Clone the repository and navigate to the examples directory:

```bash
cd periplon
cargo build
```

---

## Tutorial 1: Your First Loop

**Goal:** Create a simple loop that prints numbers 1 through 5.

### Step 1: Create the Workflow

Create a file `examples/workflows/tutorial1_first_loop.yaml`:

```yaml
name: "Tutorial 1: First Loop"
version: "1.0.0"
description: "Print numbers 1 through 5"

agents:
  printer:
    description: "Print messages"
    tools: []
    permissions:
      mode: "default"

tasks:
  print_numbers:
    description: "Printing number {{num}}"
    agent: "printer"
    loop:
      type: for_each
      collection:
        source: inline
        items: [1, 2, 3, 4, 5]
      iterator: "num"
```

### Step 2: Understanding the Code

Let's break down each part:

**Loop Type:**
```yaml
type: for_each
```
- `for_each` means "do this task for each item in a collection"
- Like a `for` loop in programming languages

**Collection Source:**
```yaml
collection:
  source: inline
  items: [1, 2, 3, 4, 5]
```
- `source: inline` means the items are listed right here in the YAML
- `items:` is the list we'll iterate over

**Iterator:**
```yaml
iterator: "num"
```
- `num` is the variable name for the current item
- You can use `{{num}}` in task description and other fields

**Variable Substitution:**
```yaml
description: "Printing number {{num}}"
```
- `{{num}}` gets replaced with the current item value
- Iteration 1: "Printing number 1"
- Iteration 2: "Printing number 2"
- And so on...

### Step 3: Run It

```bash
cargo run --example dsl_executor_example examples/workflows/tutorial1_first_loop.yaml
```

**Expected Output:**
```
Executing task: print_numbers - Printing number 1
Executing task: print_numbers - Printing number 2
Executing task: print_numbers - Printing number 3
Executing task: print_numbers - Printing number 4
Executing task: print_numbers - Printing number 5
```

### What You Learned

✅ How to create a basic ForEach loop
✅ How to use inline collections
✅ How to use variable substitution ({{iterator}})
✅ Loop executes once per item

---

## Tutorial 2: Processing Files

**Goal:** Process multiple files from a list.

### Step 1: Create File List

Create `examples/data/files.txt`:
```
report1.txt
report2.txt
report3.txt
```

### Step 2: Create the Workflow

Create `examples/workflows/tutorial2_process_files.yaml`:

```yaml
name: "Tutorial 2: Process Files"
version: "1.0.0"

agents:
  file_processor:
    description: "Process text files"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  process_files:
    description: "Processing file {{file}} ({{iteration}}/{{total}})"
    agent: "file_processor"
    loop:
      type: for_each
      collection:
        source: file
        path: "examples/data/files.txt"
        format: lines
      iterator: "file"
    loop_control:
      collect_results: true
      result_key: "processed_files"
```

### Step 3: Understanding New Concepts

**File Collection Source:**
```yaml
collection:
  source: file
  path: "examples/data/files.txt"
  format: lines
```
- Reads items from a file instead of hardcoding
- `format: lines` means each line is one item
- Other formats: `json`, `json_lines`, `csv`

**Loop Control:**
```yaml
loop_control:
  collect_results: true
  result_key: "processed_files"
```
- `collect_results: true` saves the output from each iteration
- `result_key` is where results are stored in workflow state
- Later tasks can access `state["processed_files"]`

**Progress Tracking:**
```yaml
description: "Processing file {{file}} ({{iteration}}/{{total}})"
```
- `{{iteration}}` is the current iteration number (0-based)
- `{{total}}` is the total number of items
- Helps track progress

### Step 4: Run It

```bash
cargo run --example dsl_executor_example examples/workflows/tutorial2_process_files.yaml
```

### What You Learned

✅ How to read collections from files
✅ How to collect results from iterations
✅ How to track progress with {{iteration}}
✅ File format options (lines, json, csv)

---

## Tutorial 3: Fetching from APIs

**Goal:** Fetch data from a REST API and process each item.

### Step 1: Create the Workflow

Create `examples/workflows/tutorial3_api_fetch.yaml`:

```yaml
name: "Tutorial 3: API Fetch"
version: "1.0.0"

agents:
  api_processor:
    description: "Fetch and process API data"
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
      result_key: "users"
      timeout_secs: 60
```

### Step 2: Understanding HTTP Collections

**HTTP Source:**
```yaml
collection:
  source: http
  url: "https://jsonplaceholder.typicode.com/users"
  method: "GET"
  format: json
```
- Fetches data from HTTP API
- `method` can be GET, POST, PUT, DELETE, PATCH
- `format: json` expects JSON array response

**Accessing Object Fields:**
```yaml
description: "Processing user {{user.name}} (ID: {{user.id}})"
```
- If items are objects, use dot notation
- `{{user.name}}` accesses the `name` field
- `{{user.id}}` accesses the `id` field

**Timeout Protection:**
```yaml
loop_control:
  timeout_secs: 60
```
- Loop will timeout after 60 seconds
- Prevents hanging on slow/broken APIs
- Always add timeouts for HTTP operations

### Step 3: Advanced - With Headers

```yaml
collection:
  source: http
  url: "https://api.github.com/repos/rust-lang/rust/issues"
  method: "GET"
  headers:
    Accept: "application/vnd.github.v3+json"
    User-Agent: "DSL-Workflow"
  format: json
```

### Step 4: Advanced - POST with Body

```yaml
collection:
  source: http
  url: "https://api.example.com/search"
  method: "POST"
  headers:
    Content-Type: "application/json"
  body: '{"query": "rust programming", "limit": 10}'
  format: json
```

### What You Learned

✅ How to fetch data from HTTP APIs
✅ How to access object fields with dot notation
✅ How to add custom headers
✅ How to make POST requests
✅ Always add timeouts for network operations

---

## Tutorial 4: Polling and Waiting

**Goal:** Poll an API until a condition is met.

### Step 1: Create the Workflow

Create `examples/workflows/tutorial4_polling.yaml`:

```yaml
name: "Tutorial 4: Polling"
version: "1.0.0"

agents:
  poller:
    description: "Poll for job completion"
    tools: [WebFetch]
    permissions:
      mode: "default"

tasks:
  check_status:
    description: "Checking status (attempt {{iteration}})"
    agent: "poller"
    loop:
      type: while
      condition:
        type: state_equals
        key: "job_complete"
        value: false
      max_iterations: 30
      iteration_variable: "iteration"
      delay_between_secs: 5
    loop_control:
      timeout_secs: 180
```

### Step 2: Understanding While Loops

**While Loop:**
```yaml
type: while
condition:
  type: state_equals
  key: "job_complete"
  value: false
```
- Executes WHILE condition is true
- Checks condition BEFORE each iteration
- Stops when condition becomes false

**Safety Limits:**
```yaml
max_iterations: 30
```
- REQUIRED for while loops
- Prevents infinite loops
- Hard limit: 10,000 iterations

**Polling Delay:**
```yaml
delay_between_secs: 5
```
- Wait 5 seconds between iterations
- Prevents tight loops
- Respects API rate limits
- ALWAYS add delays for polling!

**Progress Tracking:**
```yaml
iteration_variable: "iteration"
```
- Creates `{{iteration}}` variable
- Tracks how many times loop has run
- Useful for logging

### Step 3: Understanding RepeatUntil

```yaml
tasks:
  retry_operation:
    description: "Attempt {{iteration}}"
    agent: "worker"
    loop:
      type: repeat_until
      condition:
        type: state_equals
        key: "success"
        value: true
      min_iterations: 1
      max_iterations: 5
      delay_between_secs: 2
```

**Difference from While:**
- While: Check condition BEFORE iteration (may not execute)
- RepeatUntil: Check condition AFTER iteration (always executes once)
- Use RepeatUntil for retry logic

### What You Learned

✅ How to create polling loops with While
✅ How to use RepeatUntil for retry logic
✅ Always set max_iterations for safety
✅ Always add delays to prevent tight loops
✅ Use timeouts for network operations

---

## Tutorial 5: Parallel Processing

**Goal:** Process multiple items concurrently for better performance.

### Step 1: Sequential vs Parallel

**Sequential (Default):**
```yaml
loop:
  type: for_each
  collection:
    source: inline
    items: [1, 2, 3, 4, 5]
  iterator: "num"
  parallel: false  # One at a time
```
- Processes one item at a time
- Total time: 5 × item_time

**Parallel:**
```yaml
loop:
  type: for_each
  collection:
    source: inline
    items: [1, 2, 3, 4, 5]
  iterator: "num"
  parallel: true
  max_parallel: 3  # At most 3 concurrent
```
- Processes multiple items concurrently
- Total time: ≈ 2 × item_time (with max_parallel: 3)

### Step 2: Create Parallel Workflow

Create `examples/workflows/tutorial5_parallel.yaml`:

```yaml
name: "Tutorial 5: Parallel Processing"
version: "1.0.0"

agents:
  fetcher:
    description: "Fetch data in parallel"
    tools: [WebFetch]
    permissions:
      mode: "default"

tasks:
  fetch_multiple:
    description: "Fetching page {{page}}"
    agent: "fetcher"
    loop:
      type: for_each
      collection:
        source: range
        start: 1
        end: 11  # Pages 1-10
      iterator: "page"
      parallel: true
      max_parallel: 3  # Fetch 3 pages at once
    loop_control:
      collect_results: true
      result_key: "all_pages"
      timeout_secs: 120
```

### Step 3: Understanding Concurrency Control

**max_parallel:**
```yaml
max_parallel: 3
```
- Limits concurrent iterations
- Hard limit: 100 concurrent
- Choose based on:
  - API rate limits
  - System resources
  - Network bandwidth

**Best Practices:**
- Start with low concurrency (2-5)
- Increase gradually
- Monitor system resources
- Respect API rate limits

**Example Concurrency Levels:**
- CPU-bound: max_parallel = num_cores
- I/O-bound (APIs): max_parallel = 5-10
- Database: max_parallel = 2-5

### Step 4: When to Use Parallel

**Use Parallel:**
✅ I/O-bound operations (API calls, file I/O)
✅ Independent iterations
✅ Time is critical
✅ Resources available

**Use Sequential:**
✅ Order matters
✅ Shared state mutations
✅ Resource-constrained
✅ Debugging

### What You Learned

✅ How to enable parallel execution
✅ How to limit concurrency with max_parallel
✅ When to use parallel vs sequential
✅ Best practices for concurrency levels

---

## Tutorial 6: Error Handling

**Goal:** Handle errors gracefully and retry failed operations.

### Step 1: Break on Error

Create `examples/workflows/tutorial6_error_handling.yaml`:

```yaml
name: "Tutorial 6: Error Handling"
version: "1.0.0"

agents:
  processor:
    description: "Process with error handling"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  process_with_break:
    description: "Processing {{item}}"
    agent: "processor"
    loop:
      type: for_each
      collection:
        source: inline
        items: [1, 2, 3, 4, 5]
      iterator: "item"
    loop_control:
      break_condition:
        type: state_equals
        key: "critical_error"
        value: true
      collect_results: true
```

### Step 2: Understanding Break Condition

**Break Condition:**
```yaml
break_condition:
  type: state_equals
  key: "critical_error"
  value: true
```
- Checked AFTER each iteration
- Stops loop immediately if true
- Use for critical errors

**Example Flow:**
1. Iteration 1: OK, critical_error = false
2. Iteration 2: OK, critical_error = false
3. Iteration 3: ERROR, sets critical_error = true
4. Break condition triggers, loop stops
5. Iterations 4-5 never execute

### Step 3: Skip on Error (Continue)

```yaml
loop_control:
  continue_condition:
    type: state_equals
    key: "skip_item"
    value: true
```
- Checked BEFORE each iteration
- Skips iteration if true
- Continues to next item
- Use for non-critical errors

### Step 4: Retry Pattern

```yaml
tasks:
  retry_operation:
    description: "Attempt {{iteration}}/5"
    agent: "worker"
    loop:
      type: repeat_until
      condition:
        type: state_equals
        key: "success"
        value: true
      min_iterations: 1
      max_iterations: 5
      delay_between_secs: 2  # Exponential backoff
    loop_control:
      timeout_secs: 60
```

**Retry with Exponential Backoff:**
- Attempt 1: 0s delay
- Attempt 2: 2s delay
- Attempt 3: 4s delay
- Attempt 4: 8s delay
- Attempt 5: 16s delay

### What You Learned

✅ How to use break_condition to stop on errors
✅ How to use continue_condition to skip items
✅ How to implement retry logic
✅ Exponential backoff pattern

---

## Tutorial 7: Checkpointing Long Jobs

**Goal:** Process large dataset with resume capability.

### Step 1: Create Long-Running Job

Create `examples/workflows/tutorial7_checkpointing.yaml`:

```yaml
name: "Tutorial 7: Checkpointing"
version: "1.0.0"

agents:
  processor:
    description: "Process large dataset"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  process_large_dataset:
    description: "Processing item {{item}}/1000"
    agent: "processor"
    loop:
      type: for_each
      collection:
        source: range
        start: 0
        end: 1000
      iterator: "item"
    loop_control:
      checkpoint_interval: 50  # Save every 50 items
      collect_results: true
      result_key: "results"
```

### Step 2: Understanding Checkpointing

**Checkpoint Interval:**
```yaml
checkpoint_interval: 50
```
- Saves state to disk every 50 iterations
- Creates `.state` directory
- Stores completed iteration numbers

**Resume Capability:**
```bash
# First run (interrupted after 200 items)
cargo run --example dsl_executor_example workflow.yaml
^C  # Ctrl+C to interrupt

# Resume (skips first 200 items)
cargo run --example dsl_executor_example workflow.yaml
# Output: "Already completed, skipping (resume)"
```

**How Resume Works:**
1. On start, check for .state directory
2. Load saved checkpoint
3. For each iteration:
   - Check if already completed
   - Skip if yes, execute if no
4. Continue from where left off

### Step 3: Choosing Checkpoint Interval

**Guidelines:**
- Fast iterations (<1s): Every 100-1000
- Medium iterations (1-10s): Every 10-100
- Slow iterations (>10s): Every 1-10

**Trade-offs:**
- More frequent: Better resume, more I/O
- Less frequent: Lower overhead, more lost work

**Example:**
```yaml
# Processing 10,000 items, 1s each
checkpoint_interval: 100
# Saves every 100s
# Max lost work: 100s on failure
```

### Step 4: State Persistence

The executor handles state automatically:

```rust
// In your demo program
let mut executor = DSLExecutor::new(workflow)?;
executor.enable_state_persistence(Some(".state"))?;
executor.initialize().await?;

// Automatically resumes from checkpoint
executor.execute().await?;
```

### What You Learned

✅ How to add checkpointing to long loops
✅ How resume works automatically
✅ How to choose checkpoint intervals
✅ State is saved to .state directory

---

## Next Steps

### Practice Projects

1. **File Organizer**
   - List files in directory
   - Categorize by extension
   - Move to organized folders
   - Use parallel processing

2. **API Aggregator**
   - Fetch from multiple APIs
   - Transform data format
   - Combine results
   - Save to file

3. **Data Pipeline**
   - Extract from source
   - Transform with rules
   - Load to destination
   - Add checkpointing

### Advanced Topics

**Read these guides:**
- [Loop Patterns Guide](loop-patterns.md) - Comprehensive reference
- [Loop Cookbook](loop-cookbook.md) - 25 real-world patterns
- [Security Audit](SECURITY_AUDIT.md) - Safety and security

**Advanced Features:**
- Nested loops
- Complex conditions
- JSON path extraction
- Custom collection sources

### Common Patterns

**Map-Reduce:**
```yaml
tasks:
  map:
    loop:
      type: for_each
      collection: ...
      parallel: true
    loop_control:
      collect_results: true

  reduce:
    depends_on: [map]
    # Process map results
```

**Fan-Out / Fan-In:**
```yaml
tasks:
  fan_out:
    loop:
      type: for_each
      collection: ...
      parallel: true
      max_parallel: 10
    loop_control:
      collect_results: true

  fan_in:
    depends_on: [fan_out]
    # Aggregate parallel results
```

**ETL Pipeline:**
```yaml
tasks:
  extract:
    loop:
      type: for_each
      collection:
        source: http
    loop_control:
      collect_results: true

  transform:
    depends_on: [extract]
    loop:
      type: for_each
      collection:
        source: state
        key: "extracted_data"
    loop_control:
      collect_results: true

  load:
    depends_on: [transform]
    loop:
      type: for_each
      collection:
        source: state
        key: "transformed_data"
```

---

## Summary

You've learned how to:

✅ Create basic ForEach loops
✅ Use different collection sources (inline, file, HTTP, range)
✅ Fetch and process API data
✅ Poll with While loops
✅ Retry with RepeatUntil
✅ Process data in parallel
✅ Handle errors with break/continue
✅ Checkpoint long-running jobs

**Key Takeaways:**
- Always set safety limits (max_iterations, max_parallel, timeout)
- Use delays for polling loops
- Add checkpoints for long jobs (>100 iterations)
- Limit parallel concurrency appropriately
- Collect results only when needed

**Next:** Read the [Loop Patterns Guide](loop-patterns.md) for comprehensive reference and the [Loop Cookbook](loop-cookbook.md) for 25 production-ready patterns!

---

**Happy Looping! 🔄**

---

**Last Updated:** 2025-10-19
**Version:** 1.0
**Status:** ✅ Production Ready

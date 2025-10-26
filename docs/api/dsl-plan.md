# Agentic AI DSL - Comprehensive Design Plan

## 1. Executive Summary

This document outlines a comprehensive plan for designing and implementing a Domain-Specific Language (DSL) for creating agentic AI systems using the periplon. The DSL will enable users to define complex multi-agent workflows with hierarchical task decomposition, tool usage, and inter-agent collaboration through an intuitive, declarative syntax.

### Design Goals
- **Simplicity**: Easy to read and write for users with varying expertise levels
- **Expressiveness**: Powerful enough to handle simple automation to complex problem-solving
- **Composability**: Support hierarchical task systems and agent collaboration
- **Type Safety**: Leverage Rust's type system for compile-time validation
- **Testability**: Comprehensive testing framework for DSL workflows

---

## 2. DSL Architecture

### 2.1 Format Choice: YAML

**Rationale**: YAML provides:
- Human-readable syntax
- Native support in Rust ecosystem (serde_yaml)
- Hierarchical structure for nested tasks
- Comments for documentation
- Wide adoption in configuration files

**Alternative considered**: TOML (simpler but less hierarchical), Custom syntax (more control but steeper learning curve)

### 2.2 Core Components

The DSL will consist of five primary constructs:

1. **Agents**: Define AI agents with capabilities, permissions, and behavior
2. **Tasks**: Describe work units with hierarchical decomposition support
3. **Tools**: Configure available tools and constraints
4. **Workflows**: Orchestrate agent collaboration and task execution
5. **Communication**: Define message passing and coordination protocols

---

## 3. DSL Syntax Specification

### 3.1 Agent Definition

```yaml
agents:
  researcher:
    description: "Research and gather information from various sources"
    model: "claude-sonnet-4-5"
    system_prompt: |
      You are a research specialist. Gather comprehensive information
      and synthesize findings into clear, actionable insights.
    tools:
      - Read
      - WebSearch
      - Grep
      - Glob
    permissions:
      mode: "default"  # default | acceptEdits | plan | bypassPermissions
      allowed_directories:
        - "./research"
        - "./data"
    max_turns: 10

  coder:
    description: "Write, review, and refactor code"
    model: "claude-sonnet-4-5"
    system_prompt: |
      You are an expert software engineer. Write clean, efficient,
      well-tested code following best practices.
    tools:
      - Read
      - Write
      - Edit
      - Bash
      - Grep
      - Glob
    permissions:
      mode: "acceptEdits"
    max_turns: 20

  reviewer:
    description: "Review code for quality, security, and best practices"
    model: "claude-opus-4"
    system_prompt: "You are a code review expert. Provide thorough, constructive feedback."
    tools:
      - Read
      - Grep
    permissions:
      mode: "default"
    max_turns: 5
```

### 3.2 Hierarchical Task System

```yaml
tasks:
  build_web_app:
    description: "Build a full-stack web application"
    agent: "coder"
    priority: 1

    # Hierarchical subtasks
    subtasks:
      - research_requirements:
          description: "Research tech stack and requirements"
          agent: "researcher"
          priority: 1

          subtasks:
            - analyze_frameworks:
                description: "Compare web frameworks"
                agent: "researcher"
                output: "framework_analysis.md"

            - analyze_databases:
                description: "Evaluate database options"
                agent: "researcher"
                output: "database_analysis.md"

      - design_architecture:
          description: "Design system architecture"
          agent: "coder"
          priority: 2
          depends_on:
            - research_requirements
          output: "architecture.md"

      - implement_backend:
          description: "Implement backend services"
          agent: "coder"
          priority: 3
          depends_on:
            - design_architecture

          subtasks:
            - setup_project:
                description: "Initialize project structure"
                agent: "coder"

            - implement_api:
                description: "Create REST API endpoints"
                agent: "coder"

            - implement_database:
                description: "Set up database models and migrations"
                agent: "coder"

      - implement_frontend:
          description: "Build user interface"
          agent: "coder"
          priority: 4
          depends_on:
            - design_architecture
          parallel_with:
            - implement_backend

      - review_code:
          description: "Comprehensive code review"
          agent: "reviewer"
          priority: 5
          depends_on:
            - implement_backend
            - implement_frontend

      - write_tests:
          description: "Create comprehensive test suite"
          agent: "coder"
          priority: 6
          depends_on:
            - review_code

    on_complete:
      notify: "Build complete - ready for deployment"

    on_error:
      retry: 2
      fallback_agent: "researcher"
```

### 3.3 Tool Configuration

```yaml
tools:
  allowed:
    - Read
    - Write
    - Edit
    - Bash
    - Grep
    - Glob
    - WebSearch

  disallowed:
    - Task  # Prevent nested agent spawning in certain workflows

  constraints:
    Bash:
      timeout: 30000  # milliseconds
      allowed_commands:
        - git
        - npm
        - cargo
        - pytest

    Write:
      max_file_size: 1048576  # 1MB
      allowed_extensions:
        - .rs
        - .ts
        - .js
        - .md

    WebSearch:
      rate_limit: 10  # queries per minute
```

### 3.4 Workflow Orchestration

```yaml
workflows:
  code_review_pipeline:
    description: "Automated code review and improvement pipeline"

    steps:
      - stage: "analysis"
        agents:
          - coder
        tasks:
          - analyze_changes

      - stage: "review"
        agents:
          - reviewer
        tasks:
          - review_code
        depends_on:
          - analysis

      - stage: "refinement"
        agents:
          - coder
        tasks:
          - apply_feedback
        depends_on:
          - review

      - stage: "validation"
        agents:
          - coder
          - reviewer
        tasks:
          - run_tests
          - final_review
        depends_on:
          - refinement
        mode: "parallel"  # Run tasks in parallel

    hooks:
      pre_workflow:
        - create_backup

      post_workflow:
        - cleanup_temp_files
        - generate_report

      on_agent_error:
        - log_error
        - notify_user
```

### 3.5 Communication Protocols

```yaml
communication:
  channels:
    research_channel:
      description: "Share research findings"
      participants:
        - researcher
        - coder
      message_format: "markdown"

    code_review_channel:
      description: "Code review discussions"
      participants:
        - coder
        - reviewer
      message_format: "json"

  message_types:
    research_finding:
      schema:
        type: "object"
        properties:
          topic: { type: "string" }
          summary: { type: "string" }
          sources: { type: "array" }
          confidence: { type: "number" }

    code_review_comment:
      schema:
        type: "object"
        properties:
          file: { type: "string" }
          line: { type: "number" }
          severity: { type: "string", enum: ["info", "warning", "error"] }
          message: { type: "string" }
          suggestion: { type: "string" }
```

### 3.6 MCP Server Integration

```yaml
mcp_servers:
  filesystem:
    type: "stdio"
    command: "npx"
    args:
      - "-y"
      - "@modelcontextprotocol/server-filesystem"
      - "/path/to/allowed/dir"

  github:
    type: "stdio"
    command: "npx"
    args:
      - "-y"
      - "@modelcontextprotocol/server-github"
    env:
      GITHUB_TOKEN: "${GITHUB_TOKEN}"

  custom_api:
    type: "http"
    url: "https://api.example.com/mcp"
    headers:
      Authorization: "Bearer ${API_TOKEN}"
```

---

## 4. DSL Examples

### 4.1 Example 1: Simple File Organization

```yaml
# simple_file_organizer.yaml
name: "File Organizer"
version: "1.0.0"

agents:
  organizer:
    description: "Organize files by type and date"
    model: "claude-sonnet-4-5"
    tools:
      - Read
      - Bash
      - Glob
    permissions:
      mode: "acceptEdits"

tasks:
  organize_downloads:
    description: "Sort files in Downloads folder"
    agent: "organizer"

    subtasks:
      - scan_files:
          description: "List all files in Downloads"

      - categorize:
          description: "Group files by type"
          depends_on:
            - scan_files

      - move_files:
          description: "Move files to categorized folders"
          depends_on:
            - categorize
```

### 4.2 Example 2: Research and Documentation

```yaml
# research_pipeline.yaml
name: "Research Pipeline"
version: "1.0.0"

agents:
  researcher:
    description: "Conduct research on topics"
    model: "claude-sonnet-4-5"
    tools:
      - WebSearch
      - Read
      - Write
    permissions:
      mode: "acceptEdits"
    max_turns: 15

  writer:
    description: "Write comprehensive documentation"
    model: "claude-opus-4"
    tools:
      - Read
      - Write
      - Edit
    permissions:
      mode: "acceptEdits"

tasks:
  create_documentation:
    description: "Research and document a technology"

    subtasks:
      - gather_info:
          description: "Collect information from multiple sources"
          agent: "researcher"
          output: "research_notes.md"

      - write_guide:
          description: "Create comprehensive guide"
          agent: "writer"
          depends_on:
            - gather_info
          output: "user_guide.md"

      - write_api_reference:
          description: "Document API endpoints"
          agent: "writer"
          depends_on:
            - gather_info
          output: "api_reference.md"
          parallel_with:
            - write_guide
```

### 4.3 Example 3: Complex Multi-Agent Software Development

```yaml
# software_dev_pipeline.yaml
name: "Software Development Pipeline"
version: "1.0.0"

agents:
  product_manager:
    description: "Define requirements and specifications"
    model: "claude-sonnet-4-5"
    tools:
      - Read
      - Write
    permissions:
      mode: "acceptEdits"

  architect:
    description: "Design system architecture"
    model: "claude-opus-4"
    tools:
      - Read
      - Write
      - Grep
      - Glob
    permissions:
      mode: "acceptEdits"

  backend_dev:
    description: "Implement backend services"
    model: "claude-sonnet-4-5"
    tools:
      - Read
      - Write
      - Edit
      - Bash
    permissions:
      mode: "acceptEdits"

  frontend_dev:
    description: "Build user interfaces"
    model: "claude-sonnet-4-5"
    tools:
      - Read
      - Write
      - Edit
      - Bash
    permissions:
      mode: "acceptEdits"

  qa_engineer:
    description: "Write and run tests"
    model: "claude-sonnet-4-5"
    tools:
      - Read
      - Write
      - Bash
    permissions:
      mode: "acceptEdits"

  code_reviewer:
    description: "Review code quality and security"
    model: "claude-opus-4"
    tools:
      - Read
      - Grep
      - Glob
    permissions:
      mode: "default"

workflows:
  full_development_cycle:
    description: "Complete software development lifecycle"

    steps:
      - stage: "planning"
        agents:
          - product_manager
        tasks:
          - define_requirements:
              description: "Create product requirements document"
              output: "requirements.md"

      - stage: "design"
        agents:
          - architect
        tasks:
          - design_system:
              description: "Create architecture design"
              output: "architecture.md"
        depends_on:
          - planning

      - stage: "development"
        agents:
          - backend_dev
          - frontend_dev
        tasks:
          - implement_backend:
              agent: "backend_dev"
              description: "Build backend services"
              subtasks:
                - setup_project
                - implement_models
                - implement_controllers
                - implement_services

          - implement_frontend:
              agent: "frontend_dev"
              description: "Build UI components"
              subtasks:
                - setup_frontend
                - implement_components
                - implement_pages
                - integrate_api
        depends_on:
          - design
        mode: "parallel"

      - stage: "quality_assurance"
        agents:
          - qa_engineer
          - code_reviewer
        tasks:
          - write_tests:
              agent: "qa_engineer"
              description: "Create comprehensive test suite"

          - review_code:
              agent: "code_reviewer"
              description: "Security and quality review"

          - run_tests:
              agent: "qa_engineer"
              description: "Execute all tests"
        depends_on:
          - development

      - stage: "refinement"
        agents:
          - backend_dev
          - frontend_dev
        tasks:
          - fix_issues:
              description: "Address review comments and test failures"
        depends_on:
          - quality_assurance

    hooks:
      pre_workflow:
        - command: "git checkout -b feature/new-feature"

      post_workflow:
        - command: "git add ."
        - command: "git commit -m 'Implement new feature'"

      on_stage_complete:
        - notify_progress

      on_error:
        - log_error
        - create_issue

communication:
  channels:
    dev_channel:
      participants:
        - product_manager
        - architect
        - backend_dev
        - frontend_dev
        - qa_engineer
        - code_reviewer
      message_format: "markdown"
```

### 4.4 Example 4: Data Processing Pipeline

```yaml
# data_pipeline.yaml
name: "Data Processing Pipeline"
version: "1.0.0"

agents:
  data_collector:
    description: "Collect data from various sources"
    model: "claude-sonnet-4-5"
    tools:
      - Read
      - Write
      - Bash
    permissions:
      mode: "acceptEdits"

  data_cleaner:
    description: "Clean and validate data"
    model: "claude-sonnet-4-5"
    tools:
      - Read
      - Write
      - Edit
    permissions:
      mode: "acceptEdits"

  data_analyzer:
    description: "Analyze data and generate insights"
    model: "claude-opus-4"
    tools:
      - Read
      - Write
    permissions:
      mode: "acceptEdits"

tasks:
  process_dataset:
    description: "Complete data processing workflow"

    subtasks:
      - collect_data:
          description: "Fetch data from APIs and files"
          agent: "data_collector"
          output: "raw_data.json"

      - validate_schema:
          description: "Validate data structure"
          agent: "data_cleaner"
          depends_on:
            - collect_data

      - clean_data:
          description: "Remove duplicates and handle missing values"
          agent: "data_cleaner"
          depends_on:
            - validate_schema
          output: "cleaned_data.json"

      - analyze_patterns:
          description: "Identify trends and patterns"
          agent: "data_analyzer"
          depends_on:
            - clean_data

      - generate_report:
          description: "Create analysis report"
          agent: "data_analyzer"
          depends_on:
            - analyze_patterns
          output: "analysis_report.md"
```

---

## 5. Implementation Architecture

### 5.1 Component Structure

```
periplon/
├── src/
│   ├── dsl/
│   │   ├── mod.rs              # DSL module root
│   │   ├── parser.rs           # YAML parsing and validation
│   │   ├── schema.rs           # DSL schema definitions
│   │   ├── validator.rs        # Semantic validation
│   │   ├── executor.rs         # DSL execution engine
│   │   ├── task_graph.rs       # Hierarchical task management
│   │   └── communication.rs    # Inter-agent messaging
│   │
│   ├── dsl_domain/
│   │   ├── agent_def.rs        # Agent definition types
│   │   ├── task_def.rs         # Task definition types
│   │   ├── workflow_def.rs     # Workflow types
│   │   └── message_def.rs      # Communication types
│   │
│   └── dsl_runtime/
│       ├── scheduler.rs        # Task scheduling and dependency resolution
│       ├── agent_pool.rs       # Agent lifecycle management
│       ├── message_bus.rs      # Inter-agent communication bus
│       └── state_manager.rs    # Workflow state management
│
└── examples/
    └── dsl/
        ├── simple_file_organizer.yaml
        ├── research_pipeline.yaml
        ├── software_dev_pipeline.yaml
        └── data_pipeline.yaml
```

### 5.2 Core Types

```rust
// src/dsl/schema.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DSLWorkflow {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub agents: HashMap<String, AgentSpec>,
    #[serde(default)]
    pub tasks: HashMap<String, TaskSpec>,
    #[serde(default)]
    pub workflows: HashMap<String, WorkflowSpec>,
    #[serde(default)]
    pub tools: Option<ToolsConfig>,
    #[serde(default)]
    pub communication: Option<CommunicationConfig>,
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServerSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub description: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub permissions: PermissionsSpec,
    #[serde(default)]
    pub max_turns: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub description: String,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub priority: u32,
    #[serde(default)]
    pub subtasks: Vec<HashMap<String, TaskSpec>>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub parallel_with: Vec<String>,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub on_complete: Option<ActionSpec>,
    #[serde(default)]
    pub on_error: Option<ErrorHandlingSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSpec {
    pub description: String,
    pub steps: Vec<StageSpec>,
    #[serde(default)]
    pub hooks: Option<HooksSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSpec {
    pub stage: String,
    pub agents: Vec<String>,
    pub tasks: Vec<HashMap<String, TaskSpec>>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub mode: ExecutionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    Sequential,
    Parallel,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::Sequential
    }
}
```

### 5.3 Task Graph and Dependency Resolution

```rust
// src/dsl_runtime/scheduler.rs
use std::collections::{HashMap, HashSet, VecDeque};

pub struct TaskGraph {
    tasks: HashMap<String, TaskNode>,
    adjacency: HashMap<String, Vec<String>>,
}

struct TaskNode {
    id: String,
    spec: TaskSpec,
    status: TaskStatus,
    dependencies: Vec<String>,
    parallel_tasks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum TaskStatus {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
}

impl TaskGraph {
    pub fn new() -> Self {
        TaskGraph {
            tasks: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, id: String, spec: TaskSpec) {
        // Build task node with dependencies
        let node = TaskNode {
            id: id.clone(),
            spec: spec.clone(),
            status: TaskStatus::Pending,
            dependencies: spec.depends_on.clone(),
            parallel_tasks: spec.parallel_with.clone(),
        };

        self.tasks.insert(id.clone(), node);

        // Build adjacency list for dependency graph
        for dep in &spec.depends_on {
            self.adjacency
                .entry(dep.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
    }

    pub fn get_ready_tasks(&self) -> Vec<String> {
        // Find tasks where all dependencies are completed
        self.tasks
            .iter()
            .filter(|(_, node)| {
                node.status == TaskStatus::Pending
                    && node.dependencies.iter().all(|dep| {
                        self.tasks
                            .get(dep)
                            .map(|n| n.status == TaskStatus::Completed)
                            .unwrap_or(false)
                    })
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn get_parallel_tasks(&self, task_id: &str) -> Vec<String> {
        // Get tasks that can run in parallel with the given task
        self.tasks
            .get(task_id)
            .map(|node| node.parallel_tasks.clone())
            .unwrap_or_default()
    }

    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        // Kahn's algorithm for topological sorting
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut result: Vec<String> = Vec::new();

        // Calculate in-degrees
        for (id, _) in &self.tasks {
            in_degree.insert(id.clone(), 0);
        }

        for (_, neighbors) in &self.adjacency {
            for neighbor in neighbors {
                *in_degree.get_mut(neighbor).unwrap() += 1;
            }
        }

        // Find nodes with in-degree 0
        for (id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(id.clone());
            }
        }

        // Process nodes
        while let Some(id) = queue.pop_front() {
            result.push(id.clone());

            if let Some(neighbors) = self.adjacency.get(&id) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != self.tasks.len() {
            return Err("Cyclic dependency detected".to_string());
        }

        Ok(result)
    }
}
```

### 5.4 Execution Engine

```rust
// src/dsl/executor.rs
use crate::adapters::primary::PeriplonSDKClient;
use crate::dsl::schema::DSLWorkflow;
use crate::dsl_runtime::scheduler::TaskGraph;
use futures::future::join_all;
use std::collections::HashMap;

pub struct DSLExecutor {
    workflow: DSLWorkflow,
    agents: HashMap<String, PeriplonSDKClient>,
    task_graph: TaskGraph,
}

impl DSLExecutor {
    pub fn new(workflow: DSLWorkflow) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(DSLExecutor {
            workflow,
            agents: HashMap::new(),
            task_graph: TaskGraph::new(),
        })
    }

    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create agent instances
        for (name, spec) in &self.workflow.agents {
            let options = self.agent_spec_to_options(spec)?;
            let mut client = PeriplonSDKClient::new(options);
            client.connect(None).await?;
            self.agents.insert(name.clone(), client);
        }

        // Build task graph
        for (name, spec) in &self.workflow.tasks {
            self.task_graph.add_task(name.clone(), spec.clone());
        }

        Ok(())
    }

    pub async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Get execution order
        let order = self.task_graph.topological_sort()?;

        for task_id in order {
            // Get tasks that can run in parallel
            let parallel_tasks = self.task_graph.get_parallel_tasks(&task_id);

            if parallel_tasks.is_empty() {
                // Execute task sequentially
                self.execute_task(&task_id).await?;
            } else {
                // Execute tasks in parallel
                let mut futures = vec![self.execute_task(&task_id)];
                for parallel_id in parallel_tasks {
                    futures.push(self.execute_task(&parallel_id));
                }

                join_all(futures).await;
            }
        }

        Ok(())
    }

    async fn execute_task(&mut self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let spec = self.workflow.tasks.get(task_id)
            .ok_or("Task not found")?;

        let agent_name = spec.agent.as_ref()
            .ok_or("Agent not specified for task")?;

        let agent = self.agents.get_mut(agent_name)
            .ok_or("Agent not found")?;

        // Execute task query
        agent.query(&spec.description).await?;

        // Process response
        let mut stream = agent.receive_response()?;
        while let Some(_msg) = futures::StreamExt::next(&mut stream).await {
            // Handle message
        }

        Ok(())
    }

    fn agent_spec_to_options(&self, spec: &AgentSpec) -> Result<AgentOptions, Box<dyn std::error::Error>> {
        // Convert DSL AgentSpec to SDK AgentOptions
        let options = AgentOptions {
            allowed_tools: spec.tools.clone(),
            model: spec.model.clone(),
            max_turns: spec.max_turns,
            permission_mode: Some(spec.permissions.mode.clone()),
            ..Default::default()
        };
        Ok(options)
    }

    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Disconnect all agents
        for (_name, agent) in &mut self.agents {
            agent.disconnect().await?;
        }
        Ok(())
    }
}
```

---

## 6. Testing Strategy

### 6.1 Test Levels

#### 6.1.1 Unit Tests
Test individual DSL components in isolation:

```rust
// tests/dsl_parser_tests.rs
#[cfg(test)]
mod tests {
    use periplon_sdk::dsl::parser::parse_workflow;

    #[test]
    fn test_parse_simple_workflow() {
        let yaml = r#"
name: "Test Workflow"
version: "1.0.0"
agents:
  test_agent:
    description: "Test agent"
    tools:
      - Read
    permissions:
      mode: "default"
"#;

        let workflow = parse_workflow(yaml).unwrap();
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.agents.len(), 1);
    }

    #[test]
    fn test_parse_hierarchical_tasks() {
        let yaml = r#"
tasks:
  parent_task:
    description: "Parent task"
    agent: "test_agent"
    subtasks:
      - child_task:
          description: "Child task"
          agent: "test_agent"
"#;

        let workflow = parse_workflow(yaml).unwrap();
        let task = workflow.tasks.get("parent_task").unwrap();
        assert_eq!(task.subtasks.len(), 1);
    }

    #[test]
    fn test_detect_circular_dependencies() {
        // Test cycle detection in task graph
    }

    #[test]
    fn test_validate_agent_references() {
        // Test that all agent references in tasks exist
    }
}
```

#### 6.1.2 Integration Tests
Test DSL execution with mock agents:

```rust
// tests/dsl_execution_tests.rs
#[cfg(test)]
mod tests {
    use periplon_sdk::dsl::executor::DSLExecutor;
    use periplon_sdk::adapters::secondary::MockTransport;

    #[tokio::test]
    async fn test_execute_simple_workflow() {
        let yaml = r#"
name: "Simple Test"
version: "1.0.0"
agents:
  test_agent:
    description: "Test"
    tools: [Read]
tasks:
  test_task:
    description: "Run test"
    agent: "test_agent"
"#;

        let workflow = parse_workflow(yaml).unwrap();
        let mut executor = DSLExecutor::new(workflow).unwrap();

        executor.initialize().await.unwrap();
        executor.execute().await.unwrap();
        executor.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_execute_parallel_tasks() {
        // Test parallel task execution
    }

    #[tokio::test]
    async fn test_task_dependency_resolution() {
        // Test that dependencies are honored
    }
}
```

#### 6.1.3 End-to-End Tests
Test complete workflows with real agents:

```rust
// tests/e2e_tests.rs
#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore] // Run separately due to API calls
    async fn test_file_organizer_workflow() {
        // Test the simple file organizer example
    }

    #[tokio::test]
    #[ignore]
    async fn test_research_pipeline() {
        // Test the research pipeline example
    }

    #[tokio::test]
    #[ignore]
    async fn test_multi_agent_collaboration() {
        // Test complex multi-agent workflow
    }
}
```

### 6.2 Test Coverage Requirements

- **Parser**: 95%+ coverage
  - Valid YAML parsing
  - Invalid YAML error handling
  - Schema validation
  - Type conversion

- **Validator**: 95%+ coverage
  - Agent reference validation
  - Dependency cycle detection
  - Tool availability checks
  - Permission validation

- **Scheduler**: 90%+ coverage
  - Topological sort
  - Parallel task identification
  - Dependency resolution
  - Error propagation

- **Executor**: 85%+ coverage
  - Agent initialization
  - Task execution
  - Message handling
  - Error recovery

- **Integration**: 80%+ coverage
  - Complete workflow execution
  - Inter-agent communication
  - State management
  - Resource cleanup

### 6.3 Test Data and Fixtures

```
tests/
├── fixtures/
│   ├── workflows/
│   │   ├── valid/
│   │   │   ├── simple.yaml
│   │   │   ├── hierarchical.yaml
│   │   │   ├── parallel.yaml
│   │   │   └── complex.yaml
│   │   └── invalid/
│   │       ├── circular_deps.yaml
│   │       ├── missing_agent.yaml
│   │       ├── invalid_tool.yaml
│   │       └── malformed.yaml
│   │
│   └── responses/
│       ├── agent_responses.json
│       └── tool_results.json
│
└── test_utils/
    ├── mock_builder.rs
    └── assertion_helpers.rs
```

### 6.4 Property-Based Testing

Use `proptest` for property-based testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_task_graph_no_cycles(
        tasks in prop::collection::vec(arbitrary_task(), 1..20)
    ) {
        let mut graph = TaskGraph::new();
        for (id, task) in tasks.iter().enumerate() {
            graph.add_task(format!("task_{}", id), task.clone());
        }

        // Should either succeed or detect cycle
        match graph.topological_sort() {
            Ok(_) => { /* Valid graph */ }
            Err(e) => assert!(e.contains("cycle")),
        }
    }
}
```

### 6.5 Performance Tests

```rust
// tests/performance_tests.rs
#[tokio::test]
async fn test_large_task_graph_performance() {
    let start = std::time::Instant::now();

    let mut graph = TaskGraph::new();
    for i in 0..1000 {
        graph.add_task(
            format!("task_{}", i),
            TaskSpec {
                description: format!("Task {}", i),
                depends_on: if i > 0 {
                    vec![format!("task_{}", i - 1)]
                } else {
                    vec![]
                },
                ..Default::default()
            }
        );
    }

    let order = graph.topological_sort().unwrap();

    let duration = start.elapsed();
    assert!(duration.as_millis() < 100); // Should be fast
    assert_eq!(order.len(), 1000);
}
```

---

## 7. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
- [ ] Define DSL schema types
- [ ] Implement YAML parser
- [ ] Create validator for semantic checks
- [ ] Write unit tests for parser and validator
- [ ] Documentation: DSL syntax reference

### Phase 2: Core Execution (Weeks 3-4)
- [ ] Implement TaskGraph and dependency resolution
- [ ] Build DSLExecutor for sequential tasks
- [ ] Create agent pool management
- [ ] Write integration tests
- [ ] Documentation: Execution model

### Phase 3: Hierarchical Tasks (Weeks 5-6)
- [ ] Support nested subtasks
- [ ] Implement task tree traversal
- [ ] Add parent-child task communication
- [ ] Write hierarchical task tests
- [ ] Documentation: Hierarchical tasks guide

### Phase 4: Parallel Execution (Week 7)
- [ ] Implement parallel task execution
- [ ] Add synchronization primitives
- [ ] Handle parallel task results
- [ ] Write parallel execution tests
- [ ] Documentation: Parallel execution patterns

### Phase 5: Communication (Week 8)
- [ ] Implement message bus for inter-agent communication
- [ ] Create communication channels
- [ ] Add message routing
- [ ] Write communication tests
- [ ] Documentation: Communication protocols

### Phase 6: Advanced Features (Weeks 9-10)
- [ ] Add workflow hooks
- [ ] Implement error recovery
- [ ] Support MCP server integration
- [ ] Create workflow state persistence
- [ ] Write E2E tests
- [ ] Documentation: Advanced features

### Phase 7: Optimization & Polish (Weeks 11-12)
- [ ] Performance optimization
- [ ] Memory usage optimization
- [ ] Error message improvements
- [ ] CLI tool for DSL execution
- [ ] Comprehensive examples
- [ ] Final documentation review

---

## 8. Success Metrics

### 8.1 Functional Metrics
- ✓ Parse all valid DSL files without errors
- ✓ Detect and report all invalid DSL constructs
- ✓ Execute hierarchical tasks correctly
- ✓ Handle parallel tasks efficiently
- ✓ Manage inter-agent communication
- ✓ Support all SDK features (tools, permissions, MCP)

### 8.2 Quality Metrics
- Code coverage: >85% overall
- Parser coverage: >95%
- Zero memory leaks (valgrind)
- Zero unsafe code (except where necessary)
- All tests passing on CI/CD
- Documentation coverage: 100% of public APIs

### 8.3 Performance Metrics
- Parse 1000-task workflow in <100ms
- Topological sort of 1000 tasks in <50ms
- Memory usage: <10MB overhead per agent
- Support 100+ concurrent agents
- Workflow state serialization: <1s for 1000 tasks

### 8.4 Usability Metrics
- New user can create simple workflow in <10 minutes
- Documentation completeness: All features documented
- Example coverage: 10+ examples from simple to complex
- Error messages are clear and actionable
- IDE support (YAML schema) available

---

## 9. Documentation Plan

### 9.1 User Documentation
- **Getting Started Guide**: Quick introduction to DSL
- **Syntax Reference**: Complete DSL syntax documentation
- **Examples Library**: 10+ examples with explanations
- **Best Practices**: Guidelines for effective workflows
- **Troubleshooting Guide**: Common issues and solutions

### 9.2 Developer Documentation
- **Architecture Overview**: System design and components
- **API Reference**: All public types and functions
- **Extension Guide**: How to extend the DSL
- **Testing Guide**: How to test DSL workflows
- **Contributing Guide**: How to contribute to the project

### 9.3 Examples Documentation
Each example should include:
- Purpose and use case
- Prerequisites
- Complete YAML file
- Expected output
- Explanation of key concepts
- Variations and extensions

---

## 10. Future Enhancements

### 10.1 Short-term (3-6 months)
- Visual workflow designer (web UI)
- Workflow templates library
- DSL playground (interactive editor)
- Performance profiling tools
- Workflow debugging tools

### 10.2 Medium-term (6-12 months)
- Distributed workflow execution
- Workflow versioning and rollback
- Real-time workflow monitoring
- Workflow optimization suggestions
- Integration with popular CI/CD platforms

### 10.3 Long-term (12+ months)
- Machine learning for workflow optimization
- Natural language to DSL conversion
- Workflow marketplace
- Cloud-hosted workflow execution
- Advanced agent collaboration patterns

---

## 11. Risk Assessment & Mitigation

### 11.1 Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Circular dependency detection fails | High | Low | Comprehensive testing, formal verification |
| Memory leaks in long-running workflows | Medium | Medium | Rigorous testing, profiling, smart pointers |
| Parallel task race conditions | High | Medium | Careful synchronization, testing |
| DSL parsing ambiguities | Medium | Low | Clear syntax definition, validation |

### 11.2 Usability Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| DSL too complex for beginners | High | Medium | Simple examples, good documentation |
| Error messages unclear | Medium | Medium | User testing, iterative improvement |
| Limited IDE support | Low | High | Provide JSON schema for validation |

### 11.3 Performance Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Poor performance with large graphs | Medium | Low | Optimized algorithms, benchmarking |
| Excessive memory usage | Medium | Low | Memory profiling, optimization |

---

## 12. Conclusion

This DSL design provides a comprehensive, intuitive, and powerful way to create agentic AI systems. It leverages the periplon's capabilities while adding:

1. **Hierarchical task decomposition** for complex problem-solving
2. **Multi-agent collaboration** with clear communication protocols
3. **Declarative workflow definition** that's easy to read and write
4. **Comprehensive testing framework** ensuring reliability
5. **Extensibility** for future enhancements

The implementation roadmap provides a clear path forward with measurable milestones, and the testing strategy ensures quality and reliability throughout development.

The DSL will enable users of all skill levels to create sophisticated agentic workflows, from simple automation tasks to complex multi-agent systems, while maintaining type safety and leveraging Rust's performance and reliability.

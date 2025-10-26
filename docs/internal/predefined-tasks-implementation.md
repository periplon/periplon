# Predefined Tasks Implementation Plan

## Overview

This document outlines the design and implementation plan for adding predefined task functionality, ecosystem, and marketplace support to the DSL system. This will enable easy integration with third-party systems (Google, Notion, GitHub, etc.) through reusable, versioned task definitions stored locally or in git repositories.

## Goals

1. **Reusability**: Define tasks once, use across multiple workflows
2. **Discoverability**: Auto-discover available predefined tasks via manifests and registries
3. **Flexibility**: Support both task referencing and embedding
4. **Versioning**: Proper semantic versioning with dependency resolution
5. **Ecosystem**: Enable third-party task sharing via git repositories and multiple marketplaces
6. **Security**: Trust and permission management for external tasks with per-registry configuration
7. **Multi-Marketplace**: Support official, community, company-internal, and self-hosted registries
8. **Task Groups**: Bundle related tasks into cohesive integration suites
9. **Offline Support**: Work without internet connectivity using cached registries
10. **Priority Resolution**: Intelligent task resolution across multiple sources

## Architecture

### 1. Predefined Task Structure

#### Task Definition Schema

```yaml
# google/drive/upload.task.yaml
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "google-drive-upload"
  version: "1.2.0"
  author: "community/google-integrations"
  description: "Upload files to Google Drive"
  license: "MIT"
  repository: "https://github.com/claude-tasks/google-integrations"
  tags: ["google", "storage", "upload"]

spec:
  # Task implementation
  agent_template:
    description: "Upload ${input.file_path} to Google Drive folder ${input.folder_id}"
    model: "claude-sonnet-4-5"
    tools: ["Bash", "WebFetch"]
    permissions:
      mode: "acceptEdits"

  # Input schema with validation
  inputs:
    file_path:
      type: string
      required: true
      description: "Local file path to upload"
      validation:
        pattern: "^[^<>:\"|?*]+$"

    folder_id:
      type: string
      required: false
      default: "root"
      description: "Google Drive folder ID"

    credentials:
      type: secret
      required: true
      description: "Google OAuth credentials"
      source: "${env.GOOGLE_CREDENTIALS}"

  # Output schema
  outputs:
    file_id:
      type: string
      description: "Uploaded file ID"
      source:
        type: state
        key: "drive_file_id"

    share_url:
      type: string
      description: "Shareable link"
      source:
        type: state
        key: "share_link"

  # Dependencies on other predefined tasks
  dependencies:
    - name: "google-auth"
      version: "^2.0.0"
      optional: false

  # Example usage
  examples:
    - name: "Upload PDF report"
      inputs:
        file_path: "./report.pdf"
        folder_id: "abc123xyz"
      description: "Upload a PDF to a specific folder"
```

#### Package Manifest

```yaml
# package.yaml (repository root)
apiVersion: "package/v1"
kind: "TaskPackage"
metadata:
  name: "google-integrations"
  version: "2.1.0"
  description: "Google Workspace integration tasks"
  author: "community"
  homepage: "https://github.com/claude-tasks/google-integrations"
  license: "MIT"

tasks:
  - path: "drive/upload.task.yaml"
    name: "google-drive-upload"
    version: "1.2.0"

  - path: "drive/download.task.yaml"
    name: "google-drive-download"
    version: "1.1.0"

  - path: "docs/create.task.yaml"
    name: "google-docs-create"
    version: "2.0.0"

  - path: "sheets/read.task.yaml"
    name: "google-sheets-read"
    version: "1.5.0"

dependencies:
  - name: "oauth-common"
    version: "^1.0.0"
    repository: "https://github.com/claude-tasks/oauth-common"

# Compatibility
requires:
  sdk_version: ">=0.2.0"
  min_model: "claude-sonnet-4"
```

### 2. Storage & Discovery

#### Storage Locations (Priority Order)

1. **Project Local**: `./.claude/tasks/`
2. **User Global**: `~/.claude/tasks/`
3. **Git Repositories**: Configured in `~/.claude/task-sources.yaml`
4. **Marketplace Registries**: Multiple registries with different purposes:
   - Official public marketplace
   - Company/organization private registries
   - Community-driven registries
   - Self-hosted internal registries

#### Task Sources Configuration

```yaml
# ~/.claude/task-sources.yaml
sources:
  # Local directories
  - type: local
    name: "project-tasks"
    path: "./.claude/tasks"
    priority: 10

  # Git repositories
  - type: git
    name: "official-tasks"
    url: "https://github.com/claude-tasks/official"
    branch: "main"
    cache_dir: "~/.claude/cache/official-tasks"
    update_policy: "daily"  # daily, weekly, manual
    priority: 5
    trusted: true

  - type: git
    name: "google-integrations"
    url: "https://github.com/claude-tasks/google-integrations"
    branch: "main"
    tag: "v2.1.0"  # Pin to specific version
    priority: 5
    trusted: true

  # Multiple Marketplace Registries
  - type: registry
    name: "official-marketplace"
    url: "https://registry.claude-tasks.io"
    priority: 3
    auto_discover: true
    trust_level: "trusted"
    auth:
      type: "none"  # Public registry

  - type: registry
    name: "company-internal"
    url: "https://tasks.mycompany.internal"
    priority: 8
    auto_discover: true
    trust_level: "trusted"
    auth:
      type: "token"
      token: "${env.COMPANY_REGISTRY_TOKEN}"
    tls:
      verify: true
      ca_cert: "~/.claude/certs/company-ca.pem"

  - type: registry
    name: "community-hub"
    url: "https://community.claude-tasks.org"
    priority: 2
    auto_discover: false  # Manual search only
    trust_level: "community"
    auth:
      type: "none"

  - type: registry
    name: "self-hosted"
    url: "http://localhost:8080/registry"
    priority: 9
    auto_discover: true
    trust_level: "trusted"
    auth:
      type: "basic"
      username: "${env.REGISTRY_USER}"
      password: "${env.REGISTRY_PASS}"
```

#### Discovery Mechanism

```rust
// src/dsl/predefined_tasks/discovery.rs

pub struct TaskDiscovery {
    sources: Vec<TaskSource>,
    cache: TaskCache,
}

impl TaskDiscovery {
    /// Load all task sources from config
    pub async fn load_sources(config_path: &Path) -> Result<Self>;

    /// Discover all available tasks from all sources
    pub async fn discover_all(&self) -> Result<Vec<TaskMetadata>>;

    /// Find task by name, searching all registries by priority
    pub async fn find_task(&self, name: &str, version: &str) -> Result<PredefinedTask>;

    /// Find task from specific registry
    pub async fn find_task_in_registry(
        &self,
        registry: &str,
        name: &str,
        version: &str,
    ) -> Result<PredefinedTask>;

    /// Search tasks across all registries
    pub async fn search(&self, query: &str) -> Result<Vec<TaskMetadata>>;

    /// Search tasks in specific registry
    pub async fn search_registry(
        &self,
        registry: &str,
        query: &str,
    ) -> Result<Vec<TaskMetadata>>;

    /// List tasks by category/tag across all registries
    pub async fn list_by_tag(&self, tag: &str) -> Result<Vec<TaskMetadata>>;

    /// List all configured registries
    pub fn list_registries(&self) -> Vec<RegistryInfo>;

    /// Add new registry dynamically
    pub async fn add_registry(&mut self, config: RegistryConfig) -> Result<()>;

    /// Remove registry
    pub async fn remove_registry(&mut self, name: &str) -> Result<()>;

    /// Update git sources and registries
    pub async fn update_sources(&self) -> Result<()>;

    /// Sync specific registry
    pub async fn sync_registry(&self, name: &str) -> Result<()>;
}
```

### 3. Task Reference vs Embedding

#### Reference Method (Recommended)

Reference external task definitions by URI/path:

```yaml
# workflow.yaml
name: "Upload Report Workflow"

tasks:
  upload_to_drive:
    # Reference by package name and version
    uses: "google-drive-upload@1.2.0"
    inputs:
      file_path: "./report.pdf"
      folder_id: "abc123xyz"
      credentials: "${env.GOOGLE_CREDENTIALS}"
    outputs:
      file_id: "${task.file_id}"
      share_url: "${task.share_url}"
```

**Benefits**:
- Always use latest compatible version
- Automatic updates with version constraints
- Smaller workflow files
- Shared bug fixes/improvements

#### Embedding Method

Copy task definition into workflow (for stability/customization):

```yaml
# workflow.yaml
name: "Upload Report Workflow"

tasks:
  upload_to_drive:
    # Embed complete task definition
    embed: "google-drive-upload@1.2.0"
    # Override specific properties
    overrides:
      agent_template:
        model: "claude-opus-4"  # Use different model
      inputs:
        timeout:
          type: integer
          default: 300  # Add custom input
```

**Benefits**:
- Version pinning (workflow won't break)
- Customization of task behavior
- Offline execution (no external deps)
- Audit trail of exact task version

### 4. Task Groups

Task groups allow defining collections of tasks that work together as cohesive units. This is useful for multi-step workflows, integration suites, or feature bundles.

#### Task Group Definition Schema

```yaml
# google/workspace/full-suite.taskgroup.yaml
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "google-workspace-suite"
  version: "2.0.0"
  author: "community/google-integrations"
  description: "Complete Google Workspace integration suite"
  license: "MIT"
  tags: ["google", "workspace", "integration"]

spec:
  # Tasks included in this group
  tasks:
    - name: "google-drive-upload"
      version: "^1.2.0"
      required: true

    - name: "google-drive-download"
      version: "^1.1.0"
      required: true

    - name: "google-docs-create"
      version: "^2.0.0"
      required: false  # Optional task

    - name: "google-sheets-read"
      version: "^1.5.0"
      required: true

    - name: "google-auth"
      version: "^2.0.0"
      required: true

  # Shared configuration for all tasks in group
  shared_config:
    inputs:
      credentials:
        type: secret
        required: true
        description: "Google OAuth credentials for all tasks"
        source: "${env.GOOGLE_CREDENTIALS}"

    permissions:
      mode: "acceptEdits"
      max_turns: 10

  # Pre-configured workflows using these tasks
  workflows:
    - name: "backup-to-drive"
      description: "Backup files to Google Drive"
      tasks:
        - upload_files:
            uses: "google-drive-upload"
            inputs:
              credentials: "${group.credentials}"

    - name: "report-generation"
      description: "Generate and share report"
      tasks:
        - create_doc:
            uses: "google-docs-create"
            inputs:
              credentials: "${group.credentials}"

        - upload_to_drive:
            uses: "google-drive-upload"
            depends_on: [create_doc]
            inputs:
              credentials: "${group.credentials}"

  # Group-level dependencies
  dependencies:
    - name: "oauth-common"
      version: "^1.0.0"

  # Installation hooks
  hooks:
    post_install:
      - type: "command"
        command: "echo 'Google Workspace suite installed successfully'"

    pre_use:
      - type: "validate"
        check: "env.GOOGLE_CREDENTIALS"
        message: "GOOGLE_CREDENTIALS environment variable required"
```

#### Using Task Groups in Workflows

##### Method 1: Import Entire Group

```yaml
name: "My Workflow"

# Import all tasks from group
imports:
  - group: "google-workspace-suite@^2.0.0"
    as: "gws"  # Namespace alias

tasks:
  upload_report:
    # Reference task from imported group
    uses: "gws.google-drive-upload"
    inputs:
      file_path: "./report.pdf"
      credentials: "${env.GOOGLE_CREDENTIALS}"
```

##### Method 2: Use Predefined Workflow from Group

```yaml
name: "Backup Workflow"

tasks:
  backup_data:
    # Use pre-configured workflow from task group
    uses_workflow: "google-workspace-suite@^2.0.0#backup-to-drive"
    inputs:
      credentials: "${env.GOOGLE_CREDENTIALS}"
      files:
        - "./data/*.csv"
        - "./reports/*.pdf"
```

##### Method 3: Selective Import

```yaml
name: "Simple Upload"

# Import only specific tasks from group
imports:
  - group: "google-workspace-suite@^2.0.0"
    tasks: ["google-drive-upload", "google-auth"]
    as: "gws"

tasks:
  upload:
    uses: "gws.google-drive-upload"
    inputs:
      file_path: "./file.txt"
```

#### Task Group Use Cases

**1. Platform Integration Suites**
```yaml
# slack/notifications-suite.taskgroup.yaml
metadata:
  name: "slack-notifications-suite"
  version: "1.0.0"

spec:
  tasks:
    - {name: "slack-send-message", version: "^2.0.0"}
    - {name: "slack-create-channel", version: "^1.5.0"}
    - {name: "slack-upload-file", version: "^1.3.0"}
    - {name: "slack-update-status", version: "^1.0.0"}
```

**2. CI/CD Pipeline Groups**
```yaml
# ci-cd/deployment-suite.taskgroup.yaml
metadata:
  name: "deployment-suite"
  version: "3.0.0"

spec:
  tasks:
    - {name: "docker-build", version: "^2.0.0"}
    - {name: "docker-push", version: "^2.0.0"}
    - {name: "kubernetes-deploy", version: "^1.8.0"}
    - {name: "health-check", version: "^1.2.0"}
    - {name: "rollback", version: "^1.0.0"}

  workflows:
    - name: "blue-green-deployment"
      description: "Zero-downtime deployment"
      # ... workflow definition
```

**3. Data Processing Pipelines**
```yaml
# data/etl-suite.taskgroup.yaml
metadata:
  name: "etl-suite"
  version: "1.5.0"

spec:
  tasks:
    - {name: "extract-postgres", version: "^2.0.0"}
    - {name: "extract-mysql", version: "^1.8.0"}
    - {name: "transform-json", version: "^3.0.0"}
    - {name: "load-bigquery", version: "^2.5.0"}
    - {name: "load-snowflake", version: "^1.9.0"}
```

#### Task Group Discovery and Management

Task groups are discovered alongside predefined tasks:

```rust
// src/dsl/predefined_tasks/groups.rs

pub struct TaskGroup {
    pub metadata: TaskGroupMetadata,
    pub tasks: Vec<TaskReference>,
    pub shared_config: Option<SharedConfig>,
    pub workflows: Vec<PrebuiltWorkflow>,
    pub dependencies: Vec<Dependency>,
}

impl TaskGroup {
    /// Load task group from YAML
    pub fn from_yaml(yaml: &str) -> Result<Self>;

    /// Resolve all tasks in group
    pub async fn resolve_tasks(&self, discovery: &TaskDiscovery) -> Result<Vec<PredefinedTask>>;

    /// Install all tasks from group
    pub async fn install(&self, target_dir: &Path) -> Result<()>;

    /// Validate group integrity
    pub fn validate(&self) -> Result<()>;
}
```

#### Task Group Registry Schema

Task groups are published to registries alongside individual tasks:

```json
{
  "api_version": "v1",
  "kind": "registry_index",
  "tasks": [
    {
      "name": "google-drive-upload",
      "type": "task",
      "latest_version": "1.2.3"
    }
  ],
  "task_groups": [
    {
      "name": "google-workspace-suite",
      "type": "task_group",
      "latest_version": "2.0.0",
      "included_tasks": [
        "google-drive-upload@^1.2.0",
        "google-docs-create@^2.0.0",
        "google-sheets-read@^1.5.0"
      ],
      "downloads": 15234,
      "rating": 4.7
    }
  ]
}
```

#### CLI Commands for Task Groups

```bash
# List available task groups
periplon-executor groups list

# Search task groups
periplon-executor groups search "google"

# Show group details
periplon-executor groups show google-workspace-suite

# Install task group (installs all included tasks)
periplon-executor groups install google-workspace-suite@2.0.0

# Show tasks in group
periplon-executor groups tasks google-workspace-suite

# Show predefined workflows in group
periplon-executor groups workflows google-workspace-suite

# Create new task group
periplon-executor groups new my-group --tasks="task1,task2,task3"

# Validate task group definition
periplon-executor groups validate ./my-group.taskgroup.yaml

# Publish task group to registry
periplon-executor groups publish ./my-group.taskgroup.yaml --registry=company-internal
```

#### Benefits of Task Groups

1. **Cohesive Integration**: Related tasks packaged together
2. **Simplified Discovery**: Find complete solutions, not individual tasks
3. **Shared Configuration**: Common settings applied to all tasks
4. **Version Compatibility**: Guaranteed compatible task versions
5. **Predefined Workflows**: Ready-to-use workflow templates
6. **Easier Updates**: Update entire suite at once
7. **Better Documentation**: Suite-level docs and examples
8. **Namespace Management**: Avoid naming conflicts with group prefixes

### 5. Versioning & Dependency Resolution

#### Semantic Versioning

Follow semver (MAJOR.MINOR.PATCH):
- **MAJOR**: Breaking changes to inputs/outputs
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, no API changes

#### Version Constraints

```yaml
tasks:
  upload:
    uses: "google-drive-upload@^1.2.0"  # >=1.2.0 <2.0.0
    # Other formats:
    # "google-drive-upload@~1.2.0"  # >=1.2.0 <1.3.0
    # "google-drive-upload@1.2.0"   # Exact version
    # "google-drive-upload@latest"  # Latest stable
    # "google-drive-upload@1.x"     # Any 1.x version
```

#### Lock File

Generate lock file for reproducible builds:

```yaml
# workflow.lock.yaml
version: 1
generated_at: "2025-01-15T10:30:00Z"

resolved_tasks:
  google-drive-upload:
    version: "1.2.3"
    source:
      type: "git"
      url: "https://github.com/claude-tasks/google-integrations"
      commit: "a3b2c1d"
    registry: null
    checksum: "sha256:abc123..."

  google-auth:
    version: "2.0.1"
    source:
      type: "registry"
      registry: "official-marketplace"
      url: "https://registry.claude-tasks.io"
    checksum: "sha256:def456..."

  notion-create-page:
    version: "1.5.0"
    source:
      type: "registry"
      registry: "company-internal"
      url: "https://tasks.mycompany.internal"
    checksum: "sha256:xyz789..."
```

### 5. Implementation Phases

#### Phase 1: Local Predefined Tasks (Weeks 1-2)

**Goal**: Basic predefined task support with local storage

**Deliverables**:
1. Task definition schema (`PredefinedTask` struct)
2. Parser for `.task.yaml` files
3. Local task discovery (`.claude/tasks/`)
4. Task reference in workflows (`uses:` syntax)
5. Input/output binding
6. Basic validation

**Files to Create**:
- `src/dsl/predefined_tasks/mod.rs` - Module root
- `src/dsl/predefined_tasks/schema.rs` - Task definition types
- `src/dsl/predefined_tasks/parser.rs` - YAML parsing
- `src/dsl/predefined_tasks/loader.rs` - Load from filesystem
- `src/dsl/predefined_tasks/resolver.rs` - Resolve task references
- `tests/dsl/predefined_tasks_tests.rs` - Unit tests

**Workflow Changes**:
```rust
// src/dsl/schema.rs
pub struct Task {
    // Existing fields...

    // NEW: Reference to predefined task
    pub uses: Option<String>,  // e.g., "google-drive-upload@1.2.0"
    pub embed: Option<String>, // Embed instead of reference
    pub overrides: Option<serde_yaml::Value>, // Override task properties
}
```

#### Phase 2: Git Repository Support (Weeks 3-4)

**Goal**: Pull tasks from git repositories with caching

**Deliverables**:
1. Git source configuration (`task-sources.yaml`)
2. Git cloning/pulling with caching
3. Package manifest parsing (`package.yaml`)
4. Multi-source task discovery
5. Priority-based resolution
6. Update mechanism

**Files to Create**:
- `src/dsl/predefined_tasks/sources/mod.rs`
- `src/dsl/predefined_tasks/sources/git.rs` - Git integration
- `src/dsl/predefined_tasks/sources/local.rs` - Local filesystem
- `src/dsl/predefined_tasks/cache.rs` - Caching layer
- `src/dsl/predefined_tasks/discovery.rs` - Multi-source discovery

**Dependencies to Add**:
```toml
[dependencies]
git2 = "0.18"  # Git operations
dirs = "5.0"   # Platform-specific directories
```

#### Phase 3: Versioning & Dependency Resolution (Weeks 5-6)

**Goal**: Proper semver resolution with dependency graphs

**Deliverables**:
1. Semver parsing and constraint matching
2. Dependency graph construction
3. Version resolution algorithm
4. Lock file generation
5. Conflict detection
6. Update checker

**Files to Create**:
- `src/dsl/predefined_tasks/version.rs` - Semver types
- `src/dsl/predefined_tasks/resolver.rs` - Dependency resolution
- `src/dsl/predefined_tasks/lockfile.rs` - Lock file management
- `src/dsl/predefined_tasks/graph.rs` - Dependency graph

**Dependencies to Add**:
```toml
[dependencies]
semver = "1.0"  # Semantic versioning
petgraph = "0.6"  # Dependency graphs
```

#### Phase 4: Task Groups (Weeks 7-8)

**Goal**: Support for task groups and bundle workflows

**Deliverables**:
1. Task group schema and parser
2. Group import mechanism (namespace support)
3. Predefined workflow execution from groups
4. Selective task import from groups
5. Shared configuration inheritance
6. Group validation and dependency resolution

**Files to Create**:
- `src/dsl/predefined_tasks/groups/mod.rs` - Module root
- `src/dsl/predefined_tasks/groups/schema.rs` - TaskGroup types
- `src/dsl/predefined_tasks/groups/parser.rs` - YAML parsing
- `src/dsl/predefined_tasks/groups/loader.rs` - Load and resolve groups
- `src/dsl/predefined_tasks/groups/namespace.rs` - Namespace management
- `tests/dsl/task_groups_tests.rs` - Unit tests

**Workflow Changes**:
```rust
// src/dsl/schema.rs
pub struct Workflow {
    // Existing fields...

    // NEW: Group imports
    pub imports: Option<Vec<GroupImport>>,
}

pub struct GroupImport {
    pub group: String,  // "google-workspace-suite@^2.0.0"
    pub as_name: Option<String>,  // Namespace alias
    pub tasks: Option<Vec<String>>,  // Selective import
}

pub struct Task {
    // Existing fields...

    // NEW: Reference workflow from group
    pub uses_workflow: Option<String>,  // "group@version#workflow"
}
```

#### Phase 5: Multiple Marketplace Support (Weeks 9-10)

**Goal**: Enable multiple registries with priority-based resolution

**Deliverables**:
1. Multi-registry configuration support
2. Registry priority and conflict resolution
3. Registry-specific task references (`registry::task@version`)
4. Registry health monitoring and failover
5. Authentication per registry (token, basic, client cert)
6. Offline mode with registry caching
7. Registry management CLI commands

**Files to Create**:
- `src/dsl/predefined_tasks/registry/mod.rs` - Module root
- `src/dsl/predefined_tasks/registry/client.rs` - HTTP client
- `src/dsl/predefined_tasks/registry/config.rs` - Registry config
- `src/dsl/predefined_tasks/registry/auth.rs` - Authentication handlers
- `src/dsl/predefined_tasks/registry/health.rs` - Health monitoring
- `src/dsl/predefined_tasks/registry/cache.rs` - Offline caching
- `src/dsl/predefined_tasks/resolution.rs` - Multi-registry resolution

**Dependencies to Add**:
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
rustls = "0.21"  # TLS support with client certs
tokio = { version = "1.0", features = ["time"] }  # Health check timeouts
```

**CLI Extensions**:
```bash
# Registry management
periplon-executor registry list
periplon-executor registry add --name=my-reg --url=https://...
periplon-executor registry remove my-reg
periplon-executor registry test company-internal
periplon-executor registry sync official-marketplace

# Multi-registry search
periplon-executor tasks search "google" --registry=all
periplon-executor tasks search "google" --registry=company-internal
```

#### Phase 6: Marketplace & Publishing (Weeks 11-12)

**Goal**: Public registry infrastructure and publishing tools

**Deliverables**:
1. Task search functionality across registries
2. Category/tag filtering
3. Task and group validation tools
4. Publishing CLI commands
5. Trust/signature verification
6. Marketplace web UI (separate project)
7. Download stats and ratings

**Files to Create**:
- `src/dsl/predefined_tasks/search.rs` - Search engine
- `src/dsl/predefined_tasks/publish.rs` - Publishing tools
- `src/dsl/predefined_tasks/security.rs` - Trust verification
- `src/dsl/predefined_tasks/analytics.rs` - Download tracking
- `bin/task-publisher.rs` - Publishing CLI

**New Binary Target**:
```bash
# Publish task to registry
task-publisher publish ./my-task.yaml --registry=official-marketplace

# Publish task group
task-publisher publish ./my-group.taskgroup.yaml --registry=company-internal

# Search marketplace
task-publisher search "google drive" --registry=all

# Show analytics
task-publisher stats google-drive-upload

# Update all tasks
task-publisher update --check-only
```

### 6. Third-Party Integration Examples

#### Google Workspace Tasks

```yaml
# google/drive/upload.task.yaml
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "google-drive-upload"
  version: "1.2.0"

spec:
  agent_template:
    description: |
      Upload file to Google Drive using the Drive API.
      File: ${input.file_path}
      Folder: ${input.folder_id}
    tools: ["Bash", "WebFetch"]

  inputs:
    file_path: {type: string, required: true}
    folder_id: {type: string, default: "root"}
    credentials: {type: secret, required: true}

  outputs:
    file_id: {type: string}
    share_url: {type: string}
```

#### Notion Integration Tasks

```yaml
# notion/create-page.task.yaml
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "notion-create-page"
  version: "1.0.0"

spec:
  agent_template:
    description: |
      Create a new Notion page in database ${input.database_id}
      with title "${input.title}" and properties ${input.properties}
    tools: ["WebFetch", "Bash"]

  inputs:
    database_id: {type: string, required: true}
    title: {type: string, required: true}
    properties: {type: object, required: false}
    api_token: {type: secret, required: true, source: "${env.NOTION_TOKEN}"}

  outputs:
    page_id: {type: string}
    page_url: {type: string}
```

#### GitHub Integration Tasks

```yaml
# github/create-issue.task.yaml
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "github-create-issue"
  version: "2.0.0"

spec:
  agent_template:
    description: |
      Create GitHub issue in ${input.repository}:
      Title: ${input.title}
      Body: ${input.body}
      Labels: ${input.labels}
    tools: ["Bash"]

  inputs:
    repository: {type: string, required: true, description: "owner/repo"}
    title: {type: string, required: true}
    body: {type: string, required: true}
    labels: {type: array, items: string, required: false}
    token: {type: secret, required: true, source: "${env.GITHUB_TOKEN}"}

  outputs:
    issue_number: {type: integer}
    issue_url: {type: string}
```

### 7. Security & Trust Model

#### Trust Levels

```yaml
sources:
  - type: git
    name: "official-tasks"
    url: "https://github.com/claude-tasks/official"
    trust_level: "trusted"  # trusted, verified, community, untrusted
```

**Trust Levels**:
1. **trusted**: Official tasks, auto-approved
2. **verified**: Signed by known authors, minimal review
3. **community**: Require user approval before use
4. **untrusted**: Show warning, require explicit approval

#### Permission Scoping

Predefined tasks declare required permissions:

```yaml
spec:
  required_permissions:
    - type: "file_write"
      paths: ["./output/**"]
    - type: "network"
      domains: ["*.googleapis.com"]
    - type: "environment"
      variables: ["GOOGLE_CREDENTIALS"]
```

Workflow executor validates permissions before task execution.

#### Signature Verification (Future)

```yaml
metadata:
  signature:
    algorithm: "ed25519"
    public_key: "base64:..."
    signature: "base64:..."
    signed_at: "2025-01-15T10:30:00Z"
```

### 8. Multiple Marketplace Management

Supporting multiple marketplaces enables different use cases and organizational needs.

#### Marketplace Types

**1. Official Public Marketplace**
- Curated, high-quality tasks
- Vetted by maintainers
- Free to use
- Highest trust level
- Example: `https://registry.claude-tasks.io`

**2. Company/Organization Internal Registry**
- Private, organization-specific tasks
- Behind corporate firewall
- Authentication required
- Contains proprietary integrations
- Example: `https://tasks.mycompany.internal`

**3. Community Marketplace**
- User-contributed tasks
- Lower trust level
- Requires approval before use
- Diverse ecosystem
- Example: `https://community.claude-tasks.org`

**4. Self-Hosted Registry**
- Deployed on-premises or private cloud
- Full control over content
- Custom security policies
- Air-gapped environments
- Example: `http://localhost:8080/registry`

#### Registry Priority and Resolution

When multiple registries are configured, task resolution follows priority order:

```rust
// src/dsl/predefined_tasks/resolution.rs

pub struct TaskResolver {
    sources: Vec<TaskSource>,
}

impl TaskResolver {
    /// Find task across all registries by priority
    pub async fn resolve_task(
        &self,
        name: &str,
        version: &str,
    ) -> Result<ResolvedTask> {
        // Sort sources by priority (higher = searched first)
        let mut sorted_sources = self.sources.clone();
        sorted_sources.sort_by(|a, b| b.priority.cmp(&a.priority));

        for source in sorted_sources {
            if let Ok(task) = source.find_task(name, version).await {
                return Ok(ResolvedTask {
                    task,
                    source: source.name.clone(),
                });
            }
        }

        Err(Error::TaskNotFound(name.to_string()))
    }

    /// Find task from specific registry
    pub async fn resolve_from_registry(
        &self,
        registry: &str,
        name: &str,
        version: &str,
    ) -> Result<ResolvedTask> {
        let source = self.sources
            .iter()
            .find(|s| s.name == registry)
            .ok_or(Error::RegistryNotFound(registry.to_string()))?;

        let task = source.find_task(name, version).await?;
        Ok(ResolvedTask {
            task,
            source: registry.to_string(),
        })
    }
}
```

#### Registry-Specific Task References

Tasks can explicitly specify which registry to use:

```yaml
tasks:
  # Use default priority-based resolution
  upload:
    uses: "google-drive-upload@^1.2.0"

  # Explicitly use company registry
  internal_task:
    uses: "company-internal::custom-integration@1.0.0"
    # Syntax: "registry::task@version"

  # Use community registry
  community_task:
    uses: "community-hub::experimental-feature@0.5.0"
```

#### Registry Configuration Schema

```yaml
# ~/.claude/registries.yaml
registries:
  # Official marketplace (public)
  - name: "official-marketplace"
    type: "registry"
    url: "https://registry.claude-tasks.io"
    priority: 3
    trust_level: "trusted"
    enabled: true
    features:
      search: true
      auto_discover: true
      caching: true
      offline_mode: true
    metadata:
      description: "Official curated marketplace"
      contact: "support@claude-tasks.io"

  # Company internal (private)
  - name: "company-internal"
    type: "registry"
    url: "https://tasks.mycompany.internal"
    priority: 8
    trust_level: "trusted"
    enabled: true
    auth:
      type: "token"
      token: "${env.COMPANY_REGISTRY_TOKEN}"
    tls:
      verify: true
      ca_cert: "~/.claude/certs/company-ca.pem"
      client_cert: "~/.claude/certs/client.pem"
      client_key: "~/.claude/certs/client-key.pem"
    features:
      search: true
      auto_discover: true
      caching: true
    rate_limit:
      requests_per_minute: 100
    metadata:
      description: "Internal company task registry"
      admin: "devops@mycompany.com"

  # Community hub (public, lower trust)
  - name: "community-hub"
    type: "registry"
    url: "https://community.claude-tasks.org"
    priority: 2
    trust_level: "community"
    enabled: true
    features:
      search: true
      auto_discover: false  # Require explicit search
      caching: true
      approval_required: true  # Prompt before using tasks
    metadata:
      description: "Community-contributed tasks"
      website: "https://community.claude-tasks.org"

  # Self-hosted (local development)
  - name: "dev-local"
    type: "registry"
    url: "http://localhost:8080/registry"
    priority: 9
    trust_level: "trusted"
    enabled: true
    auth:
      type: "basic"
      username: "admin"
      password: "${env.LOCAL_REGISTRY_PASS}"
    tls:
      verify: false  # Dev environment
    features:
      search: true
      auto_discover: true
      caching: false  # Always fetch fresh
    metadata:
      description: "Local development registry"
```

#### Registry API Specification

All registries must implement this API:

```
GET /api/v1/tasks
  - List all available tasks
  - Query params: ?page=1&limit=50&tag=google

GET /api/v1/tasks/{name}
  - Get task metadata and versions
  - Returns: task info, all versions, download stats

GET /api/v1/tasks/{name}/{version}
  - Get specific task version
  - Returns: complete task definition

GET /api/v1/search?q={query}
  - Full-text search across tasks
  - Returns: matching tasks with relevance scores

GET /api/v1/groups
  - List all task groups
  - Query params: ?page=1&limit=50

GET /api/v1/groups/{name}
  - Get task group metadata
  - Returns: group info, included tasks

POST /api/v1/publish
  - Publish new task/group (requires auth)
  - Body: task definition + metadata

DELETE /api/v1/tasks/{name}/{version}
  - Remove task version (requires auth)
  - Returns: deletion confirmation

GET /api/v1/health
  - Health check endpoint
  - Returns: registry status, version
```

#### Registry Mirroring and Fallbacks

Configure fallback registries for high availability:

```yaml
registries:
  - name: "official-marketplace"
    url: "https://registry.claude-tasks.io"
    priority: 3
    mirrors:
      - url: "https://mirror1.claude-tasks.io"
        active: true
      - url: "https://mirror2.claude-tasks.io"
        active: true
    fallback_timeout: "5s"
```

#### Registry Health Monitoring

```rust
// src/dsl/predefined_tasks/registry_health.rs

pub struct RegistryHealth {
    pub name: String,
    pub status: HealthStatus,
    pub response_time: Duration,
    pub last_check: DateTime<Utc>,
    pub error: Option<String>,
}

pub enum HealthStatus {
    Healthy,
    Degraded,
    Unreachable,
    Unauthorized,
}

impl Registry {
    pub async fn health_check(&self) -> Result<RegistryHealth> {
        let start = Instant::now();

        match self.client.get(&format!("{}/api/v1/health", self.url)).await {
            Ok(resp) if resp.status().is_success() => {
                Ok(RegistryHealth {
                    name: self.name.clone(),
                    status: HealthStatus::Healthy,
                    response_time: start.elapsed(),
                    last_check: Utc::now(),
                    error: None,
                })
            }
            Ok(resp) if resp.status() == 401 => {
                Ok(RegistryHealth {
                    name: self.name.clone(),
                    status: HealthStatus::Unauthorized,
                    response_time: start.elapsed(),
                    last_check: Utc::now(),
                    error: Some("Authentication failed".to_string()),
                })
            }
            Err(e) => {
                Ok(RegistryHealth {
                    name: self.name.clone(),
                    status: HealthStatus::Unreachable,
                    response_time: start.elapsed(),
                    last_check: Utc::now(),
                    error: Some(e.to_string()),
                })
            }
        }
    }
}
```

#### Cross-Registry Task Discovery

Search across all enabled registries:

```bash
# Search all registries
periplon-executor tasks search "google drive"

# Output:
# official-marketplace:
#   - google-drive-upload@1.2.3 (downloads: 15.2k, rating: 4.8/5)
#   - google-drive-download@1.1.5 (downloads: 12.5k, rating: 4.7/5)
#
# company-internal:
#   - google-drive-enterprise@2.0.0 (downloads: 234, rating: 5.0/5)
#
# community-hub:
#   - google-drive-advanced@0.8.0 (downloads: 523, rating: 4.2/5)
```

#### Registry Conflict Resolution

When multiple registries have the same task name:

```yaml
# Conflict resolution strategies
conflict_resolution:
  strategy: "priority"  # priority, prompt, explicit

  # priority: Use highest priority registry
  # prompt: Ask user to choose
  # explicit: Require registry prefix (registry::task)

  # Per-task overrides
  overrides:
    "google-drive-upload":
      preferred_registry: "company-internal"
      reason: "Use enterprise version with enhanced features"
```

#### Offline Mode Support

Cache tasks locally for offline use:

```yaml
registries:
  - name: "official-marketplace"
    url: "https://registry.claude-tasks.io"
    features:
      offline_mode: true
      cache_policy:
        max_age: "7d"
        max_size: "500MB"
        location: "~/.claude/cache/official-marketplace"
```

```bash
# Pre-cache tasks for offline use
periplon-executor registry cache official-marketplace --all

# Use cached tasks when offline
periplon-executor run workflow.yaml --offline
```

### 9. CLI Commands

#### DSL Executor Extensions

```bash
# List available predefined tasks
periplon-executor tasks list

# List tasks from specific registry
periplon-executor tasks list --registry=company-internal

# Search tasks across all registries
periplon-executor tasks search "google drive"

# Search in specific registry
periplon-executor tasks search "google drive" --registry=official-marketplace

# Show task details
periplon-executor tasks show google-drive-upload

# Show task from specific registry
periplon-executor tasks show google-drive-upload --registry=official-marketplace

# Install task locally
periplon-executor tasks install google-drive-upload@1.2.0

# Install from specific registry
periplon-executor tasks install google-drive-upload@1.2.0 --registry=company-internal

# Update all task sources
periplon-executor tasks update

# Update specific registry
periplon-executor tasks update --registry=official-marketplace

# Validate task definition
periplon-executor tasks validate ./my-task.yaml

# Create new task from template
periplon-executor tasks new my-task --template=basic

# Registry Management Commands
# List all configured registries
periplon-executor registry list

# Add new registry
periplon-executor registry add \
  --name=my-registry \
  --url=https://registry.example.com \
  --trust-level=community \
  --priority=5

# Remove registry
periplon-executor registry remove my-registry

# Show registry details
periplon-executor registry show company-internal

# Test registry connectivity
periplon-executor registry test company-internal

# Sync registry cache
periplon-executor registry sync official-marketplace
```

### 9. Example Usage

#### Complete Workflow with Predefined Tasks

```yaml
name: "Automated Report Generation"
version: "1.0.0"

inputs:
  report_date:
    type: string
    required: true
  google_folder:
    type: string
    default: "root"

tasks:
  # Generate report data
  generate_data:
    description: "Generate sales report for ${workflow.report_date}"
    agent: data_analyst
    outputs:
      report_file:
        source:
          type: file
          path: "./sales_report.pdf"

  # Upload to Google Drive (predefined task)
  upload_to_drive:
    uses: "google-drive-upload@^1.2.0"
    depends_on: [generate_data]
    inputs:
      file_path: "${task.generate_data.report_file}"
      folder_id: "${workflow.google_folder}"
      credentials: "${env.GOOGLE_CREDENTIALS}"
    outputs:
      drive_url: "${task.share_url}"

  # Create Notion page (predefined task)
  create_notion_page:
    uses: "notion-create-page@^1.0.0"
    depends_on: [upload_to_drive]
    inputs:
      database_id: "${env.NOTION_DB_ID}"
      title: "Sales Report - ${workflow.report_date}"
      properties:
        Date: "${workflow.report_date}"
        DriveLink: "${task.upload_to_drive.drive_url}"
      api_token: "${env.NOTION_TOKEN}"

  # Notify team (predefined task)
  send_slack_notification:
    uses: "slack-send-message@^2.0.0"
    depends_on: [create_notion_page]
    inputs:
      channel: "#reports"
      message: "📊 Sales report for ${workflow.report_date} is ready: ${task.create_notion_page.page_url}"
      webhook_url: "${env.SLACK_WEBHOOK}"

agents:
  data_analyst:
    description: "Analyze sales data and generate PDF report"
    tools: ["Read", "Write", "Bash"]
```

### 10. Testing Strategy

#### Unit Tests
- Task schema parsing
- Version constraint matching
- Dependency resolution
- Input/output validation

#### Integration Tests
- Git repository cloning
- Multi-source discovery
- Task reference resolution
- End-to-end workflow execution

#### Example Test Suite

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_predefined_task() {
        let yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "test-task"
  version: "1.0.0"
spec:
  inputs:
    input1: {type: string, required: true}
  outputs:
    output1: {type: string}
"#;
        let task = PredefinedTask::from_yaml(yaml).unwrap();
        assert_eq!(task.metadata.name, "test-task");
    }

    #[tokio::test]
    async fn test_version_constraint_matching() {
        let constraint = VersionConstraint::parse("^1.2.0").unwrap();
        assert!(constraint.matches("1.2.3"));
        assert!(constraint.matches("1.9.0"));
        assert!(!constraint.matches("2.0.0"));
    }

    #[tokio::test]
    async fn test_dependency_resolution() {
        // Test diamond dependency resolution
        let tasks = vec![
            create_task("A", "1.0.0", vec![("B", "^1.0"), ("C", "^1.0")]),
            create_task("B", "1.0.0", vec![("D", "^2.0")]),
            create_task("C", "1.0.0", vec![("D", "^2.0")]),
            create_task("D", "2.1.0", vec![]),
        ];

        let resolved = resolve_dependencies(&tasks).await.unwrap();
        assert_eq!(resolved.get("D").unwrap(), "2.1.0");
    }
}
```

### 11. Migration Path

#### Backward Compatibility

Existing workflows continue to work unchanged. New features opt-in via `uses:` syntax.

#### Converting Inline Tasks to Predefined

Tool to extract reusable tasks:

```bash
# Extract task from workflow
periplon-executor tasks extract workflow.yaml upload_to_drive > google-drive-upload.task.yaml

# Replace inline task with reference
periplon-executor tasks replace workflow.yaml upload_to_drive google-drive-upload@1.0.0
```

### 12. Documentation Requirements

1. **User Guide**: How to use predefined tasks in workflows
2. **Author Guide**: How to create and publish tasks
3. **API Reference**: Task definition schema reference
4. **Integration Guides**: Google, Notion, GitHub examples
5. **Security Guide**: Trust model and best practices

### 13. Future Enhancements

1. **Task Composition**: Combine multiple predefined tasks into higher-level tasks
2. **Conditional Tasks**: Tasks that adapt based on runtime conditions
3. **Streaming Tasks**: Tasks that produce incremental outputs
4. **Marketplace Analytics**: Download stats, ratings, reviews
5. **IDE Integration**: VSCode extension for task discovery/completion
6. **MCP Server Bridge**: Auto-generate predefined tasks from MCP servers
7. **AI-Generated Tasks**: Use LLM to generate task definitions from natural language

## Summary

This implementation plan provides a comprehensive roadmap for adding predefined task functionality to the DSL system, including support for multiple marketplaces and task groups. The phased approach allows for incremental delivery while building toward a full-featured task ecosystem.

### Key Features

**1. Predefined Tasks**
- Reusable task definitions with semantic versioning
- Reference or embed tasks in workflows
- Comprehensive input/output validation
- Dependency management and resolution

**2. Multiple Marketplace Support**
- Official public marketplace
- Private company/organization registries
- Community-driven marketplaces
- Self-hosted registries
- Priority-based resolution across registries
- Registry-specific task references (`registry::task@version`)
- Offline mode with intelligent caching
- Health monitoring and failover

**3. Task Groups**
- Bundle related tasks into cohesive suites
- Shared configuration across grouped tasks
- Predefined workflows within groups
- Namespace management to avoid conflicts
- Selective or full group imports
- Platform integration suites (Google, Slack, GitHub, etc.)

**4. Discovery & Distribution**
- Local filesystem (`.claude/tasks/`)
- Git repositories with auto-update
- Multiple registry support with authentication
- Full-text search across all sources
- Category and tag filtering
- Manifest-based auto-discovery

**5. Security & Trust**
- Four-level trust model (trusted, verified, community, untrusted)
- Per-registry trust configuration
- Permission scoping and validation
- Signature verification (future)
- Approval workflows for community tasks

### Implementation Timeline

**Total Duration**: 12 weeks for complete implementation

- **Phase 1** (Weeks 1-2): Local predefined tasks
- **Phase 2** (Weeks 3-4): Git repository support
- **Phase 3** (Weeks 5-6): Versioning & dependency resolution
- **Phase 4** (Weeks 7-8): Task groups
- **Phase 5** (Weeks 9-10): Multiple marketplace support
- **Phase 6** (Weeks 11-12): Publishing & marketplace infrastructure

**Post-Implementation**: Ongoing ecosystem growth and community task development

### Key Success Metrics

**Technical Metrics**:
- 90%+ test coverage for predefined task system
- Sub-100ms task discovery performance
- Sub-500ms cross-registry search
- 99.9% registry uptime (with mirrors)
- Zero breaking changes to existing workflows

**Ecosystem Metrics**:
- 50+ official predefined tasks in first 3 months
- 10+ task groups (integration suites) in first 6 months
- 5+ community task packages published
- 3+ operational registries (official, community, example company)
- 100+ total tasks across all registries within 6 months

**Adoption Metrics**:
- 1000+ task downloads in first quarter
- 50+ active workflows using predefined tasks
- 10+ organizations running private registries
- 80%+ user satisfaction with task discovery

### Use Case Examples

**1. Enterprise Integration**: Company maintains internal registry with proprietary integrations (Salesforce, SAP, internal APIs)

**2. DevOps Automation**: Public task groups for CI/CD pipelines (Docker, Kubernetes, cloud providers)

**3. Data Engineering**: ETL task suites for common data processing patterns

**4. SaaS Integration**: Community marketplace for popular SaaS platforms (Google, Notion, Slack, GitHub)

**5. Rapid Prototyping**: Developers quickly assemble workflows from pre-built tasks without writing custom agent logic

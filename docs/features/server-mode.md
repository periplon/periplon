# DSL Server Mode: Workflow Orchestration at Scale

## Overview

Transform the DSL executor from a CLI tool into a scalable workflow orchestration platform with REST API, WebSocket streaming, and web-based management interface. The server functionality will be integrated into the same `periplon-executor` binary, supporting both CLI and server modes.

## Unified Binary Architecture

The `periplon-executor` binary operates in multiple modes based on command-line arguments:

```bash
# CLI Mode (existing functionality)
periplon-executor template                    # Generate template
periplon-executor generate "desc" -o file.yaml # Generate workflow
periplon-executor validate workflow.yaml      # Validate workflow
periplon-executor run workflow.yaml           # Execute workflow locally

# Server Mode (new functionality)
periplon-executor server                      # Start HTTP/WebSocket server
periplon-executor server --port 8080          # Custom port
periplon-executor server --config server.toml # Custom config
periplon-executor worker                      # Start background worker
periplon-executor worker --concurrency 5      # Multiple workers
periplon-executor migrate                     # Run database migrations
periplon-executor migrate rollback            # Rollback migrations
```

### Shared Code Architecture

```rust
// src/bin/periplon-executor.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "periplon-executor")]
#[command(about = "DSL workflow executor - CLI and server modes")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a workflow template
    Template {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate workflow from natural language
    Generate {
        description: String,
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Validate a workflow file
    Validate {
        workflow_file: PathBuf,
    },

    /// Run a workflow
    Run {
        workflow_file: PathBuf,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        resume: Option<String>,
        #[arg(long)]
        parallel: bool,
    },

    /// Start the server
    Server {
        #[arg(short, long, default_value = "8080")]
        port: u16,
        #[arg(short, long)]
        config: Option<PathBuf>,
        #[arg(long)]
        workers: bool,  // Also start workers in same process
    },

    /// Start a background worker
    Worker {
        #[arg(short, long, default_value = "3")]
        concurrency: usize,
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Database migration commands
    Migrate {
        #[command(subcommand)]
        action: Option<MigrateAction>,
    },
}

#[derive(Subcommand)]
enum MigrateAction {
    /// Run pending migrations
    Up,
    /// Rollback last migration
    Down,
    /// Show migration status
    Status,
}
```

### Feature Flags

```toml
# Cargo.toml
[features]
default = ["cli", "server"]
cli = []
server = ["axum", "sqlx", "redis", "tokio-full"]
full = ["cli", "server"]

[[bin]]
name = "periplon-executor"
path = "src/bin/periplon-executor.rs"
required-features = ["cli"]  # CLI features always included
```

### Benefits of Unified Binary

1. **Single Deployment**: One binary for all use cases
2. **Shared Code**: DSL parsing, validation, execution engine shared
3. **Simplified Maintenance**: No code duplication
4. **Flexible Deployment**: Run as CLI, server, or both
5. **Easy Testing**: Same binary for local dev and production

## Configuration Subsystem

The server supports multiple configuration sources with precedence:

```
Command Line Args > Environment Variables > Config File > Defaults
```

### Configuration File Format

```toml
# server.toml - Main configuration file

[server]
host = "0.0.0.0"
port = 8080
workers = true              # Also start workers in same process
worker_concurrency = 3
log_level = "info"
environment = "production"  # development, staging, production

[server.tls]
enabled = true
cert_path = "/etc/ssl/certs/server.crt"
key_path = "/etc/ssl/private/server.key"
min_version = "1.3"

[server.cors]
allowed_origins = ["https://app.example.com", "https://admin.example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"]
allow_credentials = true

[storage]
# Storage backend: "filesystem", "s3", "postgres"
backend = "filesystem"

[storage.filesystem]
base_path = "./data"
workflows_dir = "workflows"
executions_dir = "executions"
checkpoints_dir = "checkpoints"
logs_dir = "logs"

[storage.s3]
endpoint = "https://s3.amazonaws.com"  # Or MinIO endpoint
region = "us-east-1"
bucket = "workflow-storage"
access_key_id = "${AWS_ACCESS_KEY_ID}"  # From env var
secret_access_key = "${AWS_SECRET_ACCESS_KEY}"
path_prefix = "prod/"

[storage.postgres]
url = "postgres://user:pass@localhost:5432/workflows"
max_connections = 20
min_connections = 5
connection_timeout = 30
idle_timeout = 600

[queue]
# Queue backend: "filesystem", "s3", "postgres", "redis"
backend = "filesystem"

[queue.filesystem]
queue_dir = "./queue"
poll_interval_ms = 1000
lock_timeout_secs = 300

[queue.s3]
endpoint = "https://s3.amazonaws.com"
bucket = "workflow-queue"
region = "us-east-1"
poll_interval_ms = 2000
lock_timeout_secs = 300
visibility_timeout_secs = 600

[queue.postgres]
url = "postgres://user:pass@localhost:5432/workflows"
poll_interval_ms = 500
max_retries = 3

[queue.redis]
url = "redis://localhost:6379/0"
max_connections = 10

[auth]
jwt_secret = "${JWT_SECRET}"  # From env var
jwt_expiration_secs = 3600
refresh_token_expiration_secs = 2592000  # 30 days
session_max_idle_secs = 1800
password_min_length = 12
mfa_enabled = false

[auth.oauth]
enabled = true

[[auth.oauth.providers]]
name = "github"
client_id = "${GITHUB_CLIENT_ID}"
client_secret = "${GITHUB_CLIENT_SECRET}"
scopes = ["user:email"]

[[auth.oauth.providers]]
name = "google"
client_id = "${GOOGLE_CLIENT_ID}"
client_secret = "${GOOGLE_CLIENT_SECRET}"
scopes = ["openid", "email", "profile"]

[rate_limit]
enabled = true
global_requests_per_minute = 10000
per_user_requests_per_minute = 100
per_api_key_requests_per_minute = 1000
per_ip_requests_per_minute = 60

[monitoring]
metrics_enabled = true
metrics_port = 9090
tracing_enabled = false
tracing_endpoint = "http://jaeger:14268/api/traces"

[reliability]
# Reliability and resilience settings
max_retries = 3
retry_backoff_ms = 1000
retry_backoff_multiplier = 2.0
circuit_breaker_threshold = 5
circuit_breaker_timeout_secs = 60
health_check_interval_secs = 30
```

### Environment Variable Support

All config values support environment variable substitution:

```bash
# Override storage backend
export STORAGE_BACKEND=s3
export STORAGE_S3_BUCKET=my-workflows

# Override database URL
export STORAGE_POSTGRES_URL=postgres://...

# Secrets from env
export JWT_SECRET=super-secret-key
export AWS_ACCESS_KEY_ID=AKIA...
export AWS_SECRET_ACCESS_KEY=...

# Start server
periplon-executor server --config server.toml
```

### Configuration Validation

```rust
// Configuration is validated on startup
pub struct Config {
    server: ServerConfig,
    storage: StorageConfig,
    queue: QueueConfig,
    auth: AuthConfig,
    rate_limit: RateLimitConfig,
    monitoring: MonitoringConfig,
    reliability: ReliabilityConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load from file, env vars, and CLI args
        // Validate all required fields
        // Fail fast on invalid config
    }

    pub fn validate(&self) -> Result<()> {
        // Check storage backend is configured
        // Validate queue backend matches deployment
        // Ensure secrets are set in production
        // Validate TLS certs exist if enabled
    }
}
```

## Architecture

### Hexagonal Architecture Integration

```
┌─────────────────────────────────────────────────────────────┐
│                     Primary Adapters                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   REST API   │  │  WebSocket   │  │   Web UI     │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
└─────────┼──────────────────┼──────────────────┼──────────────┘
          │                  │                  │
┌─────────┼──────────────────┼──────────────────┼──────────────┐
│         │      Primary Ports (Inbound)        │              │
│  ┌──────▼──────────────────▼──────────────────▼───────┐     │
│  │  WorkflowService │ ExecutionService │ MonitorService│     │
│  └──────┬──────────────────┬──────────────────┬────────┘     │
└─────────┼──────────────────┼──────────────────┼──────────────┘
          │                  │                  │
┌─────────┼──────────────────┼──────────────────┼──────────────┐
│         │         Domain Core (Business Logic)│              │
│  ┌──────▼──────────────────▼──────────────────▼───────┐     │
│  │  Workflow │ Execution Engine │ State Machine       │     │
│  │  Task Graph │ Message Bus │ Hook System            │     │
│  └──────┬──────────────────┬──────────────────┬────────┘     │
└─────────┼──────────────────┼──────────────────┼──────────────┘
          │                  │                  │
┌─────────┼──────────────────┼──────────────────┼──────────────┐
│         │     Secondary Ports (Outbound)      │              │
│  ┌──────▼──────────────────▼──────────────────▼───────┐     │
│  │ WorkflowRepo │ StateRepo │ EventPublisher          │     │
│  └──────┬──────────────────┬──────────────────┬────────┘     │
└─────────┼──────────────────┼──────────────────┼──────────────┘
          │                  │                  │
┌─────────┼──────────────────┼──────────────────┼──────────────┐
│         │      Secondary Adapters             │              │
│  ┌──────▼──────┐  ┌────────▼──────┐  ┌───────▼────────┐    │
│  │  PostgreSQL │  │     Redis     │  │   S3/Blob      │    │
│  └─────────────┘  └───────────────┘  └────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

#### 1. HTTP Server (Primary Adapter)
- **Framework**: `axum` for async HTTP handling
- **Features**:
  - RESTful API for workflow CRUD
  - Health/metrics endpoints
  - Request validation middleware
  - Rate limiting
  - Authentication/authorization
  - OpenAPI/Swagger documentation

#### 2. WebSocket Server (Primary Adapter)
- **Framework**: `axum` WebSocket support
- **Features**:
  - Real-time execution updates
  - Task progress streaming
  - Log streaming
  - Multi-client broadcast
  - Connection pooling
  - Heartbeat/reconnection

#### 3. Workflow Service (Primary Port)
- **Responsibilities**:
  - Workflow CRUD operations
  - Version management
  - Validation orchestration
  - Template generation
  - Natural language conversion

#### 4. Execution Service (Primary Port)
- **Responsibilities**:
  - Workflow execution lifecycle
  - Queue management
  - Scheduling
  - Cancellation
  - Pause/resume
  - Retry logic

#### 5. Monitoring Service (Primary Port)
- **Responsibilities**:
  - Execution metrics
  - Performance analytics
  - Resource utilization
  - Error tracking
  - Audit logging

#### 6. Repository Layer (Secondary Ports)
- **WorkflowRepository**:
  - Workflow persistence
  - Version history
  - Search/filtering

- **ExecutionRepository**:
  - Execution state tracking
  - History archival
  - Result storage

- **StateRepository**:
  - Checkpoint management
  - State snapshots
  - Resume data

#### 7. Event System (Secondary Port)
- **EventPublisher**:
  - Workflow lifecycle events
  - Task state changes
  - Execution completion
  - Error notifications

## Pluggable Storage Layer

The system supports multiple storage backends through a unified interface, allowing deployment flexibility from single-machine to cloud-scale.

### Storage Abstraction

```rust
// Core storage trait - all backends implement this
#[async_trait]
pub trait WorkflowStorage: Send + Sync {
    async fn store_workflow(&self, workflow: &Workflow) -> Result<Uuid>;
    async fn get_workflow(&self, id: Uuid) -> Result<Option<Workflow>>;
    async fn update_workflow(&self, id: Uuid, workflow: &Workflow) -> Result<()>;
    async fn delete_workflow(&self, id: Uuid) -> Result<()>;
    async fn list_workflows(&self, filter: WorkflowFilter) -> Result<Vec<Workflow>>;
    async fn get_workflow_version(&self, id: Uuid, version: &str) -> Result<Option<Workflow>>;
}

#[async_trait]
pub trait ExecutionStorage: Send + Sync {
    async fn store_execution(&self, execution: &Execution) -> Result<Uuid>;
    async fn get_execution(&self, id: Uuid) -> Result<Option<Execution>>;
    async fn update_execution(&self, id: Uuid, execution: &Execution) -> Result<()>;
    async fn list_executions(&self, filter: ExecutionFilter) -> Result<Vec<Execution>>;
    async fn store_execution_log(&self, execution_id: Uuid, log: &ExecutionLog) -> Result<()>;
    async fn get_execution_logs(&self, execution_id: Uuid) -> Result<Vec<ExecutionLog>>;
}

#[async_trait]
pub trait CheckpointStorage: Send + Sync {
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<Uuid>;
    async fn get_checkpoint(&self, execution_id: Uuid, name: &str) -> Result<Option<Checkpoint>>;
    async fn list_checkpoints(&self, execution_id: Uuid) -> Result<Vec<Checkpoint>>;
}

// Factory pattern for storage backends
pub enum StorageBackend {
    Filesystem(FilesystemConfig),
    S3(S3Config),
    Postgres(PostgresConfig),
}

impl StorageBackend {
    pub fn create_workflow_storage(&self) -> Box<dyn WorkflowStorage> {
        match self {
            StorageBackend::Filesystem(config) => {
                Box::new(FilesystemWorkflowStorage::new(config))
            }
            StorageBackend::S3(config) => {
                Box::new(S3WorkflowStorage::new(config))
            }
            StorageBackend::Postgres(config) => {
                Box::new(PostgresWorkflowStorage::new(config))
            }
        }
    }
}
```

### Filesystem Storage Backend

**Use Case**: Development, small deployments, single-machine setups

**Structure**:
```
data/
├── workflows/
│   ├── {workflow-id}/
│   │   ├── metadata.json       # Workflow metadata
│   │   ├── definition.yaml     # Workflow YAML
│   │   └── versions/
│   │       ├── v1.0.0.yaml
│   │       └── v2.0.0.yaml
├── executions/
│   ├── {execution-id}/
│   │   ├── metadata.json       # Execution state
│   │   ├── result.json         # Execution result
│   │   └── logs/
│   │       ├── 001.jsonl       # Log chunks (JSONL)
│   │       ├── 002.jsonl
│   │       └── current.jsonl
├── checkpoints/
│   ├── {execution-id}/
│   │   ├── checkpoint-1.json
│   │   └── checkpoint-2.json
└── queue/
    ├── pending/
    │   ├── {job-id}.json
    │   └── {job-id}.lock       # Lock file for claiming
    ├── processing/
    │   └── {job-id}.json
    └── completed/
        └── {job-id}.json
```

**Implementation**:
```rust
pub struct FilesystemWorkflowStorage {
    base_path: PathBuf,
}

impl FilesystemWorkflowStorage {
    async fn store_workflow(&self, workflow: &Workflow) -> Result<Uuid> {
        let id = workflow.id;
        let workflow_dir = self.base_path.join("workflows").join(id.to_string());

        // Create directory structure
        tokio::fs::create_dir_all(&workflow_dir).await?;

        // Write metadata
        let metadata_path = workflow_dir.join("metadata.json");
        let metadata = serde_json::to_string_pretty(&workflow.metadata)?;
        tokio::fs::write(metadata_path, metadata).await?;

        // Write YAML definition
        let definition_path = workflow_dir.join("definition.yaml");
        let yaml = serde_yaml::to_string(&workflow.definition)?;
        tokio::fs::write(definition_path, yaml).await?;

        // Use fsync for durability
        self.sync_directory(&workflow_dir).await?;

        Ok(id)
    }

    // Ensure durability with fsync
    async fn sync_directory(&self, path: &Path) -> Result<()> {
        use std::os::unix::fs::FileExt;
        let file = std::fs::File::open(path)?;
        file.sync_all()?;
        Ok(())
    }
}
```

**Reliability Features**:
- Atomic writes using temp files + rename
- fsync for durability
- Lock files for queue coordination
- Automatic recovery from partial writes

### S3-Compatible Storage Backend

**Use Case**: Cloud deployments, distributed systems, high availability

**Supported Systems**:
- AWS S3
- MinIO
- DigitalOcean Spaces
- Backblaze B2
- Cloudflare R2
- Any S3-compatible object storage

**Object Key Structure**:
```
{prefix}/workflows/{workflow-id}/metadata.json
{prefix}/workflows/{workflow-id}/definition.yaml
{prefix}/workflows/{workflow-id}/versions/v1.0.0.yaml
{prefix}/executions/{execution-id}/metadata.json
{prefix}/executions/{execution-id}/result.json
{prefix}/executions/{execution-id}/logs/001.jsonl
{prefix}/checkpoints/{execution-id}/checkpoint-1.json
{prefix}/queue/pending/{job-id}.json
{prefix}/queue/processing/{job-id}.json
```

**Implementation**:
```rust
pub struct S3WorkflowStorage {
    client: aws_sdk_s3::Client,
    bucket: String,
    prefix: String,
}

impl S3WorkflowStorage {
    async fn store_workflow(&self, workflow: &Workflow) -> Result<Uuid> {
        let id = workflow.id;

        // Store metadata
        let metadata_key = format!(
            "{}/workflows/{}/metadata.json",
            self.prefix, id
        );
        let metadata = serde_json::to_vec_pretty(&workflow.metadata)?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&metadata_key)
            .body(metadata.into())
            .content_type("application/json")
            .storage_class(StorageClass::StandardIa)  // Infrequent access
            .server_side_encryption(ServerSideEncryption::Aes256)
            .send()
            .await?;

        // Store YAML definition
        let definition_key = format!(
            "{}/workflows/{}/definition.yaml",
            self.prefix, id
        );
        let yaml = serde_yaml::to_string(&workflow.definition)?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&definition_key)
            .body(yaml.into_bytes().into())
            .content_type("application/x-yaml")
            .send()
            .await?;

        Ok(id)
    }

    async fn get_workflow(&self, id: Uuid) -> Result<Option<Workflow>> {
        // Use GetObject with retry logic
        let metadata_key = format!("{}/workflows/{}/metadata.json", self.prefix, id);

        let result = self.retry_with_backoff(|| async {
            self.client
                .get_object()
                .bucket(&self.bucket)
                .key(&metadata_key)
                .send()
                .await
        }).await;

        // Handle not found vs error
        match result {
            Ok(output) => {
                let bytes = output.body.collect().await?.into_bytes();
                let metadata: WorkflowMetadata = serde_json::from_slice(&bytes)?;
                Ok(Some(Workflow { metadata, ..Default::default() }))
            }
            Err(e) if e.is_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }
}
```

**Reliability Features**:
- Server-side encryption (AES-256 or KMS)
- Versioning enabled on bucket
- Lifecycle policies for archival
- Retry with exponential backoff
- Multi-region replication (optional)
- Object locking for compliance

### PostgreSQL Storage Backend

**Use Case**: Transactional consistency, complex queries, relational data

**Implementation**:
```rust
pub struct PostgresWorkflowStorage {
    pool: sqlx::PgPool,
}

impl PostgresWorkflowStorage {
    async fn store_workflow(&self, workflow: &Workflow) -> Result<Uuid> {
        let id = workflow.id;
        let definition_json = serde_json::to_value(&workflow.definition)?;

        // Use transaction for atomicity
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
            INSERT INTO workflows (id, name, version, description, definition, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET definition = $5, updated_at = NOW()
            "#,
            id,
            workflow.name,
            workflow.version,
            workflow.description,
            definition_json,
            workflow.created_by
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(id)
    }
}
```

**Reliability Features**:
- ACID transactions
- Write-ahead logging (WAL)
- Point-in-time recovery
- Streaming replication
- Connection pooling
- Automatic failover with Patroni/Stolon

### Hybrid Storage Strategies

**Strategy 1: Hot/Cold Storage**
- Recent executions → PostgreSQL (fast queries)
- Old executions → S3 (cost-effective archival)
- Automatic archival after 90 days

**Strategy 2: Tiered Storage**
- Metadata → PostgreSQL (queryable)
- Large artifacts → S3 (logs, outputs)
- Queue → Redis (low latency)

**Implementation**:
```rust
pub struct HybridStorage {
    hot_storage: Box<dyn WorkflowStorage>,    // PostgreSQL
    cold_storage: Box<dyn WorkflowStorage>,   // S3
    archive_threshold: Duration,
}

impl HybridStorage {
    async fn get_execution(&self, id: Uuid) -> Result<Option<Execution>> {
        // Try hot storage first
        if let Some(execution) = self.hot_storage.get_execution(id).await? {
            return Ok(Some(execution));
        }

        // Fall back to cold storage
        if let Some(execution) = self.cold_storage.get_execution(id).await? {
            return Ok(Some(execution));
        }

        Ok(None)
    }

    async fn archive_old_executions(&self) -> Result<()> {
        let cutoff = Utc::now() - self.archive_threshold;

        let old_executions = self.hot_storage
            .list_executions(ExecutionFilter {
                completed_before: Some(cutoff),
                ..Default::default()
            })
            .await?;

        for execution in old_executions {
            // Move to cold storage
            self.cold_storage.store_execution(&execution).await?;

            // Delete from hot storage
            self.hot_storage.delete_execution(execution.id).await?;
        }

        Ok(())
    }
}
```

## Pluggable Queue System

The worker queue supports multiple backends for job distribution and coordination.

### Queue Abstraction

```rust
#[async_trait]
pub trait WorkQueue: Send + Sync {
    /// Enqueue a new job
    async fn enqueue(&self, job: Job) -> Result<Uuid>;

    /// Dequeue next available job (with locking)
    async fn dequeue(&self, worker_id: &str) -> Result<Option<Job>>;

    /// Mark job as completed
    async fn complete(&self, job_id: Uuid) -> Result<()>;

    /// Mark job as failed
    async fn fail(&self, job_id: Uuid, error: &str) -> Result<()>;

    /// Requeue job for retry
    async fn requeue(&self, job_id: Uuid, delay: Duration) -> Result<()>;

    /// Heartbeat to keep job locked
    async fn heartbeat(&self, job_id: Uuid, worker_id: &str) -> Result<()>;

    /// Release stuck jobs (dead worker recovery)
    async fn release_stale_jobs(&self, timeout: Duration) -> Result<Vec<Uuid>>;

    /// Get queue statistics
    async fn stats(&self) -> Result<QueueStats>;
}

pub struct Job {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub execution_id: Uuid,
    pub priority: i32,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub attempts: u32,
    pub max_retries: u32,
}

pub struct QueueStats {
    pub pending: usize,
    pub processing: usize,
    pub completed: usize,
    pub failed: usize,
}
```

### Filesystem Queue Backend

**Use Case**: Single machine, development, simple deployments

**Implementation**:
```rust
pub struct FilesystemQueue {
    queue_dir: PathBuf,
    poll_interval: Duration,
    lock_timeout: Duration,
}

impl FilesystemQueue {
    async fn dequeue(&self, worker_id: &str) -> Result<Option<Job>> {
        let pending_dir = self.queue_dir.join("pending");

        // List all pending jobs
        let mut entries = tokio::fs::read_dir(&pending_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip lock files
            if path.extension() == Some(OsStr::new("lock")) {
                continue;
            }

            let job_id = path.file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| Uuid::parse_str(s).ok());

            if let Some(job_id) = job_id {
                // Try to acquire lock
                if self.try_lock(&job_id, worker_id).await? {
                    // Read job file
                    let contents = tokio::fs::read_to_string(&path).await?;
                    let job: Job = serde_json::from_str(&contents)?;

                    // Move to processing
                    let processing_path = self.queue_dir
                        .join("processing")
                        .join(format!("{}.json", job_id));
                    tokio::fs::rename(&path, &processing_path).await?;

                    return Ok(Some(job));
                }
            }
        }

        Ok(None)
    }

    async fn try_lock(&self, job_id: &Uuid, worker_id: &str) -> Result<bool> {
        let lock_path = self.queue_dir
            .join("pending")
            .join(format!("{}.lock", job_id));

        // Try to create lock file exclusively
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                // Write worker ID and timestamp
                let lock_data = format!("{}:{}", worker_id, Utc::now().to_rfc3339());
                file.write_all(lock_data.as_bytes())?;
                file.sync_all()?;
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Check if lock is stale
                if self.is_lock_stale(&lock_path).await? {
                    // Remove stale lock and retry
                    tokio::fs::remove_file(&lock_path).await?;
                    self.try_lock(job_id, worker_id).await
                } else {
                    Ok(false)
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn is_lock_stale(&self, lock_path: &Path) -> Result<bool> {
        let metadata = tokio::fs::metadata(lock_path).await?;
        let age = Utc::now().signed_duration_since(
            DateTime::from(metadata.modified()?)
        );

        Ok(age.to_std()? > self.lock_timeout)
    }

    async fn heartbeat(&self, job_id: Uuid, worker_id: &str) -> Result<()> {
        let lock_path = self.queue_dir
            .join("processing")
            .join(format!("{}.lock", job_id));

        // Update lock file timestamp
        let lock_data = format!("{}:{}", worker_id, Utc::now().to_rfc3339());
        tokio::fs::write(&lock_path, lock_data).await?;

        Ok(())
    }
}
```

**Reliability Features**:
- Exclusive file locking (flock on Unix)
- Lock files with timestamps
- Stale lock detection and recovery
- Atomic file operations (rename)
- Directory fsync for durability

### S3 Queue Backend

**Use Case**: Distributed workers, cloud deployments, serverless

**Implementation**:
```rust
pub struct S3Queue {
    client: aws_sdk_s3::Client,
    bucket: String,
    prefix: String,
    visibility_timeout: Duration,
    poll_interval: Duration,
}

impl S3Queue {
    async fn dequeue(&self, worker_id: &str) -> Result<Option<Job>> {
        let pending_prefix = format!("{}/queue/pending/", self.prefix);

        // List pending jobs
        let objects = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&pending_prefix)
            .max_keys(10)  // Batch processing
            .send()
            .await?;

        for object in objects.contents().unwrap_or_default() {
            let key = object.key().unwrap();
            let job_id = self.extract_job_id(key)?;

            // Try to claim job with conditional PUT (optimistic locking)
            if self.try_claim_job(&job_id, worker_id).await? {
                // Download job data
                let job_key = format!("{}/queue/pending/{}.json", self.prefix, job_id);
                let output = self.client
                    .get_object()
                    .bucket(&self.bucket)
                    .key(&job_key)
                    .send()
                    .await?;

                let bytes = output.body.collect().await?.into_bytes();
                let job: Job = serde_json::from_slice(&bytes)?;

                // Move to processing
                let processing_key = format!(
                    "{}/queue/processing/{}.json",
                    self.prefix, job_id
                );

                self.client
                    .copy_object()
                    .bucket(&self.bucket)
                    .copy_source(format!("{}/{}", self.bucket, job_key))
                    .key(&processing_key)
                    .metadata("worker_id", worker_id)
                    .metadata("claimed_at", Utc::now().to_rfc3339())
                    .send()
                    .await?;

                // Delete from pending
                self.client
                    .delete_object()
                    .bucket(&self.bucket)
                    .key(&job_key)
                    .send()
                    .await?;

                return Ok(Some(job));
            }
        }

        Ok(None)
    }

    async fn try_claim_job(&self, job_id: &Uuid, worker_id: &str) -> Result<bool> {
        let lock_key = format!("{}/queue/locks/{}.lock", self.prefix, job_id);

        // Try to create lock object with conditional PUT
        let lock_data = format!("{}:{}", worker_id, Utc::now().to_rfc3339());

        let result = self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&lock_key)
            .body(lock_data.into_bytes().into())
            .metadata("worker_id", worker_id)
            .metadata("expires_at", (Utc::now() + self.visibility_timeout).to_rfc3339())
            .if_none_match("*")  // Only create if doesn't exist
            .send()
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(e) if e.is_precondition_failed() => {
                // Lock exists, check if stale
                self.check_stale_lock(&lock_key).await
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn heartbeat(&self, job_id: Uuid, worker_id: &str) -> Result<()> {
        let lock_key = format!("{}/queue/locks/{}.lock", self.prefix, job_id);

        // Update lock expiration
        let lock_data = format!("{}:{}", worker_id, Utc::now().to_rfc3339());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&lock_key)
            .body(lock_data.into_bytes().into())
            .metadata("expires_at", (Utc::now() + self.visibility_timeout).to_rfc3339())
            .send()
            .await?;

        Ok(())
    }
}
```

**Reliability Features**:
- Conditional PUT for optimistic locking
- Visibility timeout
- Object metadata for lock tracking
- S3 versioning for recovery
- Lifecycle policies for cleanup

### PostgreSQL Queue Backend

**Use Case**: Transactional consistency, ACID guarantees, priority queues

**Implementation**:
```rust
pub struct PostgresQueue {
    pool: sqlx::PgPool,
}

impl PostgresQueue {
    async fn dequeue(&self, worker_id: &str) -> Result<Option<Job>> {
        // Use SELECT FOR UPDATE SKIP LOCKED for efficient dequeuing
        let result = sqlx::query_as!(
            Job,
            r#"
            UPDATE execution_queue
            SET
                status = 'processing',
                worker_id = $1,
                claimed_at = NOW(),
                updated_at = NOW()
            WHERE id = (
                SELECT id
                FROM execution_queue
                WHERE status = 'pending'
                AND (scheduled_at IS NULL OR scheduled_at <= NOW())
                ORDER BY priority DESC, created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#,
            worker_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn heartbeat(&self, job_id: Uuid, worker_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE execution_queue
            SET updated_at = NOW()
            WHERE id = $1 AND worker_id = $2
            "#,
            job_id,
            worker_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn release_stale_jobs(&self, timeout: Duration) -> Result<Vec<Uuid>> {
        let timeout_secs = timeout.as_secs() as i64;

        let stale_jobs = sqlx::query_scalar!(
            r#"
            UPDATE execution_queue
            SET
                status = 'pending',
                worker_id = NULL,
                claimed_at = NULL,
                attempts = attempts + 1,
                updated_at = NOW()
            WHERE status = 'processing'
            AND updated_at < NOW() - INTERVAL '1 second' * $1
            RETURNING id
            "#,
            timeout_secs
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(stale_jobs)
    }
}
```

**Reliability Features**:
- `SELECT FOR UPDATE SKIP LOCKED` (no blocking)
- ACID transactions
- Priority queue support
- Automatic stale job recovery
- Dead letter queue for failed jobs

### Redis Queue Backend

**Use Case**: High throughput, low latency, distributed locking

**Implementation**:
```rust
pub struct RedisQueue {
    client: redis::aio::MultiplexedConnection,
}

impl RedisQueue {
    async fn dequeue(&self, worker_id: &str) -> Result<Option<Job>> {
        // Use BRPOPLPUSH for atomic dequeue
        let result: Option<String> = redis::cmd("BRPOPLPUSH")
            .arg("executions:queue:pending")
            .arg(format!("executions:queue:processing:{}", worker_id))
            .arg(5)  // 5 second timeout
            .query_async(&mut self.client)
            .await?;

        if let Some(job_data) = result {
            let job: Job = serde_json::from_str(&job_data)?;

            // Set processing lock with TTL
            let lock_key = format!("executions:lock:{}", job.id);
            redis::cmd("SETEX")
                .arg(&lock_key)
                .arg(300)  // 5 minute TTL
                .arg(worker_id)
                .query_async(&mut self.client)
                .await?;

            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    async fn heartbeat(&self, job_id: Uuid, worker_id: &str) -> Result<()> {
        let lock_key = format!("executions:lock:{}", job_id);

        // Extend lock TTL
        redis::cmd("EXPIRE")
            .arg(&lock_key)
            .arg(300)
            .query_async(&mut self.client)
            .await?;

        Ok(())
    }
}
```

**Reliability Features**:
- Atomic operations (BRPOPLPUSH)
- Automatic expiration (TTL)
- Redis Sentinel for HA
- Redis Cluster for sharding
- Persistence (AOF + RDB)

## Database Schema

### PostgreSQL Tables

```sql
-- Workflows
CREATE TABLE workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    definition JSONB NOT NULL,  -- Full YAML as JSON
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by VARCHAR(255),
    tags TEXT[],
    is_active BOOLEAN DEFAULT TRUE,
    UNIQUE(name, version)
);

CREATE INDEX idx_workflows_name ON workflows(name);
CREATE INDEX idx_workflows_tags ON workflows USING GIN(tags);
CREATE INDEX idx_workflows_created_at ON workflows(created_at DESC);

-- Executions
CREATE TABLE executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID REFERENCES workflows(id),
    workflow_version VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,  -- queued, running, completed, failed, cancelled, paused
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    triggered_by VARCHAR(255),
    trigger_type VARCHAR(50),  -- manual, scheduled, webhook, api
    input_params JSONB,
    result JSONB,
    error TEXT,
    retry_count INT DEFAULT 0,
    parent_execution_id UUID REFERENCES executions(id)  -- for retries
);

CREATE INDEX idx_executions_workflow_id ON executions(workflow_id);
CREATE INDEX idx_executions_status ON executions(status);
CREATE INDEX idx_executions_started_at ON executions(started_at DESC);

-- Task Executions (detailed tracking)
CREATE TABLE task_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID REFERENCES executions(id) ON DELETE CASCADE,
    task_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    output TEXT,
    error TEXT,
    retry_count INT DEFAULT 0,
    parent_task_id UUID REFERENCES task_executions(id)
);

CREATE INDEX idx_task_executions_execution_id ON task_executions(execution_id);
CREATE INDEX idx_task_executions_status ON task_executions(status);

-- Execution Logs (streaming logs)
CREATE TABLE execution_logs (
    id BIGSERIAL PRIMARY KEY,
    execution_id UUID REFERENCES executions(id) ON DELETE CASCADE,
    task_execution_id UUID REFERENCES task_executions(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    level VARCHAR(20),  -- debug, info, warn, error
    message TEXT,
    metadata JSONB
);

CREATE INDEX idx_execution_logs_execution_id ON execution_logs(execution_id, timestamp DESC);
CREATE INDEX idx_execution_logs_task_execution_id ON execution_logs(task_execution_id, timestamp DESC);

-- Checkpoints (for resume capability)
CREATE TABLE checkpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID REFERENCES executions(id) ON DELETE CASCADE,
    checkpoint_name VARCHAR(255),
    state JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_checkpoints_execution_id ON checkpoints(execution_id, created_at DESC);

-- Schedules
CREATE TABLE schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID REFERENCES workflows(id),
    cron_expression VARCHAR(255),
    timezone VARCHAR(100) DEFAULT 'UTC',
    is_active BOOLEAN DEFAULT TRUE,
    input_params JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ
);

CREATE INDEX idx_schedules_workflow_id ON schedules(workflow_id);
CREATE INDEX idx_schedules_next_run_at ON schedules(next_run_at) WHERE is_active = TRUE;

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE,
    password_hash VARCHAR(255),  -- NULL for SSO-only users
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    avatar_url TEXT,
    organization_id UUID REFERENCES organizations(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE,
    is_system BOOLEAN DEFAULT FALSE,  -- Service accounts
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_organization_id ON users(organization_id);
CREATE INDEX idx_users_is_active ON users(is_active);

-- Organizations (Multi-tenancy)
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    logo_url TEXT,
    plan VARCHAR(50) DEFAULT 'free',  -- free, pro, enterprise
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_organizations_slug ON organizations(slug);

-- Teams
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(organization_id, name)
);

CREATE INDEX idx_teams_organization_id ON teams(organization_id);

-- Team Members
CREATE TABLE team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) DEFAULT 'member',  -- owner, admin, member
    added_at TIMESTAMPTZ DEFAULT NOW(),
    added_by UUID REFERENCES users(id),
    UNIQUE(team_id, user_id)
);

CREATE INDEX idx_team_members_team_id ON team_members(team_id);
CREATE INDEX idx_team_members_user_id ON team_members(user_id);

-- Roles
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    is_system BOOLEAN DEFAULT FALSE,  -- System roles can't be deleted
    organization_id UUID REFERENCES organizations(id),  -- NULL for global roles
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_roles_organization_id ON roles(organization_id);

-- Permissions
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    resource_type VARCHAR(100),  -- workflow, execution, user, etc.
    action VARCHAR(100),         -- create, read, update, delete, execute
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_permissions_resource_type ON permissions(resource_type);

-- Role Permissions
CREATE TABLE role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
    UNIQUE(role_id, permission_id)
);

CREATE INDEX idx_role_permissions_role_id ON role_permissions(role_id);

-- User Roles
CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE(user_id, role_id)
);

CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);

-- Permission Policies (ABAC - Attribute-Based Access Control)
CREATE TABLE permission_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    organization_id UUID REFERENCES organizations(id),
    conditions JSONB NOT NULL,  -- Policy rules in JSON
    effect VARCHAR(20) NOT NULL CHECK (effect IN ('allow', 'deny')),
    priority INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_permission_policies_organization_id ON permission_policies(organization_id);

-- API Keys
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    key_prefix VARCHAR(20) NOT NULL,  -- First chars for identification
    name VARCHAR(255),
    description TEXT,
    scopes TEXT[],                    -- Allowed scopes
    ip_whitelist INET[],              -- Allowed IPs
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);

-- OAuth Connections
CREATE TABLE oauth_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,    -- google, github, gitlab, etc.
    provider_user_id VARCHAR(255) NOT NULL,
    access_token_encrypted TEXT,
    refresh_token_encrypted TEXT,
    expires_at TIMESTAMPTZ,
    scopes TEXT[],
    profile_data JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(provider, provider_user_id)
);

CREATE INDEX idx_oauth_connections_user_id ON oauth_connections(user_id);
CREATE INDEX idx_oauth_connections_provider ON oauth_connections(provider);

-- MFA Settings
CREATE TABLE mfa_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE UNIQUE,
    method VARCHAR(50) NOT NULL,      -- totp, sms, email
    secret_encrypted TEXT,
    backup_codes_encrypted TEXT[],
    is_enabled BOOLEAN DEFAULT FALSE,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_mfa_settings_user_id ON mfa_settings(user_id);

-- User Sessions
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_token_hash ON user_sessions(token_hash);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);

-- Password Reset Tokens
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ
);

CREATE INDEX idx_password_reset_tokens_token_hash ON password_reset_tokens(token_hash);

-- Email Verification Tokens
CREATE TABLE email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ
);

CREATE INDEX idx_email_verification_tokens_token_hash ON email_verification_tokens(token_hash);
```

### Redis Data Structures

```
# Execution Queue (priority queue)
ZSET executions:queue
  Score: priority (higher = more urgent)
  Value: execution_id

# Active Executions
SET executions:active
  Members: execution_id

# Execution Status Cache
HASH execution:{execution_id}
  status, progress, current_task, etc.

# WebSocket Connections
SET ws:execution:{execution_id}
  Members: connection_id

# Rate Limiting
STRING rate:{user_id}:{endpoint}
  Value: request_count
  TTL: 60s

# Distributed Locks
STRING lock:execution:{execution_id}
  Value: worker_id
  TTL: 30s
```

## DSL CLI Feature Mapping

The server mode implements all DSL CLI functionality via API endpoints:

| CLI Command | API Endpoint | Description |
|------------|--------------|-------------|
| `periplon-executor template` | `GET /api/v1/workflows/template` | Generate YAML template |
| `periplon-executor generate "desc" -o file.yaml` | `POST /api/v1/workflows/generate` | NL to workflow |
| `periplon-executor validate workflow.yaml` | `POST /api/v1/workflows/{id}/validate` | Validate workflow |
| `periplon-executor run workflow.yaml` | `POST /api/v1/executions` | Execute workflow |
| `periplon-executor run --dry-run` | `POST /api/v1/workflows/{id}/dry-run` | Dry-run validation |
| `periplon-executor run --resume {checkpoint}` | `POST /api/v1/executions/{id}/resume` | Resume from checkpoint |
| `periplon-executor run --parallel` | Workflow config: `parallel: true` | Parallel execution |

### Enhanced DSL Features

```rust
// Additional server-mode features beyond CLI
pub struct WorkflowExecutionRequest {
    workflow_id: Uuid,
    input_params: Option<serde_json::Value>,
    priority: Option<i32>,           // Queue priority
    schedule: Option<Schedule>,      // Deferred execution
    notifications: Vec<Notification>, // Alert on completion
    tags: Vec<String>,               // Execution metadata
    dry_run: bool,                   // Validation-only mode
    parallel: bool,                  // Force parallel execution
    timeout: Option<Duration>,       // Override workflow timeout
}
```

## REST API Design

### Authentication & Session Management
```
# Basic Authentication
POST   /api/v1/auth/register          # User registration (if enabled)
POST   /api/v1/auth/login             # Login with credentials
POST   /api/v1/auth/logout            # Logout (invalidate token)
POST   /api/v1/auth/refresh           # Refresh JWT token
GET    /api/v1/auth/me                # Get current user profile

# Multi-Factor Authentication
POST   /api/v1/auth/mfa/enable        # Enable MFA for account
POST   /api/v1/auth/mfa/disable       # Disable MFA
POST   /api/v1/auth/mfa/verify        # Verify MFA code during login
POST   /api/v1/auth/mfa/backup-codes  # Generate backup codes

# SSO / OAuth
GET    /api/v1/auth/oauth/{provider}  # OAuth login (Google, GitHub, etc)
GET    /api/v1/auth/oauth/callback    # OAuth callback handler
POST   /api/v1/auth/saml              # SAML SSO login
GET    /api/v1/auth/saml/metadata     # SAML metadata endpoint

# Password Management
POST   /api/v1/auth/password/reset    # Request password reset
POST   /api/v1/auth/password/confirm  # Confirm password reset
PUT    /api/v1/auth/password/change   # Change password (authenticated)

# Session Management
GET    /api/v1/auth/sessions          # List active sessions
DELETE /api/v1/auth/sessions/{id}     # Revoke specific session
DELETE /api/v1/auth/sessions/all      # Revoke all sessions
```

### User Management
```
# User CRUD (Admin only)
GET    /api/v1/users                  # List users (paginated, filterable)
POST   /api/v1/users                  # Create user (admin)
GET    /api/v1/users/{id}             # Get user details
PUT    /api/v1/users/{id}             # Update user
DELETE /api/v1/users/{id}             # Delete user (soft delete)
PATCH  /api/v1/users/{id}/activate    # Activate user
PATCH  /api/v1/users/{id}/deactivate  # Deactivate user

# User Profile (Self-service)
GET    /api/v1/users/me               # Get own profile
PUT    /api/v1/users/me               # Update own profile
DELETE /api/v1/users/me               # Delete own account

# User Roles & Permissions
GET    /api/v1/users/{id}/roles       # Get user roles
POST   /api/v1/users/{id}/roles       # Assign role
DELETE /api/v1/users/{id}/roles/{role_id} # Remove role
GET    /api/v1/users/{id}/permissions # Get effective permissions
```

### Teams & Organizations
```
# Organizations (Multi-tenancy)
GET    /api/v1/organizations          # List organizations
POST   /api/v1/organizations          # Create organization
GET    /api/v1/organizations/{id}     # Get organization
PUT    /api/v1/organizations/{id}     # Update organization
DELETE /api/v1/organizations/{id}     # Delete organization

# Organization Membership
GET    /api/v1/organizations/{id}/members      # List members
POST   /api/v1/organizations/{id}/members      # Add member
DELETE /api/v1/organizations/{id}/members/{uid} # Remove member
PUT    /api/v1/organizations/{id}/members/{uid}/role # Update member role

# Teams (within organization)
GET    /api/v1/organizations/{oid}/teams       # List teams
POST   /api/v1/organizations/{oid}/teams       # Create team
GET    /api/v1/teams/{id}                      # Get team
PUT    /api/v1/teams/{id}                      # Update team
DELETE /api/v1/teams/{id}                      # Delete team

# Team Membership
GET    /api/v1/teams/{id}/members              # List team members
POST   /api/v1/teams/{id}/members              # Add member
DELETE /api/v1/teams/{id}/members/{uid}        # Remove member
```

### API Keys & Service Accounts
```
# API Keys
GET    /api/v1/api-keys               # List user's API keys
POST   /api/v1/api-keys               # Create API key
GET    /api/v1/api-keys/{id}          # Get API key details
PUT    /api/v1/api-keys/{id}          # Update API key (rename, rotate)
DELETE /api/v1/api-keys/{id}          # Revoke API key
POST   /api/v1/api-keys/{id}/rotate   # Rotate API key

# Service Accounts (machine users)
GET    /api/v1/service-accounts       # List service accounts
POST   /api/v1/service-accounts       # Create service account
GET    /api/v1/service-accounts/{id}  # Get service account
PUT    /api/v1/service-accounts/{id}  # Update service account
DELETE /api/v1/service-accounts/{id}  # Delete service account
POST   /api/v1/service-accounts/{id}/keys # Generate key for service account
```

### Roles & Permissions Management
```
# Roles
GET    /api/v1/roles                  # List all roles
POST   /api/v1/roles                  # Create custom role
GET    /api/v1/roles/{id}             # Get role details
PUT    /api/v1/roles/{id}             # Update role
DELETE /api/v1/roles/{id}             # Delete custom role

# Permissions
GET    /api/v1/permissions            # List all available permissions
GET    /api/v1/roles/{id}/permissions # Get role permissions
POST   /api/v1/roles/{id}/permissions # Add permission to role
DELETE /api/v1/roles/{id}/permissions/{pid} # Remove permission

# Permission Policies (ABAC)
GET    /api/v1/policies               # List permission policies
POST   /api/v1/policies               # Create policy
GET    /api/v1/policies/{id}          # Get policy
PUT    /api/v1/policies/{id}          # Update policy
DELETE /api/v1/policies/{id}          # Delete policy
POST   /api/v1/policies/{id}/test     # Test policy against context
```

### Workflows
```
GET    /api/v1/workflows              # List workflows (paginated, filterable)
POST   /api/v1/workflows              # Create workflow
GET    /api/v1/workflows/{id}         # Get workflow by ID
PUT    /api/v1/workflows/{id}         # Update workflow
DELETE /api/v1/workflows/{id}         # Delete workflow
POST   /api/v1/workflows/{id}/validate # Validate workflow
POST   /api/v1/workflows/{id}/dry-run # Dry-run validation with inputs
POST   /api/v1/workflows/generate     # Generate from natural language
GET    /api/v1/workflows/template     # Get template

# Version management
GET    /api/v1/workflows/{id}/versions
GET    /api/v1/workflows/{id}/versions/{version}
POST   /api/v1/workflows/{id}/versions # Create new version

# Workflow Sharing & Permissions
GET    /api/v1/workflows/{id}/permissions # Get workflow ACL
POST   /api/v1/workflows/{id}/permissions # Share with user/team
DELETE /api/v1/workflows/{id}/permissions/{subject_id} # Revoke access
POST   /api/v1/workflows/{id}/clone   # Clone workflow
POST   /api/v1/workflows/{id}/export  # Export workflow (YAML)
POST   /api/v1/workflows/import       # Import workflow from YAML
```

### Executions
```
GET    /api/v1/executions             # List executions (paginated, filterable)
POST   /api/v1/executions             # Start execution
GET    /api/v1/executions/{id}        # Get execution details
DELETE /api/v1/executions/{id}        # Cancel execution
POST   /api/v1/executions/{id}/pause  # Pause execution
POST   /api/v1/executions/{id}/resume # Resume execution
POST   /api/v1/executions/{id}/retry  # Retry failed execution
GET    /api/v1/executions/{id}/logs   # Get execution logs (streaming)
GET    /api/v1/executions/{id}/tasks  # Get task execution details
```

### Schedules
```
GET    /api/v1/schedules              # List schedules
POST   /api/v1/schedules              # Create schedule
GET    /api/v1/schedules/{id}         # Get schedule
PUT    /api/v1/schedules/{id}         # Update schedule
DELETE /api/v1/schedules/{id}         # Delete schedule
POST   /api/v1/schedules/{id}/trigger # Manually trigger scheduled workflow
```

### Monitoring
```
GET    /api/v1/metrics/overview       # Overall system metrics
GET    /api/v1/metrics/workflows      # Per-workflow metrics
GET    /api/v1/metrics/executions     # Execution statistics
GET    /api/v1/health                 # Health check
GET    /api/v1/version                # Version info
```

### WebSocket Endpoints
```
WS     /api/v1/ws/executions/{id}     # Real-time execution updates
WS     /api/v1/ws/logs/{execution_id} # Real-time log streaming
WS     /api/v1/ws/events              # Global event stream
```

## Web Interface Design

### Technology Stack
- **Framework**: SvelteKit or React + Next.js
- **UI Library**: Tailwind CSS + shadcn/ui components
- **State**: Zustand or TanStack Query
- **WebSocket**: Socket.io-client or native WebSocket
- **Charts**: Recharts or Chart.js
- **Code Editor**: Monaco Editor (for YAML editing)

### Key Pages

#### 1. Dashboard
- Active execution count
- Success/failure rate charts
- Recent executions timeline
- Resource utilization graphs
- Quick actions (create workflow, start execution)

#### 2. Workflows
- **List View**:
  - Grid/table toggle
  - Search and filters (tags, status, date)
  - Bulk actions
  - Version badges

- **Detail View**:
  - YAML editor with syntax highlighting
  - Validation on save
  - Execution history
  - Version comparison
  - Visual DAG representation

- **Create/Edit**:
  - Template selection
  - Natural language input option
  - Form-based builder
  - Real-time validation
  - Preview mode

#### 3. Executions
- **List View**:
  - Status indicators (color-coded)
  - Duration timers
  - Progress bars
  - Quick actions (cancel, retry)

- **Detail View**:
  - Real-time status updates via WebSocket
  - Task execution timeline (Gantt chart)
  - Live log streaming
  - Task dependency graph
  - Input/output inspection
  - Checkpoint navigation
  - Performance metrics

#### 4. Schedules
- Calendar view
- Cron expression builder
- Upcoming runs preview
- Schedule history

#### 5. Monitoring
- System health dashboard
- Execution analytics
- Error tracking
- Performance trends
- Resource usage

#### 6. Settings
- User management
- API key management
- System configuration
- Authentication settings

### UI Components

```typescript
// Key reusable components
- WorkflowCard
- ExecutionTimeline
- TaskDAG (D3.js/Cytoscape.js)
- LogViewer (virtualized)
- YAMLEditor
- CronBuilder
- MetricsChart
- StatusBadge
- ProgressBar
- ActionMenu
```

## Scalability Architecture

### Horizontal Scaling

#### 1. Stateless API Servers
```
┌─────────────┐
│Load Balancer│
└──────┬──────┘
       │
   ┌───┴───┬───────┬───────┐
   │       │       │       │
┌──▼──┐ ┌──▼──┐ ┌──▼──┐ ┌──▼──┐
│API 1│ │API 2│ │API 3│ │API N│
└─────┘ └─────┘ └─────┘ └─────┘
```

- No session state in API servers
- JWT-based authentication
- Shared Redis for cache/sessions
- Sticky sessions for WebSocket (optional)

#### 2. Execution Workers
```
┌──────────────┐
│ Redis Queue  │
└──────┬───────┘
       │
   ┌───┴───┬───────┬───────┐
   │       │       │       │
┌──▼──┐ ┌──▼──┐ ┌──▼──┐ ┌──▼──┐
│Wrk 1│ │Wrk 2│ │Wrk 3│ │Wrk N│
└─────┘ └─────┘ └─────┘ └─────┘
```

- Worker pool consuming execution queue
- Distributed locking per execution
- Auto-scaling based on queue depth
- Graceful shutdown (drain queue)

#### 3. Database Scaling
- **PostgreSQL**:
  - Primary for writes
  - Read replicas for queries
  - Connection pooling (PgBouncer)
  - Partitioning (executions, logs by date)

- **Redis**:
  - Cluster mode for HA
  - Separate instances for cache/queue
  - Persistence for queue data

### Performance Optimizations

#### 1. Caching Strategy
```rust
// Multi-level caching
L1: In-memory LRU (per API server)
L2: Redis (shared across servers)
L3: PostgreSQL (source of truth)

// Cache workflow definitions
Cache Key: workflow:{id}:{version}
TTL: 1 hour

// Cache execution status
Cache Key: execution:{id}:status
TTL: 5 minutes
```

#### 2. Database Optimizations
- Indexes on foreign keys and query filters
- Materialized views for analytics
- Partitioning for time-series data
- Archival strategy (move old executions to cold storage)

#### 3. WebSocket Optimization
- Connection pooling
- Message batching (100ms window)
- Client-side reconnection logic
- Server-sent events (SSE) as fallback

#### 4. Execution Optimizations
- Parallel task execution (existing DAG feature)
- Lazy loading of large outputs
- Streaming results instead of buffering
- Early termination on critical failures

### Message Queue Architecture

```
┌──────────────────────────────────────┐
│         Event-Driven Flow             │
└──────────────────────────────────────┘

API Request
    │
    ▼
┌─────────────┐
│ Validation  │
└──────┬──────┘
       │
       ▼
┌─────────────┐       ┌──────────────┐
│Create Record│──────▶│  PostgreSQL  │
└──────┬──────┘       └──────────────┘
       │
       ▼
┌─────────────┐       ┌──────────────┐
│ Push Queue  │──────▶│Redis Queue   │
└──────┬──────┘       └──────┬───────┘
       │                     │
       ▼                     │
┌─────────────┐              │
│Return Job ID│              │
└─────────────┘              │
                             ▼
                      ┌──────────────┐
                      │Worker Pool   │
                      └──────┬───────┘
                             │
                             ▼
                      ┌──────────────┐
                      │Execute + Log │
                      └──────┬───────┘
                             │
                             ▼
                      ┌──────────────┐
                      │Update Status │
                      └──────┬───────┘
                             │
                             ▼
                      ┌──────────────┐
                      │Publish Events│
                      └──────┬───────┘
                             │
                             ▼
                      ┌──────────────┐
                      │ WebSocket    │
                      │  Broadcast   │
                      └──────────────┘
```

## Security Considerations

### Authentication & Authorization

#### 1. Multi-tier Authentication System

```rust
// JWT-based authentication
pub struct Claims {
    sub: Uuid,                       // user_id
    username: String,
    email: String,
    organization_id: Option<Uuid>,
    roles: Vec<String>,
    permissions: Vec<String>,
    scope: Vec<String>,              // OAuth scopes
    exp: usize,                      // expiration
    iat: usize,                      // issued at
    nbf: usize,                      // not before
    jti: String,                     // JWT ID for revocation
}

// Multiple authentication methods
pub enum AuthMethod {
    Password { username: String, password: String },
    ApiKey { key: String },
    OAuth { provider: String, code: String },
    SAML { assertion: String },
    ServiceAccount { client_id: Uuid, client_secret: String },
}

// Multi-Factor Authentication
pub struct MfaChallenge {
    user_id: Uuid,
    method: MfaMethod,
    challenge_code: String,
    expires_at: DateTime<Utc>,
}

pub enum MfaMethod {
    TOTP,           // Time-based One-Time Password (Google Authenticator)
    SMS,            // SMS verification code
    Email,          // Email verification code
    BackupCode,     // Recovery backup codes
}
```

#### 2. Role-Based Access Control (RBAC)

```rust
// Hierarchical role system
pub struct Role {
    id: Uuid,
    name: String,
    description: String,
    is_system: bool,                 // System roles cannot be modified
    organization_id: Option<Uuid>,   // Organization-specific roles
    parent_role_id: Option<Uuid>,    // Role inheritance
    permissions: Vec<Permission>,
}

// System roles (built-in)
pub enum SystemRole {
    SuperAdmin,     // Platform-wide administration
    OrgOwner,       // Organization owner
    OrgAdmin,       // Organization administrator
    TeamLead,       // Team leader
    Developer,      // Can create/execute workflows
    Viewer,         // Read-only access
    Guest,          // Limited access
}

// Fine-grained permissions
pub struct Permission {
    id: Uuid,
    name: String,               // e.g., "workflow:create"
    resource_type: ResourceType,
    action: Action,
    scope: PermissionScope,
}

pub enum ResourceType {
    Workflow,
    Execution,
    Schedule,
    User,
    Team,
    Organization,
    ApiKey,
    Role,
}

pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    Execute,
    Share,
    Manage,
}

pub enum PermissionScope {
    Global,                     // All resources
    Organization(Uuid),         // Organization-scoped
    Team(Uuid),                 // Team-scoped
    Own,                        // User's own resources
}
```

#### 3. Attribute-Based Access Control (ABAC)

```rust
// Policy-based access control
pub struct PermissionPolicy {
    id: Uuid,
    name: String,
    effect: PolicyEffect,
    conditions: PolicyConditions,
    priority: i32,              // Higher priority = evaluated first
}

pub enum PolicyEffect {
    Allow,
    Deny,                       // Explicit deny overrides allow
}

// Policy conditions using JSON-based rules
pub struct PolicyConditions {
    // Subject (who is making the request)
    subject: Option<SubjectCondition>,

    // Resource (what is being accessed)
    resource: Option<ResourceCondition>,

    // Environment (context of the request)
    environment: Option<EnvironmentCondition>,
}

// Example policy JSON
/*
{
  "name": "Allow workflow execution during business hours",
  "effect": "allow",
  "conditions": {
    "subject": {
      "role": ["developer", "team_lead"],
      "organization_id": "{{ resource.organization_id }}"
    },
    "resource": {
      "type": "workflow",
      "tags": { "contains": "production" }
    },
    "environment": {
      "time": {
        "after": "09:00",
        "before": "17:00"
      },
      "days": ["monday", "tuesday", "wednesday", "thursday", "friday"],
      "ip_range": "10.0.0.0/8"
    }
  }
}
*/
```

#### 4. API Key Management

```rust
pub struct ApiKey {
    id: Uuid,
    user_id: Uuid,
    key_hash: String,           // bcrypt hash of the key
    key_prefix: String,         // e.g., "sk_live_abc..." (first 12 chars)
    name: String,
    description: Option<String>,
    scopes: Vec<String>,        // Allowed operations
    ip_whitelist: Vec<IpAddr>,  // IP restrictions
    rate_limit: Option<RateLimit>,
    expires_at: Option<DateTime<Utc>>,
    last_used_at: Option<DateTime<Utc>>,
    is_active: bool,
}

// API key scopes
pub enum ApiKeyScope {
    WorkflowRead,
    WorkflowWrite,
    WorkflowExecute,
    ExecutionRead,
    ExecutionManage,
    All,                        // Full access (dangerous)
}

// Rate limiting per API key
pub struct RateLimit {
    requests_per_minute: u32,
    requests_per_hour: u32,
    requests_per_day: u32,
}
```

#### 5. OAuth 2.0 / OIDC Integration

```rust
// Supported OAuth providers
pub enum OAuthProvider {
    Google,
    GitHub,
    GitLab,
    Microsoft,
    Okta,
    Auth0,
    Custom { name: String, config: OAuthConfig },
}

pub struct OAuthConfig {
    client_id: String,
    client_secret: String,
    authorize_url: String,
    token_url: String,
    userinfo_url: Option<String>,
    scopes: Vec<String>,
    redirect_uri: String,
}

// OAuth connection storage
pub struct OAuthConnection {
    user_id: Uuid,
    provider: OAuthProvider,
    provider_user_id: String,
    access_token: EncryptedString,
    refresh_token: Option<EncryptedString>,
    scopes: Vec<String>,
    profile_data: serde_json::Value,
}
```

#### 6. SAML 2.0 SSO

```rust
pub struct SamlConfig {
    entity_id: String,
    sso_url: String,
    slo_url: Option<String>,      // Single Logout URL
    certificate: String,           // X.509 certificate
    name_id_format: NameIdFormat,
    attribute_mapping: AttributeMapping,
}

pub struct AttributeMapping {
    email: String,                 // SAML attribute for email
    username: String,
    first_name: Option<String>,
    last_name: Option<String>,
    roles: Option<String>,
}
```

#### 7. Session Management

```rust
pub struct UserSession {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    ip_address: IpAddr,
    user_agent: String,
    device_info: Option<DeviceInfo>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_activity_at: DateTime<Utc>,
    is_active: bool,
}

// Session security features
impl SessionManager {
    // Detect suspicious activity
    async fn detect_anomaly(&self, session: &UserSession) -> bool {
        // Check for:
        // - Impossible travel (location changes too fast)
        // - Device fingerprint mismatch
        // - Unusual activity patterns
    }

    // Automatic session expiration
    async fn enforce_idle_timeout(&self, max_idle: Duration) {
        // Expire sessions inactive for > max_idle
    }

    // Concurrent session limits
    async fn enforce_session_limit(&self, user_id: Uuid, max_sessions: usize) {
        // Revoke oldest sessions if limit exceeded
    }
}
```

#### 8. Password Security

```rust
pub struct PasswordPolicy {
    min_length: usize,              // Default: 12
    require_uppercase: bool,
    require_lowercase: bool,
    require_numbers: bool,
    require_special_chars: bool,
    prevent_common_passwords: bool,
    prevent_username_in_password: bool,
    max_age_days: Option<u32>,      // Force password rotation
    history_count: usize,           // Prevent password reuse
}

// Password hashing
impl PasswordHasher {
    fn hash(&self, password: &str) -> Result<String> {
        // Use Argon2id (OWASP recommended)
        argon2::hash_encoded(
            password.as_bytes(),
            &salt,
            &argon2::Config {
                variant: argon2::Variant::Argon2id,
                version: argon2::Version::Version13,
                mem_cost: 65536,      // 64 MB
                time_cost: 3,
                lanes: 4,
                thread_mode: argon2::ThreadMode::Parallel,
                ..Default::default()
            }
        )
    }
}
```

#### 9. Network Security

```rust
// TLS configuration
pub struct TlsConfig {
    cert_path: PathBuf,
    key_path: PathBuf,
    min_version: TlsVersion,        // TLS 1.3 minimum
    cipher_suites: Vec<CipherSuite>,
    enable_hsts: bool,              // HTTP Strict Transport Security
    hsts_max_age: Duration,
}

// CORS configuration
pub struct CorsConfig {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<Method>,
    allowed_headers: Vec<String>,
    expose_headers: Vec<String>,
    max_age: Duration,
    allow_credentials: bool,
}

// Rate limiting
pub struct RateLimitConfig {
    global: RateLimitTier,
    per_user: RateLimitTier,
    per_api_key: RateLimitTier,
    per_ip: RateLimitTier,
}

pub struct RateLimitTier {
    requests_per_second: u32,
    requests_per_minute: u32,
    requests_per_hour: u32,
    burst_size: u32,
}

// IP whitelisting/blacklisting
pub struct IpAccessControl {
    whitelist: Vec<IpNetwork>,
    blacklist: Vec<IpNetwork>,
    mode: IpFilterMode,
}

pub enum IpFilterMode {
    Whitelist,      // Only allow whitelisted IPs
    Blacklist,      // Block blacklisted IPs
    Off,            // No IP filtering
}
```

### Workflow Security

#### 1. Sandbox Isolation
```rust
// Execute workflows in isolated environments
pub struct WorkflowSandbox {
    resource_limits: ResourceLimits,
    allowed_tools: Vec<Tool>,
    network_policy: NetworkPolicy,
    file_system_policy: FileSystemPolicy,
}

pub struct ResourceLimits {
    max_memory: usize,
    max_cpu_seconds: usize,
    max_execution_time: Duration,
    max_file_operations: usize,
}
```

#### 2. Input Validation
- Strict schema validation
- Input sanitization
- SQL injection prevention
- Command injection prevention

#### 3. Output Sanitization
- Redact sensitive data
- Truncate large outputs
- Filter dangerous content

#### 4. Audit Logging
```sql
CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    user_id UUID,
    action VARCHAR(255),
    resource_type VARCHAR(100),
    resource_id UUID,
    ip_address INET,
    user_agent TEXT,
    changes JSONB,
    success BOOLEAN
);
```

### Data Protection

#### 1. Encryption
- At-rest: Database encryption (PostgreSQL TDE)
- In-transit: TLS 1.3
- Secrets: HashiCorp Vault or AWS Secrets Manager

#### 2. Data Retention
- Configurable retention policies
- Automatic archival
- Secure deletion (GDPR compliance)

#### 3. Backup & Recovery
- Automated PostgreSQL backups
- Point-in-time recovery
- Disaster recovery plan

## Implementation Roadmap

### Phase 1: Core Server Infrastructure (4-6 weeks)
**Goal**: Basic API server with workflow CRUD

**Tasks**:
1. Set up project structure
   - Create `server` binary target
   - Add dependencies: `axum`, `tokio`, `sqlx`, `redis`

2. Implement database layer
   - Create migrations
   - Implement repository traits
   - PostgreSQL adapter implementation

3. Build REST API
   - Workflow CRUD endpoints
   - Basic authentication (JWT)
   - Request validation middleware

4. Basic execution queue
   - Redis queue integration
   - Simple worker implementation
   - Execution status tracking

**Deliverables**:
- Working API server
- Database schema
- Basic workflow management
- Simple execution capability

### Phase 2: WebSocket & Real-time Updates (3-4 weeks)
**Goal**: Real-time execution monitoring

**Tasks**:
1. WebSocket server
   - Connection management
   - Event broadcasting
   - Reconnection handling

2. Event system
   - Event publisher implementation
   - WebSocket adapter
   - Event filtering

3. Log streaming
   - Structured logging
   - Log storage
   - Real-time streaming

**Deliverables**:
- WebSocket endpoints
- Real-time execution updates
- Live log streaming

### Phase 3: Web Interface (6-8 weeks)
**Goal**: Full-featured web UI

**Tasks**:
1. Project setup
   - SvelteKit/React setup
   - Component library integration
   - API client generation

2. Core pages
   - Dashboard
   - Workflow management
   - Execution monitoring

3. Advanced features
   - YAML editor
   - DAG visualization
   - Real-time updates
   - Log viewer

**Deliverables**:
- Complete web interface
- All major features
- Responsive design

### Phase 4: Scalability & Performance (4-5 weeks)
**Goal**: Production-ready scaling

**Tasks**:
1. Horizontal scaling
   - Stateless API design
   - Worker pool management
   - Distributed locking

2. Caching layer
   - Redis integration
   - Cache invalidation
   - Performance optimization

3. Database optimization
   - Query optimization
   - Indexing strategy
   - Partitioning

4. Load testing
   - Performance benchmarks
   - Stress testing
   - Capacity planning

**Deliverables**:
- Horizontally scalable architecture
- Performance benchmarks
- Optimization documentation

### Phase 5: Advanced Features (6-8 weeks)
**Goal**: Enterprise features

**Tasks**:
1. Scheduling system
   - Cron scheduler
   - Schedule management
   - Timezone handling

2. Advanced monitoring
   - Metrics collection
   - Analytics dashboard
   - Alerting system

3. Security hardening
   - Enhanced RBAC
   - Audit logging
   - Security scanning

4. Integration features
   - Webhook support
   - External API integrations
   - Plugin system

**Deliverables**:
- Scheduling system
- Comprehensive monitoring
- Production-ready security

### Phase 6: Testing & Documentation (3-4 weeks)
**Goal**: Production readiness

**Tasks**:
1. Comprehensive testing
   - Unit tests
   - Integration tests
   - End-to-end tests
   - Load tests

2. Documentation
   - API documentation (OpenAPI)
   - User guide
   - Administrator guide
   - Deployment guide

3. Deployment automation
   - Docker containers
   - Kubernetes manifests
   - CI/CD pipelines

**Deliverables**:
- Full test coverage
- Complete documentation
- Deployment automation

**Total Timeline**: 26-35 weeks (~6-8 months)

## Technology Stack Summary

### Backend
- **Language**: Rust (stable)
- **Web Framework**: `axum` v0.7+
- **Async Runtime**: `tokio` v1.0+
- **Database ORM**: `sqlx` v0.7+
- **Serialization**: `serde` + `serde_json` + `serde_yaml`
- **Authentication**: `jsonwebtoken`
- **Validation**: `validator`
- **Redis Client**: `redis-rs`
- **WebSocket**: `axum::extract::ws`
- **OpenAPI**: `utoipa`

### Database
- **Primary Database**: PostgreSQL 15+
- **Cache/Queue**: Redis 7+
- **Blob Storage**: S3-compatible (optional)

### Frontend
- **Framework**: SvelteKit 2.0+ or Next.js 14+
- **Language**: TypeScript
- **Styling**: Tailwind CSS v3+
- **UI Components**: shadcn/ui
- **State Management**: TanStack Query
- **Charts**: Recharts
- **Code Editor**: Monaco Editor
- **DAG Visualization**: Cytoscape.js or D3.js

### Infrastructure
- **Containerization**: Docker
- **Orchestration**: Kubernetes (optional)
- **Load Balancer**: NGINX or Traefik
- **Monitoring**: Prometheus + Grafana
- **Logging**: Loki or ELK Stack
- **CI/CD**: GitHub Actions

## Deployment Scenarios

The system is designed to support deployment scenarios ranging from a single binary with zero infrastructure dependencies to large-scale distributed deployments.

### Minimal Deployment (Zero Dependencies)

**Requirements**: Single `periplon-executor` binary only

**Configuration**:
```toml
# minimal-config.toml
[server]
host = "0.0.0.0"
port = 8080
workers = true  # Run workers in same process
worker_concurrency = 2

[storage]
backend = "filesystem"

[storage.filesystem]
base_path = "./data"

[queue]
backend = "filesystem"

[queue.filesystem]
queue_dir = "./queue"
poll_interval_ms = 1000
lock_timeout_secs = 300

[auth]
# Simple JWT auth, no external auth providers
jwt_secret = "your-secret-key-change-this"
jwt_expiration_secs = 3600
```

**Start Server**:
```bash
# Download/build single binary
curl -L https://github.com/.../periplon-executor -o periplon-executor
chmod +x periplon-executor

# Run with minimal config
./periplon-executor server --config minimal-config.toml

# OR use environment variables
export STORAGE_BACKEND=filesystem
export QUEUE_BACKEND=filesystem
export JWT_SECRET=your-secret-key
./periplon-executor server --workers --port 8080
```

**Features**:
- ✅ Full workflow management
- ✅ Execution with checkpoints
- ✅ Resume capability
- ✅ Basic authentication (JWT)
- ✅ Worker queue
- ✅ WebSocket streaming
- ✅ REST API
- ❌ Multi-user management (single admin only)
- ❌ OAuth/SSO
- ❌ Advanced RBAC
- ❌ High availability
- ❌ Distributed workers

**Use Cases**:
- Local development
- Personal workflows
- Small team (< 5 users)
- Edge deployments
- Air-gapped environments
- Embedded systems

### Small Deployment (SQLite)

**Requirements**: Single binary + SQLite file

**Configuration**:
```toml
[storage]
backend = "sqlite"

[storage.sqlite]
database_path = "./workflow.db"
max_connections = 5

[queue]
backend = "sqlite"  # Reuse same database

[queue.sqlite]
database_path = "./workflow.db"
poll_interval_ms = 500
```

**Features**:
- ✅ All minimal deployment features
- ✅ Better query performance
- ✅ Multi-user management
- ✅ ACID transactions
- ✅ Relational queries
- ✅ Full RBAC support
- ❌ High availability
- ❌ Horizontal scaling

**Implementation**:
```rust
// SQLite backend using same abstractions
pub struct SqliteWorkflowStorage {
    pool: sqlx::SqlitePool,
}

impl SqliteWorkflowStorage {
    pub async fn new(database_path: &Path) -> Result<Self> {
        // Create database file if doesn't exist
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite://{}", database_path.display()))
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations/sqlite").run(&pool).await?;

        Ok(Self { pool })
    }
}

// Reuse PostgreSQL implementation with minor adjustments
#[async_trait]
impl WorkflowStorage for SqliteWorkflowStorage {
    async fn store_workflow(&self, workflow: &Workflow) -> Result<Uuid> {
        // Same as PostgreSQL implementation
        // SQLite supports most PostgreSQL syntax
    }
}
```

### Medium Deployment (PostgreSQL + Redis)

**Requirements**: Binary + PostgreSQL + Redis (optional)

**Configuration**:
```toml
[storage]
backend = "postgres"

[storage.postgres]
url = "postgres://user:pass@localhost:5432/workflows"
max_connections = 20

[queue]
backend = "redis"  # Or postgres for simplicity

[queue.redis]
url = "redis://localhost:6379/0"
max_connections = 10
```

**Deployment**:
```bash
# Using Docker Compose
docker-compose up -d postgres redis

# Or managed services
export STORAGE_POSTGRES_URL=postgres://...
export QUEUE_REDIS_URL=redis://...

./periplon-executor server --config config.toml
```

**Features**:
- ✅ All small deployment features
- ✅ Better performance
- ✅ Connection pooling
- ✅ Replication (PostgreSQL)
- ✅ High availability (with setup)
- ✅ Multiple server instances
- ❌ Geographic distribution

### Large Deployment (Cloud-Native)

**Requirements**: Binary + PostgreSQL + Redis + S3

**Configuration**:
```toml
[storage]
backend = "postgres"  # Metadata

[storage.postgres]
url = "postgres://rds-endpoint/workflows"

# Large artifacts to S3
[storage.s3]
endpoint = "https://s3.amazonaws.com"
bucket = "workflow-artifacts"

[queue]
backend = "redis"

[queue.redis]
url = "redis://elasticache-endpoint:6379"
```

**Features**:
- ✅ All features
- ✅ Horizontal scaling
- ✅ Geographic distribution
- ✅ Multi-region replication
- ✅ 99.9%+ uptime
- ✅ Petabyte-scale storage

### Comparison Matrix

| Feature | Minimal | SQLite | PostgreSQL | Cloud-Native |
|---------|---------|--------|------------|--------------|
| **Infrastructure** | None | None | PostgreSQL, Redis | PostgreSQL, Redis, S3 |
| **Setup Time** | < 1 min | < 5 min | < 30 min | Hours |
| **Cost (monthly)** | $0 | $0 | ~$100 | ~$500+ |
| **Max Users** | 1-5 | 10-50 | 100-1000 | 10,000+ |
| **Max Workflows/day** | 100 | 1,000 | 10,000 | 1,000,000+ |
| **HA Support** | ❌ | ❌ | ✅ | ✅ |
| **Horizontal Scaling** | ❌ | ❌ | ✅ | ✅ |
| **Geographic Distribution** | ❌ | ❌ | ❌ | ✅ |
| **Backup Complexity** | Low | Low | Medium | Medium |
| **Maintenance** | None | Minimal | Regular | Regular |

### Migration Path

```
Minimal (Filesystem)
    ↓ (export/import)
SQLite
    ↓ (pg_dump equivalent)
PostgreSQL
    ↓ (add Redis + S3)
Cloud-Native
```

Each step is a simple config change:

```bash
# Export from filesystem
./periplon-executor export --format json > workflows.json

# Import to PostgreSQL
export STORAGE_BACKEND=postgres
./periplon-executor import workflows.json
```

## Deployment Options

### 1. Single Binary (Minimal)

```bash
# Download
curl -L https://github.com/.../periplon-executor -o periplon-executor
chmod +x periplon-executor

# Run
./periplon-executor server --port 8080

# Systemd service
sudo tee /etc/systemd/system/periplon-executor.service > /dev/null <<EOF
[Unit]
Description=DSL Executor Server
After=network.target

[Service]
Type=simple
User=dsl
WorkingDirectory=/var/lib/periplon-executor
ExecStart=/usr/local/bin/periplon-executor server --config /etc/periplon-executor/config.toml
Restart=always

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable periplon-executor
sudo systemctl start periplon-executor
```

### 2. Docker Compose (Development/Small Scale)
```yaml
version: '3.8'

services:
  api:
    build: ./server
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://user:pass@db:5432/workflows
      REDIS_URL: redis://redis:6379
    depends_on:
      - db
      - redis

  worker:
    build: ./server
    command: worker
    environment:
      DATABASE_URL: postgres://user:pass@db:5432/workflows
      REDIS_URL: redis://redis:6379
    depends_on:
      - db
      - redis
    deploy:
      replicas: 3

  web:
    build: ./web
    ports:
      - "3000:3000"
    environment:
      API_URL: http://api:8080

  db:
    image: postgres:15
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: workflows
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass

  redis:
    image: redis:7
    volumes:
      - redis_data:/data
```

### 2. Kubernetes (Production/Large Scale)
```yaml
# deployment.yaml (simplified)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: dsl-server-api
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: api
        image: dsl-server:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
---
apiVersion: v1
kind: Service
metadata:
  name: dsl-server-api
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8080
  selector:
    app: dsl-server-api
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: dsl-server-api-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: dsl-server-api
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

## Cost Analysis

### Infrastructure Costs (Monthly, AWS)

**Small Scale** (< 100 workflows/day):
- EC2: 2x t3.medium ($60)
- RDS PostgreSQL: db.t3.medium ($50)
- ElastiCache Redis: cache.t3.micro ($15)
- S3 + Data Transfer ($10)
- **Total**: ~$135/month

**Medium Scale** (< 1000 workflows/day):
- EC2: 3x t3.large ($180)
- RDS PostgreSQL: db.t3.large ($120)
- ElastiCache Redis: cache.t3.small ($35)
- Load Balancer ($20)
- S3 + Data Transfer ($30)
- **Total**: ~$385/month

**Large Scale** (< 10,000 workflows/day):
- EKS Cluster ($75)
- EC2: 6x t3.xlarge ($720)
- RDS PostgreSQL: db.r5.xlarge ($400)
- ElastiCache Redis: cache.r5.large ($150)
- Load Balancer ($50)
- S3 + Data Transfer ($100)
- **Total**: ~$1,495/month

### Development Costs

**Timeline**: 6-8 months
**Team**: 2-3 engineers
**Estimated**: 3,000-4,000 engineering hours

## Success Metrics

### Performance
- API response time < 100ms (p95)
- WebSocket latency < 50ms
- Workflow validation < 1s
- Execution startup < 2s
- Support 1000+ concurrent executions

### Reliability
- 99.9% uptime
- Zero data loss
- Graceful degradation
- < 1min recovery time

### Scalability
- Handle 10,000+ workflows
- Support 1,000+ concurrent users
- Process 100,000+ executions/day
- Store 1TB+ execution data

## Risk Mitigation

### Technical Risks
1. **Database Performance**: Early load testing, sharding strategy
2. **WebSocket Scalability**: Redis pub/sub, connection limits
3. **Memory Leaks**: Profiling, resource limits
4. **Data Loss**: WAL archiving, backup verification

### Operational Risks
1. **Downtime**: Blue-green deployments, rollback strategy
2. **Data Migration**: Versioned migrations, dry-run testing
3. **Security Breach**: Regular audits, penetration testing
4. **Resource Exhaustion**: Auto-scaling, rate limiting

## Infinite Scaling with S3-Only Backend

### Overview

This section describes a pure S3-based architecture designed for infinite horizontal scaling without operational complexity. By using S3 as the single storage backend with auto-balanced sharding, the system can scale to millions of workflows and executions while maintaining:

- **Zero operational overhead**: No database clusters to manage
- **Infinite capacity**: S3 scales automatically to petabytes
- **Cost efficiency**: Pay only for actual storage used
- **High durability**: 11 nines (99.999999999%) durability
- **Multi-region**: Built-in replication and disaster recovery

### Architecture Principles

**S3-Only Stack**:
```
┌─────────────────────────────────────────────────┐
│                 API Layer (Stateless)            │
│          ┌──────────────┬──────────────┐        │
│          │  REST API    │  WebSocket   │        │
│          └──────┬───────┴──────┬───────┘        │
└─────────────────┼──────────────┼────────────────┘
                  │              │
        ┌─────────┴──────────────┴─────────┐
        │                                   │
┌───────▼─────────┐              ┌─────────▼────────┐
│  Shard Router   │              │  Event Stream    │
│  (Consistent    │              │  (S3 + SQS)      │
│   Hashing)      │              │                  │
└───────┬─────────┘              └──────────────────┘
        │
        │  ┌───────────────────────────────┐
        └─▶│  S3 Sharded Storage           │
           │  ┌──────┬──────┬──────┬────┐  │
           │  │Shard │Shard │Shard │... │  │
           │  │  0   │  1   │  2   │ N  │  │
           │  └──────┴──────┴──────┴────┘  │
           └───────────────────────────────┘
```

**Key Features**:
- No PostgreSQL, no Redis, no managed databases
- Pure S3 for all storage (workflows, executions, queue, state)
- Consistent hashing for automatic shard distribution
- SQS for queue coordination (serverless)
- Lambda or stateless workers for execution
- DynamoDB for shard metadata only (optional, can use S3)

### Consistent Hashing Shard Distribution

#### 1. Hash Ring Architecture

```rust
use std::collections::BTreeMap;
use xxhash_rust::xxh3::xxh3_64;

/// Consistent hash ring for shard distribution
pub struct ConsistentHashRing {
    /// Virtual nodes on the ring (shard_id -> hash positions)
    ring: BTreeMap<u64, ShardId>,

    /// Number of virtual nodes per shard (higher = better distribution)
    virtual_nodes_per_shard: usize,

    /// Total number of shards
    shard_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShardId(pub u32);

impl ConsistentHashRing {
    /// Create new hash ring with specified shard count
    pub fn new(shard_count: usize) -> Self {
        let virtual_nodes_per_shard = 150; // 150 virtual nodes per shard
        let mut ring = BTreeMap::new();

        // Add virtual nodes for each shard
        for shard_idx in 0..shard_count {
            let shard_id = ShardId(shard_idx as u32);

            for vnode_idx in 0..virtual_nodes_per_shard {
                // Hash: shard_id:vnode_idx
                let key = format!("shard:{}:vnode:{}", shard_idx, vnode_idx);
                let hash = xxh3_64(key.as_bytes());
                ring.insert(hash, shard_id);
            }
        }

        Self {
            ring,
            virtual_nodes_per_shard,
            shard_count,
        }
    }

    /// Get shard for a given key
    pub fn get_shard(&self, key: &str) -> ShardId {
        let hash = xxh3_64(key.as_bytes());

        // Find next shard on ring (clockwise)
        self.ring
            .range(hash..)
            .next()
            .or_else(|| self.ring.iter().next()) // Wrap around
            .map(|(_, shard_id)| *shard_id)
            .unwrap_or(ShardId(0))
    }

    /// Get multiple shards for replication (primary + replicas)
    pub fn get_shards_with_replication(&self, key: &str, replicas: usize) -> Vec<ShardId> {
        let hash = xxh3_64(key.as_bytes());
        let mut shards = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Get next N unique shards on ring
        for (_, shard_id) in self.ring.range(hash..) {
            if seen.insert(*shard_id) {
                shards.push(*shard_id);
                if shards.len() >= replicas + 1 {
                    break;
                }
            }
        }

        // Wrap around if needed
        if shards.len() < replicas + 1 {
            for (_, shard_id) in self.ring.iter() {
                if seen.insert(*shard_id) {
                    shards.push(*shard_id);
                    if shards.len() >= replicas + 1 {
                        break;
                    }
                }
            }
        }

        shards
    }
}
```

#### 2. Shard Key Strategy

```rust
/// Generate shard key for different entity types
pub trait ShardKey {
    fn shard_key(&self) -> String;
}

impl ShardKey for Uuid {
    fn shard_key(&self) -> String {
        // Use UUID as-is for random distribution
        self.to_string()
    }
}

/// Workflow-based sharding (all executions in same shard)
pub struct WorkflowShardKey {
    pub workflow_id: Uuid,
}

impl ShardKey for WorkflowShardKey {
    fn shard_key(&self) -> String {
        // All executions of a workflow go to same shard
        format!("workflow:{}", self.workflow_id)
    }
}

/// Organization-based sharding (multi-tenancy)
pub struct OrganizationShardKey {
    pub organization_id: Uuid,
}

impl ShardKey for OrganizationShardKey {
    fn shard_key(&self) -> String {
        // All org data in same shard for locality
        format!("org:{}", self.organization_id)
    }
}

/// Time-based sharding (archival optimization)
pub struct TimeShardKey {
    pub timestamp: DateTime<Utc>,
}

impl ShardKey for TimeShardKey {
    fn shard_key(&self) -> String {
        // Shard by year-month for time-series data
        format!("time:{}", self.timestamp.format("%Y-%m"))
    }
}
```

#### 3. S3 Shard Structure

```
s3://workflow-bucket/
├── shards/
│   ├── shard-0000/
│   │   ├── workflows/
│   │   │   ├── {workflow_id}/
│   │   │   │   ├── metadata.json
│   │   │   │   ├── definition.yaml
│   │   │   │   └── versions/
│   │   ├── executions/
│   │   │   ├── {execution_id}/
│   │   │   │   ├── metadata.json
│   │   │   │   ├── result.json
│   │   │   │   └── logs/
│   │   └── checkpoints/
│   │
│   ├── shard-0001/
│   │   └── ... (same structure)
│   │
│   ├── shard-0002/
│   │   └── ...
│   │
│   └── shard-NNNN/
│       └── ...
│
├── metadata/
│   ├── shard-registry.json      # Shard metadata
│   ├── hash-ring.json            # Consistent hash ring config
│   └── rebalance-history.json   # Rebalancing audit log
│
└── queue/
    ├── pending/
    ├── processing/
    └── completed/
```

### Auto-Balanced Sharding

#### 1. Shard Metadata Management

```rust
use serde::{Deserialize, Serialize};

/// Shard registry stored in S3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardRegistry {
    /// Shard metadata
    pub shards: Vec<ShardMetadata>,

    /// Hash ring configuration
    pub hash_ring: HashRingConfig,

    /// Rebalancing state
    pub rebalance_state: Option<RebalanceState>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Version for optimistic locking
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardMetadata {
    pub shard_id: ShardId,
    pub status: ShardStatus,
    pub created_at: DateTime<Utc>,

    /// Estimated object count
    pub object_count: u64,

    /// Estimated total size in bytes
    pub total_size_bytes: u64,

    /// S3 bucket and prefix
    pub s3_location: S3Location,

    /// Replication factor
    pub replication_factor: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShardStatus {
    Active,
    Draining,      // Being migrated away
    ReadOnly,      // No new writes
    Offline,       // Temporarily unavailable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Location {
    pub bucket: String,
    pub prefix: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashRingConfig {
    pub shard_count: usize,
    pub virtual_nodes_per_shard: usize,
    pub hash_algorithm: String, // "xxh3"
}
```

#### 2. Automatic Shard Balancing

```rust
/// Shard balancer monitors and rebalances shards
pub struct ShardBalancer {
    s3_client: aws_sdk_s3::Client,
    registry_bucket: String,
    hash_ring: Arc<RwLock<ConsistentHashRing>>,
}

impl ShardBalancer {
    /// Check if rebalancing is needed
    pub async fn should_rebalance(&self) -> Result<bool> {
        let registry = self.load_registry().await?;

        // Calculate shard imbalance
        let avg_size = registry.shards.iter()
            .map(|s| s.total_size_bytes)
            .sum::<u64>() / registry.shards.len() as u64;

        let max_size = registry.shards.iter()
            .map(|s| s.total_size_bytes)
            .max()
            .unwrap_or(0);

        // Trigger rebalance if max shard is >150% of average
        let imbalance_ratio = max_size as f64 / avg_size as f64;

        Ok(imbalance_ratio > 1.5)
    }

    /// Trigger automatic rebalancing
    pub async fn rebalance(&self) -> Result<()> {
        // 1. Load current registry
        let mut registry = self.load_registry().await?;

        // 2. Calculate optimal shard count
        let total_size: u64 = registry.shards.iter()
            .map(|s| s.total_size_bytes)
            .sum();

        // Target: 10GB per shard
        let target_shard_size = 10 * 1024 * 1024 * 1024; // 10GB
        let optimal_shard_count = (total_size / target_shard_size).max(1) as usize;

        // 3. Add new shards if needed
        if optimal_shard_count > registry.shards.len() {
            let new_shard_count = optimal_shard_count - registry.shards.len();

            for _ in 0..new_shard_count {
                self.add_shard(&mut registry).await?;
            }
        }

        // 4. Create new hash ring
        let new_ring = ConsistentHashRing::new(registry.shards.len());

        // 5. Start migration
        let rebalance = RebalanceState {
            id: Uuid::new_v4(),
            started_at: Utc::now(),
            status: RebalanceStatus::InProgress,
            old_shard_count: registry.hash_ring.shard_count,
            new_shard_count: optimal_shard_count,
            migrated_objects: 0,
            total_objects: 0,
        };

        registry.rebalance_state = Some(rebalance);
        self.save_registry(&registry).await?;

        // 6. Migrate objects to new shards
        self.migrate_objects(&registry, &new_ring).await?;

        // 7. Update hash ring
        {
            let mut ring = self.hash_ring.write().await;
            *ring = new_ring;
        }

        // 8. Mark rebalance complete
        registry.rebalance_state = None;
        registry.hash_ring.shard_count = optimal_shard_count;
        self.save_registry(&registry).await?;

        Ok(())
    }

    /// Add a new shard
    async fn add_shard(&self, registry: &mut ShardRegistry) -> Result<ShardId> {
        let shard_id = ShardId(registry.shards.len() as u32);

        let metadata = ShardMetadata {
            shard_id,
            status: ShardStatus::Active,
            created_at: Utc::now(),
            object_count: 0,
            total_size_bytes: 0,
            s3_location: S3Location {
                bucket: self.registry_bucket.clone(),
                prefix: format!("shards/shard-{:04}", shard_id.0),
                region: "us-east-1".to_string(),
            },
            replication_factor: 3,
        };

        registry.shards.push(metadata);

        Ok(shard_id)
    }

    /// Migrate objects to new shard layout
    async fn migrate_objects(
        &self,
        registry: &ShardRegistry,
        new_ring: &ConsistentHashRing,
    ) -> Result<()> {
        // For each existing shard
        for old_shard in &registry.shards {
            let prefix = format!("{}/workflows/", old_shard.s3_location.prefix);

            // List all objects
            let mut continuation_token = None;

            loop {
                let mut req = self.s3_client
                    .list_objects_v2()
                    .bucket(&old_shard.s3_location.bucket)
                    .prefix(&prefix)
                    .max_keys(1000);

                if let Some(token) = continuation_token {
                    req = req.continuation_token(token);
                }

                let resp = req.send().await?;

                // Process each object
                for object in resp.contents().unwrap_or_default() {
                    let key = object.key().unwrap();

                    // Extract entity ID from key
                    let entity_id = self.extract_entity_id(key)?;

                    // Determine new shard
                    let new_shard_id = new_ring.get_shard(&entity_id);

                    // If shard changed, migrate object
                    if new_shard_id != old_shard.shard_id {
                        self.migrate_object(
                            &old_shard.s3_location,
                            key,
                            &registry.shards[new_shard_id.0 as usize].s3_location,
                        ).await?;
                    }
                }

                // Check if more pages
                if resp.is_truncated().unwrap_or(false) {
                    continuation_token = resp.next_continuation_token().map(|s| s.to_string());
                } else {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Migrate single object between shards
    async fn migrate_object(
        &self,
        from: &S3Location,
        key: &str,
        to: &S3Location,
    ) -> Result<()> {
        // 1. Copy object to new shard
        let copy_source = format!("{}/{}", from.bucket, key);
        let new_key = key.replace(&from.prefix, &to.prefix);

        self.s3_client
            .copy_object()
            .bucket(&to.bucket)
            .key(&new_key)
            .copy_source(&copy_source)
            .send()
            .await?;

        // 2. Verify copy succeeded
        self.s3_client
            .head_object()
            .bucket(&to.bucket)
            .key(&new_key)
            .send()
            .await?;

        // 3. Delete from old shard
        self.s3_client
            .delete_object()
            .bucket(&from.bucket)
            .key(key)
            .send()
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceState {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub status: RebalanceStatus,
    pub old_shard_count: usize,
    pub new_shard_count: usize,
    pub migrated_objects: u64,
    pub total_objects: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RebalanceStatus {
    InProgress,
    Paused,
    Completed,
    Failed { error: String },
}
```

#### 3. Zero-Downtime Migration

```rust
/// Shadow reads during migration
pub struct ShadowReadRouter {
    hash_ring: Arc<RwLock<ConsistentHashRing>>,
    old_ring: Option<ConsistentHashRing>,
}

impl ShadowReadRouter {
    /// Read with fallback to old shard during migration
    pub async fn get_workflow(&self, workflow_id: Uuid) -> Result<Option<Workflow>> {
        let key = workflow_id.to_string();

        // Try new shard first
        let ring = self.hash_ring.read().await;
        let new_shard_id = ring.get_shard(&key);
        drop(ring);

        if let Some(workflow) = self.read_from_shard(new_shard_id, &workflow_id).await? {
            return Ok(Some(workflow));
        }

        // Fallback to old shard if migration in progress
        if let Some(old_ring) = &self.old_ring {
            let old_shard_id = old_ring.get_shard(&key);

            if old_shard_id != new_shard_id {
                return self.read_from_shard(old_shard_id, &workflow_id).await;
            }
        }

        Ok(None)
    }
}
```

### Performance Optimizations

#### 1. S3 Request Optimization

```rust
/// Batch operations for efficiency
pub struct S3BatchOperations {
    s3_client: aws_sdk_s3::Client,
}

impl S3BatchOperations {
    /// Batch upload using multipart upload
    pub async fn batch_upload(&self, objects: Vec<S3Object>) -> Result<()> {
        // Use S3 Batch Operations API for large batches
        // https://aws.amazon.com/s3/features/batch-operations/

        for chunk in objects.chunks(1000) {
            // Parallel uploads
            let futures: Vec<_> = chunk.iter()
                .map(|obj| self.upload_object(obj))
                .collect();

            futures::future::try_join_all(futures).await?;
        }

        Ok(())
    }

    /// Use S3 Select for filtering
    pub async fn query_with_select(
        &self,
        bucket: &str,
        key: &str,
        sql: &str,
    ) -> Result<Vec<u8>> {
        let resp = self.s3_client
            .select_object_content()
            .bucket(bucket)
            .key(key)
            .expression_type(aws_sdk_s3::types::ExpressionType::Sql)
            .expression(sql)
            .input_serialization(
                aws_sdk_s3::types::InputSerialization::builder()
                    .json(aws_sdk_s3::types::JsonInput::builder().build())
                    .build()
            )
            .output_serialization(
                aws_sdk_s3::types::OutputSerialization::builder()
                    .json(aws_sdk_s3::types::JsonOutput::builder().build())
                    .build()
            )
            .send()
            .await?;

        // Collect streaming results
        let mut result = Vec::new();
        let mut stream = resp.payload;

        while let Some(event) = stream.recv().await? {
            if let Some(records) = event.as_records() {
                result.extend_from_slice(records.payload());
            }
        }

        Ok(result)
    }
}
```

#### 2. S3 Caching Layer

```rust
/// Local cache for frequently accessed objects
pub struct S3CacheLayer {
    s3_client: aws_sdk_s3::Client,
    cache: Arc<Mutex<lru::LruCache<String, CachedObject>>>,
}

#[derive(Clone)]
struct CachedObject {
    data: bytes::Bytes,
    etag: String,
    cached_at: DateTime<Utc>,
}

impl S3CacheLayer {
    /// Get with local cache
    pub async fn get_cached(&self, bucket: &str, key: &str) -> Result<bytes::Bytes> {
        let cache_key = format!("{}/{}", bucket, key);

        // Check local cache
        {
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.peek(&cache_key) {
                // Validate not expired (5 min TTL)
                if Utc::now().signed_duration_since(cached.cached_at) < Duration::minutes(5) {
                    return Ok(cached.data.clone());
                }
            }
        }

        // Cache miss - fetch from S3
        let resp = self.s3_client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;

        let etag = resp.e_tag().unwrap_or("").to_string();
        let data = resp.body.collect().await?.into_bytes();

        // Update cache
        {
            let mut cache = self.cache.lock().await;
            cache.put(
                cache_key,
                CachedObject {
                    data: data.clone(),
                    etag,
                    cached_at: Utc::now(),
                },
            );
        }

        Ok(data)
    }
}
```

#### 3. S3 Intelligent-Tiering

```toml
# Enable S3 Intelligent-Tiering for automatic cost optimization
[storage.s3]
storage_class = "INTELLIGENT_TIERING"

# Lifecycle rules for archival
[[storage.s3.lifecycle_rules]]
id = "archive-old-executions"
filter.prefix = "shards/*/executions/"
transitions = [
    { days = 90, storage_class = "GLACIER_IR" },     # Immediate retrieval Glacier
    { days = 180, storage_class = "GLACIER" },        # Standard Glacier
    { days = 365, storage_class = "DEEP_ARCHIVE" }    # Deep Archive
]

[[storage.s3.lifecycle_rules]]
id = "delete-old-logs"
filter.prefix = "shards/*/executions/*/logs/"
expiration.days = 30  # Delete logs after 30 days
```

### Queue Coordination with SQS

```rust
/// SQS-based queue for S3-only architecture
pub struct SqsWorkQueue {
    sqs_client: aws_sdk_sqs::Client,
    queue_url: String,
    s3_client: aws_sdk_s3::Client,
    shard_router: Arc<ShardRouter>,
}

impl SqsWorkQueue {
    /// Enqueue job (stores job in S3, sends reference to SQS)
    pub async fn enqueue(&self, job: Job) -> Result<Uuid> {
        let job_id = job.id;

        // 1. Determine shard for job
        let shard_id = self.shard_router.get_shard_for_execution(&job.execution_id);

        // 2. Store job payload in S3
        let s3_key = format!("shards/shard-{:04}/queue/pending/{}.json", shard_id.0, job_id);

        self.s3_client
            .put_object()
            .bucket(&self.shard_router.get_bucket())
            .key(&s3_key)
            .body(serde_json::to_vec(&job)?.into())
            .content_type("application/json")
            .send()
            .await?;

        // 3. Send reference to SQS
        let message = SqsMessage {
            job_id,
            s3_key: s3_key.clone(),
            priority: job.priority,
        };

        self.sqs_client
            .send_message()
            .queue_url(&self.queue_url)
            .message_body(serde_json::to_string(&message)?)
            .send()
            .await?;

        Ok(job_id)
    }

    /// Dequeue job
    pub async fn dequeue(&self, worker_id: &str) -> Result<Option<Job>> {
        // 1. Receive message from SQS
        let resp = self.sqs_client
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(1)
            .visibility_timeout(300) // 5 minutes
            .wait_time_seconds(20)   // Long polling
            .send()
            .await?;

        let message = match resp.messages().and_then(|msgs| msgs.first()) {
            Some(msg) => msg,
            None => return Ok(None),
        };

        // 2. Parse SQS message
        let sqs_msg: SqsMessage = serde_json::from_str(message.body().unwrap_or("{}"))?;

        // 3. Fetch job from S3
        let resp = self.s3_client
            .get_object()
            .bucket(&self.shard_router.get_bucket())
            .key(&sqs_msg.s3_key)
            .send()
            .await?;

        let bytes = resp.body.collect().await?.into_bytes();
        let job: Job = serde_json::from_slice(&bytes)?;

        Ok(Some(job))
    }
}

#[derive(Serialize, Deserialize)]
struct SqsMessage {
    job_id: Uuid,
    s3_key: String,
    priority: i32,
}
```

### Cost Analysis

**S3-Only Architecture Costs** (per month):

**Small Scale** (< 100 workflows/day):
- S3 Storage (100GB): $2.30
- S3 Requests: $5
- SQS (1M requests): $0.40
- Lambda (10M invocations): $2
- **Total**: ~$10/month

**Medium Scale** (< 1,000 workflows/day):
- S3 Storage (1TB): $23
- S3 Requests: $50
- SQS (10M requests): $4
- Lambda (100M invocations): $20
- **Total**: ~$97/month

**Large Scale** (< 10,000 workflows/day):
- S3 Storage (10TB): $230
- S3 Requests: $500
- SQS (100M requests): $40
- Lambda/Fargate Workers: $200
- **Total**: ~$970/month

**Massive Scale** (< 100,000 workflows/day):
- S3 Storage (100TB): $2,300
- S3 Requests: $5,000
- SQS (1B requests): $400
- Fargate Workers: $2,000
- **Total**: ~$9,700/month

**Cost Benefits**:
- 90% cheaper than PostgreSQL + Redis at scale
- No database management overhead
- Pay only for actual usage
- Automatic scaling

### Deployment Architecture

```yaml
# docker-compose.yml - S3-only deployment
version: '3.8'

services:
  api:
    image: workflow-api:latest
    environment:
      - STORAGE_BACKEND=s3
      - QUEUE_BACKEND=sqs
      - AWS_REGION=us-east-1
      - S3_BUCKET=workflow-storage
      - SQS_QUEUE_URL=${SQS_QUEUE_URL}
    deploy:
      replicas: 3

  worker:
    image: workflow-worker:latest
    environment:
      - STORAGE_BACKEND=s3
      - QUEUE_BACKEND=sqs
      - AWS_REGION=us-east-1
      - S3_BUCKET=workflow-storage
      - SQS_QUEUE_URL=${SQS_QUEUE_URL}
    deploy:
      replicas: 5

  # Optional: Local MinIO for development
  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    ports:
      - "9000:9000"
      - "9001:9001"
```

## Workflow Diagramming & Visualization

### Overview

Transform YAML workflow definitions into interactive, real-time visual diagrams. The system provides multiple visualization modes for different use cases: DAG (Directed Acyclic Graph) for dependencies, hierarchical tree for task structure, timeline for execution flow, and interactive 3D for complex workflows.

### Architecture

```
┌─────────────────────────────────────────────────────┐
│              Workflow Definition (YAML)              │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│           Workflow Parser & Analyzer                 │
│  ┌──────────┬──────────┬──────────┬──────────┐     │
│  │Dependency│Hierarchy │Execution │Complexity│     │
│  │Resolver  │Builder   │Flow      │Metrics   │     │
│  └──────────┴──────────┴──────────┴──────────┘     │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│              Graph Data Structure                    │
│  {                                                   │
│    nodes: [{ id, type, label, metadata }],          │
│    edges: [{ source, target, type }],               │
│    layout: { algorithm, positions }                 │
│  }                                                   │
└────────────────────┬────────────────────────────────┘
                     │
          ┌──────────┴──────────┬──────────────┐
          ▼                     ▼              ▼
┌──────────────────┐  ┌─────────────────┐  ┌──────────────┐
│   DAG Renderer   │  │  Tree Renderer  │  │   Timeline   │
│  (D3.js/Cyto)    │  │  (Hierarchical) │  │  (Gantt)     │
└──────────────────┘  └─────────────────┘  └──────────────┘
          │                     │                    │
          └──────────┬──────────┴────────────────────┘
                     ▼
┌─────────────────────────────────────────────────────┐
│         Interactive Canvas (React/Svelte)            │
│  ┌─────────────────────────────────────────────┐   │
│  │  • Pan/Zoom                                  │   │
│  │  • Node Selection                            │   │
│  │  • Edge Highlighting                         │   │
│  │  • Real-time Execution Overlay               │   │
│  │  • Minimap Navigator                         │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### Backend Implementation

#### 1. Graph Data Model

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Visual graph representation of a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGraph {
    /// Graph metadata
    pub metadata: GraphMetadata,

    /// Nodes (tasks, agents, decision points)
    pub nodes: Vec<GraphNode>,

    /// Edges (dependencies, data flow)
    pub edges: Vec<GraphEdge>,

    /// Layout information
    pub layout: GraphLayout,

    /// Complexity metrics
    pub metrics: GraphMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub workflow_id: Uuid,
    pub workflow_name: String,
    pub version: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: NodeType,
    pub label: String,
    pub description: Option<String>,

    /// Visual properties
    pub style: NodeStyle,

    /// Position (if manually positioned)
    pub position: Option<Position>,

    /// Metadata for different node types
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeType {
    Agent { agent_id: String },
    Task { task_id: String },
    SubworkflowStart,
    SubworkflowEnd,
    DecisionPoint,
    LoopStart { iterations: Option<u32> },
    LoopEnd,
    Checkpoint { name: String },
    Start,
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStyle {
    pub color: String,
    pub shape: NodeShape,
    pub size: NodeSize,
    pub icon: Option<String>,
    pub badge: Option<String>,  // For counts, status
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeShape {
    Circle,
    Rectangle,
    Diamond,      // Decision points
    Hexagon,      // Agents
    Ellipse,      // Tasks
    Stadium,      // Start/End
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeSize {
    Small,
    Medium,
    Large,
    Custom { width: u32, height: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeMetadata {
    Task {
        agent: String,
        estimated_duration: Option<Duration>,
        retry_count: u32,
        timeout: Option<Duration>,
    },
    Agent {
        model: String,
        tools: Vec<String>,
        permissions: String,
    },
    Decision {
        conditions: Vec<String>,
        branches: u32,
    },
    Loop {
        collection_size: Option<usize>,
        parallelism: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub label: Option<String>,
    pub style: EdgeStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    Dependency,       // Task dependencies
    DataFlow,         // Data passing between tasks
    ControlFlow,      // Conditional execution
    Subworkflow,      // Parent-child relationships
    LoopIteration,    // Loop backedges
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeStyle {
    pub color: String,
    pub width: u32,
    pub line_style: LineStyle,
    pub animated: bool,  // Animate during execution
    pub arrow: ArrowStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArrowStyle {
    Normal,
    Thick,
    Diamond,
    Circle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLayout {
    pub algorithm: LayoutAlgorithm,
    pub direction: LayoutDirection,
    pub spacing: LayoutSpacing,
    pub clusters: Vec<Cluster>,  // Grouped nodes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutAlgorithm {
    Dagre,           // Hierarchical DAG
    ForceDirected,   // Physics-based
    Hierarchical,    // Tree-like
    Circular,        // Circular layout
    Grid,            // Grid-based
    Manual,          // User-positioned
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSpacing {
    pub node_spacing: u32,
    pub rank_spacing: u32,
    pub edge_spacing: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub id: String,
    pub label: String,
    pub nodes: Vec<String>,
    pub style: ClusterStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStyle {
    pub background_color: String,
    pub border_color: String,
    pub border_width: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub max_depth: usize,
    pub cyclomatic_complexity: u32,
    pub parallelism_factor: f64,
    pub critical_path_length: usize,
    pub estimated_duration: Option<Duration>,
}
```

#### 2. Graph Generator

```rust
/// Generate visual graph from workflow definition
pub struct WorkflowGraphGenerator {
    workflow: Workflow,
}

impl WorkflowGraphGenerator {
    pub fn new(workflow: Workflow) -> Self {
        Self { workflow }
    }

    /// Generate complete graph structure
    pub fn generate(&self) -> Result<WorkflowGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // 1. Create start node
        nodes.push(GraphNode {
            id: "start".to_string(),
            node_type: NodeType::Start,
            label: "Start".to_string(),
            description: None,
            style: NodeStyle {
                color: "#4CAF50".to_string(),
                shape: NodeShape::Stadium,
                size: NodeSize::Medium,
                icon: Some("play_arrow".to_string()),
                badge: None,
            },
            position: None,
            metadata: NodeMetadata::Task {
                agent: String::new(),
                estimated_duration: None,
                retry_count: 0,
                timeout: None,
            },
        });

        // 2. Create agent nodes
        for (agent_id, agent) in &self.workflow.agents {
            nodes.push(self.create_agent_node(agent_id, agent)?);
        }

        // 3. Create task nodes
        for (task_id, task) in &self.workflow.tasks {
            nodes.push(self.create_task_node(task_id, task)?);

            // Create edges for dependencies
            for dep_id in &task.depends_on {
                edges.push(GraphEdge {
                    id: format!("{}→{}", dep_id, task_id),
                    source: dep_id.clone(),
                    target: task_id.clone(),
                    edge_type: EdgeType::Dependency,
                    label: None,
                    style: EdgeStyle {
                        color: "#666".to_string(),
                        width: 2,
                        line_style: LineStyle::Solid,
                        animated: false,
                        arrow: ArrowStyle::Normal,
                    },
                });
            }

            // Create edges for subtasks
            for subtask_id in &task.subtasks {
                edges.push(GraphEdge {
                    id: format!("{}⊃{}", task_id, subtask_id),
                    source: task_id.clone(),
                    target: subtask_id.clone(),
                    edge_type: EdgeType::Subworkflow,
                    label: Some("contains".to_string()),
                    style: EdgeStyle {
                        color: "#9C27B0".to_string(),
                        width: 2,
                        line_style: LineStyle::Dashed,
                        animated: false,
                        arrow: ArrowStyle::Diamond,
                    },
                });
            }
        }

        // 4. Create end node
        nodes.push(GraphNode {
            id: "end".to_string(),
            node_type: NodeType::End,
            label: "End".to_string(),
            description: None,
            style: NodeStyle {
                color: "#F44336".to_string(),
                shape: NodeShape::Stadium,
                size: NodeSize::Medium,
                icon: Some("stop".to_string()),
                badge: None,
            },
            position: None,
            metadata: NodeMetadata::Task {
                agent: String::new(),
                estimated_duration: None,
                retry_count: 0,
                timeout: None,
            },
        });

        // 5. Calculate layout
        let layout = self.calculate_layout(&nodes, &edges)?;

        // 6. Calculate metrics
        let metrics = self.calculate_metrics(&nodes, &edges)?;

        Ok(WorkflowGraph {
            metadata: GraphMetadata {
                workflow_id: self.workflow.id,
                workflow_name: self.workflow.name.clone(),
                version: self.workflow.version.clone(),
                generated_at: Utc::now(),
            },
            nodes,
            edges,
            layout,
            metrics,
        })
    }

    fn create_agent_node(&self, agent_id: &str, agent: &Agent) -> Result<GraphNode> {
        Ok(GraphNode {
            id: format!("agent:{}", agent_id),
            node_type: NodeType::Agent {
                agent_id: agent_id.to_string(),
            },
            label: agent_id.to_string(),
            description: Some(agent.description.clone()),
            style: NodeStyle {
                color: "#2196F3".to_string(),
                shape: NodeShape::Hexagon,
                size: NodeSize::Large,
                icon: Some("smart_toy".to_string()),
                badge: Some(format!("{}", agent.tools.len())),
            },
            position: None,
            metadata: NodeMetadata::Agent {
                model: agent.model.clone().unwrap_or_else(|| "claude-sonnet-4-5".to_string()),
                tools: agent.tools.iter().map(|t| format!("{:?}", t)).collect(),
                permissions: format!("{:?}", agent.permissions.mode),
            },
        })
    }

    fn create_task_node(&self, task_id: &str, task: &Task) -> Result<GraphNode> {
        Ok(GraphNode {
            id: task_id.to_string(),
            node_type: NodeType::Task {
                task_id: task_id.to_string(),
            },
            label: task_id.to_string(),
            description: Some(task.description.clone()),
            style: NodeStyle {
                color: "#FF9800".to_string(),
                shape: NodeShape::Ellipse,
                size: NodeSize::Medium,
                icon: Some("task".to_string()),
                badge: if task.subtasks.is_empty() {
                    None
                } else {
                    Some(format!("{}", task.subtasks.len()))
                },
            },
            position: None,
            metadata: NodeMetadata::Task {
                agent: task.agent.clone(),
                estimated_duration: None,
                retry_count: 0,
                timeout: None,
            },
        })
    }

    fn calculate_layout(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
    ) -> Result<GraphLayout> {
        // Use Dagre for automatic hierarchical layout
        Ok(GraphLayout {
            algorithm: LayoutAlgorithm::Dagre,
            direction: LayoutDirection::TopToBottom,
            spacing: LayoutSpacing {
                node_spacing: 100,
                rank_spacing: 150,
                edge_spacing: 50,
            },
            clusters: self.identify_clusters(nodes)?,
        })
    }

    fn identify_clusters(&self, nodes: &[GraphNode]) -> Result<Vec<Cluster>> {
        let mut clusters = Vec::new();

        // Cluster by agent
        let mut agent_groups: HashMap<String, Vec<String>> = HashMap::new();

        for node in nodes {
            if let NodeMetadata::Task { agent, .. } = &node.metadata {
                agent_groups.entry(agent.clone())
                    .or_insert_with(Vec::new)
                    .push(node.id.clone());
            }
        }

        for (agent_id, node_ids) in agent_groups {
            if node_ids.len() > 1 {
                clusters.push(Cluster {
                    id: format!("cluster:{}", agent_id),
                    label: format!("Agent: {}", agent_id),
                    nodes: node_ids,
                    style: ClusterStyle {
                        background_color: "#E3F2FD".to_string(),
                        border_color: "#2196F3".to_string(),
                        border_width: 2,
                    },
                });
            }
        }

        Ok(clusters)
    }

    fn calculate_metrics(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
    ) -> Result<GraphMetrics> {
        // Calculate graph complexity metrics
        let max_depth = self.calculate_max_depth(nodes, edges)?;
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(edges)?;
        let critical_path = self.find_critical_path(nodes, edges)?;

        Ok(GraphMetrics {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            max_depth,
            cyclomatic_complexity,
            parallelism_factor: self.calculate_parallelism(nodes, edges)?,
            critical_path_length: critical_path.len(),
            estimated_duration: None,
        })
    }

    fn calculate_max_depth(&self, nodes: &[GraphNode], edges: &[GraphEdge]) -> Result<usize> {
        // BFS to find maximum depth
        let mut depths = HashMap::new();
        let mut queue = std::collections::VecDeque::new();

        queue.push_back(("start".to_string(), 0));
        depths.insert("start".to_string(), 0);

        while let Some((node_id, depth)) = queue.pop_front() {
            for edge in edges {
                if edge.source == node_id {
                    let new_depth = depth + 1;
                    let current_depth = depths.get(&edge.target).copied().unwrap_or(0);

                    if new_depth > current_depth {
                        depths.insert(edge.target.clone(), new_depth);
                        queue.push_back((edge.target.clone(), new_depth));
                    }
                }
            }
        }

        Ok(*depths.values().max().unwrap_or(&0))
    }

    fn calculate_cyclomatic_complexity(&self, edges: &[GraphEdge]) -> Result<u32> {
        // McCabe's cyclomatic complexity: E - N + 2P
        // For workflows: number of decision points + 1
        let decision_points = edges.iter()
            .filter(|e| matches!(e.edge_type, EdgeType::ControlFlow))
            .count() as u32;

        Ok(decision_points + 1)
    }

    fn calculate_parallelism(&self, nodes: &[GraphNode], edges: &[GraphEdge]) -> Result<f64> {
        // Average parallelism across all levels
        let levels = self.group_by_level(nodes, edges)?;
        let avg_parallel = levels.iter()
            .map(|level| level.len())
            .sum::<usize>() as f64 / levels.len() as f64;

        Ok(avg_parallel)
    }

    fn group_by_level(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
    ) -> Result<Vec<Vec<String>>> {
        let mut levels: HashMap<usize, Vec<String>> = HashMap::new();
        let depths = self.calculate_node_depths(nodes, edges)?;

        for (node_id, depth) in depths {
            levels.entry(depth)
                .or_insert_with(Vec::new)
                .push(node_id);
        }

        let max_level = *levels.keys().max().unwrap_or(&0);
        Ok((0..=max_level).map(|i| levels.get(&i).cloned().unwrap_or_default()).collect())
    }

    fn calculate_node_depths(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
    ) -> Result<HashMap<String, usize>> {
        let mut depths = HashMap::new();
        let mut queue = std::collections::VecDeque::new();

        queue.push_back(("start".to_string(), 0));
        depths.insert("start".to_string(), 0);

        while let Some((node_id, depth)) = queue.pop_front() {
            for edge in edges {
                if edge.source == node_id && !depths.contains_key(&edge.target) {
                    depths.insert(edge.target.clone(), depth + 1);
                    queue.push_back((edge.target.clone(), depth + 1));
                }
            }
        }

        Ok(depths)
    }

    fn find_critical_path(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
    ) -> Result<Vec<String>> {
        // Find longest path from start to end (critical path)
        let mut path = Vec::new();
        let mut visited = std::collections::HashSet::new();

        self.dfs_longest_path("start", "end", edges, &mut visited, &mut path);

        Ok(path)
    }

    fn dfs_longest_path(
        &self,
        current: &str,
        target: &str,
        edges: &[GraphEdge],
        visited: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        if current == target {
            path.push(current.to_string());
            return true;
        }

        visited.insert(current.to_string());

        for edge in edges {
            if edge.source == current && !visited.contains(&edge.target) {
                if self.dfs_longest_path(&edge.target, target, edges, visited, path) {
                    path.push(current.to_string());
                    return true;
                }
            }
        }

        visited.remove(current);
        false
    }
}
```

#### 3. Real-time Execution Overlay

```rust
/// Live execution state overlaid on graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOverlay {
    pub execution_id: Uuid,
    pub workflow_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub status: ExecutionStatus,

    /// Node execution states
    pub node_states: HashMap<String, NodeExecutionState>,

    /// Edge traversal animations
    pub active_edges: Vec<String>,

    /// Current execution path
    pub execution_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionState {
    pub status: TaskStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: f64,  // 0.0 to 1.0
    pub error: Option<String>,
    pub output_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Cancelled,
}
```

#### 4. API Endpoints

```rust
// REST API for graph operations
use axum::{Json, Router};
use axum::routing::{get, post};

pub fn graph_routes() -> Router {
    Router::new()
        .route("/api/v1/workflows/:id/graph", get(get_workflow_graph))
        .route("/api/v1/workflows/:id/graph/layout", post(update_graph_layout))
        .route("/api/v1/executions/:id/graph/overlay", get(get_execution_overlay))
        .route("/api/v1/workflows/:id/graph/export", get(export_graph))
}

/// GET /api/v1/workflows/:id/graph
async fn get_workflow_graph(
    workflow_id: Path<Uuid>,
) -> Result<Json<WorkflowGraph>> {
    let workflow = load_workflow(*workflow_id).await?;
    let generator = WorkflowGraphGenerator::new(workflow);
    let graph = generator.generate()?;

    Ok(Json(graph))
}

/// POST /api/v1/workflows/:id/graph/layout
async fn update_graph_layout(
    workflow_id: Path<Uuid>,
    Json(layout_update): Json<LayoutUpdate>,
) -> Result<Json<WorkflowGraph>> {
    // Update manual node positions
    let mut graph = load_graph(*workflow_id).await?;

    for (node_id, position) in layout_update.positions {
        if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == node_id) {
            node.position = Some(position);
        }
    }

    graph.layout.algorithm = LayoutAlgorithm::Manual;
    save_graph(&graph).await?;

    Ok(Json(graph))
}

#[derive(Debug, Deserialize)]
struct LayoutUpdate {
    positions: HashMap<String, Position>,
}

/// GET /api/v1/executions/:id/graph/overlay
async fn get_execution_overlay(
    execution_id: Path<Uuid>,
) -> Result<Json<ExecutionOverlay>> {
    let execution = load_execution(*execution_id).await?;
    let overlay = build_execution_overlay(&execution).await?;

    Ok(Json(overlay))
}

/// GET /api/v1/workflows/:id/graph/export
async fn export_graph(
    workflow_id: Path<Uuid>,
    Query(params): Query<ExportParams>,
) -> Result<Response> {
    let graph = load_graph(*workflow_id).await?;

    let exported = match params.format {
        ExportFormat::Svg => export_to_svg(&graph)?,
        ExportFormat::Png => export_to_png(&graph)?,
        ExportFormat::Pdf => export_to_pdf(&graph)?,
        ExportFormat::Mermaid => export_to_mermaid(&graph)?,
        ExportFormat::Dot => export_to_dot(&graph)?,
    };

    Ok(Response::builder()
        .header("Content-Type", params.format.mime_type())
        .body(exported)?)
}

#[derive(Debug, Deserialize)]
struct ExportParams {
    format: ExportFormat,
}

#[derive(Debug, Deserialize)]
enum ExportFormat {
    Svg,
    Png,
    Pdf,
    Mermaid,
    Dot,
}

impl ExportFormat {
    fn mime_type(&self) -> &str {
        match self {
            Self::Svg => "image/svg+xml",
            Self::Png => "image/png",
            Self::Pdf => "application/pdf",
            Self::Mermaid => "text/plain",
            Self::Dot => "text/vnd.graphviz",
        }
    }
}
```

### Frontend Implementation

#### 1. React Component with Cytoscape.js

```typescript
// WorkflowDiagram.tsx
import React, { useEffect, useRef, useState } from 'react';
import cytoscape, { Core, NodeSingular } from 'cytoscape';
import dagre from 'cytoscape-dagre';
import { WorkflowGraph, ExecutionOverlay } from './types';

cytoscape.use(dagre);

interface WorkflowDiagramProps {
  workflowId: string;
  executionId?: string;
  interactive?: boolean;
  showMinimap?: boolean;
}

export const WorkflowDiagram: React.FC<WorkflowDiagramProps> = ({
  workflowId,
  executionId,
  interactive = true,
  showMinimap = true,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [cy, setCy] = useState<Core | null>(null);
  const [graph, setGraph] = useState<WorkflowGraph | null>(null);
  const [overlay, setOverlay] = useState<ExecutionOverlay | null>(null);

  // Load workflow graph
  useEffect(() => {
    const loadGraph = async () => {
      const response = await fetch(`/api/v1/workflows/${workflowId}/graph`);
      const data = await response.json();
      setGraph(data);
    };

    loadGraph();
  }, [workflowId]);

  // Load execution overlay if execution ID provided
  useEffect(() => {
    if (!executionId) return;

    const loadOverlay = async () => {
      const response = await fetch(`/api/v1/executions/${executionId}/graph/overlay`);
      const data = await response.json();
      setOverlay(data);
    };

    loadOverlay();

    // Subscribe to real-time updates via WebSocket
    const ws = new WebSocket(`ws://localhost:8080/api/v1/ws/executions/${executionId}`);

    ws.onmessage = (event) => {
      const update = JSON.parse(event.data);
      setOverlay(prev => ({
        ...prev,
        ...update,
      }));
    };

    return () => ws.close();
  }, [executionId]);

  // Initialize Cytoscape
  useEffect(() => {
    if (!containerRef.current || !graph) return;

    const cyInstance = cytoscape({
      container: containerRef.current,
      elements: convertGraphToCytoscape(graph),
      style: getCytoscapeStyles(),
      layout: {
        name: 'dagre',
        rankDir: 'TB',
        nodeSep: 100,
        rankSep: 150,
        animate: true,
      },
      wheelSensitivity: 0.2,
      minZoom: 0.1,
      maxZoom: 5,
    });

    // Event handlers
    if (interactive) {
      cyInstance.on('tap', 'node', (event) => {
        const node = event.target;
        showNodeDetails(node);
      });

      cyInstance.on('tap', 'edge', (event) => {
        const edge = event.target;
        highlightPath(edge);
      });
    }

    setCy(cyInstance);

    return () => {
      cyInstance.destroy();
    };
  }, [graph, interactive]);

  // Update visualization with execution overlay
  useEffect(() => {
    if (!cy || !overlay) return;

    // Update node states
    Object.entries(overlay.node_states).forEach(([nodeId, state]) => {
      const node = cy.getElementById(nodeId);
      if (!node) return;

      // Update node appearance based on status
      node.addClass(state.status.toLowerCase());

      // Add progress ring
      if (state.status === 'Running') {
        node.data('progress', state.progress);
      }

      // Add badges
      if (state.error) {
        node.data('error', true);
      }
    });

    // Animate active edges
    overlay.active_edges.forEach(edgeId => {
      const edge = cy.getElementById(edgeId);
      if (edge) {
        edge.addClass('active');
      }
    });

    // Highlight execution path
    const pathNodes = cy.collection();
    overlay.execution_path.forEach(nodeId => {
      pathNodes.merge(cy.getElementById(nodeId));
    });
    pathNodes.addClass('in-path');

  }, [cy, overlay]);

  return (
    <div className="workflow-diagram-container">
      <div ref={containerRef} className="cytoscape-canvas" />

      {showMinimap && <Minimap cy={cy} />}

      <div className="diagram-controls">
        <button onClick={() => cy?.fit()}>Fit to Screen</button>
        <button onClick={() => cy?.zoom(cy.zoom() * 1.2)}>Zoom In</button>
        <button onClick={() => cy?.zoom(cy.zoom() * 0.8)}>Zoom Out</button>
        <button onClick={() => exportDiagram()}>Export</button>
      </div>

      <div className="diagram-legend">
        <LegendItem color="#FF9800" label="Task" />
        <LegendItem color="#2196F3" label="Agent" />
        <LegendItem color="#4CAF50" label="Completed" />
        <LegendItem color="#F44336" label="Failed" />
      </div>
    </div>
  );
};

function convertGraphToCytoscape(graph: WorkflowGraph) {
  const nodes = graph.nodes.map(node => ({
    data: {
      id: node.id,
      label: node.label,
      type: node.node_type,
      metadata: node.metadata,
    },
    classes: node.style.shape.toLowerCase(),
    position: node.position,
  }));

  const edges = graph.edges.map(edge => ({
    data: {
      id: edge.id,
      source: edge.source,
      target: edge.target,
      label: edge.label,
      type: edge.edge_type,
    },
    classes: edge.edge_type.toLowerCase(),
  }));

  return [...nodes, ...edges];
}

function getCytoscapeStyles() {
  return [
    {
      selector: 'node',
      style: {
        'label': 'data(label)',
        'text-valign': 'center',
        'text-halign': 'center',
        'background-color': '#FF9800',
        'border-width': 2,
        'border-color': '#F57C00',
        'width': 80,
        'height': 80,
        'font-size': '12px',
      },
    },
    {
      selector: 'node.hexagon',
      style: {
        'shape': 'hexagon',
        'background-color': '#2196F3',
        'border-color': '#1976D2',
        'width': 100,
        'height': 100,
      },
    },
    {
      selector: 'node.stadium',
      style: {
        'shape': 'round-rectangle',
        'width': 120,
        'height': 60,
      },
    },
    {
      selector: 'node.pending',
      style: {
        'background-color': '#9E9E9E',
      },
    },
    {
      selector: 'node.running',
      style: {
        'background-color': '#2196F3',
        'border-width': 4,
        'border-color': '#1976D2',
      },
    },
    {
      selector: 'node.completed',
      style: {
        'background-color': '#4CAF50',
      },
    },
    {
      selector: 'node.failed',
      style: {
        'background-color': '#F44336',
      },
    },
    {
      selector: 'edge',
      style: {
        'width': 2,
        'line-color': '#666',
        'target-arrow-color': '#666',
        'target-arrow-shape': 'triangle',
        'curve-style': 'bezier',
      },
    },
    {
      selector: 'edge.active',
      style: {
        'width': 4,
        'line-color': '#2196F3',
        'target-arrow-color': '#2196F3',
        'line-style': 'solid',
      },
    },
    {
      selector: 'edge.subworkflow',
      style: {
        'line-style': 'dashed',
        'line-color': '#9C27B0',
        'target-arrow-shape': 'diamond',
      },
    },
  ];
}
```

#### 2. Alternative: D3.js Implementation

```typescript
// WorkflowD3Diagram.tsx
import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { WorkflowGraph } from './types';

export const WorkflowD3Diagram: React.FC<{ graph: WorkflowGraph }> = ({ graph }) => {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current) return;

    const width = 1200;
    const height = 800;

    const svg = d3.select(svgRef.current)
      .attr('width', width)
      .attr('height', height)
      .attr('viewBox', [0, 0, width, height]);

    // Clear previous content
    svg.selectAll('*').remove();

    // Create container group for pan/zoom
    const g = svg.append('g');

    // Define arrow markers
    svg.append('defs').selectAll('marker')
      .data(['dependency', 'dataflow', 'subworkflow'])
      .enter().append('marker')
        .attr('id', d => `arrow-${d}`)
        .attr('viewBox', '0 -5 10 10')
        .attr('refX', 15)
        .attr('refY', 0)
        .attr('markerWidth', 6)
        .attr('markerHeight', 6)
        .attr('orient', 'auto')
      .append('path')
        .attr('d', 'M0,-5L10,0L0,5')
        .attr('fill', d => d === 'dependency' ? '#666' : '#9C27B0');

    // Convert graph to D3 format
    const nodes = graph.nodes.map(n => ({
      ...n,
      x: n.position?.x || Math.random() * width,
      y: n.position?.y || Math.random() * height,
    }));

    const links = graph.edges.map(e => ({
      ...e,
      source: e.source,
      target: e.target,
    }));

    // Create force simulation
    const simulation = d3.forceSimulation(nodes)
      .force('link', d3.forceLink(links).id((d: any) => d.id).distance(150))
      .force('charge', d3.forceManyBody().strength(-500))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(60));

    // Draw links
    const link = g.append('g')
      .selectAll('line')
      .data(links)
      .enter().append('line')
        .attr('class', 'edge')
        .attr('stroke', d => d.style.color)
        .attr('stroke-width', d => d.style.width)
        .attr('marker-end', d => `url(#arrow-${d.edge_type.toLowerCase()})`);

    // Draw nodes
    const node = g.append('g')
      .selectAll('g')
      .data(nodes)
      .enter().append('g')
        .attr('class', 'node')
        .call(d3.drag()
          .on('start', dragstarted)
          .on('drag', dragged)
          .on('end', dragended));

    // Node shapes
    node.each(function(d: any) {
      const g = d3.select(this);

      switch (d.style.shape) {
        case 'Circle':
          g.append('circle')
            .attr('r', 40)
            .attr('fill', d.style.color);
          break;
        case 'Hexagon':
          g.append('polygon')
            .attr('points', '0,-50 43.3,-25 43.3,25 0,50 -43.3,25 -43.3,-25')
            .attr('fill', d.style.color);
          break;
        case 'Rectangle':
          g.append('rect')
            .attr('x', -50)
            .attr('y', -30)
            .attr('width', 100)
            .attr('height', 60)
            .attr('rx', 5)
            .attr('fill', d.style.color);
          break;
        default:
          g.append('ellipse')
            .attr('rx', 60)
            .attr('ry', 40)
            .attr('fill', d.style.color);
      }
    });

    // Node labels
    node.append('text')
      .attr('dy', 5)
      .attr('text-anchor', 'middle')
      .attr('fill', 'white')
      .attr('font-size', '12px')
      .text(d => d.label);

    // Update positions on tick
    simulation.on('tick', () => {
      link
        .attr('x1', (d: any) => d.source.x)
        .attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x)
        .attr('y2', (d: any) => d.target.y);

      node.attr('transform', (d: any) => `translate(${d.x},${d.y})`);
    });

    // Pan and zoom
    const zoom = d3.zoom()
      .scaleExtent([0.1, 5])
      .on('zoom', (event) => {
        g.attr('transform', event.transform);
      });

    svg.call(zoom as any);

    // Drag functions
    function dragstarted(event: any) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      event.subject.fx = event.subject.x;
      event.subject.fy = event.subject.y;
    }

    function dragged(event: any) {
      event.subject.fx = event.x;
      event.subject.fy = event.y;
    }

    function dragended(event: any) {
      if (!event.active) simulation.alphaTarget(0);
      event.subject.fx = null;
      event.subject.fy = null;
    }

  }, [graph]);

  return (
    <div className="workflow-d3-diagram">
      <svg ref={svgRef} />
    </div>
  );
};
```

## Incremental AI Chat Assistant for Workflow Refinement

### Overview

An intelligent conversational interface that helps users iteratively refine workflows through natural language. The AI assistant understands workflow structure, suggests improvements, and applies changes incrementally while maintaining workflow validity.

### Architecture

```
┌─────────────────────────────────────────────────────┐
│              Chat Interface (Frontend)               │
│  ┌─────────────────────────────────────────────┐   │
│  │  User: "Add error handling to task2"         │   │
│  │  AI: "I'll add a retry policy..."           │   │
│  │  [Apply] [Preview] [Explain]                 │   │
│  └─────────────────────────────────────────────┘   │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│        AI Workflow Assistant Service                 │
│  ┌──────────────────────────────────────────────┐  │
│  │  1. Intent Classification                     │  │
│  │  2. Workflow Context Analysis                 │  │
│  │  3. Change Generation                         │  │
│  │  4. Validation & Preview                      │  │
│  │  5. Application & Versioning                  │  │
│  └──────────────────────────────────────────────┘  │
└────────────────────┬────────────────────────────────┘
                     │
          ┌──────────┴──────────┬──────────────┐
          ▼                     ▼              ▼
┌──────────────────┐  ┌─────────────────┐  ┌──────────────┐
│   LLM Service    │  │ Workflow Engine │  │ Version Store│
│ (Claude/GPT)     │  │   (Validator)   │  │  (Git-like)  │
└──────────────────┘  └─────────────────┘  └──────────────┘
```

### Backend Implementation

#### 1. Conversation Context Management

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Conversation session for workflow refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConversation {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub user_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    /// Conversation messages
    pub messages: Vec<ConversationMessage>,

    /// Current workflow state
    pub current_workflow: Workflow,

    /// Workflow version history
    pub versions: Vec<WorkflowVersion>,

    /// Pending changes (not yet applied)
    pub pending_changes: Vec<WorkflowChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,

    /// Associated workflow changes
    pub changes: Vec<WorkflowChange>,

    /// Metadata (intent, confidence, etc.)
    pub metadata: MessageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub intent: ConversationIntent,
    pub confidence: f64,
    pub affected_entities: Vec<String>,  // Task IDs, agent IDs
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationIntent {
    AddTask,
    RemoveTask,
    ModifyTask,
    AddAgent,
    ModifyAgent,
    AddDependency,
    RemoveDependency,
    AddInput,
    AddOutput,
    AddErrorHandling,
    OptimizePerformance,
    AddParallelism,
    AddConditional,
    AddLoop,
    Explain,
    Validate,
    Preview,
    Undo,
    Redo,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVersion {
    pub version: u32,
    pub workflow: Workflow,
    pub created_at: DateTime<Utc>,
    pub created_by: MessageRole,
    pub description: String,
    pub changes: Vec<WorkflowChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowChange {
    pub id: Uuid,
    pub change_type: ChangeType,
    pub description: String,
    pub diff: WorkflowDiff,
    pub validated: bool,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    TaskAdded,
    TaskRemoved,
    TaskModified,
    AgentAdded,
    AgentModified,
    DependencyAdded,
    DependencyRemoved,
    InputAdded,
    OutputAdded,
    ConfigChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDiff {
    pub path: String,  // JSON path to changed field
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
}
```

#### 2. AI Assistant Service

```rust
use periplon_sdk::{PeriplonSDKClient, Message};

/// AI-powered workflow assistant
pub struct WorkflowAssistant {
    claude_client: PeriplonSDKClient,
    workflow_validator: WorkflowValidator,
    version_store: VersionStore,
}

impl WorkflowAssistant {
    pub fn new() -> Result<Self> {
        Ok(Self {
            claude_client: PeriplonSDKClient::new()?,
            workflow_validator: WorkflowValidator::new(),
            version_store: VersionStore::new(),
        })
    }

    /// Process user message and generate response with changes
    pub async fn process_message(
        &self,
        conversation: &mut WorkflowConversation,
        user_message: String,
    ) -> Result<AssistantResponse> {
        // 1. Add user message to conversation
        conversation.messages.push(ConversationMessage {
            id: Uuid::new_v4(),
            role: MessageRole::User,
            content: user_message.clone(),
            timestamp: Utc::now(),
            changes: vec![],
            metadata: MessageMetadata {
                intent: ConversationIntent::Unknown,
                confidence: 0.0,
                affected_entities: vec![],
                suggestions: vec![],
            },
        });

        // 2. Build context for AI
        let context = self.build_context(conversation)?;

        // 3. Generate AI response with changes
        let ai_response = self.generate_ai_response(&context, &user_message).await?;

        // 4. Parse response and extract changes
        let changes = self.extract_changes(&ai_response, &conversation.current_workflow)?;

        // 5. Validate changes
        let validated_changes = self.validate_changes(&conversation.current_workflow, &changes)?;

        // 6. Add assistant message
        conversation.messages.push(ConversationMessage {
            id: Uuid::new_v4(),
            role: MessageRole::Assistant,
            content: ai_response.text.clone(),
            timestamp: Utc::now(),
            changes: validated_changes.clone(),
            metadata: ai_response.metadata,
        });

        conversation.pending_changes.extend(validated_changes.clone());

        Ok(AssistantResponse {
            message: ai_response.text,
            changes: validated_changes,
            preview: self.generate_preview(&conversation.current_workflow, &validated_changes)?,
        })
    }

    /// Apply pending changes to workflow
    pub async fn apply_changes(
        &self,
        conversation: &mut WorkflowConversation,
    ) -> Result<Workflow> {
        // Create new version
        let mut new_workflow = conversation.current_workflow.clone();

        for change in &conversation.pending_changes {
            self.apply_change(&mut new_workflow, change)?;
        }

        // Validate complete workflow
        self.workflow_validator.validate(&new_workflow)?;

        // Save version
        conversation.versions.push(WorkflowVersion {
            version: conversation.versions.len() as u32 + 1,
            workflow: new_workflow.clone(),
            created_at: Utc::now(),
            created_by: MessageRole::Assistant,
            description: format!("Applied {} changes", conversation.pending_changes.len()),
            changes: conversation.pending_changes.drain(..).collect(),
        });

        conversation.current_workflow = new_workflow.clone();
        conversation.updated_at = Utc::now();

        Ok(new_workflow)
    }

    fn build_context(&self, conversation: &WorkflowConversation) -> Result<String> {
        let workflow_yaml = serde_yaml::to_string(&conversation.current_workflow)?;

        let mut context = format!(
            r#"You are a workflow assistant helping to refine a DSL workflow.

Current Workflow:
```yaml
{}
```

Conversation History:
"#,
            workflow_yaml
        );

        // Add recent messages for context
        for msg in conversation.messages.iter().rev().take(10).rev() {
            context.push_str(&format!("{}: {}\n",
                if matches!(msg.role, MessageRole::User) { "User" } else { "Assistant" },
                msg.content
            ));
        }

        context.push_str("\nInstructions:\n");
        context.push_str("- Understand the user's request\n");
        context.push_str("- Suggest specific YAML changes\n");
        context.push_str("- Explain the changes clearly\n");
        context.push_str("- Ensure changes maintain workflow validity\n");
        context.push_str("- Format changes as JSON patches\n");

        Ok(context)
    }

    async fn generate_ai_response(
        &self,
        context: &str,
        user_message: &str,
    ) -> Result<AIResponse> {
        let prompt = format!("{}\n\nUser Request: {}", context, user_message);

        let mut stream = self.claude_client.query(&prompt, None).await?;
        let mut response_text = String::new();

        while let Some(msg) = stream.next().await {
            if let Message::Assistant { content, .. } = msg? {
                for block in content {
                    if let ContentBlock::Text { text } = block {
                        response_text.push_str(&text);
                    }
                }
            }
        }

        // Extract intent and metadata
        let (intent, confidence) = self.classify_intent(&response_text);

        Ok(AIResponse {
            text: response_text,
            metadata: MessageMetadata {
                intent,
                confidence,
                affected_entities: vec![],
                suggestions: vec![],
            },
        })
    }

    fn classify_intent(&self, text: &str) -> (ConversationIntent, f64) {
        // Simple keyword-based classification
        // In production, use proper NLP/ML model

        let lower = text.to_lowercase();

        if lower.contains("add") && lower.contains("task") {
            (ConversationIntent::AddTask, 0.85)
        } else if lower.contains("remove") || lower.contains("delete") {
            (ConversationIntent::RemoveTask, 0.80)
        } else if lower.contains("error") || lower.contains("retry") {
            (ConversationIntent::AddErrorHandling, 0.90)
        } else if lower.contains("parallel") {
            (ConversationIntent::AddParallelism, 0.85)
        } else {
            (ConversationIntent::Unknown, 0.50)
        }
    }

    fn extract_changes(
        &self,
        ai_response: &AIResponse,
        current_workflow: &Workflow,
    ) -> Result<Vec<WorkflowChange>> {
        let mut changes = Vec::new();

        // Parse AI response for structured changes
        // Look for YAML blocks or JSON patches

        // Example: Extract YAML modifications
        if let Some(yaml_block) = self.extract_yaml_block(&ai_response.text) {
            let proposed: serde_json::Value = serde_yaml::from_str(&yaml_block)?;
            let current: serde_json::Value = serde_json::to_value(current_workflow)?;

            // Compute diff
            let diffs = self.compute_diff(&current, &proposed)?;

            for diff in diffs {
                changes.push(WorkflowChange {
                    id: Uuid::new_v4(),
                    change_type: self.infer_change_type(&diff),
                    description: format!("Update {} to {:?}", diff.path, diff.new_value),
                    diff,
                    validated: false,
                    applied: false,
                });
            }
        }

        Ok(changes)
    }

    fn validate_changes(
        &self,
        workflow: &Workflow,
        changes: &[WorkflowChange],
    ) -> Result<Vec<WorkflowChange>> {
        let mut validated = Vec::new();
        let mut test_workflow = workflow.clone();

        for change in changes {
            // Try applying change
            self.apply_change(&mut test_workflow, change)?;

            // Validate
            match self.workflow_validator.validate(&test_workflow) {
                Ok(_) => {
                    let mut validated_change = change.clone();
                    validated_change.validated = true;
                    validated.push(validated_change);
                }
                Err(e) => {
                    eprintln!("Change validation failed: {}", e);
                    // Skip invalid change
                }
            }
        }

        Ok(validated)
    }

    fn apply_change(&self, workflow: &mut Workflow, change: &WorkflowChange) -> Result<()> {
        // Apply JSON patch to workflow
        let mut workflow_value = serde_json::to_value(&workflow)?;

        // Parse path and apply change
        let path_parts: Vec<&str> = change.diff.path.split('/').collect();

        // Navigate to parent
        let mut current = &mut workflow_value;
        for part in &path_parts[..path_parts.len() - 1] {
            current = &mut current[part];
        }

        // Apply change
        let last_part = path_parts.last().unwrap();
        current[last_part] = change.diff.new_value.clone();

        // Deserialize back
        *workflow = serde_json::from_value(workflow_value)?;

        Ok(())
    }

    fn generate_preview(
        &self,
        workflow: &Workflow,
        changes: &[WorkflowChange],
    ) -> Result<WorkflowPreview> {
        let mut preview_workflow = workflow.clone();

        for change in changes {
            self.apply_change(&mut preview_workflow, change)?;
        }

        Ok(WorkflowPreview {
            before: serde_yaml::to_string(workflow)?,
            after: serde_yaml::to_string(&preview_workflow)?,
            diff: self.generate_diff_text(workflow, &preview_workflow)?,
        })
    }

    fn generate_diff_text(&self, before: &Workflow, after: &Workflow) -> Result<String> {
        let before_yaml = serde_yaml::to_string(before)?;
        let after_yaml = serde_yaml::to_string(after)?;

        // Use diff algorithm (e.g., Myers diff)
        // For simplicity, using line-by-line comparison
        let mut diff = String::new();

        let before_lines: Vec<&str> = before_yaml.lines().collect();
        let after_lines: Vec<&str> = after_yaml.lines().collect();

        for (i, (before_line, after_line)) in before_lines.iter().zip(&after_lines).enumerate() {
            if before_line != after_line {
                diff.push_str(&format!("  {}: - {}\n", i + 1, before_line));
                diff.push_str(&format!("  {}: + {}\n", i + 1, after_line));
            }
        }

        Ok(diff)
    }

    fn extract_yaml_block(&self, text: &str) -> Option<String> {
        // Extract YAML from markdown code block
        let re = regex::Regex::new(r"```yaml\n(.*?)\n```").ok()?;
        re.captures(text)?.get(1).map(|m| m.as_str().to_string())
    }

    fn compute_diff(
        &self,
        old: &serde_json::Value,
        new: &serde_json::Value,
    ) -> Result<Vec<WorkflowDiff>> {
        let mut diffs = Vec::new();
        self.compute_diff_recursive("", old, new, &mut diffs);
        Ok(diffs)
    }

    fn compute_diff_recursive(
        &self,
        path: &str,
        old: &serde_json::Value,
        new: &serde_json::Value,
        diffs: &mut Vec<WorkflowDiff>,
    ) {
        match (old, new) {
            (serde_json::Value::Object(old_obj), serde_json::Value::Object(new_obj)) => {
                for (key, new_val) in new_obj {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}/{}", path, key)
                    };

                    if let Some(old_val) = old_obj.get(key) {
                        if old_val != new_val {
                            self.compute_diff_recursive(&new_path, old_val, new_val, diffs);
                        }
                    } else {
                        // New field
                        diffs.push(WorkflowDiff {
                            path: new_path,
                            old_value: None,
                            new_value: new_val.clone(),
                        });
                    }
                }
            }
            _ => {
                if old != new {
                    diffs.push(WorkflowDiff {
                        path: path.to_string(),
                        old_value: Some(old.clone()),
                        new_value: new.clone(),
                    });
                }
            }
        }
    }

    fn infer_change_type(&self, diff: &WorkflowDiff) -> ChangeType {
        if diff.path.contains("tasks") {
            if diff.old_value.is_none() {
                ChangeType::TaskAdded
            } else {
                ChangeType::TaskModified
            }
        } else if diff.path.contains("agents") {
            ChangeType::AgentModified
        } else if diff.path.contains("depends_on") {
            ChangeType::DependencyAdded
        } else {
            ChangeType::ConfigChanged
        }
    }
}

#[derive(Debug)]
struct AIResponse {
    text: String,
    metadata: MessageMetadata,
}

#[derive(Debug, Serialize)]
pub struct AssistantResponse {
    pub message: String,
    pub changes: Vec<WorkflowChange>,
    pub preview: WorkflowPreview,
}

#[derive(Debug, Serialize)]
pub struct WorkflowPreview {
    pub before: String,
    pub after: String,
    pub diff: String,
}
```

#### 3. API Endpoints

```rust
// REST API for workflow conversations
pub fn conversation_routes() -> Router {
    Router::new()
        .route("/api/v1/workflows/:id/conversations", post(start_conversation))
        .route("/api/v1/workflows/:id/conversations/:conv_id", get(get_conversation))
        .route("/api/v1/workflows/:id/conversations/:conv_id/messages", post(send_message))
        .route("/api/v1/workflows/:id/conversations/:conv_id/apply", post(apply_changes))
        .route("/api/v1/workflows/:id/conversations/:conv_id/undo", post(undo_changes))
}

/// POST /api/v1/workflows/:id/conversations
async fn start_conversation(
    workflow_id: Path<Uuid>,
    user: CurrentUser,
) -> Result<Json<WorkflowConversation>> {
    let workflow = load_workflow(*workflow_id).await?;

    let conversation = WorkflowConversation {
        id: Uuid::new_v4(),
        workflow_id: *workflow_id,
        user_id: user.id,
        started_at: Utc::now(),
        updated_at: Utc::now(),
        messages: vec![],
        current_workflow: workflow.clone(),
        versions: vec![WorkflowVersion {
            version: 0,
            workflow,
            created_at: Utc::now(),
            created_by: MessageRole::System,
            description: "Initial version".to_string(),
            changes: vec![],
        }],
        pending_changes: vec![],
    };

    save_conversation(&conversation).await?;

    Ok(Json(conversation))
}

/// POST /api/v1/workflows/:id/conversations/:conv_id/messages
async fn send_message(
    Path((workflow_id, conv_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<MessagePayload>,
) -> Result<Json<AssistantResponse>> {
    let mut conversation = load_conversation(conv_id).await?;
    let assistant = WorkflowAssistant::new()?;

    let response = assistant.process_message(&mut conversation, payload.message).await?;

    save_conversation(&conversation).await?;

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
struct MessagePayload {
    message: String,
}

/// POST /api/v1/workflows/:id/conversations/:conv_id/apply
async fn apply_changes(
    Path((workflow_id, conv_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Workflow>> {
    let mut conversation = load_conversation(conv_id).await?;
    let assistant = WorkflowAssistant::new()?;

    let updated_workflow = assistant.apply_changes(&mut conversation).await?;

    // Save both conversation and updated workflow
    save_conversation(&conversation).await?;
    save_workflow(&updated_workflow).await?;

    Ok(Json(updated_workflow))
}

/// POST /api/v1/workflows/:id/conversations/:conv_id/undo
async fn undo_changes(
    Path((workflow_id, conv_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Workflow>> {
    let mut conversation = load_conversation(conv_id).await?;

    // Revert to previous version
    if conversation.versions.len() > 1 {
        conversation.versions.pop();
        let previous_version = conversation.versions.last().unwrap();
        conversation.current_workflow = previous_version.workflow.clone();
        conversation.pending_changes.clear();
    }

    save_conversation(&conversation).await?;

    Ok(Json(conversation.current_workflow))
}
```

### Frontend Implementation

```typescript
// WorkflowChatAssistant.tsx
import React, { useState, useEffect, useRef } from 'react';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { WorkflowConversation, AssistantResponse } from './types';

interface WorkflowChatAssistantProps {
  workflowId: string;
  onWorkflowUpdate?: (workflow: any) => void;
}

export const WorkflowChatAssistant: React.FC<WorkflowChatAssistantProps> = ({
  workflowId,
  onWorkflowUpdate,
}) => {
  const [conversation, setConversation] = useState<WorkflowConversation | null>(null);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [pendingResponse, setPendingResponse] = useState<AssistantResponse | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    startConversation();
  }, [workflowId]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [conversation?.messages]);

  const startConversation = async () => {
    const response = await fetch(`/api/v1/workflows/${workflowId}/conversations`, {
      method: 'POST',
    });
    const data = await response.json();
    setConversation(data);
  };

  const sendMessage = async () => {
    if (!input.trim() || !conversation) return;

    setLoading(true);

    try {
      const response = await fetch(
        `/api/v1/workflows/${workflowId}/conversations/${conversation.id}/messages`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ message: input }),
        }
      );

      const data: AssistantResponse = await response.json();

      // Update conversation with new messages
      const updatedConversation = await fetch(
        `/api/v1/workflows/${workflowId}/conversations/${conversation.id}`
      ).then(r => r.json());

      setConversation(updatedConversation);
      setPendingResponse(data);
      setInput('');
    } catch (error) {
      console.error('Failed to send message:', error);
    } finally {
      setLoading(false);
    }
  };

  const applyChanges = async () => {
    if (!conversation) return;

    try {
      const response = await fetch(
        `/api/v1/workflows/${workflowId}/conversations/${conversation.id}/apply`,
        { method: 'POST' }
      );

      const updatedWorkflow = await response.json();

      // Reload conversation
      const updatedConversation = await fetch(
        `/api/v1/workflows/${workflowId}/conversations/${conversation.id}`
      ).then(r => r.json());

      setConversation(updatedConversation);
      setPendingResponse(null);

      if (onWorkflowUpdate) {
        onWorkflowUpdate(updatedWorkflow);
      }
    } catch (error) {
      console.error('Failed to apply changes:', error);
    }
  };

  const undoChanges = async () => {
    if (!conversation) return;

    try {
      const response = await fetch(
        `/api/v1/workflows/${workflowId}/conversations/${conversation.id}/undo`,
        { method: 'POST' }
      );

      const revertedWorkflow = await response.json();

      const updatedConversation = await fetch(
        `/api/v1/workflows/${workflowId}/conversations/${conversation.id}`
      ).then(r => r.json());

      setConversation(updatedConversation);

      if (onWorkflowUpdate) {
        onWorkflowUpdate(revertedWorkflow);
      }
    } catch (error) {
      console.error('Failed to undo changes:', error);
    }
  };

  if (!conversation) return <div>Loading...</div>;

  return (
    <div className="workflow-chat-assistant">
      <div className="chat-header">
        <h3>Workflow Assistant</h3>
        <div className="version-info">
          Version {conversation.versions.length - 1}
          {conversation.pending_changes.length > 0 && (
            <span className="pending-badge">
              {conversation.pending_changes.length} pending
            </span>
          )}
        </div>
      </div>

      <div className="chat-messages">
        {conversation.messages.map((message, idx) => (
          <div
            key={idx}
            className={`message ${message.role.toLowerCase()}`}
          >
            <div className="message-header">
              <span className="role">{message.role}</span>
              <span className="timestamp">
                {new Date(message.timestamp).toLocaleTimeString()}
              </span>
            </div>

            <div className="message-content">
              <ReactMarkdown
                components={{
                  code({ node, inline, className, children, ...props }) {
                    const match = /language-(\w+)/.exec(className || '');
                    return !inline && match ? (
                      <SyntaxHighlighter
                        language={match[1]}
                        PreTag="div"
                        {...props}
                      >
                        {String(children).replace(/\n$/, '')}
                      </SyntaxHighlighter>
                    ) : (
                      <code className={className} {...props}>
                        {children}
                      </code>
                    );
                  },
                }}
              >
                {message.content}
              </ReactMarkdown>
            </div>

            {message.changes.length > 0 && (
              <div className="message-changes">
                <h4>Proposed Changes:</h4>
                <ul>
                  {message.changes.map((change, i) => (
                    <li key={i} className={change.validated ? 'valid' : 'invalid'}>
                      {change.description}
                      {change.validated && <span className="badge">✓</span>}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        ))}

        {loading && (
          <div className="message assistant loading">
            <div className="typing-indicator">
              <span></span>
              <span></span>
              <span></span>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {pendingResponse && (
        <div className="pending-preview">
          <h4>Preview Changes</h4>
          <div className="diff-viewer">
            <SyntaxHighlighter language="diff">
              {pendingResponse.preview.diff}
            </SyntaxHighlighter>
          </div>
          <div className="preview-actions">
            <button onClick={applyChanges} className="btn-primary">
              Apply Changes
            </button>
            <button onClick={() => setPendingResponse(null)} className="btn-secondary">
              Dismiss
            </button>
          </div>
        </div>
      )}

      <div className="chat-input">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
          placeholder="Describe the changes you want..."
          disabled={loading}
        />
        <button onClick={sendMessage} disabled={loading || !input.trim()}>
          Send
        </button>
        {conversation.versions.length > 1 && (
          <button onClick={undoChanges} className="btn-undo">
            Undo
          </button>
        )}
      </div>

      <div className="chat-suggestions">
        <p>Try asking:</p>
        <button onClick={() => setInput('Add error handling with retry logic to task2')}>
          Add error handling
        </button>
        <button onClick={() => setInput('Make task1 and task2 run in parallel')}>
          Add parallelism
        </button>
        <button onClick={() => setInput('Add a checkpoint after task3')}>
          Add checkpoint
        </button>
      </div>
    </div>
  );
};
```

## Workflow Assisted Editor with Syntax Completion and Coloring

### Overview

An intelligent YAML editor with real-time syntax highlighting, auto-completion, validation, and inline suggestions powered by Language Server Protocol (LSP) and Monaco Editor.

### Architecture

```
┌─────────────────────────────────────────────────────┐
│           Monaco Editor (Frontend)                   │
│  ┌─────────────────────────────────────────────┐   │
│  │  • Syntax Highlighting                       │   │
│  │  • Auto-completion                           │   │
│  │  • Error Squiggles                           │   │
│  │  • Inline Suggestions                        │   │
│  │  • Code Actions (Quick Fixes)                │   │
│  └─────────────────────────────────────────────┘   │
└────────────────────┬────────────────────────────────┘
                     │ LSP Protocol
                     ▼
┌─────────────────────────────────────────────────────┐
│        Workflow LSP Server (Backend)                 │
│  ┌──────────────────────────────────────────────┐  │
│  │  • Parsing & Validation                       │  │
│  │  • Completion Provider                        │  │
│  │  │  - Task IDs                                 │  │
│  │  │  - Agent References                         │  │
│  │  │  - Tool Names                               │  │
│  │  │  - Permission Modes                         │  │
│  │  • Hover Provider (Documentation)             │  │
│  │  • Diagnostics (Errors/Warnings)              │  │
│  │  • Code Actions (Quick Fixes)                 │  │
│  └──────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### Backend: LSP Server Implementation

```rust
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// Workflow DSL Language Server
pub struct WorkflowLspServer {
    client: Client,
    document_map: Arc<RwLock<HashMap<Url, DocumentState>>>,
    validator: WorkflowValidator,
}

struct DocumentState {
    text: String,
    workflow: Option<Workflow>,
    diagnostics: Vec<Diagnostic>,
}

#[tower_lsp::async_trait]
impl LanguageServer for WorkflowLspServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Workflow DSL Language Server".to_string(),
                version: Some("1.0.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec![":".to_string(), " ".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        ..Default::default()
                    },
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Workflow LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        self.process_document(uri.clone(), text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.content_changes[0].text.clone();

        self.process_document(uri, text).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let completions = self.get_completions(&uri, position).await?;

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let hover_content = self.get_hover_content(&uri, position).await?;

        Ok(hover_content)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;

        let actions = self.get_code_actions(&uri, params.range).await?;

        Ok(Some(actions))
    }
}

impl WorkflowLspServer {
    async fn process_document(&self, uri: Url, text: String) {
        // Parse workflow
        let workflow_result = serde_yaml::from_str::<Workflow>(&text);

        let mut diagnostics = Vec::new();

        let workflow = match workflow_result {
            Ok(wf) => {
                // Validate workflow
                if let Err(e) = self.validator.validate(&wf) {
                    diagnostics.push(Diagnostic {
                        range: Range::default(),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        source: Some("workflow-validator".to_string()),
                        message: format!("Validation error: {}", e),
                        ..Default::default()
                    });
                }
                Some(wf)
            }
            Err(e) => {
                diagnostics.push(Diagnostic {
                    range: Range::default(),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    source: Some("yaml-parser".to_string()),
                    message: format!("Parse error: {}", e),
                    ..Default::default()
                });
                None
            }
        };

        // Store document state
        {
            let mut map = self.document_map.write().await;
            map.insert(
                uri.clone(),
                DocumentState {
                    text,
                    workflow,
                    diagnostics: diagnostics.clone(),
                },
            );
        }

        // Publish diagnostics
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn get_completions(
        &self,
        uri: &Url,
        position: Position,
    ) -> Result<Vec<CompletionItem>> {
        let map = self.document_map.read().await;
        let doc = map.get(uri).ok_or_else(|| {
            tower_lsp::jsonrpc::Error::invalid_params("Document not found")
        })?;

        let mut completions = Vec::new();

        // Get current line context
        let lines: Vec<&str> = doc.text.lines().collect();
        if let Some(line) = lines.get(position.line as usize) {
            let line_up_to_cursor = &line[..position.character.min(line.len() as u32) as usize];

            // Agent completion
            if line_up_to_cursor.contains("agent:") {
                if let Some(workflow) = &doc.workflow {
                    for agent_id in workflow.agents.keys() {
                        completions.push(CompletionItem {
                            label: agent_id.clone(),
                            kind: Some(CompletionItemKind::REFERENCE),
                            detail: Some("Agent".to_string()),
                            documentation: workflow.agents.get(agent_id).map(|a| {
                                Documentation::String(a.description.clone())
                            }),
                            ..Default::default()
                        });
                    }
                }
            }

            // Tool completion
            if line_up_to_cursor.contains("tools:") || line_up_to_cursor.trim().starts_with("- ") {
                let tools = vec![
                    "Read", "Write", "Edit", "Bash", "WebSearch", "WebFetch",
                    "Glob", "Grep", "NotebookEdit",
                ];

                for tool in tools {
                    completions.push(CompletionItem {
                        label: tool.to_string(),
                        kind: Some(CompletionItemKind::ENUM),
                        detail: Some("Tool".to_string()),
                        ..Default::default()
                    });
                }
            }

            // Permission mode completion
            if line_up_to_cursor.contains("mode:") {
                let modes = vec!["default", "acceptEdits", "plan", "bypassPermissions"];

                for mode in modes {
                    completions.push(CompletionItem {
                        label: mode.to_string(),
                        kind: Some(CompletionItemKind::ENUM),
                        detail: Some("Permission Mode".to_string()),
                        ..Default::default()
                    });
                }
            }

            // Task ID completion in depends_on
            if line_up_to_cursor.contains("depends_on:") ||
               (line_up_to_cursor.trim().starts_with("- ") && lines.iter().rev().skip(1).any(|l| l.contains("depends_on:"))) {
                if let Some(workflow) = &doc.workflow {
                    for task_id in workflow.tasks.keys() {
                        completions.push(CompletionItem {
                            label: task_id.clone(),
                            kind: Some(CompletionItemKind::REFERENCE),
                            detail: Some("Task ID".to_string()),
                            documentation: workflow.tasks.get(task_id).map(|t| {
                                Documentation::String(t.description.clone())
                            }),
                            ..Default::default()
                        });
                    }
                }
            }

            // Schema-based completions
            if line_up_to_cursor.trim().is_empty() || line_up_to_cursor.ends_with(':') {
                completions.extend(self.get_schema_completions(line_up_to_cursor));
            }
        }

        Ok(completions)
    }

    fn get_schema_completions(&self, context: &str) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Top-level keys
        if context.is_empty() || !context.contains(':') {
            completions.extend(vec![
                CompletionItem {
                    label: "name".to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    insert_text: Some("name: \"\"".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                },
                CompletionItem {
                    label: "version".to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    insert_text: Some("version: \"1.0.0\"".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "agents".to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    insert_text: Some("agents:\n  $1:".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                },
                CompletionItem {
                    label: "tasks".to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    insert_text: Some("tasks:\n  $1:".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                },
            ]);
        }

        // Agent properties
        if context.contains("agents:") {
            completions.extend(vec![
                CompletionItem {
                    label: "description".to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    insert_text: Some("description: \"$1\"".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                },
                CompletionItem {
                    label: "model".to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    insert_text: Some("model: \"claude-sonnet-4-5\"".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "tools".to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    insert_text: Some("tools:\n  - $1".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                },
            ]);
        }

        completions
    }

    async fn get_hover_content(&self, uri: &Url, position: Position) -> Result<Option<Hover>> {
        let map = self.document_map.read().await;
        let doc = map.get(uri).ok_or_else(|| {
            tower_lsp::jsonrpc::Error::invalid_params("Document not found")
        })?;

        let lines: Vec<&str> = doc.text.lines().collect();
        if let Some(line) = lines.get(position.line as usize) {
            // Detect hovered entity
            if line.contains("agent:") {
                if let Some(workflow) = &doc.workflow {
                    // Extract agent ID from line
                    if let Some(agent_id) = self.extract_identifier(line, "agent:") {
                        if let Some(agent) = workflow.agents.get(&agent_id) {
                            let hover_text = format!(
                                "**Agent**: {}\n\n{}\n\n**Model**: {}\n\n**Tools**: {}",
                                agent_id,
                                agent.description,
                                agent.model.as_ref().unwrap_or(&"default".to_string()),
                                agent.tools.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>().join(", ")
                            );

                            return Ok(Some(Hover {
                                contents: HoverContents::Markup(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: hover_text,
                                }),
                                range: None,
                            }));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn get_code_actions(
        &self,
        uri: &Url,
        _range: Range,
    ) -> Result<Vec<CodeActionOrCommand>> {
        let map = self.document_map.read().await;
        let doc = map.get(uri).ok_or_else(|| {
            tower_lsp::jsonrpc::Error::invalid_params("Document not found")
        })?;

        let mut actions = Vec::new();

        // Suggest fixes for validation errors
        for diagnostic in &doc.diagnostics {
            if diagnostic.message.contains("missing field") {
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Add missing field".to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::new()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }
        }

        Ok(actions)
    }

    fn extract_identifier(&self, line: &str, prefix: &str) -> Option<String> {
        line.find(prefix)
            .and_then(|pos| {
                let rest = &line[pos + prefix.len()..];
                rest.trim().split_whitespace().next().map(|s| s.to_string())
            })
    }
}
```

### Frontend: Monaco Editor Integration

```typescript
// WorkflowEditor.tsx
import React, { useEffect, useRef, useState } from 'react';
import * as monaco from 'monaco-editor';
import { MonacoLanguageClient, CloseAction, ErrorAction, MonacoServices } from 'monaco-languageclient';
import { listen, MessageConnection } from 'vscode-ws-jsonrpc';

interface WorkflowEditorProps {
  workflowId: string;
  initialValue?: string;
  onChange?: (value: string) => void;
  onSave?: (value: string) => void;
}

export const WorkflowEditor: React.FC<WorkflowEditorProps> = ({
  workflowId,
  initialValue = '',
  onChange,
  onSave,
}) => {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [lspClient, setLspClient] = useState<MonacoLanguageClient | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    // Register YAML language
    monaco.languages.register({ id: 'workflow-yaml' });

    // Set language configuration
    monaco.languages.setLanguageConfiguration('workflow-yaml', {
      comments: {
        lineComment: '#',
      },
      brackets: [
        ['{', '}'],
        ['[', ']'],
      ],
      autoClosingPairs: [
        { open: '{', close: '}' },
        { open: '[', close: ']' },
        { open: '"', close: '"' },
        { open: "'", close: "'" },
      ],
      surroundingPairs: [
        { open: '{', close: '}' },
        { open: '[', close: ']' },
        { open: '"', close: '"' },
        { open: "'", close: "'" },
      ],
      indentationRules: {
        increaseIndentPattern: /^.*:\s*$/,
        decreaseIndentPattern: /^\s*$/,
      },
    });

    // Set syntax highlighting
    monaco.languages.setMonarchTokensProvider('workflow-yaml', {
      tokenizer: {
        root: [
          [/^(\s*)(name|version|description|agents|tasks):/, ['white', 'type']],
          [/^(\s*)([a-zA-Z_][a-zA-Z0-9_]*):/, ['white', 'key']],
          [/:\s*([^#\n]+)/, 'string'],
          [/#.*$/, 'comment'],
          [/\d+/, 'number'],
          [/(true|false)/, 'keyword'],
          [/([a-zA-Z_][a-zA-Z0-9_]*)/, 'identifier'],
        ],
      },
    });

    // Define theme
    monaco.editor.defineTheme('workflow-theme', {
      base: 'vs-dark',
      inherit: true,
      rules: [
        { token: 'type', foreground: 'C586C0', fontStyle: 'bold' },
        { token: 'key', foreground: '9CDCFE' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'keyword', foreground: '569CD6' },
        { token: 'comment', foreground: '6A9955' },
      ],
      colors: {
        'editor.background': '#1E1E1E',
      },
    });

    // Create editor
    const editor = monaco.editor.create(containerRef.current, {
      value: initialValue,
      language: 'workflow-yaml',
      theme: 'workflow-theme',
      automaticLayout: true,
      minimap: { enabled: true },
      fontSize: 14,
      tabSize: 2,
      insertSpaces: true,
      wordWrap: 'on',
      folding: true,
      lineNumbers: 'on',
      renderWhitespace: 'boundary',
      scrollBeyondLastLine: false,
      suggestOnTriggerCharacters: true,
      quickSuggestions: {
        other: true,
        comments: false,
        strings: true,
      },
    });

    editorRef.current = editor;

    // Set up change handler
    editor.onDidChangeModelContent(() => {
      const value = editor.getValue();
      if (onChange) {
        onChange(value);
      }
    });

    // Set up save command
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      const value = editor.getValue();
      if (onSave) {
        onSave(value);
      }
    });

    // Connect to LSP server
    connectToLsp();

    return () => {
      editor.dispose();
      lspClient?.stop();
    };
  }, []);

  const connectToLsp = () => {
    // Install Monaco services
    MonacoServices.install(monaco);

    // Create WebSocket connection to LSP server
    const webSocket = new WebSocket('ws://localhost:8080/lsp');

    webSocket.onopen = () => {
      const socket: MessageConnection = listen({
        webSocket,
        onConnection: (connection) => {
          // Create language client
          const client = new MonacoLanguageClient({
            name: 'Workflow YAML Client',
            clientOptions: {
              documentSelector: [{ language: 'workflow-yaml' }],
              errorHandler: {
                error: () => ErrorAction.Continue,
                closed: () => CloseAction.DoNotRestart,
              },
            },
            connectionProvider: {
              get: (errorHandler, closeHandler) => {
                return Promise.resolve(connection);
              },
            },
          });

          client.start();
          setLspClient(client);
        },
      });
    };
  };

  return (
    <div className="workflow-editor-container">
      <div className="editor-toolbar">
        <button onClick={() => formatDocument()}>Format</button>
        <button onClick={() => validateWorkflow()}>Validate</button>
        <button onClick={() => showDiagram()}>Show Diagram</button>
        <span className="editor-status">
          Lines: {editorRef.current?.getModel()?.getLineCount() || 0}
        </span>
      </div>

      <div ref={containerRef} className="editor-content" style={{ height: '600px' }} />

      <div className="editor-footer">
        <span className="hint">Press Ctrl+Space for suggestions</span>
        <span className="hint">Press Ctrl+S to save</span>
      </div>
    </div>
  );

  function formatDocument() {
    if (!editorRef.current) return;

    editorRef.current.getAction('editor.action.formatDocument')?.run();
  }

  async function validateWorkflow() {
    if (!editorRef.current) return;

    const value = editorRef.current.getValue();

    try {
      const response = await fetch(`/api/v1/workflows/${workflowId}/validate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-yaml' },
        body: value,
      });

      if (response.ok) {
        alert('Workflow is valid!');
      } else {
        const error = await response.json();
        alert(`Validation error: ${error.message}`);
      }
    } catch (error) {
      alert(`Error: ${error}`);
    }
  }

  function showDiagram() {
    // Open diagram view
    window.open(`/workflows/${workflowId}/diagram`, '_blank');
  }
};
```

## Future Enhancements

### Phase 7+ (Beyond MVP)
- **Multi-tenancy**: Workspace isolation
- **Workflow Marketplace**: Share/import templates
- **Advanced Analytics**: ML-based insights
- **Distributed Tracing**: OpenTelemetry integration
- **Workflow Versioning**: GitOps integration
- **Plugin System**: Custom tool extensions
- **Mobile App**: iOS/Android monitoring
- **Notifications**: Slack/Email/PagerDuty
- **Cost Tracking**: Resource consumption analytics
- **Compliance**: SOC2, HIPAA certification

---

**Document Version**: 1.0
**Last Updated**: 2025-10-19
**Status**: Planning Phase

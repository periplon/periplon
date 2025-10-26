# Server Mode Implementation Summary

## Overview

This document summarizes the complete implementation of the server mode for the DSL executor, transforming it from a CLI tool into a scalable workflow orchestration platform with REST API, background job processing, and comprehensive monitoring.

## Architecture

### Hexagonal Architecture (Ports & Adapters)

The implementation follows strict hexagonal architecture principles:

- **Domain Core**: Pure business logic with zero external dependencies
- **Ports**: Abstract interfaces for primary (inbound) and secondary (outbound) interactions
- **Adapters**: Concrete implementations of ports (filesystem, PostgreSQL, Redis, etc.)

### Key Components

```
src/server/
├── config.rs              # Configuration management with TOML & env vars
├── storage/               # Pluggable storage layer
│   ├── traits.rs          # Storage abstractions (WorkflowStorage, ExecutionStorage, CheckpointStorage)
│   ├── filesystem.rs      # File-based storage backend
│   └── postgres.rs        # PostgreSQL storage backend
├── queue/                 # Pluggable queue system
│   ├── traits.rs          # Queue abstractions (WorkQueue)
│   ├── filesystem.rs      # File-based queue backend
│   └── postgres.rs        # PostgreSQL queue backend (with SELECT FOR UPDATE SKIP LOCKED)
├── worker.rs              # Background worker for async execution
├── api/                   # REST API
│   ├── routes.rs          # Route definitions
│   └── handlers/          # Request handlers
│       ├── workflows.rs   # Workflow CRUD operations
│       ├── executions.rs  # Execution management
│       ├── queue.rs       # Queue stats
│       ├── health.rs      # Health checks
│       └── monitoring.rs  # Metrics & monitoring
└── db/migrations/         # Database schema migrations
    └── 001_initial_schema.sql
```

## Completed Features

### ✅ 1. Unified Binary Architecture

- **Binary**: `periplon-executor` supports multiple modes:
  - `template` - Generate workflow templates
  - `generate` - Create workflows from natural language
  - `validate` - Validate workflow definitions
  - `run` - Execute workflows (CLI mode)
  - `server` - Start REST API server
  - `worker` - Start background worker
  - `migrate` - Run database migrations

### ✅ 2. Configuration Subsystem

**File**: `src/server/config.rs`

Features:
- TOML-based configuration
- Environment variable substitution (`${VAR_NAME}`)
- Validation on startup
- Support for multiple backends (filesystem, PostgreSQL, S3, Redis)
- Structured config for server, storage, queue, auth, rate limiting, monitoring, and reliability

Example:
```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[storage]
backend = "postgres"
postgres_url = "${DATABASE_URL}"

[queue]
backend = "postgres"
postgres_url = "${DATABASE_URL}"
```

### ✅ 3. Pluggable Storage Layer

**Traits** (`src/server/storage/traits.rs`):
- `WorkflowStorage` - Store and retrieve workflow definitions
- `ExecutionStorage` - Manage execution records and logs
- `CheckpointStorage` - Save/restore execution state
- `Storage` - Unified trait combining all storage capabilities

**Implementations**:

#### Filesystem Backend (`src/server/storage/filesystem.rs`)
- Directory-based structure
- JSON metadata + YAML workflows
- Atomic writes with fsync
- Version support

#### PostgreSQL Backend (`src/server/storage/postgres.rs`)
- Full ACID compliance
- JSONB columns for flexible schema
- Optimistic locking with version numbers
- Efficient querying with indexes
- **Runtime queries** (no DATABASE_URL required at compile time)

### ✅ 4. Pluggable Queue System

**Traits** (`src/server/queue/traits.rs`):
- `WorkQueue` - Job distribution interface
  - `enqueue` - Add jobs to queue
  - `dequeue` - Claim jobs for processing
  - `complete` - Mark job as done
  - `fail` - Mark job as failed
  - `requeue` - Retry with optional delay
  - `heartbeat` - Keep job alive
  - `release_stale_jobs` - Recover stuck jobs
  - `stats` - Queue statistics

**Implementations**:

#### Filesystem Backend (`src/server/queue/filesystem.rs`)
- File-based job queue
- Exclusive file locking
- Status directories (pending, processing, completed, failed)
- Stale job detection

#### PostgreSQL Backend (`src/server/queue/postgres.rs`)
- **SELECT FOR UPDATE SKIP LOCKED** for non-blocking concurrent dequeue
- Priority-based job ordering
- Scheduled job support
- Worker tracking
- Automatic stale job recovery
- **Runtime queries** (no DATABASE_URL required at compile time)

### ✅ 5. Background Worker System

**File**: `src/server/worker.rs`

Features:
- Async job processing
- Configurable concurrency
- Heartbeat mechanism to prevent timeouts
- Automatic retry on failure
- Graceful shutdown
- Integration with pluggable queue and storage

Worker loop:
1. Dequeue job from queue
2. Load workflow from storage
3. Execute workflow
4. Send periodic heartbeats
5. Store results or handle errors
6. Mark job as complete/failed

### ✅ 6. REST API (Axum)

**Routes** (`src/server/api/routes.rs`):

#### Health & Monitoring
- `GET /health` - Health check with version
- `GET /ready` - Readiness check (database, queue, storage)
- `GET /live` - Liveness probe
- `GET /metrics` - Prometheus-formatted metrics
- `GET /stats` - JSON statistics
- `GET /version` - Version information

#### Workflows
- `GET /api/v1/workflows` - List workflows
- `POST /api/v1/workflows` - Create workflow
- `GET /api/v1/workflows/:id` - Get workflow details
- `PUT /api/v1/workflows/:id` - Update workflow
- `DELETE /api/v1/workflows/:id` - Delete workflow
- `POST /api/v1/workflows/:id/validate` - Validate workflow

#### Executions
- `GET /api/v1/executions` - List executions
- `POST /api/v1/executions` - Start execution
- `GET /api/v1/executions/:id` - Get execution status
- `POST /api/v1/executions/:id/cancel` - Cancel execution
- `GET /api/v1/executions/:id/logs` - Get execution logs

#### Queue
- `GET /api/v1/queue/stats` - Queue statistics

### ✅ 7. Database Schema

**File**: `src/server/db/migrations/001_initial_schema.sql`

Tables:
- `organizations` - Multi-tenancy support
- `users` - User accounts
- `workflows` - Workflow definitions (JSONB)
- `executions` - Execution records
- `task_executions` - Individual task runs
- `execution_logs` - Structured logging
- `checkpoints` - State snapshots
- `execution_queue` - Job queue
- `schedules` - Recurring workflows
- `roles`, `permissions`, `role_permissions`, `user_roles` - RBAC
- `api_keys` - API authentication
- `user_sessions` - Session management
- `oauth_connections` - OAuth integration
- `user_mfa_settings` - Multi-factor auth

Features:
- Updated timestamps (via triggers)
- Foreign key constraints
- Indexes for performance
- Initial seed data (system roles/permissions)

### ✅ 8. Monitoring & Metrics

**File**: `src/server/api/handlers/monitoring.rs`

Prometheus Metrics:
- `workflow_executions_total` - Total workflow executions (counter)
- `workflow_executions_active` - Active executions (gauge)
- `workflow_executions_duration_seconds` - Execution duration (histogram)
- `queue_jobs_total` - Jobs by status (gauge)
- `http_requests_total` - HTTP request count (counter)
- `http_request_duration_seconds` - Request latency (histogram)
- `system_memory_usage_bytes` - Memory usage (gauge)
- `system_cpu_usage_percent` - CPU usage (gauge)

Endpoints:
- `/metrics` - Prometheus text format
- `/stats` - JSON statistics
- `/health` - Health with version
- `/ready` - Readiness check
- `/live` - Liveness probe

### ✅ 9. Runtime SQL Queries (No DATABASE_URL at Compile Time)

All PostgreSQL backends use runtime queries (`sqlx::query()`) instead of compile-time macros (`sqlx::query!()`):

Benefits:
- ✅ No database connection required during compilation
- ✅ Faster CI/CD builds
- ✅ Easier local development
- ✅ Same runtime performance and safety

Implementation:
- Manual parameter binding with `.bind()`
- Manual row value extraction with `row.get()` and `row.try_get()`
- Type-safe with Rust's type system

### ✅ 10. Database Migration Runner

**File**: `src/server/db/migrations.rs`

Full-featured migration system for PostgreSQL:
- Automatic migration table creation (`_migrations`)
- Load migrations from SQL files
- Version tracking
- Transactional migration application
- Rollback support
- Migration status reporting

Commands:
```bash
# Run pending migrations
DATABASE_URL=postgresql://... ./target/release/periplon-executor migrate up

# Rollback last migration
DATABASE_URL=postgresql://... ./target/release/periplon-executor migrate down

# Show migration status
DATABASE_URL=postgresql://... ./target/release/periplon-executor migrate status
```

Features:
- Filename-based versioning (`001_initial_schema.sql`)
- Atomic migrations (wrapped in transactions)
- Idempotent - can run multiple times safely
- Clear status output with colored terminal UI

### ✅ 11. JWT Authentication System

**Files**:
- `src/server/auth/jwt.rs` - JWT token management
- `src/server/auth/middleware.rs` - Axum authentication middleware
- `src/server/api/handlers/auth.rs` - Authentication endpoints

Features:

#### JWT Token Management
- HS256 algorithm (configurable)
- Customizable expiration (default: 24 hours)
- Claims include: user ID, email, roles, issued/expiration times
- Role-based claims for authorization

#### Middleware
- Extract Bearer tokens from Authorization header
- Validate JWT signatures and expiration
- Public path configuration (health, metrics, auth endpoints)
- Store claims in request extensions for handlers
- Role-based authorization helpers

#### API Endpoints
- `POST /api/v1/auth/login` - User login (returns JWT)
- `POST /api/v1/auth/register` - User registration (returns JWT)
- `GET /api/v1/auth/me` - Get current user info (requires auth)
- `POST /api/v1/auth/refresh` - Refresh JWT token (requires auth)

Example Usage:
```bash
# Register
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secret","name":"John Doe"}'

# Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secret"}'

# Use token
curl http://localhost:8080/api/v1/auth/me \
  -H "Authorization: Bearer <token>"
```

**Note**: Auth endpoints are currently stubs. Production implementation requires:
- Password hashing (argon2/bcrypt)
- Database user storage
- Email validation
- Password strength requirements

To enable authentication middleware, see the commented example in `src/server/api/routes.rs`.

## Usage

### Build

```bash
# Build with server features
cargo build --release --features server

# Build CLI only
cargo build --release
```

### Run Server

```bash
# Using filesystem backends
./target/release/periplon-executor server --port 8080

# Using PostgreSQL backends
export DATABASE_URL="postgresql://user:pass@localhost/dbname"
./target/release/periplon-executor server --port 8080 --config config.toml

# Start background workers
./target/release/periplon-executor worker --config config.toml
```

### Configuration File

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[storage]
backend = "postgres"  # or "filesystem" or "s3"
postgres_url = "${DATABASE_URL}"

[queue]
backend = "postgres"  # or "filesystem" or "redis"
postgres_url = "${DATABASE_URL}"
poll_interval_ms = 1000
max_retries = 3

[monitoring]
metrics_enabled = true
```

### API Examples

```bash
# Health check
curl http://localhost:8080/health

# Prometheus metrics
curl http://localhost:8080/metrics

# Create workflow
curl -X POST http://localhost:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -d @workflow.json

# List executions
curl http://localhost:8080/api/v1/executions

# Queue stats
curl http://localhost:8080/api/v1/queue/stats
```

### Database Migrations

```bash
# Set database URL
export DATABASE_URL="postgresql://user:pass@localhost/dbname"

# Run migrations
./target/release/periplon-executor migrate up

# Show migration status
./target/release/periplon-executor migrate status

# Rollback last migration
./target/release/periplon-executor migrate down
```

## Pending Features

### 🔄 High Priority

1. **Complete Authentication Integration**
   - Connect auth handlers to database
   - Implement password hashing (argon2)
   - Add email validation
   - Implement password reset flow
   - Enable authentication middleware in routes

2. **Authorization System (RBAC)**
   - Implement role-based middleware
   - Connect to database roles/permissions
   - Fine-grained permission checks
   - Admin panel for role management

3. **S3 Storage Backend**
   - Implement `Storage` trait for AWS S3
   - Support S3-compatible services (MinIO, DigitalOcean Spaces)

4. **Redis Queue Backend**
   - Implement `WorkQueue` trait for Redis
   - Use Redis Streams or Lists
   - Support for priority queues

### 🔄 Medium Priority

5. **WebSocket Support**
   - Real-time execution updates
   - Live log streaming
   - Progress notifications

6. **OAuth 2.0 Integration**
   - Google OAuth
   - GitHub OAuth
   - Generic OIDC provider support

7. **Multi-Factor Authentication (MFA)**
   - TOTP support
   - Backup codes
   - Recovery options

### 🔄 Future Enhancements

8. **Web UI**
   - Workflow editor (visual or YAML)
   - Execution dashboard
   - Logs viewer
   - Queue monitoring

9. **Advanced Monitoring**
   - Distributed tracing (Jaeger, Zipkin)
   - APM integration (Datadog, New Relic)
   - Log aggregation (Elasticsearch)

10. **High Availability**
    - Leader election for workers
    - Horizontal scaling
    - Load balancing

11. **Workflow Scheduling**
    - Cron-based triggers
    - Event-based triggers
    - Recurring workflows

## Testing

```bash
# Run tests
cargo test --features server

# Run tests with output
cargo test --features server -- --nocapture

# Run specific test
cargo test test_postgres_storage --features server
```

## Deployment

### Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features server

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/periplon-executor /usr/local/bin/
EXPOSE 8080
CMD ["periplon-executor", "server", "--port", "8080"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: periplon-executor-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: periplon-executor
  template:
    metadata:
      labels:
        app: periplon-executor
    spec:
      containers:
      - name: server
        image: periplon-executor:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: url
        livenessProbe:
          httpGet:
            path: /live
            port: 8080
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: periplon-executor-worker
spec:
  replicas: 5
  selector:
    matchLabels:
      app: periplon-executor-worker
  template:
    metadata:
      labels:
        app: periplon-executor-worker
    spec:
      containers:
      - name: worker
        image: periplon-executor:latest
        command: ["periplon-executor", "worker"]
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: url
```

## Performance Considerations

### Queue Optimization

The PostgreSQL queue uses `SELECT FOR UPDATE SKIP LOCKED` which provides:
- **Non-blocking**: Multiple workers can dequeue concurrently
- **No lock contention**: Workers skip locked rows instead of waiting
- **High throughput**: Optimal for distributed job processing

### Storage Optimization

- **Indexes**: Created on frequently queried columns
- **JSONB**: Fast querying with GIN indexes
- **Prepared statements**: Runtime queries are cached by PostgreSQL
- **Connection pooling**: SQLx pool management

### Monitoring

- **Prometheus metrics**: Standard monitoring integration
- **Health checks**: Kubernetes-compatible probes
- **Structured logging**: Easy log aggregation

## Security Notes

1. **Database**: Use environment variables for credentials
2. **API**: Implement authentication middleware (pending)
3. **CORS**: Configure for production deployments
4. **Rate Limiting**: Configure in `config.toml`
5. **TLS**: Use reverse proxy (nginx, Traefik) for HTTPS

## Contributing

When adding new features:
1. Follow hexagonal architecture principles
2. Add trait abstractions before implementations
3. Support multiple backends where applicable
4. Include tests
5. Update this documentation

## License

Same as parent project.

# DSL Executor CLI Usage Guide

Complete command-line interface guide for the DSL Executor workflow orchestration platform.

## Table of Contents
- [Installation](#installation)
- [CLI Mode (Local Execution)](#cli-mode-local-execution)
- [Server Mode (Production)](#server-mode-production)
- [Configuration](#configuration)
- [Examples](#examples)

## Installation

```bash
# Build from source
cargo build --release --features full

# Install globally
cargo install --path . --features full

# The binary will be available as `periplon-executor`
```

## CLI Mode (Local Execution)

### Generate Workflow Template

Generate a comprehensive workflow template with all features documented:

```bash
# Print to stdout
periplon-executor template

# Save to file
periplon-executor template -o my-workflow.yaml
```

### Generate from Natural Language

Create workflows using natural language descriptions:

```bash
# Generate new workflow
periplon-executor generate "Create a workflow that fetches data from an API and processes it" -o workflow.yaml

# Modify existing workflow
periplon-executor generate "Add error handling and retries" -w existing.yaml -o modified.yaml

# With verbose output
periplon-executor generate "Multi-step data pipeline with validation" -o pipeline.yaml --verbose
```

### Validate Workflow

Validate workflow files without executing:

```bash
# Basic validation
periplon-executor validate workflow.yaml

# Verbose validation (shows agents and tasks)
periplon-executor validate workflow.yaml --verbose

# JSON output
periplon-executor validate workflow.yaml --json
```

### Run Workflow

Execute workflows locally:

```bash
# Basic execution
periplon-executor run workflow.yaml

# With verbose output
periplon-executor run workflow.yaml --verbose

# Dry run (validate only, don't execute)
periplon-executor run workflow.yaml --dry-run

# Resume from saved state
periplon-executor run workflow.yaml --resume

# Clean state before execution
periplon-executor run workflow.yaml --clean

# Custom state directory
periplon-executor run workflow.yaml --state-dir ./my-states

# With input variables
periplon-executor run workflow.yaml -i name=John -i age=30 -i config='{"key":"value"}'

# JSON output
periplon-executor run workflow.yaml --json
```

### Manage Workflow States

```bash
# List saved workflow states
periplon-executor list

# List with custom state directory
periplon-executor list --state-dir ./my-states

# JSON output
periplon-executor list --json

# Show workflow status
periplon-executor status my-workflow

# Clean saved states
periplon-executor clean my-workflow

# Clean all states (with confirmation)
periplon-executor clean

# Skip confirmation
periplon-executor clean --yes
```

### Version Information

```bash
# Show DSL grammar version
periplon-executor version
```

## Server Mode (Production)

### Start HTTP/WebSocket Server

Start the REST API server:

```bash
# Basic server (default port 8080, no config file required)
periplon-executor server

# Custom port
periplon-executor server --port 3000

# With embedded workers
periplon-executor server --workers --worker-concurrency 5

# With configuration file (optional)
periplon-executor server --config config.toml
```

**No Configuration File Required!** The server uses sensible defaults:
- Filesystem storage backend (no database needed)
- Development JWT secret (auto-generated)
- Port 8080 (unless overridden via `--port`)

Server will start with:
- **Web UI** at `http://0.0.0.0:8080/` (Embedded Next.js interface!)
- REST API at `http://0.0.0.0:8080/api/v1`
- Health check at `http://0.0.0.0:8080/health`
- WebSocket support at `ws://0.0.0.0:8080/api/v1/ws`

**🎉 Bonus: Embedded Web Interface!** The server includes a complete web UI built with Next.js - no separate web server needed!

### Start Background Worker

Start a dedicated worker process:

```bash
# Basic worker (3 concurrent jobs)
periplon-executor worker

# Custom concurrency
periplon-executor worker --concurrency 10

# With configuration file
periplon-executor worker --config config.toml

# With custom worker ID
periplon-executor worker --worker-id prod-worker-1
```

Workers poll the queue and execute workflows in the background.

### Database Migrations

Manage database schema:

```bash
# Run pending migrations (default action)
periplon-executor migrate

# Explicit up migration
periplon-executor migrate --action up

# Rollback last migration
periplon-executor migrate --action down

# Show migration status
periplon-executor migrate --action status

# With custom configuration
periplon-executor migrate --config config.toml
```

**Environment Variable Required:**
```bash
export DATABASE_URL=postgres://user:password@localhost:5432/dsl_executor
```

## Configuration

### Configuration File (Optional)

**The server works out-of-the-box without any configuration file!** It uses sensible defaults for development.

For production or custom setups, create `config.toml`:

```bash
# Copy example configuration
cp config.example.toml config.toml

# Edit configuration
nano config.toml
```

See `config.example.toml` for full configuration options.

### Environment Variables

Create `.env` file:

```bash
# Copy example environment file
cp .env.example .env

# Edit environment variables
nano .env

# Load environment variables
export $(cat .env | xargs)
```

### Configuration Precedence

```
Command Line Args > Environment Variables > Config File > Defaults
```

### Environment Variables for Server Mode

**All environment variables are optional for development!** The server auto-generates secure defaults.

For production deployments:

```bash
# JWT Secret (highly recommended for production)
export JWT_SECRET=your-super-secret-jwt-key-change-this

# Database URL (required for PostgreSQL backend)
export DATABASE_URL=postgres://user:password@localhost:5432/dsl_executor

# Optional: AWS credentials for S3 storage
export AWS_ACCESS_KEY_ID=your-access-key
export AWS_SECRET_ACCESS_KEY=your-secret-key
```

**Development**: If `JWT_SECRET` is not set, a secure random secret is auto-generated.
**Production**: You must explicitly set `JWT_SECRET` via environment variable.

## Examples

### Quick Start: Zero-Config Server

**Start a fully functional server in one command - no setup required!**

```bash
# Build the project
cargo build --release --features full

# Start server (uses filesystem storage, auto-generates JWT secret)
./target/release/periplon-executor server --port 8080 --workers

# Server is now running at http://0.0.0.0:8080
# Health check: curl http://localhost:8080/health
```

That's it! No database, no config file, no environment variables. Perfect for:
- Local development
- Testing
- CI/CD pipelines
- Quick demos

### Example 1: Local Workflow Development

```bash
# 1. Generate template
periplon-executor template -o my-workflow.yaml

# 2. Edit workflow (in your editor)
nano my-workflow.yaml

# 3. Validate
periplon-executor validate my-workflow.yaml --verbose

# 4. Run locally
periplon-executor run my-workflow.yaml --verbose

# 5. Check status
periplon-executor status my-workflow

# 6. Resume if interrupted
periplon-executor run my-workflow.yaml --resume
```

### Example 2: Natural Language Workflow Generation

```bash
# Create a workflow from description
periplon-executor generate \
  "Create a multi-agent workflow for processing customer feedback. \
   Agent 1 fetches feedback from API, Agent 2 performs sentiment analysis, \
   Agent 3 generates summary report" \
  -o feedback-workflow.yaml \
  --verbose

# Validate generated workflow
periplon-executor validate feedback-workflow.yaml

# Run it
periplon-executor run feedback-workflow.yaml
```

### Example 3: Production Server Deployment

**Quick Start (Development/Testing - No Setup Required):**

```bash
# Just run it! No config, no database, no environment variables needed
periplon-executor server --port 8080 --workers
```

**Production Deployment (with PostgreSQL):**

```bash
# 1. Set up environment
export DATABASE_URL=postgres://prod_user:pass@db.example.com:5432/dsl_executor
export JWT_SECRET=$(openssl rand -base64 32)

# 2. Run database migrations
periplon-executor migrate --action up

# 3. Start server with embedded workers
periplon-executor server \
  --port 8080 \
  --config production.toml \
  --workers \
  --worker-concurrency 10

# Or start separately:
# Terminal 1: Server
periplon-executor server --port 8080 --config production.toml

# Terminal 2: Worker pool
periplon-executor worker --concurrency 10 --config production.toml
```

### Example 4: Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features full

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/periplon-executor /usr/local/bin/
COPY config.toml /etc/periplon-executor/config.toml
COPY migrations /app/migrations

# Run migrations and start server
CMD ["periplon-executor", "server", "--config", "/etc/periplon-executor/config.toml"]
```

```bash
# Build image
docker build -t periplon-executor:latest .

# Run with environment variables
docker run -p 8080:8080 \
  -e DATABASE_URL=postgres://user:pass@db:5432/dsl_executor \
  -e JWT_SECRET=your-secret \
  periplon-executor:latest
```

### Example 5: Development with File Watching

```bash
# Run workflow and watch for changes
while true; do
  periplon-executor validate workflow.yaml && \
  periplon-executor run workflow.yaml
  sleep 2
done
```

### Example 6: CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Test Workflows

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Build CLI
        run: cargo build --release --features cli

      - name: Validate workflows
        run: |
          ./target/release/periplon-executor validate workflows/*.yaml

      - name: Run tests
        run: |
          ./target/release/periplon-executor run test-workflow.yaml --dry-run
```

## Command Reference

### CLI Mode Commands

| Command | Description | Example |
|---------|-------------|---------|
| `template` | Generate workflow template | `periplon-executor template -o template.yaml` |
| `generate` | Generate from natural language | `periplon-executor generate "description" -o workflow.yaml` |
| `validate` | Validate workflow file | `periplon-executor validate workflow.yaml` |
| `run` | Execute workflow | `periplon-executor run workflow.yaml` |
| `list` | List saved states | `periplon-executor list` |
| `status` | Show workflow status | `periplon-executor status my-workflow` |
| `clean` | Clean saved states | `periplon-executor clean my-workflow` |
| `version` | Show version info | `periplon-executor version` |

### Server Mode Commands

| Command | Description | Example |
|---------|-------------|---------|
| `server` | Start HTTP server | `periplon-executor server --port 8080` |
| `worker` | Start background worker | `periplon-executor worker --concurrency 5` |
| `migrate` | Run database migrations | `periplon-executor migrate --action up` |

### Common Flags

| Flag | Description | Applies To |
|------|-------------|------------|
| `-v, --verbose` | Enable verbose output | `run`, `validate`, `generate` |
| `-j, --json` | JSON output format | `run`, `validate`, `list`, `status` |
| `-o, --output` | Output file path | `template`, `generate` |
| `-c, --config` | Configuration file | `server`, `worker`, `migrate` |
| `-s, --state-dir` | State directory | `run`, `list`, `status`, `clean` |
| `-r, --resume` | Resume from state | `run` |
| `--clean` | Clean state before run | `run` |
| `--dry-run` | Validate without executing | `run` |

## Troubleshooting

### Common Issues

**1. Database connection failed**
```bash
# Check DATABASE_URL is set
echo $DATABASE_URL

# Test connection
psql $DATABASE_URL -c "SELECT 1"

# Check migrations
periplon-executor migrate --action status
```

**2. JWT token errors**
```bash
# Ensure JWT_SECRET is set
echo $JWT_SECRET

# Generate new secret
export JWT_SECRET=$(openssl rand -base64 32)
```

**3. Permission errors**
```bash
# Check file permissions
ls -la .dsl_storage/

# Fix permissions
chmod -R 755 .dsl_storage/
```

**4. Port already in use**
```bash
# Check what's using the port
lsof -i :8080

# Use different port
periplon-executor server --port 3000
```

**5. Migration errors**
```bash
# Check migration files
ls -la migrations/

# Reset migrations (CAREFUL: drops all data)
psql $DATABASE_URL -c "DROP TABLE _migrations CASCADE"
periplon-executor migrate --action up
```

## Best Practices

1. **Always validate before running**: `periplon-executor validate workflow.yaml`
2. **Use verbose mode during development**: `--verbose`
3. **Version control your workflows**: Commit `.yaml` files to git
4. **Use environment variables for secrets**: Never commit passwords
5. **Run migrations before starting server**: `periplon-executor migrate`
6. **Monitor worker logs**: Check for failed jobs
7. **Use separate workers for production**: Don't use `--workers` flag
8. **Backup database regularly**: Especially before `migrate down`
9. **Use JSON output for scripting**: `--json` flag
10. **Clean old states periodically**: `periplon-executor clean`

## Getting Help

```bash
# Global help
periplon-executor --help

# Command-specific help
periplon-executor run --help
periplon-executor server --help
periplon-executor migrate --help

# Show version
periplon-executor --version
```

## Resources

- **Project Repository**: https://github.com/yourusername/periplon
- **Documentation**: See `docs/` directory
- **Examples**: See `examples/` directory
- **Web UI**: See `web/` directory
- **API Documentation**: http://localhost:8080/api/v1 (when server running)

## License

MIT OR Apache-2.0

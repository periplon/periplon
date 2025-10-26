# Installation Guide

## Adding Periplon to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
periplon = "0.1.0"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

## Building from Source

### Build the Library

```bash
# Build library and binaries
cargo build

# Build in release mode
cargo build --release
```

### Build the Executor CLI

```bash
# Build DSL executor binary
cargo build --release --bin periplon-executor

# Build with full features (embedded web UI)
cargo build --release --features full
```

### Build the TUI

```bash
# Build the TUI interface
cargo build --release --bin periplon-tui --features tui
```

## Requirements

- **Minimum CLI version**: 2.0.0
- **Rust**: 1.70 or later
- **Tokio runtime**: Required for async operations

## CLI Discovery

The SDK automatically finds the CLI binary in this order:

1. User-provided `cli_path` option
2. `which claude` (PATH lookup with symlink resolution)
3. Shell alias resolution (via `command -v` in interactive shell)
4. Common installation paths:
   - `~/.npm-global/bin/claude`
   - `/usr/local/bin/claude`
   - `~/.local/bin/claude`
   - `~/node_modules/.bin/claude`
   - `~/.yarn/bin/claude`

## Environment Variables

To skip version checking:

```bash
export PERIPLON_SKIP_VERSION_CHECK=1
```

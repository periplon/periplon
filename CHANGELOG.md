# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-10-27

### Added
- Initial release of Periplon Rust SDK
- Hexagonal architecture with ports and adapters pattern
- Type-safe async I/O using tokio
- Stream-based message processing
- Multi-agent workflow orchestration
- DSL system for YAML-based workflow definitions
- Natural language workflow generation
- State management with checkpoint and resume capability
- Dependency resolution with DAG-based task execution
- Variable system with scoped interpolation
- Loop support (ForEach, While, RepeatUntil, Repeat patterns)
- Executor CLI tool with embedded web UI
- TUI (Terminal User Interface) for workflow management
- Comprehensive test coverage (166+ tests)
- Authentication and authorization system
- Queue backend operations
- Storage backend operations
- Schedule management API
- Execution management API
- WebSocket streaming support
- Workflow API integration
- HTTP collections for API integration
- MCP server integration support
- Hook system for lifecycle events
- Permission system with multiple modes
- Notification delivery system with multi-channel support

### Documentation
- Installation guide
- Quick start guide
- Configuration guide
- Architecture guide
- Error handling guide
- Testing guide
- DSL overview and tutorials
- Loop patterns cookbook (25 production-ready patterns)
- TUI user guide with keyboard shortcuts
- API reference documentation
- Security audit documentation
- 50+ example workflows

### CI/CD
- Automated conventional commits workflow
- Release automation with semantic versioning
- Comprehensive test suite
- Clippy and rustfmt checks
- Build validation for all targets
- Web UI build integration

[Unreleased]: https://github.com/periplon/periplon/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/periplon/periplon/releases/tag/v0.1.0

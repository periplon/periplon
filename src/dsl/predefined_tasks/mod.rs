//! Predefined Tasks System
//!
//! This module implements support for reusable predefined tasks that can be referenced
//! in workflows via the `uses:` syntax. Predefined tasks enable task sharing, versioning,
//! and discovery from local directories.
//!
//! # Overview
//!
//! Predefined tasks are standalone task definitions stored in `.task.yaml` files that can be
//! discovered and referenced across multiple workflows. They provide:
//!
//! - **Reusability**: Define a task once, use it across multiple workflows
//! - **Discoverability**: Auto-discover tasks from `.claude/tasks/` directories
//! - **Input/Output Contracts**: Well-defined interfaces with validation
//! - **Versioning**: Semantic versioning support (Phase 2+)
//!
//! # Usage
//!
//! ## Defining a Predefined Task
//!
//! ```yaml
//! # .claude/tasks/google-drive-upload.task.yaml
//! apiVersion: "task/v1"
//! kind: "PredefinedTask"
//! metadata:
//!   name: "google-drive-upload"
//!   version: "1.2.0"
//!   description: "Upload files to Google Drive"
//!
//! spec:
//!   agent_template:
//!     description: "Upload ${input.file_path} to Google Drive"
//!     model: "claude-sonnet-4-5"
//!     tools: ["Bash", "WebFetch"]
//!     permissions:
//!       mode: "acceptEdits"
//!
//!   inputs:
//!     file_path:
//!       type: string
//!       required: true
//!       description: "Local file path to upload"
//!     folder_id:
//!       type: string
//!       required: false
//!       default: "root"
//!
//!   outputs:
//!     file_id:
//!       type: string
//!       description: "Uploaded file ID"
//!       source:
//!         type: state
//!         key: "drive_file_id"
//! ```
//!
//! ## Using a Predefined Task in a Workflow
//!
//! ```yaml
//! # workflow.yaml
//! name: "Upload Report"
//! version: "1.0.0"
//!
//! tasks:
//!   upload:
//!     uses: "google-drive-upload@1.2.0"
//!     inputs:
//!       file_path: "./report.pdf"
//!       folder_id: "abc123xyz"
//!     outputs:
//!       url: "${task.file_id}"
//! ```
//!
//! # Architecture
//!
//! The predefined tasks system consists of:
//!
//! - **Schema** (`schema.rs`): Type definitions for predefined tasks
//! - **Parser** (`parser.rs`): YAML parsing for `.task.yaml` files
//! - **Loader** (`loader.rs`): Filesystem discovery and loading
//! - **Resolver** (`resolver.rs`): Task reference resolution and instantiation
//!
//! # Discovery
//!
//! Tasks are discovered from the following locations (in priority order):
//!
//! 1. **Project Local**: `./.claude/tasks/`
//! 2. **User Global**: `~/.claude/tasks/`
//! 3. **Git Repositories**: Configured sources (Phase 2+)
//! 4. **Registries**: Marketplace support (Phase 5+)

pub mod cache;
pub mod deps;
pub mod discovery;
pub mod groups;
pub mod loader;
pub mod lockfile;
pub mod manifest;
pub mod parser;
pub mod resolver;
pub mod schema;
pub mod sources;
pub mod update;
pub mod version;

pub use cache::TaskCache;
pub use deps::{DependencyError, DependencyResolver, ResolvedTask};
pub use discovery::{CacheStats, TaskDiscovery};
pub use groups::{
    load_task_group, GroupDependency, GroupHooks, GroupLoadError, Hook, PrebuiltWorkflow,
    ResolvedTaskGroup, SharedConfig, TaskGroup, TaskGroupApiVersion, TaskGroupKind,
    TaskGroupLoader, TaskGroupMetadata, TaskGroupReference, TaskGroupSpec, TaskGroupTask,
};
pub use loader::{load_predefined_task, TaskLoader};
pub use lockfile::{
    compute_task_checksum, generate_lock_file, validate_lock_file, LocalSourceResolver, LockFile,
    LockFileError, LockFileMetadata, LockedTask, LockedTaskMetadata, SourceResolver,
    TaskSource as LockFileTaskSource, ValidationIssue, ValidationResult, LOCK_FILE_NAME,
    LOCK_FILE_VERSION,
};
pub use manifest::{
    PackageDependency, PackageMetadata, PackageRequirements, TaskPackage, TaskReference,
};
pub use parser::parse_predefined_task;
pub use resolver::{resolve_task_reference, TaskResolver};
pub use schema::{
    AgentTemplate, PredefinedTask, PredefinedTaskInputSpec, PredefinedTaskMetadata,
    PredefinedTaskOutputSpec, PredefinedTaskSpec, TaskApiVersion, TaskDependency, TaskKind,
};
pub use sources::{
    GitTaskSource, HealthStatus, LocalTaskSource, SourceConfig, SourceInfo, SourceType,
    TaskMetadata, TaskSource, TaskSourcesConfig, UpdatePolicy, UpdateResult,
};
pub use update::{
    BreakingChangeInfo, UpdateChecker, UpdateError, UpdateInfo, UpdateRecommendation,
    UpdateResult as TaskUpdateResult, UpdateStats, VersionUpdatePolicy,
};
pub use version::{find_best_match, VersionConstraint, VersionError};

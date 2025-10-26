//! Task Groups Module
//!
//! This module provides support for task groups - collections of related tasks
//! that work together as cohesive units for integration suites, feature bundles,
//! or multi-step workflows.

pub mod loader;
pub mod namespace;
pub mod parser;
pub mod schema;

pub use loader::{load_task_group, GroupLoadError, ResolvedTaskGroup, TaskGroupLoader};
pub use namespace::{NamespaceResolver, ResolverError};
pub use parser::{parse_task_group, ParseError};
pub use schema::{
    GroupDependency, GroupHooks, Hook, PrebuiltWorkflow, SharedConfig, TaskGroup,
    TaskGroupApiVersion, TaskGroupKind, TaskGroupMetadata, TaskGroupReference, TaskGroupSpec,
    TaskGroupTask,
};

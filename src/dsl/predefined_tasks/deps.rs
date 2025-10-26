//! Dependency Resolution for Predefined Tasks
//!
//! This module handles dependency graph construction and resolution for predefined tasks,
//! ensuring all dependencies are satisfied with compatible versions.

use super::schema::TaskReference;
use super::version::{find_best_match, VersionConstraint, VersionError};
use super::PredefinedTask;
use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors that can occur during dependency resolution
#[derive(Debug, Error)]
pub enum DependencyError {
    /// Task not found
    #[error("Task '{0}' not found")]
    TaskNotFound(String),

    /// Circular dependency detected
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Version conflict between dependencies
    #[error("Version conflict for task '{task}': {details}")]
    VersionConflict { task: String, details: String },

    /// No version satisfies all constraints
    #[error("No version of '{task}' satisfies all constraints: {constraints:?}")]
    NoSatisfyingVersion {
        task: String,
        constraints: Vec<String>,
    },

    /// Missing required dependency
    #[error("Missing required dependency '{dependency}' for task '{task}'")]
    MissingDependency { task: String, dependency: String },

    /// Version parsing error
    #[error("Version error: {0}")]
    VersionError(#[from] VersionError),
}

/// A resolved task with its specific version
#[derive(Debug, Clone)]
pub struct ResolvedTask {
    /// Task name
    pub name: String,

    /// Resolved version
    pub version: String,

    /// The predefined task definition
    pub task: PredefinedTask,

    /// Dependencies of this task (name -> version)
    pub dependencies: HashMap<String, String>,
}

/// Dependency resolver
pub struct DependencyResolver {
    /// Available tasks by name and version
    tasks: HashMap<String, HashMap<String, PredefinedTask>>,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    /// Add a task to the resolver's registry
    pub fn add_task(&mut self, task: PredefinedTask) {
        let name = task.metadata.name.clone();
        let version = task.metadata.version.clone();

        self.tasks.entry(name).or_default().insert(version, task);
    }

    /// Add multiple tasks
    pub fn add_tasks(&mut self, tasks: Vec<PredefinedTask>) {
        for task in tasks {
            self.add_task(task);
        }
    }

    /// Resolve dependencies for a task
    ///
    /// Returns a list of resolved tasks in topological order (dependencies first)
    pub fn resolve(&self, task_ref: &TaskReference) -> Result<Vec<ResolvedTask>, DependencyError> {
        let mut resolved = HashMap::new();
        let mut constraints: HashMap<String, Vec<VersionConstraint>> = HashMap::new();

        // Collect all dependencies and constraints
        self.collect_dependencies(task_ref, &mut resolved, &mut constraints)?;

        // Resolve version constraints to specific versions
        let versions = self.resolve_constraints(&constraints)?;

        // Build dependency graph
        let mut graph = DiGraph::new();
        let mut node_indices = HashMap::new();

        // Create nodes for all resolved tasks
        for (name, version) in &versions {
            let task = self
                .get_task(name, version)
                .ok_or_else(|| DependencyError::TaskNotFound(format!("{}@{}", name, version)))?;

            let resolved_task = ResolvedTask {
                name: name.clone(),
                version: version.clone(),
                task: task.clone(),
                dependencies: HashMap::new(),
            };

            let idx = graph.add_node(resolved_task);
            node_indices.insert(name.clone(), idx);
        }

        // Add edges for dependencies
        for (name, version) in &versions {
            let task = self.get_task(name, version).unwrap();

            if let Some(from_idx) = node_indices.get(name) {
                for dep in &task.spec.dependencies {
                    if let Some(to_idx) = node_indices.get(&dep.name) {
                        graph.add_edge(*from_idx, *to_idx, ());
                    }
                }
            }
        }

        // Check for cycles
        if is_cyclic_directed(&graph) {
            return Err(DependencyError::CircularDependency(
                self.find_cycle_path(&graph, &node_indices),
            ));
        }

        // Topological sort (toposort returns reverse order, so reverse it to get dependencies first)
        let mut sorted = toposort(&graph, None).map_err(|_| {
            DependencyError::CircularDependency("Failed to sort dependencies".to_string())
        })?;
        sorted.reverse(); // Dependencies first

        // Build result with dependency information
        let mut result = Vec::new();
        for node_idx in sorted {
            let mut resolved_task = graph[node_idx].clone();

            // Populate dependencies map
            for edge in graph.edges(node_idx) {
                let dep_task = &graph[edge.target()];
                resolved_task
                    .dependencies
                    .insert(dep_task.name.clone(), dep_task.version.clone());
            }

            result.push(resolved_task);
        }

        Ok(result)
    }

    /// Collect all dependencies recursively
    fn collect_dependencies(
        &self,
        task_ref: &TaskReference,
        resolved: &mut HashMap<String, String>,
        constraints: &mut HashMap<String, Vec<VersionConstraint>>,
    ) -> Result<(), DependencyError> {
        // Check if already processed
        if resolved.contains_key(&task_ref.name) {
            return Ok(());
        }

        // Find a version that matches the constraint
        let constraint = VersionConstraint::parse(&task_ref.version)?;
        let available_versions = self.get_available_versions(&task_ref.name);

        let version = find_best_match(&constraint, &available_versions)?
            .ok_or_else(|| DependencyError::TaskNotFound(task_ref.to_string()))?;

        // Get the task
        let task = self.get_task(&task_ref.name, &version).ok_or_else(|| {
            DependencyError::TaskNotFound(format!("{}@{}", task_ref.name, version))
        })?;

        // Mark as resolved (tentatively)
        resolved.insert(task_ref.name.clone(), version);

        // Add constraint
        constraints
            .entry(task_ref.name.clone())
            .or_default()
            .push(constraint);

        // Process dependencies
        for dep in &task.spec.dependencies {
            let dep_ref = TaskReference {
                name: dep.name.clone(),
                version: dep.version.clone(),
            };

            self.collect_dependencies(&dep_ref, resolved, constraints)?;
        }

        Ok(())
    }

    /// Resolve version constraints to specific versions
    fn resolve_constraints(
        &self,
        constraints: &HashMap<String, Vec<VersionConstraint>>,
    ) -> Result<HashMap<String, String>, DependencyError> {
        let mut resolved = HashMap::new();

        for (name, constraint_list) in constraints {
            let available = self.get_available_versions(name);

            // Find version that satisfies ALL constraints
            let mut candidates: Option<HashSet<String>> = None;

            for constraint in constraint_list {
                let matching: HashSet<String> = available
                    .iter()
                    .filter(|v| constraint.matches(v).unwrap_or(false))
                    .cloned()
                    .collect();

                candidates = Some(match candidates {
                    None => matching,
                    Some(current) => current.intersection(&matching).cloned().collect(),
                });
            }

            let candidates = candidates.unwrap_or_default();

            if candidates.is_empty() {
                return Err(DependencyError::NoSatisfyingVersion {
                    task: name.clone(),
                    constraints: constraint_list.iter().map(|c| c.to_string()).collect(),
                });
            }

            // Select highest version from candidates
            let mut candidate_vec: Vec<_> = candidates.into_iter().collect();
            candidate_vec.sort_by(|a, b| {
                semver::Version::parse(a)
                    .unwrap()
                    .cmp(&semver::Version::parse(b).unwrap())
            });

            let best_version = candidate_vec.last().unwrap().clone();
            resolved.insert(name.clone(), best_version);
        }

        Ok(resolved)
    }

    /// Get a specific task by name and version
    fn get_task(&self, name: &str, version: &str) -> Option<&PredefinedTask> {
        self.tasks.get(name)?.get(version)
    }

    /// Get all available versions for a task
    fn get_available_versions(&self, name: &str) -> Vec<String> {
        self.tasks
            .get(name)
            .map(|versions| versions.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Find a cycle path for error reporting
    fn find_cycle_path(
        &self,
        graph: &DiGraph<ResolvedTask, ()>,
        _node_indices: &HashMap<String, NodeIndex>,
    ) -> String {
        // Simple cycle detection - just report that a cycle exists
        // A more sophisticated implementation could trace the actual cycle
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        for node_idx in graph.node_indices() {
            if Self::dfs_find_cycle(graph, node_idx, &mut visited, &mut path) {
                return path
                    .iter()
                    .map(|idx| graph[*idx].name.clone())
                    .collect::<Vec<_>>()
                    .join(" -> ");
            }
        }

        "Unknown cycle".to_string()
    }

    /// DFS to find cycle
    fn dfs_find_cycle(
        graph: &DiGraph<ResolvedTask, ()>,
        node: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        path: &mut Vec<NodeIndex>,
    ) -> bool {
        if path.contains(&node) {
            path.push(node);
            return true;
        }

        if visited.contains(&node) {
            return false;
        }

        visited.insert(node);
        path.push(node);

        for neighbor in graph.neighbors(node) {
            if Self::dfs_find_cycle(graph, neighbor, visited, path) {
                return true;
            }
        }

        path.pop();
        false
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::predefined_tasks::{
        AgentTemplate, PredefinedTaskMetadata, PredefinedTaskSpec, TaskApiVersion, TaskDependency,
        TaskKind,
    };
    use crate::dsl::schema::PermissionsSpec;

    fn create_test_task(name: &str, version: &str, deps: Vec<(&str, &str)>) -> PredefinedTask {
        PredefinedTask {
            api_version: TaskApiVersion::V1,
            kind: TaskKind::PredefinedTask,
            metadata: PredefinedTaskMetadata {
                name: name.to_string(),
                version: version.to_string(),
                description: None,
                author: None,
                license: None,
                repository: None,
                tags: vec![],
            },
            spec: PredefinedTaskSpec {
                agent_template: AgentTemplate {
                    description: "Test task".to_string(),
                    model: None,
                    system_prompt: None,
                    tools: vec![],
                    permissions: PermissionsSpec::default(),
                    max_turns: None,
                },
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                dependencies: deps
                    .into_iter()
                    .map(|(n, v)| TaskDependency {
                        name: n.to_string(),
                        version: v.to_string(),
                        optional: false,
                    })
                    .collect(),
                examples: vec![],
            },
        }
    }

    #[test]
    fn test_resolve_simple() {
        let mut resolver = DependencyResolver::new();

        let task_a = create_test_task("task-a", "1.0.0", vec![]);
        resolver.add_task(task_a);

        let task_ref = TaskReference {
            name: "task-a".to_string(),
            version: "1.0.0".to_string(),
        };

        let resolved = resolver.resolve(&task_ref).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].name, "task-a");
        assert_eq!(resolved[0].version, "1.0.0");
    }

    #[test]
    fn test_resolve_with_dependency() {
        let mut resolver = DependencyResolver::new();

        let task_b = create_test_task("task-b", "1.0.0", vec![]);
        let task_a = create_test_task("task-a", "1.0.0", vec![("task-b", "^1.0.0")]);

        resolver.add_task(task_a);
        resolver.add_task(task_b);

        let task_ref = TaskReference {
            name: "task-a".to_string(),
            version: "1.0.0".to_string(),
        };

        let resolved = resolver.resolve(&task_ref).unwrap();
        assert_eq!(resolved.len(), 2);

        // Dependencies should come first
        assert_eq!(resolved[0].name, "task-b");
        assert_eq!(resolved[1].name, "task-a");
    }

    #[test]
    fn test_resolve_diamond_dependency() {
        let mut resolver = DependencyResolver::new();

        // Diamond: A depends on B and C, both B and C depend on D
        let task_d = create_test_task("task-d", "2.0.0", vec![]);
        let task_c = create_test_task("task-c", "1.0.0", vec![("task-d", "^2.0.0")]);
        let task_b = create_test_task("task-b", "1.0.0", vec![("task-d", "^2.0.0")]);
        let task_a = create_test_task(
            "task-a",
            "1.0.0",
            vec![("task-b", "^1.0.0"), ("task-c", "^1.0.0")],
        );

        resolver.add_task(task_a);
        resolver.add_task(task_b);
        resolver.add_task(task_c);
        resolver.add_task(task_d);

        let task_ref = TaskReference {
            name: "task-a".to_string(),
            version: "1.0.0".to_string(),
        };

        let resolved = resolver.resolve(&task_ref).unwrap();
        assert_eq!(resolved.len(), 4);

        // D should be first (deepest dependency)
        assert_eq!(resolved[0].name, "task-d");
        assert_eq!(resolved[0].version, "2.0.0");

        // A should be last
        assert_eq!(resolved[3].name, "task-a");
    }

    #[test]
    fn test_resolve_version_constraint() {
        let mut resolver = DependencyResolver::new();

        let task_b_v1 = create_test_task("task-b", "1.0.0", vec![]);
        let task_b_v2 = create_test_task("task-b", "1.5.0", vec![]);
        let task_b_v3 = create_test_task("task-b", "2.0.0", vec![]);
        let task_a = create_test_task("task-a", "1.0.0", vec![("task-b", "^1.0.0")]);

        resolver.add_task(task_a);
        resolver.add_task(task_b_v1);
        resolver.add_task(task_b_v2);
        resolver.add_task(task_b_v3);

        let task_ref = TaskReference {
            name: "task-a".to_string(),
            version: "1.0.0".to_string(),
        };

        let resolved = resolver.resolve(&task_ref).unwrap();

        // Should select highest version matching ^1.0.0 (which is 1.5.0)
        let task_b = resolved.iter().find(|t| t.name == "task-b").unwrap();
        assert_eq!(task_b.version, "1.5.0");
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut resolver = DependencyResolver::new();

        // Create circular dependency: A -> B -> C -> A
        let task_c = create_test_task("task-c", "1.0.0", vec![("task-a", "^1.0.0")]);
        let task_b = create_test_task("task-b", "1.0.0", vec![("task-c", "^1.0.0")]);
        let task_a = create_test_task("task-a", "1.0.0", vec![("task-b", "^1.0.0")]);

        resolver.add_task(task_a);
        resolver.add_task(task_b);
        resolver.add_task(task_c);

        let task_ref = TaskReference {
            name: "task-a".to_string(),
            version: "1.0.0".to_string(),
        };

        let result = resolver.resolve(&task_ref);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DependencyError::CircularDependency(_)
        ));
    }

    #[test]
    fn test_no_satisfying_version() {
        let mut resolver = DependencyResolver::new();

        let task_b = create_test_task("task-b", "1.0.0", vec![]);
        let task_a = create_test_task("task-a", "1.0.0", vec![("task-b", "^2.0.0")]);

        resolver.add_task(task_a);
        resolver.add_task(task_b);

        let task_ref = TaskReference {
            name: "task-a".to_string(),
            version: "1.0.0".to_string(),
        };

        let result = resolver.resolve(&task_ref);
        assert!(result.is_err());

        // The error could be NoSatisfyingVersion or TaskNotFound depending on resolution path
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            DependencyError::NoSatisfyingVersion { .. } | DependencyError::TaskNotFound(_)
        ));
    }

    #[test]
    fn test_task_not_found() {
        let resolver = DependencyResolver::new();

        let task_ref = TaskReference {
            name: "nonexistent".to_string(),
            version: "1.0.0".to_string(),
        };

        let result = resolver.resolve(&task_ref);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DependencyError::TaskNotFound(_)
        ));
    }
}

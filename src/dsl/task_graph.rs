//! Task Graph and Dependency Resolution
//!
//! This module provides task scheduling and dependency resolution using
//! topological sorting and graph algorithms.

use crate::dsl::schema::TaskSpec;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Task node in the execution graph
#[derive(Debug, Clone)]
pub struct TaskNode {
    pub id: String,
    pub spec: TaskSpec,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
    pub parallel_tasks: Vec<String>,
}

/// Task graph for managing task execution and dependencies
pub struct TaskGraph {
    tasks: HashMap<String, TaskNode>,
    adjacency: HashMap<String, Vec<String>>,
}

impl TaskGraph {
    /// Create a new empty task graph
    pub fn new() -> Self {
        TaskGraph {
            tasks: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }

    /// Add a task to the graph
    ///
    /// # Arguments
    ///
    /// * `id` - Unique task identifier
    /// * `spec` - Task specification
    pub fn add_task(&mut self, id: String, spec: TaskSpec) {
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
                .or_default()
                .push(id.clone());
        }
    }

    /// Get all tasks that are ready to execute
    ///
    /// A task is ready if all its dependencies are completed
    pub fn get_ready_tasks(&self) -> Vec<String> {
        self.tasks
            .iter()
            .filter(|(_, node)| {
                node.status == TaskStatus::Pending
                    && node
                        .dependencies
                        .iter()
                        .all(|dep| self.is_task_completed(dep))
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Check if a task is completed
    ///
    /// A task is considered completed if it has finished successfully or was skipped.
    /// Skipped tasks (due to unmet conditions) are treated as completed for dependency purposes.
    fn is_task_completed(&self, task_id: &str) -> bool {
        self.tasks
            .get(task_id)
            .map(|n| n.status == TaskStatus::Completed || n.status == TaskStatus::Skipped)
            .unwrap_or(false)
    }

    /// Get tasks that can run in parallel with the given task
    pub fn get_parallel_tasks(&self, task_id: &str) -> Vec<String> {
        self.tasks
            .get(task_id)
            .map(|node| node.parallel_tasks.clone())
            .unwrap_or_default()
    }

    /// Update task status
    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<()> {
        self.tasks
            .get_mut(task_id)
            .map(|node| node.status = status)
            .ok_or_else(|| Error::InvalidInput(format!("Task '{}' not found", task_id)))
    }

    /// Get task status
    ///
    /// # Performance
    ///
    /// Returns Copy type without allocation
    pub fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        self.tasks.get(task_id).map(|node| node.status)
    }

    /// Get task specification
    pub fn get_task(&self, task_id: &str) -> Option<&TaskNode> {
        self.tasks.get(task_id)
    }

    /// Perform topological sort on the task graph
    ///
    /// Returns an ordered list of task IDs or an error if a cycle is detected
    ///
    /// # Algorithm
    ///
    /// Uses Kahn's algorithm for topological sorting
    ///
    /// # Performance
    ///
    /// Pre-allocates collections with task count for better performance
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        let task_count = self.tasks.len();

        // Pre-allocate with capacity for better performance
        let mut in_degree: HashMap<String, usize> = HashMap::with_capacity(task_count);
        let mut queue: VecDeque<String> = VecDeque::with_capacity(task_count);
        let mut result: Vec<String> = Vec::with_capacity(task_count);

        // Calculate in-degrees
        for id in self.tasks.keys() {
            in_degree.insert(id.clone(), 0);
        }

        for neighbors in self.adjacency.values() {
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
            // Find tasks that couldn't be sorted (part of cycle)
            let sorted_set: std::collections::HashSet<_> = result.iter().collect();
            let unsorted: Vec<String> = self
                .tasks
                .keys()
                .filter(|id| !sorted_set.contains(id))
                .cloned()
                .collect();

            // Show which tasks are in the cycle and their dependencies
            let cycle_info: Vec<String> = unsorted
                .iter()
                .map(|id| {
                    let deps = self
                        .tasks
                        .get(id)
                        .map(|node| node.spec.depends_on.clone())
                        .unwrap_or_default();
                    format!("  - {} (depends on: {:?})", id, deps)
                })
                .collect();

            return Err(Error::InvalidInput(format!(
                "Cyclic dependency detected in task graph. Tasks in cycle:\n{}",
                cycle_info.join("\n")
            )));
        }

        Ok(result)
    }

    /// Get all tasks in the graph
    pub fn get_all_tasks(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }

    /// Get the number of tasks in the graph
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Check if all tasks are completed
    pub fn is_complete(&self) -> bool {
        self.tasks
            .values()
            .all(|node| node.status == TaskStatus::Completed)
    }

    /// Get tasks that depend on the given task
    pub fn get_dependents(&self, task_id: &str) -> Vec<String> {
        self.adjacency.get(task_id).cloned().unwrap_or_default()
    }

    /// Update task dependencies
    ///
    /// Replaces the dependencies of a task with new dependencies and
    /// updates the adjacency list accordingly.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task whose dependencies should be updated
    /// * `new_dependencies` - The new list of dependencies
    pub fn update_task_dependencies(
        &mut self,
        task_id: &str,
        new_dependencies: Vec<String>,
    ) -> Result<()> {
        let task_node = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| Error::InvalidInput(format!("Task '{}' not found", task_id)))?;

        // Remove old adjacency list entries
        for old_dep in &task_node.dependencies {
            if let Some(dependents) = self.adjacency.get_mut(old_dep) {
                dependents.retain(|t| t != task_id);
                // Clean up empty entries
                if dependents.is_empty() {
                    self.adjacency.remove(old_dep);
                }
            }
        }

        // Update task dependencies
        task_node.dependencies = new_dependencies.clone();
        task_node.spec.depends_on = new_dependencies.clone();

        // Add new adjacency list entries
        for new_dep in &new_dependencies {
            self.adjacency
                .entry(new_dep.clone())
                .or_default()
                .push(task_id.to_string());
        }

        Ok(())
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph = TaskGraph::new();
        assert_eq!(graph.task_count(), 0);
        assert!(graph.is_complete());
    }

    #[test]
    fn test_add_single_task() {
        let mut graph = TaskGraph::new();
        let task = TaskSpec {
            description: "Test task".to_string(),
            ..Default::default()
        };

        graph.add_task("task1".to_string(), task);
        assert_eq!(graph.task_count(), 1);
    }

    #[test]
    fn test_topological_sort_simple() {
        let mut graph = TaskGraph::new();

        let task1 = TaskSpec {
            description: "Task 1".to_string(),
            ..Default::default()
        };

        let task2 = TaskSpec {
            description: "Task 2".to_string(),
            depends_on: vec!["task1".to_string()],
            ..Default::default()
        };

        graph.add_task("task1".to_string(), task1);
        graph.add_task("task2".to_string(), task2);

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 2);

        let task1_pos = sorted.iter().position(|x| x == "task1").unwrap();
        let task2_pos = sorted.iter().position(|x| x == "task2").unwrap();
        assert!(task1_pos < task2_pos);
    }

    #[test]
    fn test_topological_sort_complex() {
        let mut graph = TaskGraph::new();

        let task1 = TaskSpec {
            description: "Task 1".to_string(),
            ..Default::default()
        };

        let task2 = TaskSpec {
            description: "Task 2".to_string(),
            depends_on: vec!["task1".to_string()],
            ..Default::default()
        };

        let task3 = TaskSpec {
            description: "Task 3".to_string(),
            depends_on: vec!["task1".to_string()],
            ..Default::default()
        };

        let task4 = TaskSpec {
            description: "Task 4".to_string(),
            depends_on: vec!["task2".to_string(), "task3".to_string()],
            ..Default::default()
        };

        graph.add_task("task1".to_string(), task1);
        graph.add_task("task2".to_string(), task2);
        graph.add_task("task3".to_string(), task3);
        graph.add_task("task4".to_string(), task4);

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 4);

        let task1_pos = sorted.iter().position(|x| x == "task1").unwrap();
        let task2_pos = sorted.iter().position(|x| x == "task2").unwrap();
        let task3_pos = sorted.iter().position(|x| x == "task3").unwrap();
        let task4_pos = sorted.iter().position(|x| x == "task4").unwrap();

        assert!(task1_pos < task2_pos);
        assert!(task1_pos < task3_pos);
        assert!(task2_pos < task4_pos);
        assert!(task3_pos < task4_pos);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = TaskGraph::new();

        let task1 = TaskSpec {
            description: "Task 1".to_string(),
            depends_on: vec!["task2".to_string()],
            ..Default::default()
        };

        let task2 = TaskSpec {
            description: "Task 2".to_string(),
            depends_on: vec!["task1".to_string()],
            ..Default::default()
        };

        graph.add_task("task1".to_string(), task1);
        graph.add_task("task2".to_string(), task2);

        let result = graph.topological_sort();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_ready_tasks() {
        let mut graph = TaskGraph::new();

        let task1 = TaskSpec {
            description: "Task 1".to_string(),
            ..Default::default()
        };

        let task2 = TaskSpec {
            description: "Task 2".to_string(),
            depends_on: vec!["task1".to_string()],
            ..Default::default()
        };

        graph.add_task("task1".to_string(), task1);
        graph.add_task("task2".to_string(), task2);

        let ready = graph.get_ready_tasks();
        assert_eq!(ready.len(), 1);
        assert!(ready.contains(&"task1".to_string()));

        graph
            .update_task_status("task1", TaskStatus::Completed)
            .unwrap();
        let ready = graph.get_ready_tasks();
        assert_eq!(ready.len(), 1);
        assert!(ready.contains(&"task2".to_string()));
    }

    #[test]
    fn test_parallel_tasks() {
        let mut graph = TaskGraph::new();

        let task1 = TaskSpec {
            description: "Task 1".to_string(),
            parallel_with: vec!["task2".to_string()],
            ..Default::default()
        };

        let task2 = TaskSpec {
            description: "Task 2".to_string(),
            ..Default::default()
        };

        graph.add_task("task1".to_string(), task1);
        graph.add_task("task2".to_string(), task2);

        let parallel = graph.get_parallel_tasks("task1");
        assert_eq!(parallel.len(), 1);
        assert_eq!(parallel[0], "task2");
    }
}

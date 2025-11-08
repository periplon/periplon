//! Breakpoint Management System
//!
//! Provides breakpoint functionality for pausing execution at specific points:
//! - Task breakpoints
//! - Conditional breakpoints
//! - Loop iteration breakpoints
//! - Variable watch breakpoints
use crate::dsl::task_graph::TaskStatus;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Breakpoint manager
#[derive(Debug, Clone)]
pub struct BreakpointManager {
    /// Task breakpoints (break when task starts)
    task_breakpoints: HashSet<String>,

    /// Conditional breakpoints
    conditional_breakpoints: HashMap<String, ConditionalBreakpoint>,

    /// Loop iteration breakpoints (task_id -> set of iterations)
    loop_breakpoints: HashMap<String, HashSet<usize>>,

    /// Variable watch breakpoints
    watch_breakpoints: HashMap<String, WatchBreakpoint>,

    /// Next breakpoint ID
    next_id: usize,

    /// Global enabled/disabled flag
    enabled: bool,
}

/// Conditional breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalBreakpoint {
    /// Breakpoint ID
    pub id: String,

    /// Condition to check
    pub condition: BreakCondition,

    /// Whether this breakpoint is enabled
    pub enabled: bool,

    /// Hit count (how many times this breakpoint has triggered)
    pub hit_count: usize,

    /// Description
    pub description: Option<String>,
}

/// Break condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BreakCondition {
    /// Break when task reaches specific status
    TaskStatus { task_id: String, status: TaskStatus },

    /// Break when variable equals value
    VariableEquals {
        scope: VariableScope,
        name: String,
        value: serde_json::Value,
    },

    /// Break when variable changes
    VariableChanged { scope: VariableScope, name: String },

    /// Break on any error
    OnError,

    /// Break on specific task error
    TaskError { task_id: String },

    /// Custom expression (for future implementation)
    Expression(String),
}

/// Variable scope for breakpoints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum VariableScope {
    Workflow,
    Agent(String),
    Task(String),
    Loop { task_id: String },
}

/// Variable watch breakpoint
#[derive(Debug, Clone)]
pub struct WatchBreakpoint {
    /// Breakpoint ID
    pub id: String,

    /// Variable scope
    pub scope: VariableScope,

    /// Variable name
    pub name: String,

    /// Last known value (for change detection)
    pub last_value: Option<serde_json::Value>,

    /// Break on any change or specific value
    pub condition: WatchCondition,

    /// Whether enabled
    pub enabled: bool,

    /// Hit count
    pub hit_count: usize,
}

/// Watch condition
#[derive(Debug, Clone)]
pub enum WatchCondition {
    /// Break on any change
    AnyChange,

    /// Break when equals specific value
    Equals(serde_json::Value),

    /// Break when not equals specific value
    NotEquals(serde_json::Value),
}

impl Default for BreakpointManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BreakpointManager {
    /// Create a new breakpoint manager
    pub fn new() -> Self {
        Self {
            task_breakpoints: HashSet::new(),
            conditional_breakpoints: HashMap::new(),
            loop_breakpoints: HashMap::new(),
            watch_breakpoints: HashMap::new(),
            next_id: 0,
            enabled: true,
        }
    }

    /// Enable all breakpoints
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable all breakpoints
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if breakpoints are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Add task breakpoint
    pub fn add_task_breakpoint(&mut self, task_id: String) -> String {
        self.task_breakpoints.insert(task_id.clone());
        format!("task:{}", task_id)
    }

    /// Remove task breakpoint
    pub fn remove_task_breakpoint(&mut self, task_id: &str) -> bool {
        self.task_breakpoints.remove(task_id)
    }

    /// Check if should break on task
    pub fn should_break_on_task(&self, task_id: &str) -> bool {
        self.enabled && self.task_breakpoints.contains(task_id)
    }

    /// Add conditional breakpoint
    pub fn add_conditional_breakpoint(
        &mut self,
        condition: BreakCondition,
        description: Option<String>,
    ) -> String {
        let id = format!("cond:{}", self.next_id);
        self.next_id += 1;

        let breakpoint = ConditionalBreakpoint {
            id: id.clone(),
            condition,
            enabled: true,
            hit_count: 0,
            description,
        };

        self.conditional_breakpoints.insert(id.clone(), breakpoint);
        id
    }

    /// Remove conditional breakpoint
    pub fn remove_conditional_breakpoint(&mut self, id: &str) -> bool {
        self.conditional_breakpoints.remove(id).is_some()
    }

    /// Enable conditional breakpoint
    pub fn enable_conditional(&mut self, id: &str) -> bool {
        if let Some(bp) = self.conditional_breakpoints.get_mut(id) {
            bp.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable conditional breakpoint
    pub fn disable_conditional(&mut self, id: &str) -> bool {
        if let Some(bp) = self.conditional_breakpoints.get_mut(id) {
            bp.enabled = false;
            true
        } else {
            false
        }
    }

    /// Check if should break on condition
    pub fn check_conditional(
        &mut self,
        task_id: &str,
        status: TaskStatus,
        variables: &HashMap<VariableScope, HashMap<String, serde_json::Value>>,
    ) -> Option<String> {
        if !self.enabled {
            return None;
        }

        for (id, bp) in self.conditional_breakpoints.iter_mut() {
            if !bp.enabled {
                continue;
            }

            let should_break = match &bp.condition {
                BreakCondition::TaskStatus {
                    task_id: bp_task,
                    status: bp_status,
                } => task_id == bp_task && status == *bp_status,

                BreakCondition::VariableEquals { scope, name, value } => variables
                    .get(scope)
                    .and_then(|vars| vars.get(name))
                    .map(|v| v == value)
                    .unwrap_or(false),

                BreakCondition::TaskError { task_id: bp_task } => {
                    task_id == bp_task && status == TaskStatus::Failed
                }

                BreakCondition::OnError => status == TaskStatus::Failed,

                _ => false,
            };

            if should_break {
                bp.hit_count += 1;
                return Some(id.clone());
            }
        }

        None
    }

    /// Add loop iteration breakpoint
    pub fn add_loop_breakpoint(&mut self, task_id: String, iteration: usize) -> String {
        self.loop_breakpoints
            .entry(task_id.clone())
            .or_default()
            .insert(iteration);
        format!("loop:{}:{}", task_id, iteration)
    }

    /// Remove loop iteration breakpoint
    pub fn remove_loop_breakpoint(&mut self, task_id: &str, iteration: usize) -> bool {
        if let Some(iterations) = self.loop_breakpoints.get_mut(task_id) {
            iterations.remove(&iteration)
        } else {
            false
        }
    }

    /// Check if should break on loop iteration
    pub fn should_break_on_iteration(&self, task_id: &str, iteration: usize) -> bool {
        self.enabled
            && self
                .loop_breakpoints
                .get(task_id)
                .map(|iterations| iterations.contains(&iteration))
                .unwrap_or(false)
    }

    /// Add variable watch breakpoint
    pub fn add_watch(
        &mut self,
        scope: VariableScope,
        name: String,
        condition: WatchCondition,
    ) -> String {
        let id = format!("watch:{}", self.next_id);
        self.next_id += 1;

        let watch = WatchBreakpoint {
            id: id.clone(),
            scope,
            name,
            last_value: None,
            condition,
            enabled: true,
            hit_count: 0,
        };

        self.watch_breakpoints.insert(id.clone(), watch);
        id
    }

    /// Remove watch breakpoint
    pub fn remove_watch(&mut self, id: &str) -> bool {
        self.watch_breakpoints.remove(id).is_some()
    }

    /// Check watch breakpoints for variable change
    pub fn check_watch(
        &mut self,
        scope: &VariableScope,
        name: &str,
        new_value: &serde_json::Value,
    ) -> Option<String> {
        if !self.enabled {
            return None;
        }

        for (id, watch) in self.watch_breakpoints.iter_mut() {
            if !watch.enabled {
                continue;
            }

            if &watch.scope != scope || watch.name != name {
                continue;
            }

            let should_break = match &watch.condition {
                WatchCondition::AnyChange => {
                    if let Some(ref last) = watch.last_value {
                        last != new_value
                    } else {
                        true // First time seeing this variable
                    }
                }

                WatchCondition::Equals(target) => new_value == target,

                WatchCondition::NotEquals(target) => new_value != target,
            };

            // Update last value
            watch.last_value = Some(new_value.clone());

            if should_break {
                watch.hit_count += 1;
                return Some(id.clone());
            }
        }

        None
    }

    /// Get all breakpoints
    pub fn list_all(&self) -> Vec<BreakpointInfo> {
        let mut breakpoints = Vec::new();

        // Task breakpoints
        for task_id in &self.task_breakpoints {
            breakpoints.push(BreakpointInfo {
                id: format!("task:{}", task_id),
                breakpoint_type: BreakpointType::Task,
                description: format!("Break on task: {}", task_id),
                enabled: self.enabled,
                hit_count: 0,
            });
        }

        // Conditional breakpoints
        for bp in self.conditional_breakpoints.values() {
            breakpoints.push(BreakpointInfo {
                id: bp.id.clone(),
                breakpoint_type: BreakpointType::Conditional,
                description: bp
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("{:?}", bp.condition)),
                enabled: bp.enabled,
                hit_count: bp.hit_count,
            });
        }

        // Loop breakpoints
        for (task_id, iterations) in &self.loop_breakpoints {
            for iteration in iterations {
                breakpoints.push(BreakpointInfo {
                    id: format!("loop:{}:{}", task_id, iteration),
                    breakpoint_type: BreakpointType::Loop,
                    description: format!("Break on task {} iteration {}", task_id, iteration),
                    enabled: self.enabled,
                    hit_count: 0,
                });
            }
        }

        // Watch breakpoints
        for watch in self.watch_breakpoints.values() {
            breakpoints.push(BreakpointInfo {
                id: watch.id.clone(),
                breakpoint_type: BreakpointType::Watch,
                description: format!("Watch {:?}.{}", watch.scope, watch.name),
                enabled: watch.enabled,
                hit_count: watch.hit_count,
            });
        }

        breakpoints
    }

    /// Clear all breakpoints
    pub fn clear_all(&mut self) {
        self.task_breakpoints.clear();
        self.conditional_breakpoints.clear();
        self.loop_breakpoints.clear();
        self.watch_breakpoints.clear();
    }

    /// Get breakpoint count
    pub fn count(&self) -> usize {
        self.task_breakpoints.len()
            + self.conditional_breakpoints.len()
            + self
                .loop_breakpoints
                .values()
                .map(|s| s.len())
                .sum::<usize>()
            + self.watch_breakpoints.len()
    }
}

/// Breakpoint information for display
#[derive(Debug, Clone)]
pub struct BreakpointInfo {
    pub id: String,
    pub breakpoint_type: BreakpointType,
    pub description: String,
    pub enabled: bool,
    pub hit_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointType {
    Task,
    Conditional,
    Loop,
    Watch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_breakpoints() {
        let mut manager = BreakpointManager::new();

        manager.add_task_breakpoint("task1".to_string());
        manager.add_task_breakpoint("task2".to_string());

        assert!(manager.should_break_on_task("task1"));
        assert!(manager.should_break_on_task("task2"));
        assert!(!manager.should_break_on_task("task3"));

        manager.remove_task_breakpoint("task1");
        assert!(!manager.should_break_on_task("task1"));
    }

    #[test]
    fn test_conditional_breakpoints() {
        let mut manager = BreakpointManager::new();

        let id = manager.add_conditional_breakpoint(
            BreakCondition::TaskStatus {
                task_id: "task1".to_string(),
                status: TaskStatus::Failed,
            },
            Some("Break on task1 failure".to_string()),
        );

        let variables = HashMap::new();

        // Should not break on success
        assert!(manager
            .check_conditional("task1", TaskStatus::Completed, &variables)
            .is_none());

        // Should break on failure
        let result = manager.check_conditional("task1", TaskStatus::Failed, &variables);
        assert_eq!(result, Some(id.clone()));

        // Hit count should increment
        assert_eq!(manager.conditional_breakpoints[&id].hit_count, 1);
    }

    #[test]
    fn test_loop_breakpoints() {
        let mut manager = BreakpointManager::new();

        manager.add_loop_breakpoint("loop_task".to_string(), 5);
        manager.add_loop_breakpoint("loop_task".to_string(), 10);

        assert!(manager.should_break_on_iteration("loop_task", 5));
        assert!(manager.should_break_on_iteration("loop_task", 10));
        assert!(!manager.should_break_on_iteration("loop_task", 7));

        manager.remove_loop_breakpoint("loop_task", 5);
        assert!(!manager.should_break_on_iteration("loop_task", 5));
    }

    #[test]
    fn test_watch_breakpoints() {
        let mut manager = BreakpointManager::new();

        let scope = VariableScope::Task("task1".to_string());
        manager.add_watch(
            scope.clone(),
            "counter".to_string(),
            WatchCondition::AnyChange,
        );

        // First value change should trigger
        let result = manager.check_watch(&scope, "counter", &serde_json::json!(1));
        assert!(result.is_some());

        // Same value should not trigger
        let result = manager.check_watch(&scope, "counter", &serde_json::json!(1));
        assert!(result.is_none());

        // Different value should trigger
        let result = manager.check_watch(&scope, "counter", &serde_json::json!(2));
        assert!(result.is_some());
    }

    #[test]
    fn test_enable_disable() {
        let mut manager = BreakpointManager::new();

        manager.add_task_breakpoint("task1".to_string());
        assert!(manager.should_break_on_task("task1"));

        manager.disable();
        assert!(!manager.should_break_on_task("task1"));

        manager.enable();
        assert!(manager.should_break_on_task("task1"));
    }

    #[test]
    fn test_list_all() {
        let mut manager = BreakpointManager::new();

        manager.add_task_breakpoint("task1".to_string());
        manager.add_loop_breakpoint("loop1".to_string(), 5);
        manager.add_conditional_breakpoint(
            BreakCondition::OnError,
            Some("Break on any error".to_string()),
        );

        let breakpoints = manager.list_all();
        assert_eq!(breakpoints.len(), 3);
    }

    #[test]
    fn test_clear_all() {
        let mut manager = BreakpointManager::new();

        manager.add_task_breakpoint("task1".to_string());
        manager.add_loop_breakpoint("loop1".to_string(), 5);

        assert_eq!(manager.count(), 2);

        manager.clear_all();
        assert_eq!(manager.count(), 0);
    }
}

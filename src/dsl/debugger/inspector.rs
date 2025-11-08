//! Inspector API
//!
//! Provides introspection capabilities for debugging:
//! - Variable inspection across all scopes
//! - Task state and execution details
//! - Call stack visualization
//! - Side effect history
//! - Execution timeline
use super::breakpoints::VariableScope;
use super::side_effects::{SideEffect, SideEffectType};
use super::state::DebuggerState;
use crate::dsl::state::{TaskOutput, WorkflowState};
use crate::dsl::task_graph::TaskStatus;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;

/// Inspector for runtime state introspection
pub struct Inspector {
    debugger: Arc<Mutex<DebuggerState>>,
    state: Arc<Mutex<Option<WorkflowState>>>,
}

impl Inspector {
    /// Create a new inspector
    pub fn new(
        debugger: Arc<Mutex<DebuggerState>>,
        state: Arc<Mutex<Option<WorkflowState>>>,
    ) -> Self {
        Self { debugger, state }
    }

    /// Get current execution position
    pub async fn current_position(&self) -> ExecutionPosition {
        let debugger = self.debugger.lock().await;

        ExecutionPosition {
            current_task: debugger.pointer.current_task.clone(),
            loop_position: debugger.pointer.loop_position.clone(),
            call_stack: debugger.pointer.execution_stack.clone(),
            step_count: debugger.step_count,
        }
    }

    /// Inspect all variables in scope
    pub async fn inspect_variables(
        &self,
        _scope_filter: Option<VariableScope>,
    ) -> VariableSnapshot {
        let debugger = self.debugger.lock().await;
        let state_guard = self.state.lock().await;

        let mut snapshot = VariableSnapshot {
            workflow_vars: HashMap::new(),
            agent_vars: HashMap::new(),
            task_vars: HashMap::new(),
            loop_vars: HashMap::new(),
        };

        // Get workflow-level variables from state
        if let Some(ref state) = *state_guard {
            // Extract from metadata (if stored there)
            for (key, value) in &state.metadata {
                if key.starts_with("workflow.") {
                    let var_name = key.strip_prefix("workflow.").unwrap();
                    snapshot
                        .workflow_vars
                        .insert(var_name.to_string(), value.clone());
                }
            }

            // Extract loop variables
            for (task_id, loop_state) in &state.loop_states {
                for (var_name, value) in &loop_state.loop_variables {
                    snapshot
                        .loop_vars
                        .entry(task_id.clone())
                        .or_default()
                        .insert(var_name.clone(), value.clone());
                }
            }
        }

        // Get current frame variables
        if let Some(frame) = debugger.pointer.current_frame() {
            snapshot
                .task_vars
                .insert(frame.task_id.clone(), frame.local_vars.clone());
        }

        snapshot
    }

    /// Get task execution details
    pub async fn inspect_task(&self, task_id: &str) -> Option<TaskInspection> {
        let state_guard = self.state.lock().await;

        if let Some(ref state) = *state_guard {
            let status = state.get_task_status(task_id)?;

            let duration = state
                .task_start_times
                .get(task_id)
                .and_then(|start_time| {
                    state
                        .task_end_times
                        .get(task_id)
                        .map(|end_time| end_time.duration_since(*start_time).ok())
                })
                .flatten();

            let error = state.task_errors.get(task_id).cloned();

            let outputs = state.get_task_output(task_id).cloned();

            Some(TaskInspection {
                task_id: task_id.to_string(),
                status,
                inputs: HashMap::new(), // TODO: Extract from task spec
                outputs,
                duration,
                error,
                subtasks: Vec::new(),     // TODO: Extract from task graph
                dependencies: Vec::new(), // TODO: Extract from task graph
                attempts: state.get_task_attempts(task_id),
            })
        } else {
            None
        }
    }

    /// Get execution call stack
    pub async fn call_stack(&self) -> Vec<super::pointer::ExecutionFrame> {
        let debugger = self.debugger.lock().await;
        debugger.pointer.execution_stack.clone()
    }

    /// Get call stack as formatted string
    pub async fn call_stack_string(&self) -> String {
        let debugger = self.debugger.lock().await;
        debugger.pointer.call_stack_string()
    }

    /// Get side effect history
    pub async fn side_effects(&self, filter: Option<SideEffectFilter>) -> Vec<SideEffect> {
        let debugger = self.debugger.lock().await;

        let all_effects = debugger.side_effects.all_effects();

        if let Some(filter) = filter {
            all_effects
                .iter()
                .filter(|e| filter.matches(e))
                .cloned()
                .collect()
        } else {
            all_effects.to_vec()
        }
    }

    /// Get side effect summary
    pub async fn side_effect_summary(&self) -> HashMap<String, usize> {
        let debugger = self.debugger.lock().await;
        debugger.side_effects.summary()
    }

    /// Get execution timeline
    pub async fn timeline(&self) -> ExecutionTimeline {
        let debugger = self.debugger.lock().await;
        let state_guard = self.state.lock().await;

        let mut events = Vec::new();

        // Add task events from state
        if let Some(ref state) = *state_guard {
            for (task_id, status) in &state.task_statuses {
                if let Some(&start_time) = state.task_start_times.get(task_id) {
                    events.push(TimelineEvent {
                        timestamp: start_time,
                        event_type: EventType::TaskStarted {
                            task_id: task_id.clone(),
                        },
                    });
                }

                if *status == TaskStatus::Completed || *status == TaskStatus::Failed {
                    if let Some(&end_time) = state.task_end_times.get(task_id) {
                        events.push(TimelineEvent {
                            timestamp: end_time,
                            event_type: if *status == TaskStatus::Completed {
                                EventType::TaskCompleted {
                                    task_id: task_id.clone(),
                                }
                            } else {
                                EventType::TaskFailed {
                                    task_id: task_id.clone(),
                                    error: state.task_errors.get(task_id).cloned(),
                                }
                            },
                        });
                    }
                }
            }
        }

        // Add side effect events
        for effect in debugger.side_effects.all_effects() {
            events.push(TimelineEvent {
                timestamp: effect.timestamp,
                event_type: EventType::SideEffect {
                    task_id: effect.task_id.clone(),
                    effect_type: format!("{:?}", effect.effect_type),
                },
            });
        }

        // Sort by timestamp
        events.sort_by_key(|e| e.timestamp);

        ExecutionTimeline { events }
    }

    /// Get debugger status
    pub async fn status(&self) -> super::state::DebuggerStatus {
        let debugger = self.debugger.lock().await;
        debugger.status_summary()
    }

    /// Get snapshot count
    pub async fn snapshot_count(&self) -> usize {
        let debugger = self.debugger.lock().await;
        debugger.history.len()
    }

    /// Get all snapshots
    pub async fn snapshots(&self) -> Vec<SnapshotInfo> {
        let debugger = self.debugger.lock().await;

        debugger
            .history
            .all()
            .iter()
            .map(|s| SnapshotInfo {
                id: s.metadata.id,
                description: s.metadata.description.clone(),
                elapsed: s.metadata.elapsed,
                task: s.pointer.current_task.clone(),
            })
            .collect()
    }
}

/// Current execution position
#[derive(Debug, Clone)]
pub struct ExecutionPosition {
    pub current_task: Option<String>,
    pub loop_position: Option<super::pointer::LoopPosition>,
    pub call_stack: Vec<super::pointer::ExecutionFrame>,
    pub step_count: usize,
}

/// Snapshot of all variables
#[derive(Debug, Clone)]
pub struct VariableSnapshot {
    pub workflow_vars: HashMap<String, serde_json::Value>,
    pub agent_vars: HashMap<String, HashMap<String, serde_json::Value>>,
    pub task_vars: HashMap<String, HashMap<String, serde_json::Value>>,
    pub loop_vars: HashMap<String, HashMap<String, serde_json::Value>>,
}

impl VariableSnapshot {
    /// Get total variable count
    pub fn total_count(&self) -> usize {
        self.workflow_vars.len()
            + self.agent_vars.values().map(|m| m.len()).sum::<usize>()
            + self.task_vars.values().map(|m| m.len()).sum::<usize>()
            + self.loop_vars.values().map(|m| m.len()).sum::<usize>()
    }

    /// Find variable by name (searches all scopes)
    pub fn find(&self, name: &str) -> Vec<(VariableScope, &serde_json::Value)> {
        let mut results = Vec::new();

        // Search workflow vars
        if let Some(value) = self.workflow_vars.get(name) {
            results.push((VariableScope::Workflow, value));
        }

        // Search agent vars
        for (agent_id, vars) in &self.agent_vars {
            if let Some(value) = vars.get(name) {
                results.push((VariableScope::Agent(agent_id.clone()), value));
            }
        }

        // Search task vars
        for (task_id, vars) in &self.task_vars {
            if let Some(value) = vars.get(name) {
                results.push((VariableScope::Task(task_id.clone()), value));
            }
        }

        results
    }
}

/// Task inspection details
#[derive(Debug, Clone)]
pub struct TaskInspection {
    pub task_id: String,
    pub status: TaskStatus,
    pub inputs: HashMap<String, serde_json::Value>,
    pub outputs: Option<TaskOutput>,
    pub duration: Option<Duration>,
    pub error: Option<String>,
    pub subtasks: Vec<String>,
    pub dependencies: Vec<String>,
    pub attempts: u32,
}

/// Side effect filter
#[derive(Debug, Clone)]
pub struct SideEffectFilter {
    pub task_id: Option<String>,
    pub effect_type: Option<SideEffectFilterType>,
    pub compensated: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum SideEffectFilterType {
    FileOperations,
    StateChanges,
    VariableChanges,
    Commands,
    Network,
}

impl SideEffectFilter {
    pub fn matches(&self, effect: &SideEffect) -> bool {
        // Filter by task ID
        if let Some(ref task_id) = self.task_id {
            if &effect.task_id != task_id {
                return false;
            }
        }

        // Filter by compensation status
        if let Some(compensated) = self.compensated {
            if effect.compensated != compensated {
                return false;
            }
        }

        // Filter by effect type
        if let Some(ref filter_type) = self.effect_type {
            let matches_type = matches!(
                (&effect.effect_type, filter_type),
                (
                    SideEffectType::FileCreated { .. },
                    SideEffectFilterType::FileOperations
                ) | (
                    SideEffectType::FileModified { .. },
                    SideEffectFilterType::FileOperations
                ) | (
                    SideEffectType::FileDeleted { .. },
                    SideEffectFilterType::FileOperations
                ) | (
                    SideEffectType::DirectoryCreated { .. },
                    SideEffectFilterType::FileOperations
                ) | (
                    SideEffectType::DirectoryDeleted { .. },
                    SideEffectFilterType::FileOperations
                ) | (
                    SideEffectType::StateChanged { .. },
                    SideEffectFilterType::StateChanges
                ) | (
                    SideEffectType::VariableSet { .. },
                    SideEffectFilterType::VariableChanges
                ) | (
                    SideEffectType::CommandExecuted { .. },
                    SideEffectFilterType::Commands
                ) | (
                    SideEffectType::NetworkRequest { .. },
                    SideEffectFilterType::Network
                )
            );

            if !matches_type {
                return false;
            }
        }

        true
    }
}

/// Execution timeline
#[derive(Debug, Clone)]
pub struct ExecutionTimeline {
    pub events: Vec<TimelineEvent>,
}

impl ExecutionTimeline {
    /// Get events for a specific task
    pub fn events_for_task(&self, task_id: &str) -> Vec<&TimelineEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type.task_id() == Some(task_id))
            .collect()
    }

    /// Get event count
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if timeline is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Timeline event
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    pub timestamp: SystemTime,
    pub event_type: EventType,
}

/// Event type
#[derive(Debug, Clone)]
pub enum EventType {
    TaskStarted {
        task_id: String,
    },
    TaskCompleted {
        task_id: String,
    },
    TaskFailed {
        task_id: String,
        error: Option<String>,
    },
    SideEffect {
        task_id: String,
        effect_type: String,
    },
    BreakpointHit {
        breakpoint_id: String,
    },
    Snapshot {
        snapshot_id: usize,
    },
}

impl EventType {
    pub fn task_id(&self) -> Option<&str> {
        match self {
            EventType::TaskStarted { task_id }
            | EventType::TaskCompleted { task_id }
            | EventType::TaskFailed { task_id, .. }
            | EventType::SideEffect { task_id, .. } => Some(task_id),
            _ => None,
        }
    }
}

/// Snapshot information
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    pub id: usize,
    pub description: String,
    pub elapsed: Duration,
    pub task: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::debugger::DebuggerState;

    #[tokio::test]
    async fn test_inspector_position() {
        let debugger = Arc::new(Mutex::new(DebuggerState::new()));
        let state = Arc::new(Mutex::new(None));
        let inspector = Inspector::new(debugger.clone(), state);

        {
            let mut dbg = debugger.lock().await;
            dbg.enter_task("task1".to_string(), None);
        }

        let position = inspector.current_position().await;
        assert_eq!(position.current_task, Some("task1".to_string()));
        assert_eq!(position.call_stack.len(), 1);
    }

    #[tokio::test]
    async fn test_inspector_variables() {
        let debugger = Arc::new(Mutex::new(DebuggerState::new()));
        let state = Arc::new(Mutex::new(None));
        let inspector = Inspector::new(debugger, state);

        let snapshot = inspector.inspect_variables(None).await;
        assert_eq!(snapshot.total_count(), 0);
    }

    #[tokio::test]
    async fn test_variable_snapshot_find() {
        let mut snapshot = VariableSnapshot {
            workflow_vars: HashMap::new(),
            agent_vars: HashMap::new(),
            task_vars: HashMap::new(),
            loop_vars: HashMap::new(),
        };

        snapshot
            .workflow_vars
            .insert("key1".to_string(), serde_json::json!("value1"));

        let results = snapshot.find("key1");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, VariableScope::Workflow);
    }
}

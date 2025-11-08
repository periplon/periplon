//! Execution Pointer and Snapshot System
//!
//! Tracks current execution position, maintains execution history for time-travel debugging,
//! and provides snapshot/restore capabilities.
use crate::dsl::state::WorkflowState;
use crate::dsl::task_graph::TaskStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Current execution position in the workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionPointer {
    /// Current task being executed (None if not started or completed)
    pub current_task: Option<String>,

    /// Current loop iteration (if inside a loop)
    pub loop_position: Option<LoopPosition>,

    /// Stack of execution frames (for nested subtasks)
    pub execution_stack: Vec<ExecutionFrame>,

    /// Execution mode
    pub mode: ExecutionMode,
}

/// Loop execution position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoopPosition {
    /// Task ID that owns the loop
    pub task_id: String,

    /// Current iteration index (0-based)
    pub iteration: usize,

    /// Total iterations (if known)
    pub total_iterations: Option<usize>,
}

/// Execution frame representing a task in the call stack
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionFrame {
    /// Task ID
    pub task_id: String,

    /// Parent task ID (None for root tasks)
    pub parent_task: Option<String>,

    /// Depth in execution tree (0 for root)
    pub depth: usize,

    /// Frame-local variables
    pub local_vars: HashMap<String, serde_json::Value>,
}

/// Execution mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Normal execution
    Normal,

    /// Debug mode - paused
    Paused,

    /// Stepping through tasks
    Stepping,

    /// Replaying from history
    Replaying,
}

impl Default for ExecutionPointer {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionPointer {
    /// Create a new execution pointer
    pub fn new() -> Self {
        Self {
            current_task: None,
            loop_position: None,
            execution_stack: Vec::new(),
            mode: ExecutionMode::Normal,
        }
    }

    /// Enter a task (push onto execution stack)
    pub fn enter_task(&mut self, task_id: String, parent_task: Option<String>) {
        let depth = self.execution_stack.len();
        let frame = ExecutionFrame {
            task_id: task_id.clone(),
            parent_task,
            depth,
            local_vars: HashMap::new(),
        };

        self.execution_stack.push(frame);
        self.current_task = Some(task_id);
    }

    /// Exit current task (pop from execution stack)
    pub fn exit_task(&mut self) -> Option<ExecutionFrame> {
        let frame = self.execution_stack.pop();
        self.current_task = self.execution_stack.last().map(|f| f.task_id.clone());
        frame
    }

    /// Enter a loop
    pub fn enter_loop(&mut self, task_id: String, total_iterations: Option<usize>) {
        self.loop_position = Some(LoopPosition {
            task_id,
            iteration: 0,
            total_iterations,
        });
    }

    /// Advance loop iteration
    pub fn next_iteration(&mut self) -> Option<usize> {
        if let Some(ref mut pos) = self.loop_position {
            pos.iteration += 1;
            Some(pos.iteration)
        } else {
            None
        }
    }

    /// Exit loop
    pub fn exit_loop(&mut self) -> Option<LoopPosition> {
        self.loop_position.take()
    }

    /// Get current execution depth
    pub fn depth(&self) -> usize {
        self.execution_stack.len()
    }

    /// Get current frame
    pub fn current_frame(&self) -> Option<&ExecutionFrame> {
        self.execution_stack.last()
    }

    /// Get mutable current frame
    pub fn current_frame_mut(&mut self) -> Option<&mut ExecutionFrame> {
        self.execution_stack.last_mut()
    }

    /// Set variable in current frame
    pub fn set_local_var(&mut self, name: String, value: serde_json::Value) {
        if let Some(frame) = self.current_frame_mut() {
            frame.local_vars.insert(name, value);
        }
    }

    /// Get variable from current frame
    pub fn get_local_var(&self, name: &str) -> Option<&serde_json::Value> {
        self.current_frame()
            .and_then(|frame| frame.local_vars.get(name))
    }

    /// Get the full call stack as a string for display
    pub fn call_stack_string(&self) -> String {
        self.execution_stack
            .iter()
            .map(|frame| {
                let indent = "  ".repeat(frame.depth);
                format!("{}└─ {}", indent, frame.task_id)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Check if currently inside a loop
    pub fn is_in_loop(&self) -> bool {
        self.loop_position.is_some()
    }
}

/// Snapshot of execution state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSnapshot {
    /// Timestamp when snapshot was taken
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,

    /// Execution pointer state
    pub pointer: ExecutionPointer,

    /// Workflow state checkpoint
    pub state_checkpoint: StateCheckpoint,

    /// Snapshot metadata
    pub metadata: SnapshotMetadata,
}

/// State checkpoint for snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateCheckpoint {
    /// Task statuses
    pub task_statuses: HashMap<String, TaskStatus>,

    /// Task results
    pub task_results: HashMap<String, String>,

    /// Workflow variables
    pub workflow_vars: HashMap<String, serde_json::Value>,

    /// Agent variables
    pub agent_vars: HashMap<String, HashMap<String, serde_json::Value>>,

    /// Task variables
    pub task_vars: HashMap<String, HashMap<String, serde_json::Value>>,

    /// Loop states
    pub loop_states: HashMap<String, crate::dsl::state::LoopState>,
}

impl StateCheckpoint {
    /// Create checkpoint from workflow state
    pub fn from_workflow_state(state: &WorkflowState) -> Self {
        Self {
            task_statuses: state.task_statuses.clone(),
            task_results: state.task_results.clone(),
            workflow_vars: HashMap::new(), // TODO: Extract from state
            agent_vars: HashMap::new(),
            task_vars: HashMap::new(),
            loop_states: state.loop_states.clone(),
        }
    }

    /// Apply checkpoint to workflow state
    pub fn apply_to_state(&self, state: &mut WorkflowState) {
        state.task_statuses = self.task_statuses.clone();
        state.task_results = self.task_results.clone();
        state.loop_states = self.loop_states.clone();
        // TODO: Apply variables
    }
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Snapshot ID
    pub id: usize,

    /// Description of what happened at this point
    pub description: String,

    /// Elapsed time since workflow start
    #[serde(skip)]
    pub elapsed: Duration,
}

impl ExecutionSnapshot {
    /// Create a new snapshot
    pub fn new(
        id: usize,
        pointer: ExecutionPointer,
        state: &WorkflowState,
        description: String,
        elapsed: Duration,
    ) -> Self {
        Self {
            timestamp: Instant::now(),
            pointer,
            state_checkpoint: StateCheckpoint::from_workflow_state(state),
            metadata: SnapshotMetadata {
                id,
                description,
                elapsed,
            },
        }
    }
}

/// Execution history for time-travel debugging
#[derive(Debug, Clone)]
pub struct ExecutionHistory {
    /// All execution snapshots
    snapshots: Vec<ExecutionSnapshot>,

    /// Current position in history (for back/forward navigation)
    current_index: usize,

    /// Maximum history size
    max_size: usize,
}

impl Default for ExecutionHistory {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl ExecutionHistory {
    /// Create new execution history
    pub fn new(max_size: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            current_index: 0,
            max_size,
        }
    }

    /// Add a snapshot to history
    pub fn push(&mut self, snapshot: ExecutionSnapshot) {
        // If we're not at the end of history, truncate everything after current position
        if self.current_index < self.snapshots.len() {
            self.snapshots.truncate(self.current_index);
        }

        // Add new snapshot
        self.snapshots.push(snapshot);

        // Enforce max size (keep most recent)
        if self.snapshots.len() > self.max_size {
            self.snapshots
                .drain(0..self.snapshots.len() - self.max_size);
        }

        // Update current index to point to the new snapshot
        self.current_index = self.snapshots.len();
    }

    /// Move back in history
    pub fn back(&mut self, steps: usize) -> Option<&ExecutionSnapshot> {
        if self.current_index > steps {
            self.current_index -= steps;
            self.snapshots.get(self.current_index)
        } else if !self.snapshots.is_empty() {
            self.current_index = 0;
            self.snapshots.first()
        } else {
            None
        }
    }

    /// Move forward in history
    pub fn forward(&mut self, steps: usize) -> Option<&ExecutionSnapshot> {
        let new_index = self.current_index + steps;
        if new_index < self.snapshots.len() {
            self.current_index = new_index;
            self.snapshots.get(self.current_index)
        } else if !self.snapshots.is_empty() {
            self.current_index = self.snapshots.len() - 1;
            self.snapshots.last()
        } else {
            None
        }
    }

    /// Jump to specific snapshot
    pub fn goto(&mut self, index: usize) -> Option<&ExecutionSnapshot> {
        if index < self.snapshots.len() {
            self.current_index = index;
            self.snapshots.get(index)
        } else {
            None
        }
    }

    /// Get current snapshot
    pub fn current(&self) -> Option<&ExecutionSnapshot> {
        if self.current_index > 0 && self.current_index <= self.snapshots.len() {
            self.snapshots.get(self.current_index - 1)
        } else {
            None
        }
    }

    /// Get all snapshots
    pub fn all(&self) -> &[ExecutionSnapshot] {
        &self.snapshots
    }

    /// Get snapshot count
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Get current position in history
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.current_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_pointer_basic() {
        let mut pointer = ExecutionPointer::new();

        assert_eq!(pointer.current_task, None);
        assert_eq!(pointer.depth(), 0);

        pointer.enter_task("task1".to_string(), None);
        assert_eq!(pointer.current_task, Some("task1".to_string()));
        assert_eq!(pointer.depth(), 1);

        pointer.exit_task();
        assert_eq!(pointer.current_task, None);
        assert_eq!(pointer.depth(), 0);
    }

    #[test]
    fn test_nested_tasks() {
        let mut pointer = ExecutionPointer::new();

        pointer.enter_task("parent".to_string(), None);
        pointer.enter_task("child".to_string(), Some("parent".to_string()));

        assert_eq!(pointer.current_task, Some("child".to_string()));
        assert_eq!(pointer.depth(), 2);

        let frame = pointer.exit_task().unwrap();
        assert_eq!(frame.task_id, "child");
        assert_eq!(pointer.current_task, Some("parent".to_string()));
    }

    #[test]
    fn test_loop_tracking() {
        let mut pointer = ExecutionPointer::new();

        pointer.enter_loop("loop_task".to_string(), Some(10));
        assert!(pointer.is_in_loop());

        assert_eq!(pointer.next_iteration(), Some(1));
        assert_eq!(pointer.next_iteration(), Some(2));

        pointer.exit_loop();
        assert!(!pointer.is_in_loop());
    }

    #[test]
    fn test_local_variables() {
        let mut pointer = ExecutionPointer::new();

        pointer.enter_task("task1".to_string(), None);
        pointer.set_local_var("key".to_string(), serde_json::json!("value"));

        assert_eq!(
            pointer.get_local_var("key"),
            Some(&serde_json::json!("value"))
        );

        pointer.exit_task();
        assert_eq!(pointer.get_local_var("key"), None);
    }

    #[test]
    fn test_execution_history() {
        let mut history = ExecutionHistory::new(10);

        assert!(history.is_empty());
        assert_eq!(history.len(), 0);

        let state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

        // Add snapshots
        for i in 0..5 {
            let snapshot = ExecutionSnapshot::new(
                i,
                ExecutionPointer::new(),
                &state,
                format!("Step {}", i),
                Duration::from_secs(i as u64),
            );
            history.push(snapshot);
        }

        assert_eq!(history.len(), 5);
        assert_eq!(history.current_index(), 5);

        // Navigate back
        let snapshot = history.back(2).unwrap();
        assert_eq!(snapshot.metadata.id, 3);

        // Navigate forward
        let snapshot = history.forward(1).unwrap();
        assert_eq!(snapshot.metadata.id, 4);
    }

    #[test]
    fn test_history_truncation_on_new_path() {
        let mut history = ExecutionHistory::new(10);
        let state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

        // Add 5 snapshots
        for i in 0..5 {
            let snapshot = ExecutionSnapshot::new(
                i,
                ExecutionPointer::new(),
                &state,
                format!("Step {}", i),
                Duration::from_secs(i as u64),
            );
            history.push(snapshot);
        }

        // Go back 2 steps
        history.back(2);
        assert_eq!(history.current_index(), 3);

        // Add new snapshot - should truncate everything after current position
        let snapshot = ExecutionSnapshot::new(
            100,
            ExecutionPointer::new(),
            &state,
            "New path".to_string(),
            Duration::from_secs(100),
        );
        history.push(snapshot);

        assert_eq!(history.len(), 4); // 0, 1, 2, 100
        assert_eq!(history.current_index(), 4);
    }

    #[test]
    fn test_history_max_size() {
        let mut history = ExecutionHistory::new(3);
        let state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

        // Add 5 snapshots (exceeds max size)
        for i in 0..5 {
            let snapshot = ExecutionSnapshot::new(
                i,
                ExecutionPointer::new(),
                &state,
                format!("Step {}", i),
                Duration::from_secs(i as u64),
            );
            history.push(snapshot);
        }

        // Should only keep last 3
        assert_eq!(history.len(), 3);
        assert_eq!(history.snapshots[0].metadata.id, 2);
        assert_eq!(history.snapshots[2].metadata.id, 4);
    }
}

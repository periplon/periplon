//! Debugger State Machine
//!
//! Central state management for debugging, coordinating execution pointer,
//! breakpoints, side effects, and execution control.
use super::breakpoints::BreakpointManager;
use super::pointer::{ExecutionHistory, ExecutionMode, ExecutionPointer, ExecutionSnapshot};
use super::side_effects::SideEffectJournal;
use crate::dsl::state::WorkflowState;
use crate::error::{Error, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Debugger state machine
pub struct DebuggerState {
    /// Current execution mode
    pub mode: DebugMode,

    /// Execution pointer tracking current position
    pub pointer: ExecutionPointer,

    /// Breakpoint manager
    pub breakpoints: BreakpointManager,

    /// Side effect journal for undo capability
    pub side_effects: SideEffectJournal,

    /// Execution history for time-travel
    pub history: ExecutionHistory,

    /// Current step mode
    pub step_mode: StepMode,

    /// Workflow start time
    pub start_time: Option<Instant>,

    /// Number of steps taken
    pub step_count: usize,

    /// Last breakpoint hit
    pub last_breakpoint: Option<String>,
}

/// Debug mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugMode {
    /// Normal execution without debugging
    Running,

    /// Paused at breakpoint
    Paused,

    /// Single-stepping through execution
    Stepping,

    /// Time-traveling through history
    TimeTraveling,

    /// Suspended (waiting for user input)
    Suspended,
}

/// Step execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    /// Execute one task
    StepTask,

    /// Step into subtasks
    StepInto,

    /// Step over subtasks (complete them without pausing)
    StepOver,

    /// Step out of current subtask context
    StepOut,

    /// Step one loop iteration
    StepIteration,

    /// Continue until next breakpoint or completion
    Continue,

    /// Step backward (undo last step)
    StepBack,

    /// Step forward (redo undone step)
    StepForward,
}

impl Default for DebuggerState {
    fn default() -> Self {
        Self::new()
    }
}

impl DebuggerState {
    /// Create new debugger state
    pub fn new() -> Self {
        Self {
            mode: DebugMode::Running,
            pointer: ExecutionPointer::new(),
            breakpoints: BreakpointManager::new(),
            side_effects: SideEffectJournal::new(),
            history: ExecutionHistory::default(),
            step_mode: StepMode::Continue,
            start_time: None,
            step_count: 0,
            last_breakpoint: None,
        }
    }

    /// Start debugging session
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.mode = DebugMode::Running;
        self.pointer.mode = ExecutionMode::Normal;
    }

    /// Pause execution
    pub fn pause(&mut self) {
        self.mode = DebugMode::Paused;
        self.pointer.mode = ExecutionMode::Paused;
    }

    /// Resume execution
    pub fn resume(&mut self) {
        self.mode = DebugMode::Running;
        self.pointer.mode = ExecutionMode::Normal;
        self.last_breakpoint = None;
    }

    /// Set step mode and enter stepping
    pub fn set_step_mode(&mut self, mode: StepMode) {
        self.step_mode = mode;
        self.mode = DebugMode::Stepping;
        self.pointer.mode = ExecutionMode::Stepping;
    }

    /// Check if execution should pause
    ///
    /// Returns true if:
    /// - A breakpoint was hit
    /// - Step mode requires pausing
    pub fn should_pause(&self, task_id: &str) -> bool {
        match self.mode {
            DebugMode::Paused | DebugMode::Suspended => true,

            DebugMode::Stepping => match self.step_mode {
                StepMode::StepTask | StepMode::StepInto => true,
                StepMode::StepOver => {
                    // Only pause if we're not deeper in the call stack
                    true // TODO: Check call stack depth
                }
                StepMode::StepOut => {
                    // Only pause when we return to parent level
                    false // TODO: Check if we've exited
                }
                _ => false,
            },

            DebugMode::Running => {
                // Check for breakpoints
                self.breakpoints.should_break_on_task(task_id)
            }

            DebugMode::TimeTraveling => false,
        }
    }

    /// Record task entry
    pub fn enter_task(&mut self, task_id: String, parent_task: Option<String>) {
        self.pointer.enter_task(task_id, parent_task);
        self.step_count += 1;
    }

    /// Record task exit
    pub fn exit_task(&mut self) {
        self.pointer.exit_task();
    }

    /// Record loop entry
    pub fn enter_loop(&mut self, task_id: String, total_iterations: Option<usize>) {
        self.pointer.enter_loop(task_id, total_iterations);
    }

    /// Record loop iteration
    pub fn next_iteration(&mut self) -> Option<usize> {
        let iteration = self.pointer.next_iteration()?;

        // Check for loop breakpoints
        let should_break = if let Some(ref pos) = self.pointer.loop_position {
            let task_id = pos.task_id.clone();
            let should_break = self
                .breakpoints
                .should_break_on_iteration(&task_id, iteration);
            if should_break {
                self.last_breakpoint = Some(format!("loop:{}:{}", task_id, iteration));
            }
            should_break
        } else {
            false
        };

        if should_break {
            self.pause();
        }

        Some(iteration)
    }

    /// Record loop exit
    pub fn exit_loop(&mut self) {
        self.pointer.exit_loop();
    }

    /// Create snapshot of current state
    pub fn create_snapshot(&mut self, state: &WorkflowState, description: String) {
        let elapsed = self
            .start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO);

        let snapshot = ExecutionSnapshot::new(
            self.history.len(),
            self.pointer.clone(),
            state,
            description,
            elapsed,
        );

        self.history.push(snapshot);
    }

    /// Navigate back in history
    pub async fn step_back(&mut self, steps: usize, state: &mut WorkflowState) -> Result<()> {
        let snapshot = self
            .history
            .back(steps)
            .ok_or_else(|| Error::InvalidInput("Cannot step back: no history".to_string()))?;

        // Apply snapshot to current state
        snapshot.state_checkpoint.apply_to_state(state);
        self.pointer = snapshot.pointer.clone();
        self.mode = DebugMode::TimeTraveling;

        // Compensate side effects
        let effect_count = self.side_effects.len();
        if effect_count > 0 {
            self.side_effects
                .compensate_since(snapshot.metadata.id)
                .await?;
        }

        Ok(())
    }

    /// Navigate forward in history
    pub fn step_forward(&mut self, steps: usize, state: &mut WorkflowState) -> Result<()> {
        let snapshot = self.history.forward(steps).ok_or_else(|| {
            Error::InvalidInput("Cannot step forward: at end of history".to_string())
        })?;

        // Apply snapshot
        snapshot.state_checkpoint.apply_to_state(state);
        self.pointer = snapshot.pointer.clone();

        Ok(())
    }

    /// Jump to specific snapshot
    pub fn goto_snapshot(&mut self, index: usize, state: &mut WorkflowState) -> Result<()> {
        let snapshot = self
            .history
            .goto(index)
            .ok_or_else(|| Error::InvalidInput(format!("Invalid snapshot index: {}", index)))?;

        snapshot.state_checkpoint.apply_to_state(state);
        self.pointer = snapshot.pointer.clone();
        self.mode = DebugMode::TimeTraveling;

        Ok(())
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Get debugger status summary
    pub fn status_summary(&self) -> DebuggerStatus {
        DebuggerStatus {
            mode: self.mode,
            step_mode: self.step_mode,
            current_task: self.pointer.current_task.clone(),
            call_stack_depth: self.pointer.depth(),
            breakpoint_count: self.breakpoints.count(),
            side_effect_count: self.side_effects.len(),
            snapshot_count: self.history.len(),
            step_count: self.step_count,
            elapsed: self.elapsed(),
            last_breakpoint: self.last_breakpoint.clone(),
        }
    }

    /// Reset debugger state
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Debugger status summary
#[derive(Debug, Clone)]
pub struct DebuggerStatus {
    pub mode: DebugMode,
    pub step_mode: StepMode,
    pub current_task: Option<String>,
    pub call_stack_depth: usize,
    pub breakpoint_count: usize,
    pub side_effect_count: usize,
    pub snapshot_count: usize,
    pub step_count: usize,
    pub elapsed: Duration,
    pub last_breakpoint: Option<String>,
}

impl std::fmt::Display for DebuggerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Debugger Status:")?;
        writeln!(f, "  Mode: {:?}", self.mode)?;
        writeln!(f, "  Step Mode: {:?}", self.step_mode)?;
        writeln!(
            f,
            "  Current Task: {}",
            self.current_task.as_deref().unwrap_or("None")
        )?;
        writeln!(f, "  Call Stack Depth: {}", self.call_stack_depth)?;
        writeln!(f, "  Breakpoints: {}", self.breakpoint_count)?;
        writeln!(f, "  Side Effects: {}", self.side_effect_count)?;
        writeln!(f, "  Snapshots: {}", self.snapshot_count)?;
        writeln!(f, "  Steps: {}", self.step_count)?;
        writeln!(f, "  Elapsed: {:.2}s", self.elapsed.as_secs_f64())?;
        if let Some(ref bp) = self.last_breakpoint {
            writeln!(f, "  Last Breakpoint: {}", bp)?;
        }
        Ok(())
    }
}

/// Thread-safe debugger state
pub type SharedDebuggerState = Arc<Mutex<DebuggerState>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_lifecycle() {
        let mut debugger = DebuggerState::new();

        assert_eq!(debugger.mode, DebugMode::Running);

        debugger.start();
        assert!(debugger.start_time.is_some());

        debugger.pause();
        assert_eq!(debugger.mode, DebugMode::Paused);

        debugger.resume();
        assert_eq!(debugger.mode, DebugMode::Running);
    }

    #[test]
    fn test_task_tracking() {
        let mut debugger = DebuggerState::new();

        debugger.enter_task("task1".to_string(), None);
        assert_eq!(debugger.pointer.current_task, Some("task1".to_string()));
        assert_eq!(debugger.pointer.depth(), 1);

        debugger.enter_task("task2".to_string(), Some("task1".to_string()));
        assert_eq!(debugger.pointer.depth(), 2);

        debugger.exit_task();
        assert_eq!(debugger.pointer.depth(), 1);
    }

    #[test]
    fn test_step_modes() {
        let mut debugger = DebuggerState::new();

        debugger.set_step_mode(StepMode::StepTask);
        assert_eq!(debugger.mode, DebugMode::Stepping);
        assert_eq!(debugger.step_mode, StepMode::StepTask);

        debugger.set_step_mode(StepMode::Continue);
        assert_eq!(debugger.step_mode, StepMode::Continue);
    }

    #[test]
    fn test_loop_tracking() {
        let mut debugger = DebuggerState::new();

        debugger.enter_loop("loop1".to_string(), Some(10));
        assert!(debugger.pointer.is_in_loop());

        debugger.next_iteration();
        debugger.next_iteration();

        debugger.exit_loop();
        assert!(!debugger.pointer.is_in_loop());
    }

    #[test]
    fn test_breakpoint_pause() {
        let mut debugger = DebuggerState::new();

        debugger
            .breakpoints
            .add_task_breakpoint("task1".to_string());

        assert!(debugger.should_pause("task1"));
        assert!(!debugger.should_pause("task2"));
    }

    #[test]
    fn test_status_summary() {
        let mut debugger = DebuggerState::new();

        debugger.start();
        debugger.enter_task("task1".to_string(), None);
        debugger
            .breakpoints
            .add_task_breakpoint("task2".to_string());

        let status = debugger.status_summary();

        assert_eq!(status.mode, DebugMode::Running);
        assert_eq!(status.current_task, Some("task1".to_string()));
        assert_eq!(status.call_stack_depth, 1);
        assert_eq!(status.breakpoint_count, 1);
    }

    #[tokio::test]
    async fn test_snapshot_history() {
        let mut debugger = DebuggerState::new();
        let state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

        debugger.create_snapshot(&state, "Initial state".to_string());
        debugger.enter_task("task1".to_string(), None);
        debugger.create_snapshot(&state, "After task1".to_string());

        assert_eq!(debugger.history.len(), 2);
    }
}

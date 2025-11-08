//! Debugger Module
//!
//! Comprehensive debugging infrastructure for DSL workflow execution including:
//! - Execution pointer tracking and call stack management
//! - Breakpoints (task, conditional, loop, variable watch)
//! - Side effect journaling and compensation (undo)
//! - Time-travel debugging with execution history
//! - State inspection and introspection APIs
pub mod breakpoints;
pub mod pointer;
pub mod side_effects;
pub mod state;

mod inspector;

// Re-export public API
pub use breakpoints::{
    BreakCondition, BreakpointInfo, BreakpointManager, BreakpointType, VariableScope,
    WatchCondition,
};
pub use inspector::{Inspector, TaskInspection, VariableSnapshot};
pub use pointer::{
    ExecutionFrame, ExecutionHistory, ExecutionMode, ExecutionPointer, ExecutionSnapshot,
};
pub use side_effects::{
    CompensationStrategy, DirectoryTree, SideEffect, SideEffectJournal, SideEffectType,
};
pub use state::{DebugMode, DebuggerState, DebuggerStatus, StepMode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_imports() {
        // Verify all public types are accessible
        let _pointer = ExecutionPointer::new();
        let _breakpoints = BreakpointManager::new();
        let _journal = SideEffectJournal::new();
        let _debugger = DebuggerState::new();
    }
}

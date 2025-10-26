//! Workflow State Persistence
//!
//! This module provides state management for workflows, enabling checkpointing,
//! resuming interrupted workflows, and tracking execution progress.

use crate::dsl::schema::{CleanupStrategy, TruncationStrategy};
use crate::dsl::task_graph::TaskStatus;
use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Loop execution state for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopState {
    /// Task ID that owns this loop
    pub task_id: String,
    /// Current iteration number (0-based)
    pub current_iteration: usize,
    /// Total number of iterations (known for Repeat, ForEach; None for While/RepeatUntil)
    pub total_iterations: Option<usize>,
    /// Current iterator value (for ForEach loops)
    pub iterator_value: Option<serde_json::Value>,
    /// Loop-specific variables
    pub loop_variables: HashMap<String, serde_json::Value>,
    /// Status of each iteration
    pub iteration_statuses: Vec<TaskStatus>,
    /// Loop start time
    pub started_at: DateTime<Utc>,
    /// Last iteration time
    pub last_iteration_at: Option<DateTime<Utc>>,
}

impl LoopState {
    /// Create a new loop state
    pub fn new(task_id: String, total_iterations: Option<usize>) -> Self {
        Self {
            task_id,
            current_iteration: 0,
            total_iterations,
            iterator_value: None,
            loop_variables: HashMap::new(),
            iteration_statuses: Vec::new(),
            started_at: Utc::now(),
            last_iteration_at: None,
        }
    }
}

/// Task output with metadata for context management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutput {
    /// Task ID
    pub task_id: String,
    /// Output type
    pub output_type: OutputType,
    /// Actual content (truncated or summarized)
    pub content: String,
    /// Original size in bytes
    pub original_size: usize,
    /// Was truncated/summarized?
    pub truncated: bool,
    /// Truncation strategy used
    pub strategy: TruncationStrategy,
    /// File path if stored externally
    pub file_path: Option<PathBuf>,
    /// Relevance score (0.0-1.0) for context injection
    pub relevance_score: f64,
    /// Last accessed timestamp (for LRU cleanup)
    pub last_accessed: SystemTime,
    /// Tasks that depend on this output
    pub depended_by: Vec<String>,
}

/// Output type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputType {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
    /// Combined stdout and stderr
    Combined,
    /// File output
    File,
    /// AI-generated summary
    Summary,
}

impl TaskOutput {
    /// Create a new task output
    pub fn new(
        task_id: String,
        output_type: OutputType,
        content: String,
        original_size: usize,
        truncated: bool,
        strategy: TruncationStrategy,
    ) -> Self {
        Self {
            task_id,
            output_type,
            content,
            original_size,
            truncated,
            strategy,
            file_path: None,
            relevance_score: 0.0,
            last_accessed: SystemTime::now(),
            depended_by: Vec::new(),
        }
    }

    /// Mark this output as accessed (updates LRU timestamp)
    pub fn mark_accessed(&mut self) {
        self.last_accessed = SystemTime::now();
    }

    /// Add a dependent task
    pub fn add_dependent(&mut self, task_id: String) {
        if !self.depended_by.contains(&task_id) {
            self.depended_by.push(task_id);
        }
    }

    /// Update relevance score
    pub fn set_relevance(&mut self, score: f64) {
        self.relevance_score = score.clamp(0.0, 1.0);
    }
}

/// Context metrics for observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetrics {
    /// Total bytes stored
    pub total_bytes: usize,
    /// Number of task outputs
    pub task_count: usize,
    /// Number of truncated outputs
    pub truncated_count: usize,
    /// Number of externally stored outputs
    pub external_count: usize,
    /// Average relevance score
    pub avg_relevance: f64,
    /// Last pruning time
    pub last_pruned_at: Option<SystemTime>,
}

impl Default for ContextMetrics {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            task_count: 0,
            truncated_count: 0,
            external_count: 0,
            avg_relevance: 0.0,
            last_pruned_at: None,
        }
    }
}

/// Workflow execution state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Workflow name
    pub workflow_name: String,
    /// Workflow version
    pub workflow_version: String,
    /// Task statuses
    pub task_statuses: HashMap<String, TaskStatus>,
    /// Task start times
    pub task_start_times: HashMap<String, SystemTime>,
    /// Task end times
    pub task_end_times: HashMap<String, SystemTime>,
    /// Task attempt counts (for retry tracking)
    pub task_attempts: HashMap<String, u32>,
    /// Task error messages
    pub task_errors: HashMap<String, String>,
    /// Task results/outputs (truncated summaries for context - deprecated, use task_outputs)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub task_results: HashMap<String, String>,
    /// Task outputs with metadata (new context management system)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub task_outputs: HashMap<String, TaskOutput>,
    /// Overall workflow status
    pub status: WorkflowStatus,
    /// Workflow start time
    pub started_at: SystemTime,
    /// Workflow end time (if completed)
    pub ended_at: Option<SystemTime>,
    /// Checkpoint timestamp
    pub checkpoint_at: SystemTime,
    /// Metadata for extensibility
    pub metadata: HashMap<String, serde_json::Value>,
    /// Loop states for iterative tasks
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub loop_states: HashMap<String, LoopState>,
    /// Loop results collected from iterations
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub loop_results: HashMap<String, Vec<serde_json::Value>>,
}

/// Overall workflow execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is currently running
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow was paused/checkpointed
    Paused,
}

impl WorkflowState {
    /// Create a new workflow state
    pub fn new(workflow_name: String, workflow_version: String) -> Self {
        Self {
            workflow_name,
            workflow_version,
            task_statuses: HashMap::new(),
            task_start_times: HashMap::new(),
            task_end_times: HashMap::new(),
            task_attempts: HashMap::new(),
            task_errors: HashMap::new(),
            task_results: HashMap::new(),
            task_outputs: HashMap::new(),
            status: WorkflowStatus::Running,
            started_at: SystemTime::now(),
            ended_at: None,
            checkpoint_at: SystemTime::now(),
            metadata: HashMap::new(),
            loop_states: HashMap::new(),
            loop_results: HashMap::new(),
        }
    }

    /// Update task status
    ///
    /// # Performance
    ///
    /// Caches timestamp to avoid multiple SystemTime::now() calls
    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) {
        let now = SystemTime::now(); // Cache timestamp

        self.task_statuses.insert(task_id.to_string(), status);

        match status {
            TaskStatus::Running => {
                if !self.task_start_times.contains_key(task_id) {
                    self.task_start_times.insert(task_id.to_string(), now);
                }
            }
            TaskStatus::Completed | TaskStatus::Failed => {
                self.task_end_times.insert(task_id.to_string(), now);
            }
            _ => {}
        }

        self.checkpoint_at = now; // Use cached timestamp
    }

    /// Record task attempt
    pub fn record_task_attempt(&mut self, task_id: &str) {
        let attempts = self.task_attempts.entry(task_id.to_string()).or_insert(0);
        *attempts += 1;
        self.checkpoint_at = SystemTime::now();
    }

    /// Record task error
    pub fn record_task_error(&mut self, task_id: &str, error: &str) {
        self.task_errors
            .insert(task_id.to_string(), error.to_string());
        self.checkpoint_at = SystemTime::now();
    }

    /// Record task result/output (truncated for context)
    pub fn record_task_result(&mut self, task_id: &str, result: &str) {
        // Truncate to 500 chars for context summary
        const MAX_RESULT_LEN: usize = 500;
        let truncated = if result.len() > MAX_RESULT_LEN {
            format!("{}... [truncated]", &result[..MAX_RESULT_LEN])
        } else {
            result.to_string()
        };
        self.task_results.insert(task_id.to_string(), truncated);
        self.checkpoint_at = SystemTime::now();
    }

    /// Build workflow context summary for agent tasks
    ///
    /// Returns a summary of the workflow execution state including:
    /// - Workflow name and description
    /// - Previously completed tasks and their results
    /// - Current progress
    pub fn build_context_summary(
        &self,
        workflow_name: &str,
        workflow_description: Option<&str>,
        output_file: Option<&str>,
    ) -> String {
        let mut context = String::new();

        context.push_str("=== WORKFLOW CONTEXT ===\n\n");
        context.push_str(&format!("Workflow: {}\n", workflow_name));

        if let Some(desc) = workflow_description {
            context.push_str(&format!("Description: {}\n", desc));
        }

        context.push_str(&format!("Version: {}\n", self.workflow_version));
        context.push_str(&format!("Progress: {:.0}%\n", self.get_progress() * 100.0));

        if let Some(output) = output_file {
            context.push_str(&format!("Output File: {}\n", output));
        }

        context.push('\n');

        // List completed tasks with results
        let completed_tasks = self.get_completed_tasks();
        if !completed_tasks.is_empty() {
            context.push_str("Previously Completed Tasks:\n");
            for task_id in &completed_tasks {
                context.push_str(&format!("  ✓ {}", task_id));
                if let Some(result) = self.task_results.get(task_id) {
                    if !result.trim().is_empty() {
                        context.push_str(&format!("\n    Result: {}", result.trim()));
                    }
                }
                context.push('\n');
            }
            context.push('\n');
        }

        // List failed tasks
        let failed_tasks = self.get_failed_tasks();
        if !failed_tasks.is_empty() {
            context.push_str("Failed Tasks:\n");
            for task_id in &failed_tasks {
                context.push_str(&format!("  ✗ {}", task_id));
                if let Some(error) = self.task_errors.get(task_id) {
                    context.push_str(&format!("\n    Error: {}", error));
                }
                context.push('\n');
            }
            context.push('\n');
        }

        context.push_str("=== END WORKFLOW CONTEXT ===\n\n");
        context
    }

    /// Mark workflow as completed
    pub fn mark_completed(&mut self) {
        self.status = WorkflowStatus::Completed;
        self.ended_at = Some(SystemTime::now());
        self.checkpoint_at = SystemTime::now();
    }

    /// Mark workflow as failed
    pub fn mark_failed(&mut self) {
        self.status = WorkflowStatus::Failed;
        self.ended_at = Some(SystemTime::now());
        self.checkpoint_at = SystemTime::now();
    }

    /// Mark workflow as paused
    pub fn mark_paused(&mut self) {
        self.status = WorkflowStatus::Paused;
        self.checkpoint_at = SystemTime::now();
    }

    /// Get task status
    pub fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        self.task_statuses.get(task_id).copied()
    }

    /// Get task attempts count
    pub fn get_task_attempts(&self, task_id: &str) -> u32 {
        self.task_attempts.get(task_id).copied().unwrap_or(0)
    }

    /// Get completed tasks
    pub fn get_completed_tasks(&self) -> Vec<String> {
        self.task_statuses
            .iter()
            .filter(|(_, status)| **status == TaskStatus::Completed)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get failed tasks
    pub fn get_failed_tasks(&self) -> Vec<String> {
        self.task_statuses
            .iter()
            .filter(|(_, status)| **status == TaskStatus::Failed)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get pending tasks
    pub fn get_pending_tasks(&self) -> Vec<String> {
        self.task_statuses
            .iter()
            .filter(|(_, status)| **status == TaskStatus::Pending)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Check if workflow can be resumed
    pub fn can_resume(&self) -> bool {
        matches!(
            self.status,
            WorkflowStatus::Running | WorkflowStatus::Paused
        )
    }

    /// Get total number of executable tasks including loop iterations
    pub fn get_total_task_count(&self) -> usize {
        let base_tasks = self.task_statuses.len();
        let loop_iterations: usize = self
            .loop_states
            .values()
            .filter_map(|loop_state| loop_state.total_iterations)
            .sum();

        // Don't double-count: if a task has a loop, we count only iterations, not the base task
        let tasks_with_loops = self.loop_states.len();
        base_tasks + loop_iterations - tasks_with_loops
    }

    /// Get number of completed tasks including completed loop iterations
    pub fn get_completed_task_count(&self) -> usize {
        // Count completed base tasks (excluding tasks with loops)
        let completed_base_tasks = self
            .task_statuses
            .iter()
            .filter(|(task_id, status)| {
                **status == TaskStatus::Completed && !self.loop_states.contains_key(*task_id)
            })
            .count();

        // Count completed loop iterations
        let completed_iterations: usize = self
            .loop_states
            .values()
            .map(|loop_state| {
                loop_state
                    .iteration_statuses
                    .iter()
                    .filter(|s| **s == TaskStatus::Completed)
                    .count()
            })
            .sum();

        completed_base_tasks + completed_iterations
    }

    /// Get progress percentage (0.0 to 1.0) accounting for loop iterations
    pub fn get_progress(&self) -> f64 {
        let total = self.get_total_task_count();
        if total == 0 {
            return 0.0;
        }

        let completed = self.get_completed_task_count();
        completed as f64 / total as f64
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
        self.checkpoint_at = SystemTime::now();
    }

    /// Get metadata entry
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Initialize a loop for a task
    pub fn init_loop(&mut self, task_id: &str, total_iterations: Option<usize>) {
        self.loop_states.insert(
            task_id.to_string(),
            LoopState::new(task_id.to_string(), total_iterations),
        );
        self.checkpoint_at = SystemTime::now();
    }

    /// Update loop iteration state
    pub fn update_loop_iteration(
        &mut self,
        task_id: &str,
        iteration: usize,
        status: TaskStatus,
        iterator_value: Option<serde_json::Value>,
    ) {
        if let Some(loop_state) = self.loop_states.get_mut(task_id) {
            loop_state.current_iteration = iteration;
            loop_state.iterator_value = iterator_value;
            loop_state.last_iteration_at = Some(Utc::now());

            // Ensure iteration_statuses has enough capacity
            if iteration >= loop_state.iteration_statuses.len() {
                loop_state
                    .iteration_statuses
                    .resize(iteration + 1, TaskStatus::Pending);
            }
            loop_state.iteration_statuses[iteration] = status;

            self.checkpoint_at = SystemTime::now();
        }
    }

    /// Set a loop variable
    pub fn set_loop_variable(&mut self, task_id: &str, name: String, value: serde_json::Value) {
        if let Some(loop_state) = self.loop_states.get_mut(task_id) {
            loop_state.loop_variables.insert(name, value);
            self.checkpoint_at = SystemTime::now();
        }
    }

    /// Get loop progress as a percentage (0.0 to 100.0)
    pub fn get_loop_progress(&self, task_id: &str) -> Option<f64> {
        self.loop_states.get(task_id).and_then(|ls| {
            ls.total_iterations.map(|total| {
                if total == 0 {
                    0.0
                } else {
                    (ls.current_iteration as f64 / total as f64) * 100.0
                }
            })
        })
    }

    /// Store a loop result from an iteration
    pub fn store_loop_result(&mut self, task_id: &str, result: serde_json::Value) {
        self.loop_results
            .entry(task_id.to_string())
            .or_default()
            .push(result);
        self.checkpoint_at = SystemTime::now();
    }

    /// Get all loop results for a task
    pub fn get_loop_results(&self, task_id: &str) -> Option<&Vec<serde_json::Value>> {
        self.loop_results.get(task_id)
    }

    /// Get loop state for a task
    pub fn get_loop_state(&self, task_id: &str) -> Option<&LoopState> {
        self.loop_states.get(task_id)
    }

    /// Check if a task has an active loop
    pub fn has_loop(&self, task_id: &str) -> bool {
        self.loop_states.contains_key(task_id)
    }

    /// Check if a specific loop iteration was already completed
    ///
    /// This is used for resume capability - allows skipping already-completed iterations
    /// when resuming an interrupted loop.
    pub fn is_iteration_completed(&self, task_id: &str, iteration: usize) -> bool {
        if let Some(loop_state) = self.loop_states.get(task_id) {
            if iteration < loop_state.iteration_statuses.len() {
                matches!(
                    loop_state.iteration_statuses[iteration],
                    TaskStatus::Completed
                )
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get the last completed iteration for a loop (for resume capability)
    ///
    /// Returns None if no iterations completed, otherwise returns the highest
    /// completed iteration index.
    pub fn get_last_completed_iteration(&self, task_id: &str) -> Option<usize> {
        if let Some(loop_state) = self.loop_states.get(task_id) {
            loop_state
                .iteration_statuses
                .iter()
                .enumerate()
                .rev()
                .find(|(_, status)| matches!(status, TaskStatus::Completed))
                .map(|(idx, _)| idx)
        } else {
            None
        }
    }

    // ========================================================================
    // Context Management Methods
    // ========================================================================

    /// Store task output with metadata
    pub fn store_task_output(&mut self, output: TaskOutput) {
        self.task_outputs.insert(output.task_id.clone(), output);
        self.checkpoint_at = SystemTime::now();
    }

    /// Get task output
    pub fn get_task_output(&self, task_id: &str) -> Option<&TaskOutput> {
        self.task_outputs.get(task_id)
    }

    /// Get mutable task output (for updating relevance, access time, etc.)
    pub fn get_task_output_mut(&mut self, task_id: &str) -> Option<&mut TaskOutput> {
        self.task_outputs.get_mut(task_id)
    }

    /// Calculate and return context metrics
    pub fn get_context_metrics(&self) -> ContextMetrics {
        let total_bytes = self.task_outputs.values().map(|o| o.content.len()).sum();
        let task_count = self.task_outputs.len();
        let truncated_count = self.task_outputs.values().filter(|o| o.truncated).count();
        let external_count = self
            .task_outputs
            .values()
            .filter(|o| o.file_path.is_some())
            .count();

        let avg_relevance = if task_count > 0 {
            self.task_outputs.values().map(|o| o.relevance_score).sum::<f64>() / task_count as f64
        } else {
            0.0
        };

        ContextMetrics {
            total_bytes,
            task_count,
            truncated_count,
            external_count,
            avg_relevance,
            last_pruned_at: None, // TODO: Track this
        }
    }

    /// Log context metrics
    pub fn log_metrics(&self) {
        let metrics = self.get_context_metrics();
        println!("Context Metrics:");
        println!("  Total bytes: {}", metrics.total_bytes);
        println!("  Task outputs: {}", metrics.task_count);
        println!("  Truncated: {}", metrics.truncated_count);
        println!("  External storage: {}", metrics.external_count);
        println!("  Avg relevance: {:.2}", metrics.avg_relevance);
    }

    /// Prune outputs based on cleanup strategy
    pub fn prune_outputs(&mut self, strategy: &CleanupStrategy) {
        match strategy {
            CleanupStrategy::MostRecent { keep_count } => {
                self.prune_most_recent(*keep_count);
            }
            CleanupStrategy::HighestRelevance { keep_count } => {
                self.prune_by_relevance(*keep_count);
            }
            CleanupStrategy::Lru { keep_count } => {
                self.prune_lru(*keep_count);
            }
            CleanupStrategy::DirectDependencies => {
                self.prune_non_dependencies();
            }
        }
    }

    fn prune_most_recent(&mut self, keep_count: usize) {
        if self.task_outputs.len() <= keep_count {
            return;
        }

        // Sort by last access time
        let mut outputs: Vec<_> = self.task_outputs.iter().collect();
        outputs.sort_by_key(|(_, output)| output.last_accessed);
        outputs.reverse();

        let keep_ids: std::collections::HashSet<_> = outputs
            .iter()
            .take(keep_count)
            .map(|(id, _)| (*id).clone())
            .collect();

        self.task_outputs.retain(|id, _| keep_ids.contains(id));
    }

    fn prune_by_relevance(&mut self, keep_count: usize) {
        if self.task_outputs.len() <= keep_count {
            return;
        }

        let mut outputs: Vec<_> = self.task_outputs.iter().collect();
        outputs.sort_by(|(_, a), (_, b)| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let keep_ids: std::collections::HashSet<_> = outputs
            .iter()
            .take(keep_count)
            .map(|(id, _)| (*id).clone())
            .collect();

        self.task_outputs.retain(|id, _| keep_ids.contains(id));
    }

    fn prune_lru(&mut self, keep_count: usize) {
        if self.task_outputs.len() <= keep_count {
            return;
        }

        let mut outputs: Vec<_> = self.task_outputs.iter().collect();
        outputs.sort_by_key(|(_, output)| output.last_accessed);
        outputs.reverse();

        let keep_ids: std::collections::HashSet<_> = outputs
            .iter()
            .take(keep_count)
            .map(|(id, _)| (*id).clone())
            .collect();

        self.task_outputs.retain(|id, _| keep_ids.contains(id));
    }

    fn prune_non_dependencies(&mut self) {
        // Keep only outputs that have dependent tasks
        self.task_outputs
            .retain(|_, output| !output.depended_by.is_empty());
    }

    /// Save checkpoint to default location (.state directory)
    ///
    /// This is a convenience method for in-loop checkpointing that doesn't require
    /// passing StatePersistence around. Creates a .state directory in the current
    /// working directory.
    pub fn save_checkpoint(&self) -> Result<()> {
        let state_dir = std::path::PathBuf::from(".state");
        if !state_dir.exists() {
            std::fs::create_dir_all(&state_dir).map_err(|e| {
                Error::InvalidInput(format!("Failed to create .state directory: {}", e))
            })?;
        }

        let persistence = StatePersistence::new(&state_dir)?;
        persistence.save_state(self)?;
        Ok(())
    }
}

/// State persistence manager
#[derive(Clone)]
pub struct StatePersistence {
    /// Directory for storing state files
    state_dir: PathBuf,
}

impl StatePersistence {
    /// Create a new state persistence manager
    ///
    /// # Arguments
    ///
    /// * `state_dir` - Directory to store state files
    pub fn new<P: AsRef<Path>>(state_dir: P) -> Result<Self> {
        let state_dir = state_dir.as_ref().to_path_buf();

        // Create state directory if it doesn't exist
        if !state_dir.exists() {
            fs::create_dir_all(&state_dir).map_err(|e| {
                Error::InvalidInput(format!("Failed to create state directory: {}", e))
            })?;
        }

        Ok(Self { state_dir })
    }

    /// Get state file path for a workflow
    fn get_state_file_path(&self, workflow_name: &str) -> PathBuf {
        self.state_dir.join(format!("{}.state.json", workflow_name))
    }

    /// Save workflow state to disk
    ///
    /// # Arguments
    ///
    /// * `state` - The workflow state to save
    ///
    /// # Performance
    ///
    /// Uses buffered I/O for faster writes
    pub fn save_state(&self, state: &WorkflowState) -> Result<()> {
        let file_path = self.get_state_file_path(&state.workflow_name);

        // Create file with buffered writer for better I/O performance
        let file = File::create(&file_path)
            .map_err(|e| Error::InvalidInput(format!("Failed to create state file: {}", e)))?;

        let mut writer = BufWriter::new(file);

        // Serialize directly to the buffered writer
        serde_json::to_writer_pretty(&mut writer, state).map_err(|e| {
            Error::InvalidInput(format!("Failed to serialize workflow state: {}", e))
        })?;

        // Ensure all data is flushed to disk
        writer
            .flush()
            .map_err(|e| Error::InvalidInput(format!("Failed to flush state file: {}", e)))?;

        println!("Saved workflow state to: {}", file_path.display());
        Ok(())
    }

    /// Load workflow state from disk
    ///
    /// # Arguments
    ///
    /// * `workflow_name` - Name of the workflow to load
    ///
    /// # Performance
    ///
    /// Uses buffered I/O for faster reads
    pub fn load_state(&self, workflow_name: &str) -> Result<WorkflowState> {
        let file_path = self.get_state_file_path(workflow_name);

        if !file_path.exists() {
            return Err(Error::InvalidInput(format!(
                "No state file found for workflow '{}'",
                workflow_name
            )));
        }

        // Open file with buffered reader for better I/O performance
        let file = File::open(&file_path)
            .map_err(|e| Error::InvalidInput(format!("Failed to open state file: {}", e)))?;

        let reader = BufReader::new(file);

        // Deserialize directly from the buffered reader
        let state: WorkflowState = serde_json::from_reader(reader).map_err(|e| {
            Error::InvalidInput(format!("Failed to deserialize workflow state: {}", e))
        })?;

        println!("Loaded workflow state from: {}", file_path.display());
        Ok(state)
    }

    /// Check if state exists for a workflow
    pub fn has_state(&self, workflow_name: &str) -> bool {
        self.get_state_file_path(workflow_name).exists()
    }

    /// Delete saved state for a workflow
    pub fn delete_state(&self, workflow_name: &str) -> Result<()> {
        let file_path = self.get_state_file_path(workflow_name);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| Error::InvalidInput(format!("Failed to delete state file: {}", e)))?;
            println!("Deleted state file: {}", file_path.display());
        }

        Ok(())
    }

    /// List all workflow states
    pub fn list_states(&self) -> Result<Vec<String>> {
        let mut workflows = Vec::new();

        if !self.state_dir.exists() {
            return Ok(workflows);
        }

        for entry in fs::read_dir(&self.state_dir)
            .map_err(|e| Error::InvalidInput(format!("Failed to read state directory: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                Error::InvalidInput(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Some(workflow_name) = name.strip_suffix(".state") {
                        workflows.push(workflow_name.to_string());
                    }
                }
            }
        }

        Ok(workflows)
    }

    /// Get state directory path
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }
}

impl Default for StatePersistence {
    fn default() -> Self {
        Self::new(".workflow_states").expect("Failed to create default state persistence")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workflow_state_creation() {
        let state = WorkflowState::new("test_workflow".to_string(), "1.0.0".to_string());
        assert_eq!(state.workflow_name, "test_workflow");
        assert_eq!(state.workflow_version, "1.0.0");
        assert_eq!(state.status, WorkflowStatus::Running);
        assert!(state.can_resume());
    }

    #[test]
    fn test_update_task_status() {
        let mut state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

        state.update_task_status("task1", TaskStatus::Running);
        assert_eq!(state.get_task_status("task1"), Some(TaskStatus::Running));
        assert!(state.task_start_times.contains_key("task1"));

        state.update_task_status("task1", TaskStatus::Completed);
        assert_eq!(state.get_task_status("task1"), Some(TaskStatus::Completed));
        assert!(state.task_end_times.contains_key("task1"));
    }

    #[test]
    fn test_task_attempts() {
        let mut state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

        assert_eq!(state.get_task_attempts("task1"), 0);

        state.record_task_attempt("task1");
        assert_eq!(state.get_task_attempts("task1"), 1);

        state.record_task_attempt("task1");
        assert_eq!(state.get_task_attempts("task1"), 2);
    }

    #[test]
    fn test_workflow_status_transitions() {
        let mut state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

        assert_eq!(state.status, WorkflowStatus::Running);
        assert!(state.can_resume());

        state.mark_paused();
        assert_eq!(state.status, WorkflowStatus::Paused);
        assert!(state.can_resume());

        state.mark_completed();
        assert_eq!(state.status, WorkflowStatus::Completed);
        assert!(!state.can_resume());
        assert!(state.ended_at.is_some());
    }

    #[test]
    fn test_get_progress() {
        let mut state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

        state.update_task_status("task1", TaskStatus::Completed);
        state.update_task_status("task2", TaskStatus::Pending);
        state.update_task_status("task3", TaskStatus::Running);

        let progress = state.get_progress();
        assert!((progress - 0.333).abs() < 0.01); // 1/3 completed
    }

    #[test]
    fn test_get_task_lists() {
        let mut state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

        state.update_task_status("task1", TaskStatus::Completed);
        state.update_task_status("task2", TaskStatus::Failed);
        state.update_task_status("task3", TaskStatus::Pending);

        assert_eq!(state.get_completed_tasks().len(), 1);
        assert_eq!(state.get_failed_tasks().len(), 1);
        assert_eq!(state.get_pending_tasks().len(), 1);
    }

    #[test]
    fn test_state_persistence_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let mut state = WorkflowState::new("test_workflow".to_string(), "1.0.0".to_string());
        state.update_task_status("task1", TaskStatus::Completed);
        state.add_metadata("key1".to_string(), serde_json::json!("value1"));

        // Save state
        persistence.save_state(&state).unwrap();
        assert!(persistence.has_state("test_workflow"));

        // Load state
        let loaded_state = persistence.load_state("test_workflow").unwrap();
        assert_eq!(loaded_state.workflow_name, "test_workflow");
        assert_eq!(
            loaded_state.get_task_status("task1"),
            Some(TaskStatus::Completed)
        );
        assert_eq!(
            loaded_state.metadata.get("key1"),
            Some(&serde_json::json!("value1"))
        );
    }

    #[test]
    fn test_state_persistence_list() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let state1 = WorkflowState::new("workflow1".to_string(), "1.0.0".to_string());
        let state2 = WorkflowState::new("workflow2".to_string(), "1.0.0".to_string());

        persistence.save_state(&state1).unwrap();
        persistence.save_state(&state2).unwrap();

        let workflows = persistence.list_states().unwrap();
        assert_eq!(workflows.len(), 2);
        assert!(workflows.contains(&"workflow1".to_string()));
        assert!(workflows.contains(&"workflow2".to_string()));
    }

    #[test]
    fn test_state_persistence_delete() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = StatePersistence::new(temp_dir.path()).unwrap();

        let state = WorkflowState::new("test_workflow".to_string(), "1.0.0".to_string());
        persistence.save_state(&state).unwrap();
        assert!(persistence.has_state("test_workflow"));

        persistence.delete_state("test_workflow").unwrap();
        assert!(!persistence.has_state("test_workflow"));
    }

    #[test]
    fn test_record_task_error() {
        let mut state = WorkflowState::new("test".to_string(), "1.0.0".to_string());

        state.record_task_error("task1", "Connection timeout");
        assert_eq!(
            state.task_errors.get("task1"),
            Some(&"Connection timeout".to_string())
        );
    }

    #[test]
    fn test_build_context_summary_includes_output_file() {
        let state = WorkflowState::new("test_workflow".to_string(), "1.0.0".to_string());

        // Test with output file specified
        let context = state.build_context_summary(
            "test_workflow",
            Some("Test workflow description"),
            Some("/tmp/output.txt"),
        );

        assert!(context.contains("=== WORKFLOW CONTEXT ==="));
        assert!(context.contains("Workflow: test_workflow"));
        assert!(context.contains("Description: Test workflow description"));
        assert!(context.contains("Output File: /tmp/output.txt"));
    }

    #[test]
    fn test_build_context_summary_without_output_file() {
        let mut state = WorkflowState::new("test_workflow".to_string(), "1.0.0".to_string());

        // Add a completed task
        state.update_task_status("task1", TaskStatus::Completed);
        state.record_task_result("task1", "Task completed successfully");

        // Test without output file
        let context = state.build_context_summary("test_workflow", None, None);

        assert!(context.contains("=== WORKFLOW CONTEXT ==="));
        assert!(context.contains("Workflow: test_workflow"));
        assert!(!context.contains("Output File:"));
        assert!(context.contains("Previously Completed Tasks:"));
        assert!(context.contains("task1"));
    }

    #[test]
    fn test_build_context_summary_with_both_output_and_tasks() {
        let mut state = WorkflowState::new("test_workflow".to_string(), "1.0.0".to_string());

        // Add a completed task
        state.update_task_status("task1", TaskStatus::Completed);
        state.record_task_result("task1", "First task done");

        // Test with both output file and completed tasks
        let context = state.build_context_summary(
            "test_workflow",
            Some("Multi-task workflow"),
            Some("./results/final_output.md"),
        );

        assert!(context.contains("Output File: ./results/final_output.md"));
        assert!(context.contains("Previously Completed Tasks:"));
        assert!(context.contains("task1"));
        assert!(context.contains("First task done"));
    }
}

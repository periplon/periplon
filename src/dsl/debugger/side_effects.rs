//! Side Effect Journal and Compensation System
//!
//! Tracks all side effects during workflow execution to enable undo/redo and time-travel debugging.
//! Provides compensation strategies to reverse side effects when stepping backward.
use crate::dsl::task_graph::TaskStatus;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

/// Side effect that occurred during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    /// Unique ID for this side effect
    pub id: usize,

    /// Task that caused this side effect
    pub task_id: String,

    /// Type of side effect
    pub effect_type: SideEffectType,

    /// Timestamp when effect occurred
    pub timestamp: SystemTime,

    /// Whether this effect has been compensated (undone)
    pub compensated: bool,
}

/// Type of side effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SideEffectType {
    /// File was created
    FileCreated { path: PathBuf },

    /// File was modified
    FileModified {
        path: PathBuf,
        original_content: Vec<u8>,
        new_content: Vec<u8>,
        original_size: usize,
        new_size: usize,
    },

    /// File was deleted
    FileDeleted {
        path: PathBuf,
        original_content: Vec<u8>,
    },

    /// Directory was created
    DirectoryCreated { path: PathBuf },

    /// Directory was deleted
    DirectoryDeleted {
        path: PathBuf,
        /// Saved directory tree structure
        tree: DirectoryTree,
    },

    /// Workflow state was changed
    StateChanged {
        field: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },

    /// Variable was set
    VariableSet {
        scope: VariableScope,
        name: String,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
    },

    /// Task status was changed
    TaskStatusChanged {
        task_id: String,
        old_status: TaskStatus,
        new_status: TaskStatus,
    },

    /// External command was executed
    CommandExecuted {
        command: String,
        working_dir: PathBuf,
        exit_code: i32,
        stdout: String,
        stderr: String,
    },

    /// Network request was made
    NetworkRequest {
        url: String,
        method: String,
        response_status: u16,
        response_body: Option<String>,
    },

    /// Environment variable was set
    EnvVarSet {
        name: String,
        old_value: Option<String>,
        new_value: String,
    },
}

/// Variable scope identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum VariableScope {
    Workflow,
    Agent(String),
    Task(String),
    Loop { task_id: String, iteration: usize },
}

/// Directory tree for backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryTree {
    pub path: PathBuf,
    pub files: Vec<FileEntry>,
    pub subdirs: Vec<DirectoryTree>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub content: Vec<u8>,
}

impl DirectoryTree {
    /// Capture a directory tree
    pub fn capture(path: &PathBuf) -> Result<Self> {
        let mut files = Vec::new();
        let mut subdirs = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Ok(content) = fs::read(&entry_path) {
                        files.push(FileEntry {
                            name: entry.file_name().to_string_lossy().to_string(),
                            content,
                        });
                    }
                } else if entry_path.is_dir() {
                    if let Ok(subtree) = Self::capture(&entry_path) {
                        subdirs.push(subtree);
                    }
                }
            }
        }

        Ok(DirectoryTree {
            path: path.clone(),
            files,
            subdirs,
        })
    }

    /// Restore a directory tree
    pub fn restore(&self) -> Result<()> {
        // Create directory if it doesn't exist
        fs::create_dir_all(&self.path)
            .map_err(|e| Error::InvalidInput(format!("Failed to create directory: {}", e)))?;

        // Restore files
        for file in &self.files {
            let file_path = self.path.join(&file.name);
            fs::write(&file_path, &file.content).map_err(|e| {
                Error::InvalidInput(format!("Failed to restore file {:?}: {}", file_path, e))
            })?;
        }

        // Restore subdirectories recursively
        for subdir in &self.subdirs {
            subdir.restore()?;
        }

        Ok(())
    }
}

/// Compensation strategy for undoing side effects
#[async_trait::async_trait]
pub trait CompensationStrategy: Send + Sync {
    /// Execute compensation to undo the side effect
    async fn compensate(&self) -> Result<()>;

    /// Get human-readable description of compensation
    fn description(&self) -> String;

    /// Check if compensation is safe to execute
    fn is_safe(&self) -> bool;
}

/// File creation compensation (delete the file)
pub struct FileCreationCompensation {
    pub path: PathBuf,
}

#[async_trait::async_trait]
impl CompensationStrategy for FileCreationCompensation {
    async fn compensate(&self) -> Result<()> {
        if self.path.exists() {
            tokio::fs::remove_file(&self.path).await.map_err(|e| {
                Error::InvalidInput(format!("Failed to delete file {:?}: {}", self.path, e))
            })?;
        }
        Ok(())
    }

    fn description(&self) -> String {
        format!("Delete file: {:?}", self.path)
    }

    fn is_safe(&self) -> bool {
        true
    }
}

/// File modification compensation (restore original content)
pub struct FileModificationCompensation {
    pub path: PathBuf,
    pub original_content: Vec<u8>,
}

#[async_trait::async_trait]
impl CompensationStrategy for FileModificationCompensation {
    async fn compensate(&self) -> Result<()> {
        tokio::fs::write(&self.path, &self.original_content)
            .await
            .map_err(|e| {
                Error::InvalidInput(format!("Failed to restore file {:?}: {}", self.path, e))
            })?;
        Ok(())
    }

    fn description(&self) -> String {
        format!("Restore file: {:?}", self.path)
    }

    fn is_safe(&self) -> bool {
        true
    }
}

/// File deletion compensation (restore the file)
pub struct FileDeletionCompensation {
    pub path: PathBuf,
    pub original_content: Vec<u8>,
}

#[async_trait::async_trait]
impl CompensationStrategy for FileDeletionCompensation {
    async fn compensate(&self) -> Result<()> {
        tokio::fs::write(&self.path, &self.original_content)
            .await
            .map_err(|e| {
                Error::InvalidInput(format!("Failed to restore file {:?}: {}", self.path, e))
            })?;
        Ok(())
    }

    fn description(&self) -> String {
        format!("Restore deleted file: {:?}", self.path)
    }

    fn is_safe(&self) -> bool {
        true
    }
}

/// Directory creation compensation (delete the directory)
pub struct DirectoryCreationCompensation {
    pub path: PathBuf,
}

#[async_trait::async_trait]
impl CompensationStrategy for DirectoryCreationCompensation {
    async fn compensate(&self) -> Result<()> {
        if self.path.exists() {
            tokio::fs::remove_dir_all(&self.path).await.map_err(|e| {
                Error::InvalidInput(format!("Failed to delete directory {:?}: {}", self.path, e))
            })?;
        }
        Ok(())
    }

    fn description(&self) -> String {
        format!("Delete directory: {:?}", self.path)
    }

    fn is_safe(&self) -> bool {
        // Check if directory is empty or only contains files we created
        true // TODO: Implement safety check
    }
}

/// Directory deletion compensation (restore the directory)
pub struct DirectoryDeletionCompensation {
    pub tree: DirectoryTree,
}

#[async_trait::async_trait]
impl CompensationStrategy for DirectoryDeletionCompensation {
    async fn compensate(&self) -> Result<()> {
        self.tree.restore()
    }

    fn description(&self) -> String {
        format!("Restore directory: {:?}", self.tree.path)
    }

    fn is_safe(&self) -> bool {
        true
    }
}

/// Variable change compensation (restore old value)
pub struct VariableChangeCompensation {
    pub scope: VariableScope,
    pub name: String,
    pub old_value: Option<serde_json::Value>,
}

#[async_trait::async_trait]
impl CompensationStrategy for VariableChangeCompensation {
    async fn compensate(&self) -> Result<()> {
        // This would need access to the variable store
        // For now, just log the intended compensation
        println!(
            "Would restore variable {:?}.{} to {:?}",
            self.scope, self.name, self.old_value
        );
        Ok(())
    }

    fn description(&self) -> String {
        format!("Restore variable {:?}.{}", self.scope, self.name)
    }

    fn is_safe(&self) -> bool {
        true
    }
}

/// Task status change compensation
pub struct TaskStatusCompensation {
    pub task_id: String,
    pub old_status: TaskStatus,
}

#[async_trait::async_trait]
impl CompensationStrategy for TaskStatusCompensation {
    async fn compensate(&self) -> Result<()> {
        // This would need access to task graph
        println!(
            "Would restore task {} status to {:?}",
            self.task_id, self.old_status
        );
        Ok(())
    }

    fn description(&self) -> String {
        format!("Restore task {} status", self.task_id)
    }

    fn is_safe(&self) -> bool {
        true
    }
}

/// Side effect journal
pub struct SideEffectJournal {
    /// All recorded side effects
    effects: Vec<SideEffect>,

    /// Compensation strategies
    compensations: HashMap<usize, Arc<dyn CompensationStrategy>>,

    /// Next effect ID
    next_id: usize,

    /// Whether to record side effects
    recording: bool,
}

impl Default for SideEffectJournal {
    fn default() -> Self {
        Self::new()
    }
}

impl SideEffectJournal {
    /// Create a new side effect journal
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            compensations: HashMap::new(),
            next_id: 0,
            recording: true,
        }
    }

    /// Start recording side effects
    pub fn start_recording(&mut self) {
        self.recording = true;
    }

    /// Stop recording side effects
    pub fn stop_recording(&mut self) {
        self.recording = false;
    }

    /// Record a side effect
    pub fn record(
        &mut self,
        task_id: String,
        effect_type: SideEffectType,
        compensation: Arc<dyn CompensationStrategy>,
    ) -> usize {
        if !self.recording {
            return 0;
        }

        let id = self.next_id;
        self.next_id += 1;

        let effect = SideEffect {
            id,
            task_id,
            effect_type,
            timestamp: SystemTime::now(),
            compensated: false,
        };

        self.effects.push(effect);
        self.compensations.insert(id, compensation);

        id
    }

    /// Get all side effects
    pub fn all_effects(&self) -> &[SideEffect] {
        &self.effects
    }

    /// Get effects for a specific task
    pub fn effects_for_task(&self, task_id: &str) -> Vec<&SideEffect> {
        self.effects
            .iter()
            .filter(|e| e.task_id == task_id)
            .collect()
    }

    /// Get uncompensated effects
    pub fn uncompensated_effects(&self) -> Vec<&SideEffect> {
        self.effects.iter().filter(|e| !e.compensated).collect()
    }

    /// Compensate (undo) effects since a given ID
    pub async fn compensate_since(&mut self, since_id: usize) -> Result<Vec<String>> {
        let mut compensated = Vec::new();

        // Compensate in reverse order (LIFO)
        for effect in self.effects.iter_mut().rev() {
            if effect.id >= since_id && !effect.compensated {
                if let Some(strategy) = self.compensations.get(&effect.id) {
                    if strategy.is_safe() {
                        strategy.compensate().await?;
                        effect.compensated = true;
                        compensated.push(strategy.description());
                    }
                }
            }
        }

        Ok(compensated)
    }

    /// Compensate specific effects
    pub async fn compensate_effects(&mut self, effect_ids: &[usize]) -> Result<Vec<String>> {
        let mut compensated = Vec::new();

        // Sort IDs in reverse order for LIFO compensation
        let mut sorted_ids = effect_ids.to_vec();
        sorted_ids.sort_by(|a, b| b.cmp(a));

        for effect_id in sorted_ids {
            if let Some(effect) = self.effects.iter_mut().find(|e| e.id == effect_id) {
                if !effect.compensated {
                    if let Some(strategy) = self.compensations.get(&effect_id) {
                        if strategy.is_safe() {
                            strategy.compensate().await?;
                            effect.compensated = true;
                            compensated.push(strategy.description());
                        }
                    }
                }
            }
        }

        Ok(compensated)
    }

    /// Get effect count
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Check if journal is empty
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    /// Clear all effects
    pub fn clear(&mut self) {
        self.effects.clear();
        self.compensations.clear();
        self.next_id = 0;
    }

    /// Get summary of side effects by type
    pub fn summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();

        for effect in &self.effects {
            let type_name = match &effect.effect_type {
                SideEffectType::FileCreated { .. } => "FileCreated",
                SideEffectType::FileModified { .. } => "FileModified",
                SideEffectType::FileDeleted { .. } => "FileDeleted",
                SideEffectType::DirectoryCreated { .. } => "DirectoryCreated",
                SideEffectType::DirectoryDeleted { .. } => "DirectoryDeleted",
                SideEffectType::StateChanged { .. } => "StateChanged",
                SideEffectType::VariableSet { .. } => "VariableSet",
                SideEffectType::TaskStatusChanged { .. } => "TaskStatusChanged",
                SideEffectType::CommandExecuted { .. } => "CommandExecuted",
                SideEffectType::NetworkRequest { .. } => "NetworkRequest",
                SideEffectType::EnvVarSet { .. } => "EnvVarSet",
            };

            *summary.entry(type_name.to_string()).or_insert(0) += 1;
        }

        summary
    }
}

/// Thread-safe side effect journal
pub type SharedSideEffectJournal = Arc<Mutex<SideEffectJournal>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_side_effect() {
        let mut journal = SideEffectJournal::new();

        let compensation = Arc::new(FileCreationCompensation {
            path: PathBuf::from("/tmp/test.txt"),
        });

        let id = journal.record(
            "task1".to_string(),
            SideEffectType::FileCreated {
                path: PathBuf::from("/tmp/test.txt"),
            },
            compensation,
        );

        assert_eq!(id, 0);
        assert_eq!(journal.len(), 1);
    }

    #[tokio::test]
    async fn test_filter_by_task() {
        let mut journal = SideEffectJournal::new();

        let compensation = Arc::new(FileCreationCompensation {
            path: PathBuf::from("/tmp/test.txt"),
        });

        journal.record(
            "task1".to_string(),
            SideEffectType::FileCreated {
                path: PathBuf::from("/tmp/test1.txt"),
            },
            compensation.clone(),
        );

        journal.record(
            "task2".to_string(),
            SideEffectType::FileCreated {
                path: PathBuf::from("/tmp/test2.txt"),
            },
            compensation.clone(),
        );

        journal.record(
            "task1".to_string(),
            SideEffectType::FileCreated {
                path: PathBuf::from("/tmp/test3.txt"),
            },
            compensation,
        );

        let task1_effects = journal.effects_for_task("task1");
        assert_eq!(task1_effects.len(), 2);

        let task2_effects = journal.effects_for_task("task2");
        assert_eq!(task2_effects.len(), 1);
    }

    #[test]
    fn test_summary() {
        let mut journal = SideEffectJournal::new();

        journal.record(
            "task1".to_string(),
            SideEffectType::FileCreated {
                path: PathBuf::from("/tmp/test1.txt"),
            },
            Arc::new(FileCreationCompensation {
                path: PathBuf::from("/tmp/test1.txt"),
            }),
        );

        journal.record(
            "task1".to_string(),
            SideEffectType::FileCreated {
                path: PathBuf::from("/tmp/test2.txt"),
            },
            Arc::new(FileCreationCompensation {
                path: PathBuf::from("/tmp/test2.txt"),
            }),
        );

        journal.record(
            "task1".to_string(),
            SideEffectType::FileModified {
                path: PathBuf::from("/tmp/test3.txt"),
                original_content: vec![],
                new_content: vec![],
                original_size: 0,
                new_size: 0,
            },
            Arc::new(FileModificationCompensation {
                path: PathBuf::from("/tmp/test3.txt"),
                original_content: vec![],
            }),
        );

        let summary = journal.summary();
        assert_eq!(summary.get("FileCreated"), Some(&2));
        assert_eq!(summary.get("FileModified"), Some(&1));
    }
}

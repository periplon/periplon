use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("CLI not found")]
    CliNotFound,

    #[error("CLI connection error: {0}")]
    CliConnection(String),

    #[error("Process failed with exit code {exit_code}")]
    ProcessFailed { exit_code: i32 },

    #[error("Not connected")]
    NotConnected,

    #[error("Not ready")]
    NotReady,

    #[error("Not in streaming mode")]
    NotStreamingMode,

    #[error("Query already started")]
    AlreadyStarted,

    #[error("Stdio error")]
    StdioError,

    #[error("JSON parse error: {0}")]
    ParseError(String),

    #[error("JSON buffer overflow")]
    BufferOverflow,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Timeout error")]
    Timeout(#[from] tokio::time::error::Elapsed),

    #[error("Control request failed: {0}")]
    ControlRequestFailed(String),

    #[error("Oneshot receive error")]
    OneshotRecv(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    // Predefined Tasks errors
    #[error("Task '{name}' not found in source '{source_name}'")]
    TaskNotFound {
        name: String,
        version: Option<String>,
        source_name: String,
    },

    #[error("Task '{name}' not found in any configured sources. Searched: {searched_sources:?}")]
    TaskNotFoundInAnySources {
        name: String,
        version: Option<String>,
        searched_sources: Vec<String>,
    },

    #[error("Source '{0}' not found")]
    SourceNotFound(String),

    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),

    #[error("Invalid package kind: expected 'TaskPackage', found '{0}'")]
    InvalidPackageKind(String),

    #[error("Task reference mismatch: expected '{expected}', found '{found}'")]
    TaskReferenceMismatch { expected: String, found: String },

    #[error("Task version mismatch for '{task}': expected '{expected}', found '{found}'")]
    TaskVersionMismatch {
        task: String,
        expected: String,
        found: String,
    },

    #[error("Task file not found: {path} (package: {package})")]
    TaskFileNotFound { path: String, package: String },

    #[error("Source configuration error: {0}")]
    SourceConfigError(String),

    #[error("Invalid version '{version}' in {context}")]
    InvalidVersion { version: String, context: String },

    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Task load error: {0}")]
    LoadError(#[from] crate::dsl::predefined_tasks::loader::LoadError),

    #[error("Version error: {0}")]
    VersionError(#[from] crate::dsl::predefined_tasks::version::VersionError),

    #[error("Update error: {0}")]
    UpdateError(#[from] crate::dsl::predefined_tasks::update::UpdateError),
}

pub type Result<T> = std::result::Result<T, Error>;

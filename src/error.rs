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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_cli_not_found() {
        let err = Error::CliNotFound;
        assert_eq!(err.to_string(), "CLI not found");
    }

    #[test]
    fn test_error_cli_connection() {
        let err = Error::CliConnection("Connection refused".to_string());
        assert_eq!(err.to_string(), "CLI connection error: Connection refused");
    }

    #[test]
    fn test_error_process_failed() {
        let err = Error::ProcessFailed { exit_code: 1 };
        assert_eq!(err.to_string(), "Process failed with exit code 1");
    }

    #[test]
    fn test_error_not_connected() {
        let err = Error::NotConnected;
        assert_eq!(err.to_string(), "Not connected");
    }

    #[test]
    fn test_error_not_ready() {
        let err = Error::NotReady;
        assert_eq!(err.to_string(), "Not ready");
    }

    #[test]
    fn test_error_not_streaming_mode() {
        let err = Error::NotStreamingMode;
        assert_eq!(err.to_string(), "Not in streaming mode");
    }

    #[test]
    fn test_error_already_started() {
        let err = Error::AlreadyStarted;
        assert_eq!(err.to_string(), "Query already started");
    }

    #[test]
    fn test_error_stdio_error() {
        let err = Error::StdioError;
        assert_eq!(err.to_string(), "Stdio error");
    }

    #[test]
    fn test_error_parse_error() {
        let err = Error::ParseError("Invalid JSON".to_string());
        assert_eq!(err.to_string(), "JSON parse error: Invalid JSON");
    }

    #[test]
    fn test_error_buffer_overflow() {
        let err = Error::BufferOverflow;
        assert_eq!(err.to_string(), "JSON buffer overflow");
    }

    #[test]
    fn test_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::Io(io_err);
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_error_json() {
        let json_str = "{invalid json";
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let err = Error::Json(json_err);
        assert!(err.to_string().contains("JSON error"));
    }

    #[test]
    fn test_error_control_request_failed() {
        let err = Error::ControlRequestFailed("Request timeout".to_string());
        assert_eq!(err.to_string(), "Control request failed: Request timeout");
    }

    #[test]
    fn test_error_channel_closed() {
        let err = Error::ChannelClosed;
        assert_eq!(err.to_string(), "Channel closed");
    }

    #[test]
    fn test_error_invalid_input() {
        let err = Error::InvalidInput("Missing required field".to_string());
        assert_eq!(err.to_string(), "Invalid input: Missing required field");
    }

    #[test]
    fn test_error_task_not_found() {
        let err = Error::TaskNotFound {
            name: "my-task".to_string(),
            version: Some("1.0.0".to_string()),
            source_name: "github".to_string(),
        };
        assert!(err.to_string().contains("Task 'my-task' not found"));
        assert!(err.to_string().contains("source 'github'"));
    }

    #[test]
    fn test_error_task_not_found_without_version() {
        let err = Error::TaskNotFound {
            name: "task".to_string(),
            version: None,
            source_name: "local".to_string(),
        };
        assert!(err.to_string().contains("Task 'task' not found"));
    }

    #[test]
    fn test_error_task_not_found_in_any_sources() {
        let err = Error::TaskNotFoundInAnySources {
            name: "missing-task".to_string(),
            version: Some("2.0.0".to_string()),
            searched_sources: vec!["github".to_string(), "local".to_string()],
        };
        assert!(err.to_string().contains("Task 'missing-task' not found"));
        assert!(err.to_string().contains("github"));
        assert!(err.to_string().contains("local"));
    }

    #[test]
    fn test_error_source_not_found() {
        let err = Error::SourceNotFound("unknown-source".to_string());
        assert_eq!(err.to_string(), "Source 'unknown-source' not found");
    }

    #[test]
    fn test_error_invalid_package_kind() {
        let err = Error::InvalidPackageKind("InvalidType".to_string());
        assert_eq!(
            err.to_string(),
            "Invalid package kind: expected 'TaskPackage', found 'InvalidType'"
        );
    }

    #[test]
    fn test_error_task_reference_mismatch() {
        let err = Error::TaskReferenceMismatch {
            expected: "task-a".to_string(),
            found: "task-b".to_string(),
        };
        assert!(err
            .to_string()
            .contains("Task reference mismatch: expected 'task-a', found 'task-b'"));
    }

    #[test]
    fn test_error_task_version_mismatch() {
        let err = Error::TaskVersionMismatch {
            task: "my-task".to_string(),
            expected: "1.0.0".to_string(),
            found: "2.0.0".to_string(),
        };
        assert!(err.to_string().contains("Task version mismatch"));
        assert!(err.to_string().contains("my-task"));
        assert!(err.to_string().contains("1.0.0"));
        assert!(err.to_string().contains("2.0.0"));
    }

    #[test]
    fn test_error_task_file_not_found() {
        let err = Error::TaskFileNotFound {
            path: "/path/to/task.yaml".to_string(),
            package: "task-package".to_string(),
        };
        assert!(err.to_string().contains("Task file not found"));
        assert!(err.to_string().contains("/path/to/task.yaml"));
        assert!(err.to_string().contains("task-package"));
    }

    #[test]
    fn test_error_source_config_error() {
        let err = Error::SourceConfigError("Invalid URL format".to_string());
        assert_eq!(
            err.to_string(),
            "Source configuration error: Invalid URL format"
        );
    }

    #[test]
    fn test_error_invalid_version() {
        let err = Error::InvalidVersion {
            version: "abc.def".to_string(),
            context: "task manifest".to_string(),
        };
        assert!(err.to_string().contains("Invalid version"));
        assert!(err.to_string().contains("abc.def"));
        assert!(err.to_string().contains("task manifest"));
    }

    #[test]
    fn test_error_yaml_error() {
        let yaml_str = "invalid: yaml: structure:\n  - bad";
        let yaml_err = serde_yaml::from_str::<serde_json::Value>(yaml_str).unwrap_err();
        let err = Error::YamlError(yaml_err);
        assert!(err.to_string().contains("YAML parsing error"));
    }

    #[test]
    fn test_error_debug() {
        let err = Error::NotConnected;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NotConnected"));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(Error::NotConnected);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotConnected));
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_error_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Json(_)));
    }

    #[test]
    fn test_error_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Error>();
        assert_sync::<Error>();
    }
}

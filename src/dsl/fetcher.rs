//! Subflow Fetcher
//!
//! This module handles fetching subflow definitions from various sources:
//! - Local files
//! - Git repositories
//! - HTTP/HTTPS URLs
//!
//! Includes caching for remote sources to avoid repeated network requests.

use crate::dsl::parser::parse_workflow;
use crate::dsl::schema::{DSLWorkflow, SubflowSource};
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Cache for fetched subflows to avoid redundant network requests
#[derive(Clone)]
pub struct SubflowCache {
    cache: Arc<Mutex<HashMap<String, DSLWorkflow>>>,
}

impl Default for SubflowCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SubflowCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a cached workflow if it exists
    pub fn get(&self, key: &str) -> Option<DSLWorkflow> {
        self.cache.lock().unwrap().get(key).cloned()
    }

    /// Store a workflow in the cache
    pub fn insert(&self, key: String, workflow: DSLWorkflow) {
        self.cache.lock().unwrap().insert(key, workflow);
    }
}

/// Fetch a subflow from a source
///
/// # Arguments
///
/// * `source` - The subflow source specification
/// * `base_path` - Base path for resolving relative file paths
/// * `cache` - Cache for storing fetched workflows
///
/// # Returns
///
/// The fetched workflow
pub async fn fetch_subflow(
    source: &SubflowSource,
    base_path: Option<&Path>,
    cache: &SubflowCache,
) -> Result<DSLWorkflow> {
    match source {
        SubflowSource::File { path } => fetch_file(path, base_path).await,
        SubflowSource::Git {
            url,
            path,
            reference,
        } => fetch_git(url, path, reference.as_deref(), cache).await,
        SubflowSource::Http { url, checksum } => fetch_http(url, checksum.as_deref(), cache).await,
    }
}

/// Fetch a subflow from a local file
async fn fetch_file(path: &str, base_path: Option<&Path>) -> Result<DSLWorkflow> {
    let resolved_path = if let Some(base) = base_path {
        base.join(path)
    } else {
        PathBuf::from(path)
    };

    let contents = tokio::fs::read_to_string(&resolved_path)
        .await
        .map_err(|e| {
            Error::InvalidInput(format!(
                "Failed to read subflow file '{}': {}",
                resolved_path.display(),
                e
            ))
        })?;

    parse_workflow(&contents)
}

/// Fetch a subflow from a git repository
async fn fetch_git(
    repo_url: &str,
    file_path: &str,
    reference: Option<&str>,
    cache: &SubflowCache,
) -> Result<DSLWorkflow> {
    let cache_key = format!(
        "git:{}:{}:{}",
        repo_url,
        file_path,
        reference.unwrap_or("HEAD")
    );

    // Check cache first
    if let Some(cached) = cache.get(&cache_key) {
        return Ok(cached);
    }

    // Create a temporary directory for cloning
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Error::InvalidInput(format!("Failed to create temp dir: {}", e)))?;

    // Clone the repository (shallow clone for efficiency)
    let clone_output = tokio::process::Command::new("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--single-branch")
        .args(reference.map(|r| vec!["--branch", r]).unwrap_or_default())
        .arg(repo_url)
        .arg(temp_dir.path())
        .output()
        .await
        .map_err(|e| Error::InvalidInput(format!("Failed to execute git clone: {}", e)))?;

    if !clone_output.status.success() {
        let stderr = String::from_utf8_lossy(&clone_output.stderr);
        return Err(Error::InvalidInput(format!(
            "Git clone failed for '{}': {}",
            repo_url, stderr
        )));
    }

    // Read the workflow file from the cloned repo
    let workflow_path = temp_dir.path().join(file_path);
    let contents = tokio::fs::read_to_string(&workflow_path)
        .await
        .map_err(|e| {
            Error::InvalidInput(format!(
                "Failed to read '{}' from git repo '{}': {}",
                file_path, repo_url, e
            ))
        })?;

    let workflow = parse_workflow(&contents)?;

    // Cache the result
    cache.insert(cache_key, workflow.clone());

    Ok(workflow)
}

/// Fetch a subflow from an HTTP/HTTPS URL
async fn fetch_http(
    url: &str,
    checksum: Option<&str>,
    cache: &SubflowCache,
) -> Result<DSLWorkflow> {
    let cache_key = format!("http:{}", url);

    // Check cache first
    if let Some(cached) = cache.get(&cache_key) {
        return Ok(cached);
    }

    // Fetch the content
    let response = reqwest::get(url)
        .await
        .map_err(|e| Error::InvalidInput(format!("Failed to fetch '{}': {}", url, e)))?;

    if !response.status().is_success() {
        return Err(Error::InvalidInput(format!(
            "HTTP request failed for '{}': {}",
            url,
            response.status()
        )));
    }

    let contents = response.text().await.map_err(|e| {
        Error::InvalidInput(format!("Failed to read response from '{}': {}", url, e))
    })?;

    // Verify checksum if provided
    if let Some(expected_checksum) = checksum {
        verify_checksum(&contents, expected_checksum)?;
    }

    let workflow = parse_workflow(&contents)?;

    // Cache the result
    cache.insert(cache_key, workflow.clone());

    Ok(workflow)
}

/// Verify the SHA-256 checksum of content
fn verify_checksum(content: &str, expected: &str) -> Result<()> {
    use sha2::{Digest, Sha256};

    if !expected.starts_with("sha256:") {
        return Err(Error::InvalidInput(format!(
            "Invalid checksum format '{}', expected 'sha256:hash'",
            expected
        )));
    }

    let expected_hash = &expected[7..]; // Remove "sha256:" prefix
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let computed_hash = format!("{:x}", hasher.finalize());

    if computed_hash != expected_hash {
        return Err(Error::InvalidInput(format!(
            "Checksum mismatch: expected '{}', got '{}'",
            expected_hash, computed_hash
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_subflow_cache_new() {
        let cache = SubflowCache::new();
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_subflow_cache_default() {
        let cache = SubflowCache::default();
        assert!(cache.get("key").is_none());
    }

    #[test]
    fn test_subflow_cache_insert_and_get() {
        let cache = SubflowCache::new();
        let workflow = DSLWorkflow {
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        cache.insert("test_key".to_string(), workflow.clone());
        let retrieved = cache.get("test_key");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test");
    }

    #[test]
    fn test_subflow_cache_get_nonexistent() {
        let cache = SubflowCache::new();
        assert!(cache.get("missing_key").is_none());
    }

    #[test]
    fn test_subflow_cache_multiple_entries() {
        let cache = SubflowCache::new();

        let workflow1 = DSLWorkflow {
            name: "Workflow1".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        let workflow2 = DSLWorkflow {
            name: "Workflow2".to_string(),
            version: "2.0.0".to_string(),
            ..workflow1.clone()
        };

        cache.insert("key1".to_string(), workflow1);
        cache.insert("key2".to_string(), workflow2);

        assert_eq!(cache.get("key1").unwrap().name, "Workflow1");
        assert_eq!(cache.get("key2").unwrap().name, "Workflow2");
    }

    #[test]
    fn test_subflow_cache_overwrite() {
        let cache = SubflowCache::new();

        let workflow1 = DSLWorkflow {
            name: "Version1".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        let workflow2 = DSLWorkflow {
            name: "Version2".to_string(),
            ..workflow1.clone()
        };

        cache.insert("key".to_string(), workflow1);
        cache.insert("key".to_string(), workflow2);

        assert_eq!(cache.get("key").unwrap().name, "Version2");
    }

    #[test]
    fn test_subflow_cache_clone() {
        let cache1 = SubflowCache::new();
        let workflow = DSLWorkflow {
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        cache1.insert("key".to_string(), workflow);

        // Clone the cache
        let cache2 = cache1.clone();

        // Both caches should share the same underlying data
        assert!(cache2.get("key").is_some());
        assert_eq!(cache2.get("key").unwrap().name, "Test");
    }

    #[tokio::test]
    async fn test_fetch_file() {
        // Create a temporary workflow file
        let temp_dir = tempfile::tempdir().unwrap();
        let workflow_path = temp_dir.path().join("test.yaml");

        let yaml = r#"
name: "Test Workflow"
version: "1.0.0"
agents:
  test:
    description: "Test agent"
"#;
        std::fs::write(&workflow_path, yaml).unwrap();

        let workflow = fetch_file(workflow_path.to_str().unwrap(), None)
            .await
            .unwrap();

        assert_eq!(workflow.name, "Test Workflow");
    }

    #[tokio::test]
    async fn test_fetch_file_with_base_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let workflow_path = temp_dir.path().join("subdir").join("test.yaml");
        std::fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();

        let yaml = r#"
name: "Subdir Workflow"
version: "1.0.0"
"#;
        std::fs::write(&workflow_path, yaml).unwrap();

        let workflow = fetch_file("subdir/test.yaml", Some(temp_dir.path()))
            .await
            .unwrap();

        assert_eq!(workflow.name, "Subdir Workflow");
    }

    #[tokio::test]
    async fn test_fetch_file_not_found() {
        let result = fetch_file("/nonexistent/file.yaml", None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read subflow file"));
    }

    #[tokio::test]
    async fn test_fetch_file_invalid_yaml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let workflow_path = temp_dir.path().join("invalid.yaml");

        let invalid_yaml = r#"
name: "Invalid YAML
version: 1.0.0
"#;
        std::fs::write(&workflow_path, invalid_yaml).unwrap();

        let result = fetch_file(workflow_path.to_str().unwrap(), None).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_checksum_valid() {
        let content = "test content";
        // Pre-computed SHA-256 hash of "test content"
        let checksum = "sha256:6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";
        assert!(verify_checksum(content, checksum).is_ok());
    }

    #[test]
    fn test_verify_checksum_invalid() {
        let content = "test content";
        let checksum = "sha256:invalid_hash";
        assert!(verify_checksum(content, checksum).is_err());
    }

    #[test]
    fn test_verify_checksum_bad_format() {
        let content = "test content";
        let checksum = "md5:somehash";
        let result = verify_checksum(content, checksum);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid checksum format"));
    }

    #[test]
    fn test_verify_checksum_mismatch() {
        let content = "test content";
        let checksum = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
        let result = verify_checksum(content, checksum);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Checksum mismatch"));
    }

    #[test]
    fn test_verify_checksum_empty_content() {
        let content = "";
        // SHA-256 of empty string
        let checksum = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert!(verify_checksum(content, checksum).is_ok());
    }
}

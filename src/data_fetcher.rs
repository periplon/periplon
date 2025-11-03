//! Data fetching module for APIs and files
//!
//! Provides unified interface for fetching data from various sources:
//! - HTTP/HTTPS APIs (GET, POST, etc.)
//! - Local file system (text, JSON, binary)
//! - Async operations with error handling

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

/// Errors that can occur during data fetching
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

/// HTTP methods supported
#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// HTTP request configuration
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub timeout_secs: u64,
}

impl HttpRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
            timeout_secs: 30,
        }
    }

    pub fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn json_body<T: Serialize>(mut self, data: &T) -> Result<Self, FetchError> {
        self.body = Some(serde_json::to_string(data)?);
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }

    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

/// HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    /// Parse response body as JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, FetchError> {
        Ok(serde_json::from_str(&self.body)?)
    }

    /// Parse response body as dynamic JSON value
    pub fn json_value(&self) -> Result<Value, FetchError> {
        Ok(serde_json::from_str(&self.body)?)
    }

    /// Get response body as text
    pub fn text(&self) -> &str {
        &self.body
    }

    /// Check if response was successful (2xx status code)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Main data fetcher for APIs and files
pub struct DataFetcher {
    user_agent: String,
    default_headers: HashMap<String, String>,
}

impl Default for DataFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl DataFetcher {
    /// Create a new data fetcher with default settings
    pub fn new() -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert("User-Agent".to_string(), "DataFetcher/1.0".to_string());

        Self {
            user_agent: "DataFetcher/1.0".to_string(),
            default_headers,
        }
    }

    /// Set custom user agent
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = agent.into();
        self.default_headers
            .insert("User-Agent".to_string(), self.user_agent.clone());
        self
    }

    /// Add default header for all requests
    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    /// Fetch data from HTTP API
    pub async fn fetch_http(&self, request: HttpRequest) -> Result<HttpResponse, FetchError> {
        // In a real implementation, use reqwest or hyper
        // For now, provide a mock implementation that demonstrates the interface

        // Validate URL
        if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
            return Err(FetchError::InvalidUrl(format!(
                "URL must start with http:// or https://, got: {}",
                request.url
            )));
        }

        // Simulate network request (in production, replace with actual HTTP client)
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Mock response
        let response = HttpResponse {
            status: 200,
            headers: HashMap::from([("content-type".to_string(), "application/json".to_string())]),
            body: r#"{"message": "Mock API response", "status": "success"}"#.to_string(),
        };

        Ok(response)
    }

    /// Simple GET request
    pub async fn get(&self, url: impl Into<String>) -> Result<HttpResponse, FetchError> {
        self.fetch_http(HttpRequest::new(url)).await
    }

    /// Simple POST request with JSON body
    pub async fn post_json<T: Serialize>(
        &self,
        url: impl Into<String>,
        body: &T,
    ) -> Result<HttpResponse, FetchError> {
        let request = HttpRequest::new(url)
            .method(HttpMethod::Post)
            .json_body(body)?;
        self.fetch_http(request).await
    }

    /// Read text file from filesystem
    pub async fn read_text_file<P: AsRef<Path>>(&self, path: P) -> Result<String, FetchError> {
        Ok(fs::read_to_string(path).await?)
    }

    /// Read binary file from filesystem
    pub async fn read_binary_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>, FetchError> {
        Ok(fs::read(path).await?)
    }

    /// Read and parse JSON file
    pub async fn read_json_file<P: AsRef<Path>, T: for<'de> Deserialize<'de>>(
        &self,
        path: P,
    ) -> Result<T, FetchError> {
        let content = self.read_text_file(path).await?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Read JSON file as dynamic value
    pub async fn read_json_value<P: AsRef<Path>>(&self, path: P) -> Result<Value, FetchError> {
        let content = self.read_text_file(path).await?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Read file line by line
    pub async fn read_lines<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>, FetchError> {
        let content = self.read_text_file(path).await?;
        Ok(content.lines().map(|s| s.to_string()).collect())
    }

    /// Check if file exists
    pub async fn file_exists<P: AsRef<Path>>(&self, path: P) -> bool {
        fs::metadata(path).await.is_ok()
    }

    /// Get file metadata (size, modified time, etc.)
    pub async fn file_metadata<P: AsRef<Path>>(&self, path: P) -> Result<FileMetadata, FetchError> {
        let metadata = fs::metadata(path).await?;
        Ok(FileMetadata {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            read_only: metadata.permissions().readonly(),
        })
    }
}

/// File metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub read_only: bool,
}

/// Convenience functions for quick operations
pub mod quick {
    use super::*;

    /// Quick GET request
    pub async fn get(url: impl Into<String>) -> Result<HttpResponse, FetchError> {
        DataFetcher::new().get(url).await
    }

    /// Quick POST with JSON
    pub async fn post_json<T: Serialize>(
        url: impl Into<String>,
        body: &T,
    ) -> Result<HttpResponse, FetchError> {
        DataFetcher::new().post_json(url, body).await
    }

    /// Quick file read
    pub async fn read_file<P: AsRef<Path>>(path: P) -> Result<String, FetchError> {
        DataFetcher::new().read_text_file(path).await
    }

    /// Quick JSON file read
    pub async fn read_json<P: AsRef<Path>, T: for<'de> Deserialize<'de>>(
        path: P,
    ) -> Result<T, FetchError> {
        DataFetcher::new().read_json_file(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_http_request_builder() {
        let request = HttpRequest::new("https://api.example.com/data")
            .method(HttpMethod::Post)
            .header("Authorization", "Bearer token123")
            .body(r#"{"key": "value"}"#)
            .timeout(60);

        assert_eq!(request.url, "https://api.example.com/data");
        assert!(matches!(request.method, HttpMethod::Post));
        assert_eq!(
            request.headers.get("Authorization").unwrap(),
            "Bearer token123"
        );
        assert_eq!(request.timeout_secs, 60);
    }

    #[tokio::test]
    async fn test_json_body() {
        #[derive(Serialize)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let request = HttpRequest::new("https://api.example.com/data")
            .json_body(&data)
            .unwrap();

        assert!(request.body.is_some());
        assert_eq!(
            request.headers.get("Content-Type").unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn test_mock_http_fetch() {
        let fetcher = DataFetcher::new();
        let response = fetcher.get("https://api.example.com/test").await.unwrap();

        assert!(response.is_success());
        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let fetcher = DataFetcher::new();
        let result = fetcher.get("not-a-valid-url").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FetchError::InvalidUrl(_)));
    }

    #[tokio::test]
    async fn test_read_text_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();
        writeln!(temp_file, "Second line").unwrap();

        let fetcher = DataFetcher::new();
        let content = fetcher.read_text_file(temp_file.path()).await.unwrap();

        assert!(content.contains("Hello, World!"));
        assert!(content.contains("Second line"));
    }

    #[tokio::test]
    async fn test_read_json_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"name": "test", "value": 42}}"#).unwrap();

        let fetcher = DataFetcher::new();
        let json: Value = fetcher.read_json_value(temp_file.path()).await.unwrap();

        assert_eq!(json["name"], "test");
        assert_eq!(json["value"], 42);
    }

    #[tokio::test]
    async fn test_read_lines() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();

        let fetcher = DataFetcher::new();
        let lines = fetcher.read_lines(temp_file.path()).await.unwrap();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
        assert_eq!(lines[2], "Line 3");
    }

    #[tokio::test]
    async fn test_file_exists() {
        let temp_file = NamedTempFile::new().unwrap();

        let fetcher = DataFetcher::new();
        assert!(fetcher.file_exists(temp_file.path()).await);
        assert!(!fetcher.file_exists("/nonexistent/path/file.txt").await);
    }

    #[tokio::test]
    async fn test_file_metadata() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content").unwrap();

        let fetcher = DataFetcher::new();
        let metadata = fetcher.file_metadata(temp_file.path()).await.unwrap();

        assert!(metadata.is_file);
        assert!(!metadata.is_dir);
        assert!(metadata.size > 0);
    }

    #[tokio::test]
    async fn test_quick_functions() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Quick read test").unwrap();

        let content = quick::read_file(temp_file.path()).await.unwrap();
        assert!(content.contains("Quick read test"));
    }

    #[tokio::test]
    async fn test_data_fetcher_default() {
        let fetcher = DataFetcher::default();
        assert_eq!(fetcher.user_agent, "DataFetcher/1.0");
        assert!(fetcher.default_headers.contains_key("User-Agent"));
    }

    #[tokio::test]
    async fn test_data_fetcher_custom_user_agent() {
        let fetcher = DataFetcher::new().user_agent("CustomAgent/2.0");
        assert_eq!(fetcher.user_agent, "CustomAgent/2.0");
        assert_eq!(
            fetcher.default_headers.get("User-Agent").unwrap(),
            "CustomAgent/2.0"
        );
    }

    #[tokio::test]
    async fn test_data_fetcher_default_header() {
        let fetcher = DataFetcher::new()
            .default_header("X-Custom", "value123")
            .default_header("Authorization", "Bearer token");

        assert_eq!(fetcher.default_headers.get("X-Custom").unwrap(), "value123");
        assert_eq!(
            fetcher.default_headers.get("Authorization").unwrap(),
            "Bearer token"
        );
    }

    #[tokio::test]
    async fn test_http_method_variants() {
        let get_req = HttpRequest::new("https://example.com").method(HttpMethod::Get);
        assert!(matches!(get_req.method, HttpMethod::Get));

        let post_req = HttpRequest::new("https://example.com").method(HttpMethod::Post);
        assert!(matches!(post_req.method, HttpMethod::Post));

        let put_req = HttpRequest::new("https://example.com").method(HttpMethod::Put);
        assert!(matches!(put_req.method, HttpMethod::Put));

        let delete_req = HttpRequest::new("https://example.com").method(HttpMethod::Delete);
        assert!(matches!(delete_req.method, HttpMethod::Delete));

        let patch_req = HttpRequest::new("https://example.com").method(HttpMethod::Patch);
        assert!(matches!(patch_req.method, HttpMethod::Patch));
    }

    #[tokio::test]
    async fn test_http_response_json_parsing() {
        let response = HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body: r#"{"name": "test", "count": 5}"#.to_string(),
        };

        #[derive(Deserialize)]
        struct TestResponse {
            name: String,
            count: i32,
        }

        let parsed: TestResponse = response.json().unwrap();
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.count, 5);
    }

    #[tokio::test]
    async fn test_http_response_json_value() {
        let response = HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body: r#"{"key": "value"}"#.to_string(),
        };

        let json = response.json_value().unwrap();
        assert_eq!(json["key"], "value");
    }

    #[tokio::test]
    async fn test_http_response_text() {
        let response = HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body: "Plain text response".to_string(),
        };

        assert_eq!(response.text(), "Plain text response");
    }

    #[tokio::test]
    async fn test_http_response_is_success() {
        let success = HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body: String::new(),
        };
        assert!(success.is_success());

        let redirect = HttpResponse {
            status: 301,
            headers: HashMap::new(),
            body: String::new(),
        };
        assert!(!redirect.is_success());

        let error = HttpResponse {
            status: 404,
            headers: HashMap::new(),
            body: String::new(),
        };
        assert!(!error.is_success());

        let server_error = HttpResponse {
            status: 500,
            headers: HashMap::new(),
            body: String::new(),
        };
        assert!(!server_error.is_success());
    }

    #[tokio::test]
    async fn test_fetch_error_display() {
        let http_err = FetchError::HttpError("Connection failed".to_string());
        assert!(http_err.to_string().contains("HTTP request failed"));

        let url_err = FetchError::InvalidUrl("bad url".to_string());
        assert!(url_err.to_string().contains("Invalid URL"));

        let net_err = FetchError::NetworkError("timeout".to_string());
        assert!(net_err.to_string().contains("Network error"));
    }

    #[tokio::test]
    async fn test_post_json() {
        #[derive(Serialize)]
        struct PostData {
            message: String,
        }

        let fetcher = DataFetcher::new();
        let data = PostData {
            message: "test".to_string(),
        };

        let response = fetcher
            .post_json("https://api.example.com/post", &data)
            .await
            .unwrap();

        assert!(response.is_success());
    }

    #[tokio::test]
    async fn test_read_binary_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(&[0x48, 0x65, 0x6c, 0x6c, 0x6f])
            .unwrap(); // "Hello" in hex

        let fetcher = DataFetcher::new();
        let content = fetcher.read_binary_file(temp_file.path()).await.unwrap();

        assert_eq!(content, vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]);
    }

    #[tokio::test]
    async fn test_read_json_file_typed() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Config {
            enabled: bool,
            timeout: u32,
        }

        let mut temp_file = NamedTempFile::new().unwrap();
        let config = Config {
            enabled: true,
            timeout: 30,
        };
        writeln!(temp_file, "{}", serde_json::to_string(&config).unwrap()).unwrap();

        let fetcher = DataFetcher::new();
        let loaded: Config = fetcher.read_json_file(temp_file.path()).await.unwrap();

        assert_eq!(loaded, config);
    }

    #[tokio::test]
    async fn test_file_not_found() {
        let fetcher = DataFetcher::new();
        let result = fetcher.read_text_file("/nonexistent/file.txt").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FetchError::IoError(_)));
    }

    #[tokio::test]
    async fn test_invalid_json_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{invalid json").unwrap();

        let fetcher = DataFetcher::new();
        let result = fetcher.read_json_value(temp_file.path()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FetchError::JsonError(_)));
    }

    #[tokio::test]
    async fn test_quick_get() {
        let response = quick::get("https://api.example.com").await.unwrap();
        assert!(response.is_success());
    }

    #[tokio::test]
    async fn test_quick_post_json() {
        #[derive(Serialize)]
        struct Data {
            value: i32,
        }

        let response = quick::post_json("https://api.example.com", &Data { value: 42 })
            .await
            .unwrap();
        assert!(response.is_success());
    }

    #[tokio::test]
    async fn test_quick_read_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"test": true}}"#).unwrap();

        let json: Value = quick::read_json(temp_file.path()).await.unwrap();
        assert_eq!(json["test"], true);
    }

    #[tokio::test]
    async fn test_file_metadata_readonly() {
        let temp_file = NamedTempFile::new().unwrap();
        let fetcher = DataFetcher::new();

        let metadata = fetcher.file_metadata(temp_file.path()).await.unwrap();
        assert!(metadata.is_file);
        assert!(!metadata.is_dir);
        // readonly status depends on OS and permissions
    }

    #[tokio::test]
    async fn test_http_request_default_timeout() {
        let request = HttpRequest::new("https://example.com");
        assert_eq!(request.timeout_secs, 30);
    }

    #[tokio::test]
    async fn test_http_request_multiple_headers() {
        let request = HttpRequest::new("https://example.com")
            .header("X-Header-1", "value1")
            .header("X-Header-2", "value2")
            .header("X-Header-3", "value3");

        assert_eq!(request.headers.len(), 3);
        assert_eq!(request.headers.get("X-Header-1").unwrap(), "value1");
        assert_eq!(request.headers.get("X-Header-2").unwrap(), "value2");
        assert_eq!(request.headers.get("X-Header-3").unwrap(), "value3");
    }

    #[tokio::test]
    async fn test_empty_file_read() {
        let temp_file = NamedTempFile::new().unwrap();

        let fetcher = DataFetcher::new();
        let content = fetcher.read_text_file(temp_file.path()).await.unwrap();

        assert_eq!(content, "");
    }

    #[tokio::test]
    async fn test_read_lines_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();

        let fetcher = DataFetcher::new();
        let lines = fetcher.read_lines(temp_file.path()).await.unwrap();

        assert_eq!(lines.len(), 0);
    }
}

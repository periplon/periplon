// Configuration subsystem with TOML support and environment variable substitution

#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use std::path::PathBuf;
#[cfg(feature = "server")]
use thiserror::Error;

#[cfg(feature = "server")]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    LoadError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Environment variable not set: {0}")]
    EnvVarNotSet(String),
}

#[cfg(feature = "server")]
pub type Result<T> = std::result::Result<T, ConfigError>;

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub user_storage: UserStorageConfig,
    #[serde(default)]
    pub queue: QueueConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub monitoring: MonitoringConfig,
    #[serde(default)]
    pub reliability: ReliabilityConfig,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default)]
    pub workers: bool,

    #[serde(default = "default_worker_concurrency")]
    pub worker_concurrency: usize,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default = "default_environment")]
    pub environment: String,

    #[serde(default)]
    pub tls: Option<TlsConfig>,

    #[serde(default)]
    pub cors: Option<CorsConfig>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,

    #[serde(default = "default_tls_version")]
    pub min_version: String,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allow_credentials: bool,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    pub backend: StorageBackend,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "lowercase")]
pub enum StorageBackend {
    Filesystem(FilesystemStorageConfig),
    S3(S3StorageConfig),
    Postgres(PostgresStorageConfig),
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilesystemStorageConfig {
    #[serde(default = "default_base_path")]
    pub base_path: PathBuf,

    #[serde(default = "default_workflows_dir")]
    pub workflows_dir: String,

    #[serde(default = "default_executions_dir")]
    pub executions_dir: String,

    #[serde(default = "default_checkpoints_dir")]
    pub checkpoints_dir: String,

    #[serde(default = "default_logs_dir")]
    pub logs_dir: String,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3StorageConfig {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,

    #[serde(default)]
    pub path_prefix: String,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgresStorageConfig {
    pub url: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,

    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserStorageConfig {
    pub backend: UserStorageBackend,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "lowercase")]
pub enum UserStorageBackend {
    Filesystem(FilesystemUserStorageConfig),
    S3(S3UserStorageConfig),
    Postgres(PostgresUserStorageConfig),
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilesystemUserStorageConfig {
    #[serde(default = "default_users_base_path")]
    pub base_path: PathBuf,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3UserStorageConfig {
    pub bucket: String,

    #[serde(default)]
    pub prefix: Option<String>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgresUserStorageConfig {
    pub url: String,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueConfig {
    pub backend: QueueBackend,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "lowercase")]
pub enum QueueBackend {
    Filesystem(FilesystemQueueConfig),
    S3(S3QueueConfig),
    Postgres(PostgresQueueConfig),
    Redis(RedisQueueConfig),
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilesystemQueueConfig {
    #[serde(default = "default_queue_dir")]
    pub queue_dir: PathBuf,

    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,

    #[serde(default = "default_lock_timeout_secs")]
    pub lock_timeout_secs: u64,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3QueueConfig {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,

    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,

    #[serde(default = "default_lock_timeout_secs")]
    pub lock_timeout_secs: u64,

    #[serde(default = "default_visibility_timeout_secs")]
    pub visibility_timeout_secs: u64,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgresQueueConfig {
    pub url: String,

    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,

    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisQueueConfig {
    pub url: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub jwt_secret: String,

    #[serde(default = "default_jwt_expiration_secs")]
    pub jwt_expiration_secs: u64,

    #[serde(default = "default_refresh_token_expiration_secs")]
    pub refresh_token_expiration_secs: u64,

    #[serde(default = "default_session_max_idle_secs")]
    pub session_max_idle_secs: u64,

    #[serde(default = "default_password_min_length")]
    pub password_min_length: usize,

    #[serde(default)]
    pub mfa_enabled: bool,

    #[serde(default)]
    pub oauth: Option<OAuthConfig>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuthConfig {
    pub enabled: bool,
    pub providers: Vec<OAuthProvider>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuthProvider {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Vec<String>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_global_requests_per_minute")]
    pub global_requests_per_minute: u32,

    #[serde(default = "default_per_user_requests_per_minute")]
    pub per_user_requests_per_minute: u32,

    #[serde(default = "default_per_api_key_requests_per_minute")]
    pub per_api_key_requests_per_minute: u32,

    #[serde(default = "default_per_ip_requests_per_minute")]
    pub per_ip_requests_per_minute: u32,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    #[serde(default = "default_true")]
    pub metrics_enabled: bool,

    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,

    #[serde(default)]
    pub tracing_enabled: bool,

    #[serde(default)]
    pub tracing_endpoint: Option<String>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReliabilityConfig {
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    #[serde(default = "default_retry_backoff_ms")]
    pub retry_backoff_ms: u64,

    #[serde(default = "default_retry_backoff_multiplier")]
    pub retry_backoff_multiplier: f64,

    #[serde(default = "default_circuit_breaker_threshold")]
    pub circuit_breaker_threshold: u32,

    #[serde(default = "default_circuit_breaker_timeout_secs")]
    pub circuit_breaker_timeout_secs: u64,

    #[serde(default = "default_health_check_interval_secs")]
    pub health_check_interval_secs: u64,
}

// Default value functions
#[cfg(feature = "server")]
fn default_host() -> String {
    "0.0.0.0".to_string()
}

#[cfg(feature = "server")]
fn default_port() -> u16 {
    8080
}

#[cfg(feature = "server")]
fn default_worker_concurrency() -> usize {
    3
}

#[cfg(feature = "server")]
fn default_log_level() -> String {
    "info".to_string()
}

#[cfg(feature = "server")]
fn default_environment() -> String {
    "development".to_string()
}

#[cfg(feature = "server")]
fn default_tls_version() -> String {
    "1.3".to_string()
}

#[cfg(feature = "server")]
fn default_base_path() -> PathBuf {
    PathBuf::from("./data")
}

#[cfg(feature = "server")]
fn default_workflows_dir() -> String {
    "workflows".to_string()
}

#[cfg(feature = "server")]
fn default_executions_dir() -> String {
    "executions".to_string()
}

#[cfg(feature = "server")]
fn default_checkpoints_dir() -> String {
    "checkpoints".to_string()
}

#[cfg(feature = "server")]
fn default_logs_dir() -> String {
    "logs".to_string()
}

#[cfg(feature = "server")]
fn default_users_base_path() -> PathBuf {
    PathBuf::from("./data/users")
}

#[cfg(feature = "server")]
fn default_queue_dir() -> PathBuf {
    PathBuf::from("./queue")
}

#[cfg(feature = "server")]
fn default_max_connections() -> u32 {
    20
}

#[cfg(feature = "server")]
fn default_min_connections() -> u32 {
    5
}

#[cfg(feature = "server")]
fn default_connection_timeout() -> u64 {
    30
}

#[cfg(feature = "server")]
fn default_idle_timeout() -> u64 {
    600
}

#[cfg(feature = "server")]
fn default_poll_interval_ms() -> u64 {
    1000
}

#[cfg(feature = "server")]
fn default_lock_timeout_secs() -> u64 {
    300
}

#[cfg(feature = "server")]
fn default_visibility_timeout_secs() -> u64 {
    600
}

#[cfg(feature = "server")]
fn default_max_retries() -> u32 {
    3
}

#[cfg(feature = "server")]
fn default_jwt_expiration_secs() -> u64 {
    3600
}

#[cfg(feature = "server")]
fn default_refresh_token_expiration_secs() -> u64 {
    2_592_000 // 30 days
}

#[cfg(feature = "server")]
fn default_session_max_idle_secs() -> u64 {
    1800
}

#[cfg(feature = "server")]
fn default_password_min_length() -> usize {
    12
}

#[cfg(feature = "server")]
fn default_true() -> bool {
    true
}

#[cfg(feature = "server")]
fn default_global_requests_per_minute() -> u32 {
    10_000
}

#[cfg(feature = "server")]
fn default_per_user_requests_per_minute() -> u32 {
    100
}

#[cfg(feature = "server")]
fn default_per_api_key_requests_per_minute() -> u32 {
    1000
}

#[cfg(feature = "server")]
fn default_per_ip_requests_per_minute() -> u32 {
    60
}

#[cfg(feature = "server")]
fn default_metrics_port() -> u16 {
    9090
}

#[cfg(feature = "server")]
fn default_retry_backoff_ms() -> u64 {
    1000
}

#[cfg(feature = "server")]
fn default_retry_backoff_multiplier() -> f64 {
    2.0
}

#[cfg(feature = "server")]
fn default_circuit_breaker_threshold() -> u32 {
    5
}

#[cfg(feature = "server")]
fn default_circuit_breaker_timeout_secs() -> u64 {
    60
}

#[cfg(feature = "server")]
fn default_health_check_interval_secs() -> u64 {
    30
}

// Default implementations for all config structs
#[cfg(feature = "server")]
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            workers: false,
            worker_concurrency: default_worker_concurrency(),
            log_level: default_log_level(),
            environment: default_environment(),
            tls: None,
            cors: None,
        }
    }
}

#[cfg(feature = "server")]
impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Filesystem(FilesystemStorageConfig::default()),
        }
    }
}

#[cfg(feature = "server")]
impl Default for FilesystemStorageConfig {
    fn default() -> Self {
        Self {
            base_path: default_base_path(),
            workflows_dir: default_workflows_dir(),
            executions_dir: default_executions_dir(),
            checkpoints_dir: default_checkpoints_dir(),
            logs_dir: default_logs_dir(),
        }
    }
}

#[cfg(feature = "server")]
impl Default for UserStorageConfig {
    fn default() -> Self {
        Self {
            backend: UserStorageBackend::Filesystem(FilesystemUserStorageConfig::default()),
        }
    }
}

#[cfg(feature = "server")]
impl Default for FilesystemUserStorageConfig {
    fn default() -> Self {
        Self {
            base_path: default_users_base_path(),
        }
    }
}

#[cfg(feature = "server")]
impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            backend: QueueBackend::Filesystem(FilesystemQueueConfig::default()),
        }
    }
}

#[cfg(feature = "server")]
impl Default for FilesystemQueueConfig {
    fn default() -> Self {
        Self {
            queue_dir: default_queue_dir(),
            poll_interval_ms: default_poll_interval_ms(),
            lock_timeout_secs: default_lock_timeout_secs(),
        }
    }
}

#[cfg(feature = "server")]
impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(), // Must be set via env var or config file
            jwt_expiration_secs: default_jwt_expiration_secs(),
            refresh_token_expiration_secs: default_refresh_token_expiration_secs(),
            session_max_idle_secs: default_session_max_idle_secs(),
            password_min_length: default_password_min_length(),
            mfa_enabled: false,
            oauth: None,
        }
    }
}

#[cfg(feature = "server")]
impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            global_requests_per_minute: default_global_requests_per_minute(),
            per_user_requests_per_minute: default_per_user_requests_per_minute(),
            per_api_key_requests_per_minute: default_per_api_key_requests_per_minute(),
            per_ip_requests_per_minute: default_per_ip_requests_per_minute(),
        }
    }
}

#[cfg(feature = "server")]
impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            metrics_port: default_metrics_port(),
            tracing_enabled: false,
            tracing_endpoint: None,
        }
    }
}

#[cfg(feature = "server")]
impl Default for ReliabilityConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            retry_backoff_ms: default_retry_backoff_ms(),
            retry_backoff_multiplier: default_retry_backoff_multiplier(),
            circuit_breaker_threshold: default_circuit_breaker_threshold(),
            circuit_breaker_timeout_secs: default_circuit_breaker_timeout_secs(),
            health_check_interval_secs: default_health_check_interval_secs(),
        }
    }
}

#[cfg(feature = "server")]
impl Config {
    /// Load configuration from file, environment variables, and CLI args
    /// Precedence: CLI Args > Environment Variables > Config File > Defaults
    pub fn load(config_path: Option<PathBuf>) -> Result<Self> {
        use config::{File, FileFormat};

        let mut builder = config::Config::builder();

        // Load from file if provided
        if let Some(path) = config_path {
            builder = builder.add_source(File::from(path).format(FileFormat::Toml).required(true));
        }

        // Load from environment variables
        builder =
            builder.add_source(config::Environment::with_prefix("CLAUDE_DSL").separator("__"));

        let config = builder
            .build()
            .map_err(|e| ConfigError::LoadError(e.to_string()))?;

        let mut loaded: Config = config
            .try_deserialize()
            .map_err(|e| ConfigError::LoadError(e.to_string()))?;

        // Substitute environment variables in string fields
        loaded.substitute_env_vars()?;

        // Validate configuration
        loaded.validate()?;

        Ok(loaded)
    }

    /// Substitute environment variables in configuration values
    fn substitute_env_vars(&mut self) -> Result<()> {
        use regex::Regex;
        use std::env;

        let re = Regex::new(r"\$\{([^}]+)\}").unwrap();

        // Helper to substitute env vars in a string
        let substitute = |s: &str| -> Result<String> {
            let mut result = s.to_string();
            for cap in re.captures_iter(s) {
                let var_name = &cap[1];
                let value = env::var(var_name)
                    .map_err(|_| ConfigError::EnvVarNotSet(var_name.to_string()))?;
                result = result.replace(&format!("${{{}}}", var_name), &value);
            }
            Ok(result)
        };

        // Substitute in auth config
        self.auth.jwt_secret = substitute(&self.auth.jwt_secret)?;

        // Substitute in OAuth providers
        if let Some(oauth) = &mut self.auth.oauth {
            for provider in &mut oauth.providers {
                provider.client_id = substitute(&provider.client_id)?;
                provider.client_secret = substitute(&provider.client_secret)?;
            }
        }

        // Substitute in storage config
        match &mut self.storage.backend {
            StorageBackend::S3(s3) => {
                s3.access_key_id = substitute(&s3.access_key_id)?;
                s3.secret_access_key = substitute(&s3.secret_access_key)?;
            }
            StorageBackend::Postgres(pg) => {
                pg.url = substitute(&pg.url)?;
            }
            _ => {}
        }

        // Substitute in queue config
        match &mut self.queue.backend {
            QueueBackend::Postgres(pg) => {
                pg.url = substitute(&pg.url)?;
            }
            QueueBackend::Redis(redis) => {
                redis.url = substitute(&redis.url)?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Check storage backend is configured
        match &self.storage.backend {
            StorageBackend::Postgres(pg) if pg.url.is_empty() => {
                return Err(ConfigError::ValidationError(
                    "PostgreSQL URL must be set".to_string(),
                ));
            }
            StorageBackend::S3(s3) if s3.bucket.is_empty() => {
                return Err(ConfigError::ValidationError(
                    "S3 bucket must be set".to_string(),
                ));
            }
            _ => {}
        }

        // Ensure secrets are set (required for JWT signing)
        if self.auth.jwt_secret.is_empty() || self.auth.jwt_secret.contains("${") {
            if self.server.environment == "production" {
                return Err(ConfigError::ValidationError(
                    "JWT secret must be set in production via JWT_SECRET environment variable"
                        .to_string(),
                ));
            } else {
                // In development, warn but allow (a default secret will be generated)
                eprintln!("⚠️  Warning: JWT_SECRET not set. Using development secret (DO NOT use in production)");
            }
        }

        // Validate TLS certs exist if enabled
        if let Some(tls) = &self.server.tls {
            if tls.enabled {
                if !tls.cert_path.exists() {
                    return Err(ConfigError::ValidationError(format!(
                        "TLS cert not found: {:?}",
                        tls.cert_path
                    )));
                }
                if !tls.key_path.exists() {
                    return Err(ConfigError::ValidationError(format!(
                        "TLS key not found: {:?}",
                        tls.key_path
                    )));
                }
            }
        }

        Ok(())
    }

    /// Generate a development JWT secret if none is set
    pub fn ensure_jwt_secret(&mut self) {
        if self.auth.jwt_secret.is_empty() || self.auth.jwt_secret.contains("${") {
            // Generate a random development secret
            use sha2::{Digest, Sha256};
            let random_bytes: [u8; 32] = rand::random();
            let mut hasher = Sha256::new();
            hasher.update(random_bytes);
            self.auth.jwt_secret = format!("{:x}", hasher.finalize());
        }
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config {
            server: ServerConfig {
                host: default_host(),
                port: default_port(),
                workers: false,
                worker_concurrency: default_worker_concurrency(),
                log_level: default_log_level(),
                environment: default_environment(),
                tls: None,
                cors: None,
            },
            storage: StorageConfig {
                backend: StorageBackend::Filesystem(FilesystemStorageConfig {
                    base_path: default_base_path(),
                    workflows_dir: default_workflows_dir(),
                    executions_dir: default_executions_dir(),
                    checkpoints_dir: default_checkpoints_dir(),
                    logs_dir: default_logs_dir(),
                }),
            },
            user_storage: UserStorageConfig {
                backend: UserStorageBackend::Filesystem(FilesystemUserStorageConfig {
                    base_path: default_users_base_path(),
                }),
            },
            queue: QueueConfig {
                backend: QueueBackend::Filesystem(FilesystemQueueConfig {
                    queue_dir: default_queue_dir(),
                    poll_interval_ms: default_poll_interval_ms(),
                    lock_timeout_secs: default_lock_timeout_secs(),
                }),
            },
            auth: AuthConfig {
                jwt_secret: "test-secret".to_string(),
                jwt_expiration_secs: default_jwt_expiration_secs(),
                refresh_token_expiration_secs: default_refresh_token_expiration_secs(),
                session_max_idle_secs: default_session_max_idle_secs(),
                password_min_length: default_password_min_length(),
                mfa_enabled: false,
                oauth: None,
            },
            rate_limit: RateLimitConfig {
                enabled: true,
                global_requests_per_minute: default_global_requests_per_minute(),
                per_user_requests_per_minute: default_per_user_requests_per_minute(),
                per_api_key_requests_per_minute: default_per_api_key_requests_per_minute(),
                per_ip_requests_per_minute: default_per_ip_requests_per_minute(),
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                metrics_port: default_metrics_port(),
                tracing_enabled: false,
                tracing_endpoint: None,
            },
            reliability: ReliabilityConfig {
                max_retries: default_max_retries(),
                retry_backoff_ms: default_retry_backoff_ms(),
                retry_backoff_multiplier: default_retry_backoff_multiplier(),
                circuit_breaker_threshold: default_circuit_breaker_threshold(),
                circuit_breaker_timeout_secs: default_circuit_breaker_timeout_secs(),
                health_check_interval_secs: default_health_check_interval_secs(),
            },
        };

        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.worker_concurrency, 3);
    }
}

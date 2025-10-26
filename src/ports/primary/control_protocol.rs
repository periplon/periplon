use crate::domain::{ControlConfig, PermissionMode};
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ControlProtocol: Send + Sync {
    async fn initialize(&self, config: ControlConfig) -> Result<()>;
    async fn set_permission_mode(&self, mode: PermissionMode) -> Result<()>;
    async fn set_model(&self, model: Option<String>) -> Result<()>;
}

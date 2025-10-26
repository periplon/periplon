use crate::domain::{PermissionDecision, ToolPermissionContext};
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PermissionService: Send + Sync {
    async fn can_use_tool(
        &self,
        tool_name: &str,
        input: &serde_json::Value,
        context: ToolPermissionContext,
    ) -> Result<PermissionDecision>;
}

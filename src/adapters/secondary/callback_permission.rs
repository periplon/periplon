use crate::domain::{PermissionDecision, PermissionResult, ToolPermissionContext};
use crate::error::Result;
use crate::options::CanUseToolCallback;
use crate::ports::secondary::PermissionService;
use async_trait::async_trait;

pub struct CallbackPermissionService {
    callback: CanUseToolCallback,
}

impl CallbackPermissionService {
    pub fn new(callback: CanUseToolCallback) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl PermissionService for CallbackPermissionService {
    async fn can_use_tool(
        &self,
        tool_name: &str,
        input: &serde_json::Value,
        context: ToolPermissionContext,
    ) -> Result<PermissionDecision> {
        let result = (self.callback)(tool_name.to_string(), input.clone(), context).await;

        Ok(match result {
            PermissionResult::Allow { updated_input, .. } => {
                PermissionDecision::Allow { updated_input }
            }
            PermissionResult::Deny { message, .. } => PermissionDecision::Deny { reason: message },
        })
    }
}

/// Mock permission service that denies all requests
pub struct DenyAllPermissionService;

#[async_trait]
impl PermissionService for DenyAllPermissionService {
    async fn can_use_tool(
        &self,
        _tool_name: &str,
        _input: &serde_json::Value,
        _context: ToolPermissionContext,
    ) -> Result<PermissionDecision> {
        Ok(PermissionDecision::Deny {
            reason: "All requests denied by mock".to_string(),
        })
    }
}

/// Mock permission service that allows all requests
pub struct AllowAllPermissionService;

#[async_trait]
impl PermissionService for AllowAllPermissionService {
    async fn can_use_tool(
        &self,
        _tool_name: &str,
        _input: &serde_json::Value,
        _context: ToolPermissionContext,
    ) -> Result<PermissionDecision> {
        Ok(PermissionDecision::Allow {
            updated_input: None,
        })
    }
}

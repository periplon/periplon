use crate::domain::{HookContext, HookInput, HookJSONOutput};
use crate::error::Result;
use async_trait::async_trait;

pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    UserPromptSubmit,
    Stop,
    SubagentStop,
    PreCompact,
}

#[async_trait]
pub trait HookService: Send + Sync {
    async fn execute_hook(
        &self,
        event: HookEvent,
        input: HookInput,
        context: HookContext,
    ) -> Result<HookJSONOutput>;
}

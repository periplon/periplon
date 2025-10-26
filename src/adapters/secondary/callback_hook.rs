use crate::domain::{HookCallback, HookContext, HookInput, HookJSONOutput};
use crate::error::Result;
use crate::ports::secondary::{HookEvent, HookService};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct CallbackHookService {
    hooks: HashMap<String, HookCallback>,
}

impl CallbackHookService {
    pub fn new(hooks: HashMap<String, HookCallback>) -> Self {
        Self { hooks }
    }

    pub fn add_hook(&mut self, id: String, callback: HookCallback) {
        self.hooks.insert(id, callback);
    }
}

#[async_trait]
impl HookService for CallbackHookService {
    async fn execute_hook(
        &self,
        _event: HookEvent,
        _input: HookInput,
        _context: HookContext,
    ) -> Result<HookJSONOutput> {
        // In real implementation, we would match the event and input to find the right hook
        // For now, just return a default sync output
        Ok(HookJSONOutput::Sync {
            should_continue: Some(true),
            suppress_output: None,
            stop_reason: None,
            decision: None,
            system_message: None,
            reason: None,
            hook_specific_output: None,
        })
    }
}

/// No-op hook service that does nothing
pub struct NoOpHookService;

#[async_trait]
impl HookService for NoOpHookService {
    async fn execute_hook(
        &self,
        _event: HookEvent,
        _input: HookInput,
        _context: HookContext,
    ) -> Result<HookJSONOutput> {
        Ok(HookJSONOutput::Sync {
            should_continue: Some(true),
            suppress_output: None,
            stop_reason: None,
            decision: None,
            system_message: None,
            reason: None,
            hook_specific_output: None,
        })
    }
}

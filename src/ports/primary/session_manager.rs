use crate::domain::SessionId;
use crate::error::Result;
use crate::options::AgentOptions;
use async_trait::async_trait;

#[async_trait]
pub trait SessionManager: Send + Sync {
    async fn create_session(&self, options: AgentOptions) -> Result<SessionId>;
    async fn resume_session(&self, session_id: SessionId) -> Result<()>;
    async fn close_session(&self, session_id: SessionId) -> Result<()>;
}

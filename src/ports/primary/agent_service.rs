use crate::domain::{AgentSession, Message, SessionId};
use crate::error::Result;
use crate::options::AgentOptions;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

pub type MessageStream = Pin<Box<dyn Stream<Item = Result<Message>> + Send>>;

#[async_trait]
pub trait AgentService: Send + Sync {
    async fn query(&self, prompt: String, options: AgentOptions) -> Result<MessageStream>;
    async fn send_message(&self, session: SessionId, message: Message) -> Result<()>;
    async fn interrupt(&self, session: SessionId) -> Result<()>;
    async fn get_session(&self, session: SessionId) -> Result<AgentSession>;
}

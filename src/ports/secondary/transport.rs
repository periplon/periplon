use crate::error::Result;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

#[async_trait]
pub trait Transport: Send + Sync {
    /// Initialize connection to CLI
    async fn connect(&mut self) -> Result<()>;

    /// Write data to CLI stdin (must end with newline)
    async fn write(&mut self, data: &str) -> Result<()>;

    /// Read messages from CLI stdout
    fn read_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>>;

    /// Close stdin (signal end of input)
    async fn end_input(&mut self) -> Result<()>;

    /// Close the transport
    async fn close(&mut self) -> Result<()>;

    /// Check if transport is ready
    fn is_ready(&self) -> bool;
}

use crate::error::Result;
use crate::ports::secondary::Transport;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

pub struct MockTransport {
    messages: Vec<serde_json::Value>,
    write_log: Vec<String>,
}

impl MockTransport {
    pub fn new(messages: Vec<serde_json::Value>) -> Self {
        Self {
            messages,
            write_log: Vec::new(),
        }
    }

    pub fn with_responses(messages: Vec<serde_json::Value>) -> Self {
        Self::new(messages)
    }

    pub fn write_log(&self) -> &[String] {
        &self.write_log
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn connect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn write(&mut self, data: &str) -> Result<()> {
        self.write_log.push(data.to_string());
        Ok(())
    }

    fn read_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>> {
        Box::pin(async_stream::try_stream! {
            for msg in &self.messages {
                yield msg.clone();
            }
        })
    }

    async fn end_input(&mut self) -> Result<()> {
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn is_ready(&self) -> bool {
        true
    }
}

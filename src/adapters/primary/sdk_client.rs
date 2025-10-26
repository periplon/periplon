use crate::adapters::secondary::{PromptType, SubprocessCLITransport};
use crate::application::Query;
use crate::domain::Message;
use crate::error::{Error, Result};
use crate::options::AgentOptions;
use futures::{Stream, StreamExt};
use serde_json::json;

pub struct PeriplonSDKClient {
    options: AgentOptions,
    query: Option<Query>,
}

impl PeriplonSDKClient {
    pub fn new(options: AgentOptions) -> Self {
        Self {
            options,
            query: None,
        }
    }

    /// Connect to CLI
    pub async fn connect(&mut self, prompt: Option<String>) -> Result<()> {
        let prompt_type = if let Some(text) = prompt {
            PromptType::String(text)
        } else {
            PromptType::Stream
        };

        let transport = Box::new(SubprocessCLITransport::new(
            prompt_type,
            self.options.clone(),
        ));

        let (mut query, write_rx) = Query::new(
            transport,
            true, // Always streaming mode for PeriplonSDKClient
            self.options.can_use_tool.clone(),
            self.options.hooks.clone(),
        );

        // Connect transport
        {
            let transport_arc = query.transport();
            let mut transport_lock = transport_arc.lock().await;
            transport_lock.connect().await?;
        }

        query.start(write_rx).await?;

        // TODO: Build hooks config and initialize
        // Skip initialization if no hooks configured to avoid hanging
        if self.options.hooks.is_some() && !self.options.hooks.as_ref().unwrap().is_empty() {
            query.initialize(None).await?;
        }

        self.query = Some(query);

        Ok(())
    }

    /// Send a query
    pub async fn query(&mut self, prompt: impl Into<String>) -> Result<()> {
        let query = self.query.as_mut().ok_or(Error::NotConnected)?;

        let message = json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": prompt.into()
            },
            "session_id": "default"
        });

        let json_str = serde_json::to_string(&message)?;
        query.write(format!("{}\n", json_str))?;

        Ok(())
    }

    /// Receive messages
    pub fn receive_messages(&self) -> Result<impl Stream<Item = Message> + '_> {
        let query = self.query.as_ref().ok_or(Error::NotConnected)?;
        Ok(query.receive_messages())
    }

    /// Receive until ResultMessage
    pub fn receive_response(&self) -> Result<impl Stream<Item = Message> + '_> {
        let stream = self.receive_messages()?;

        Ok(async_stream::stream! {
            futures::pin_mut!(stream);
            while let Some(msg) = stream.next().await {
                let is_result = matches!(msg, Message::Result(_));
                yield msg;
                if is_result {
                    break;
                }
            }
        })
    }

    /// Send interrupt
    pub async fn interrupt(&self) -> Result<()> {
        let query = self.query.as_ref().ok_or(Error::NotConnected)?;
        query.interrupt().await
    }

    /// Set permission mode
    pub async fn set_permission_mode(&self, mode: impl Into<String>) -> Result<()> {
        let query = self.query.as_ref().ok_or(Error::NotConnected)?;
        query.set_permission_mode(mode.into()).await
    }

    /// Set model
    pub async fn set_model(&self, model: Option<impl Into<String>>) -> Result<()> {
        let query = self.query.as_ref().ok_or(Error::NotConnected)?;
        query.set_model(model.map(|m| m.into())).await
    }

    /// Disconnect
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut query) = self.query.take() {
            query.close().await?;
        }
        Ok(())
    }
}

//! Message Bus for Inter-Agent Communication
//!
//! This module provides a publish-subscribe message bus that enables agents to
//! communicate and coordinate during workflow execution.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Maximum number of messages in a channel before old ones are dropped
const CHANNEL_CAPACITY: usize = 1000;

/// A message sent between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Source agent name
    pub from: String,
    /// Target agent name (or channel name)
    pub to: String,
    /// Message type identifier
    pub message_type: String,
    /// Message payload (JSON)
    pub payload: serde_json::Value,
    /// Message timestamp
    pub timestamp: std::time::SystemTime,
}

impl AgentMessage {
    /// Create a new agent message
    pub fn new(from: String, to: String, message_type: String, payload: serde_json::Value) -> Self {
        Self {
            from,
            to,
            message_type,
            payload,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

/// Communication channel for agents
#[derive(Clone)]
pub struct Channel {
    /// Channel name
    pub name: String,
    /// Description of the channel
    pub description: String,
    /// Agents allowed to participate in this channel
    pub participants: Vec<String>,
    /// Message format (e.g., "json", "markdown")
    pub message_format: String,
    /// Broadcast sender for this channel
    sender: broadcast::Sender<AgentMessage>,
}

impl Channel {
    /// Create a new communication channel
    pub fn new(
        name: String,
        description: String,
        participants: Vec<String>,
        message_format: String,
    ) -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self {
            name,
            description,
            participants,
            message_format,
            sender,
        }
    }

    /// Send a message to this channel
    pub fn send(&self, message: AgentMessage) -> Result<()> {
        self.sender
            .send(message)
            .map_err(|e| Error::InvalidInput(format!("Failed to send message: {}", e)))?;
        Ok(())
    }

    /// Subscribe to this channel
    pub fn subscribe(&self) -> broadcast::Receiver<AgentMessage> {
        self.sender.subscribe()
    }

    /// Check if an agent is a participant in this channel
    pub fn has_participant(&self, agent_name: &str) -> bool {
        self.participants.contains(&agent_name.to_string())
    }
}

/// Message bus for inter-agent communication
pub struct MessageBus {
    /// Communication channels indexed by name
    channels: Arc<RwLock<HashMap<String, Channel>>>,
    /// Direct message queues for each agent
    direct_messages: Arc<RwLock<HashMap<String, broadcast::Sender<AgentMessage>>>>,
}

impl MessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            direct_messages: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a communication channel
    pub async fn create_channel(
        &self,
        name: String,
        description: String,
        participants: Vec<String>,
        message_format: String,
    ) -> Result<()> {
        let channel = Channel::new(name.clone(), description, participants, message_format);

        let mut channels = self.channels.write().await;
        if channels.contains_key(&name) {
            return Err(Error::InvalidInput(format!(
                "Channel '{}' already exists",
                name
            )));
        }

        channels.insert(name, channel);
        Ok(())
    }

    /// Register an agent for direct messaging
    pub async fn register_agent(&self, agent_name: String) -> Result<()> {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);

        let mut direct_messages = self.direct_messages.write().await;
        if direct_messages.contains_key(&agent_name) {
            return Err(Error::InvalidInput(format!(
                "Agent '{}' already registered",
                agent_name
            )));
        }

        direct_messages.insert(agent_name, sender);
        Ok(())
    }

    /// Send a message to a channel
    pub async fn send_to_channel(&self, channel_name: &str, message: AgentMessage) -> Result<()> {
        let channels = self.channels.read().await;
        let channel = channels
            .get(channel_name)
            .ok_or_else(|| Error::InvalidInput(format!("Channel '{}' not found", channel_name)))?;

        // Check if sender is a participant
        if !channel.has_participant(&message.from) {
            return Err(Error::InvalidInput(format!(
                "Agent '{}' is not a participant in channel '{}'",
                message.from, channel_name
            )));
        }

        channel.send(message)
    }

    /// Send a direct message to an agent
    pub async fn send_direct(&self, message: AgentMessage) -> Result<()> {
        let direct_messages = self.direct_messages.read().await;
        let sender = direct_messages
            .get(&message.to)
            .ok_or_else(|| Error::InvalidInput(format!("Agent '{}' not registered", message.to)))?;

        sender
            .send(message)
            .map_err(|e| Error::InvalidInput(format!("Failed to send direct message: {}", e)))?;

        Ok(())
    }

    /// Subscribe to a channel
    pub async fn subscribe_to_channel(
        &self,
        channel_name: &str,
        agent_name: &str,
    ) -> Result<broadcast::Receiver<AgentMessage>> {
        let channels = self.channels.read().await;
        let channel = channels
            .get(channel_name)
            .ok_or_else(|| Error::InvalidInput(format!("Channel '{}' not found", channel_name)))?;

        // Check if agent is a participant
        if !channel.has_participant(agent_name) {
            return Err(Error::InvalidInput(format!(
                "Agent '{}' is not a participant in channel '{}'",
                agent_name, channel_name
            )));
        }

        Ok(channel.subscribe())
    }

    /// Subscribe to direct messages for an agent
    pub async fn subscribe_to_direct(
        &self,
        agent_name: &str,
    ) -> Result<broadcast::Receiver<AgentMessage>> {
        let direct_messages = self.direct_messages.read().await;
        let sender = direct_messages
            .get(agent_name)
            .ok_or_else(|| Error::InvalidInput(format!("Agent '{}' not registered", agent_name)))?;

        Ok(sender.subscribe())
    }

    /// Get channel information
    pub async fn get_channel(&self, channel_name: &str) -> Result<Channel> {
        let channels = self.channels.read().await;
        channels
            .get(channel_name)
            .cloned()
            .ok_or_else(|| Error::InvalidInput(format!("Channel '{}' not found", channel_name)))
    }

    /// List all channels
    pub async fn list_channels(&self) -> Vec<String> {
        let channels = self.channels.read().await;
        channels.keys().cloned().collect()
    }

    /// Get number of registered agents
    pub async fn agent_count(&self) -> usize {
        let direct_messages = self.direct_messages.read().await;
        direct_messages.len()
    }

    /// Get number of channels
    pub async fn channel_count(&self) -> usize {
        let channels = self.channels.read().await;
        channels.len()
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_message_bus_creation() {
        let bus = MessageBus::new();
        assert_eq!(bus.agent_count().await, 0);
        assert_eq!(bus.channel_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_agent() {
        let bus = MessageBus::new();
        bus.register_agent("agent1".to_string()).await.unwrap();
        assert_eq!(bus.agent_count().await, 1);
    }

    #[tokio::test]
    async fn test_create_channel() {
        let bus = MessageBus::new();
        bus.create_channel(
            "test_channel".to_string(),
            "Test channel".to_string(),
            vec!["agent1".to_string(), "agent2".to_string()],
            "json".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(bus.channel_count().await, 1);
        let channel = bus.get_channel("test_channel").await.unwrap();
        assert_eq!(channel.participants.len(), 2);
    }

    #[tokio::test]
    async fn test_send_to_channel() {
        let bus = MessageBus::new();

        // Create channel
        bus.create_channel(
            "test_channel".to_string(),
            "Test channel".to_string(),
            vec!["agent1".to_string(), "agent2".to_string()],
            "json".to_string(),
        )
        .await
        .unwrap();

        // Subscribe before sending
        let mut receiver = bus
            .subscribe_to_channel("test_channel", "agent2")
            .await
            .unwrap();

        // Send message
        let message = AgentMessage::new(
            "agent1".to_string(),
            "test_channel".to_string(),
            "test".to_string(),
            serde_json::json!({"data": "test"}),
        );

        bus.send_to_channel("test_channel", message.clone())
            .await
            .unwrap();

        // Receive message
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.from, "agent1");
        assert_eq!(received.message_type, "test");
    }

    #[tokio::test]
    async fn test_direct_message() {
        let bus = MessageBus::new();

        // Register agents
        bus.register_agent("agent1".to_string()).await.unwrap();
        bus.register_agent("agent2".to_string()).await.unwrap();

        // Subscribe to direct messages
        let mut receiver = bus.subscribe_to_direct("agent2").await.unwrap();

        // Send direct message
        let message = AgentMessage::new(
            "agent1".to_string(),
            "agent2".to_string(),
            "greeting".to_string(),
            serde_json::json!({"message": "hello"}),
        );

        bus.send_direct(message).await.unwrap();

        // Receive message
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.from, "agent1");
        assert_eq!(received.to, "agent2");
    }

    #[tokio::test]
    async fn test_non_participant_cannot_send() {
        let bus = MessageBus::new();

        bus.create_channel(
            "private_channel".to_string(),
            "Private channel".to_string(),
            vec!["agent1".to_string()],
            "json".to_string(),
        )
        .await
        .unwrap();

        // agent2 is not a participant
        let message = AgentMessage::new(
            "agent2".to_string(),
            "private_channel".to_string(),
            "test".to_string(),
            serde_json::json!({}),
        );

        let result = bus.send_to_channel("private_channel", message).await;
        assert!(result.is_err());
    }
}

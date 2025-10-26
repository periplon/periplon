use super::message::Message;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum SessionState {
    Idle,
    Running,
    WaitingForInput,
    Completed,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct AgentSession {
    pub id: SessionId,
    pub messages: Vec<Message>,
    pub state: SessionState,
}

impl AgentSession {
    pub fn new(id: SessionId) -> Self {
        Self {
            id,
            messages: Vec::new(),
            state: SessionState::Idle,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn set_state(&mut self, state: SessionState) {
        self.state = state;
    }
}

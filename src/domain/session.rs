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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_new_generates_unique_ids() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1.as_str(), id2.as_str());
        assert_eq!(id1.as_str().len(), 36); // UUID v4 format
    }

    #[test]
    fn test_session_id_from_string() {
        let test_str = "test-session-123".to_string();
        let id = SessionId::from_string(test_str.clone());
        assert_eq!(id.as_str(), test_str);
    }

    #[test]
    fn test_session_id_as_str() {
        let id = SessionId::from_string("my-id".to_string());
        assert_eq!(id.as_str(), "my-id");
    }

    #[test]
    fn test_session_id_default() {
        let id = SessionId::default();
        assert_eq!(id.as_str().len(), 36);
    }

    #[test]
    fn test_session_id_display() {
        let id = SessionId::from_string("display-id".to_string());
        assert_eq!(format!("{}", id), "display-id");
    }

    #[test]
    fn test_agent_session_new() {
        let id = SessionId::from_string("test".to_string());
        let session = AgentSession::new(id.clone());
        assert_eq!(session.id, id);
        assert!(session.messages.is_empty());
        assert!(matches!(session.state, SessionState::Idle));
    }

    #[test]
    fn test_agent_session_add_message() {
        let mut session = AgentSession::new(SessionId::new());
        let msg = Message::user("test");
        session.add_message(msg);
        assert_eq!(session.messages.len(), 1);
    }

    #[test]
    fn test_agent_session_set_state() {
        let mut session = AgentSession::new(SessionId::new());
        session.set_state(SessionState::Running);
        assert!(matches!(session.state, SessionState::Running));
        session.set_state(SessionState::Completed);
        assert!(matches!(session.state, SessionState::Completed));
    }
}

use periplon_sdk::domain::session::{AgentSession, SessionId, SessionState};
use periplon_sdk::domain::Message;

#[test]
fn test_session_id_new() {
    let id1 = SessionId::new();
    let id2 = SessionId::new();

    // IDs should be unique
    assert_ne!(id1.as_str(), id2.as_str());

    // IDs should be valid UUID v4 strings (36 chars with hyphens)
    assert_eq!(id1.as_str().len(), 36);
    assert!(id1.as_str().contains('-'));
}

#[test]
fn test_session_id_from_string() {
    let test_str = "test-session-123";
    let id = SessionId::from_string(test_str.to_string());

    assert_eq!(id.as_str(), test_str);
}

#[test]
fn test_session_id_as_str() {
    let test_str = "my-session-id";
    let id = SessionId::from_string(test_str.to_string());

    assert_eq!(id.as_str(), test_str);
}

#[test]
fn test_session_id_default() {
    let id = SessionId::default();

    // Default should create a new UUID
    assert_eq!(id.as_str().len(), 36);
}

#[test]
fn test_session_id_display() {
    let test_str = "display-test-id";
    let id = SessionId::from_string(test_str.to_string());

    assert_eq!(format!("{}", id), test_str);
}

#[test]
fn test_session_id_debug() {
    let test_str = "debug-test-id";
    let id = SessionId::from_string(test_str.to_string());

    let debug_str = format!("{:?}", id);
    assert!(debug_str.contains("SessionId"));
    assert!(debug_str.contains(test_str));
}

#[test]
fn test_session_id_clone() {
    let id1 = SessionId::from_string("clone-test".to_string());
    let id2 = id1.clone();

    assert_eq!(id1.as_str(), id2.as_str());
}

#[test]
fn test_session_id_partial_eq() {
    let id1 = SessionId::from_string("same-id".to_string());
    let id2 = SessionId::from_string("same-id".to_string());
    let id3 = SessionId::from_string("different-id".to_string());

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_session_id_hash() {
    use std::collections::HashMap;

    let id1 = SessionId::from_string("hashable-id".to_string());
    let id2 = id1.clone();

    let mut map = HashMap::new();
    map.insert(id1, "value1");

    // Should be able to retrieve using cloned ID
    assert_eq!(map.get(&id2), Some(&"value1"));
}

#[test]
fn test_session_state_idle() {
    let state = SessionState::Idle;
    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("Idle"));
}

#[test]
fn test_session_state_running() {
    let state = SessionState::Running;
    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("Running"));
}

#[test]
fn test_session_state_waiting_for_input() {
    let state = SessionState::WaitingForInput;
    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("WaitingForInput"));
}

#[test]
fn test_session_state_completed() {
    let state = SessionState::Completed;
    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("Completed"));
}

#[test]
fn test_session_state_error() {
    let error_msg = "Something went wrong";
    let state = SessionState::Error(error_msg.to_string());

    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("Error"));
    assert!(debug_str.contains(error_msg));
}

#[test]
fn test_session_state_clone() {
    let state1 = SessionState::Error("test error".to_string());
    let state2 = state1.clone();

    match (state1, state2) {
        (SessionState::Error(msg1), SessionState::Error(msg2)) => {
            assert_eq!(msg1, msg2);
        }
        _ => panic!("Expected both to be Error states"),
    }
}

#[test]
fn test_agent_session_new() {
    let id = SessionId::from_string("test-session".to_string());
    let session = AgentSession::new(id.clone());

    assert_eq!(session.id, id);
    assert!(session.messages.is_empty());

    match session.state {
        SessionState::Idle => {}
        _ => panic!("Expected initial state to be Idle"),
    }
}

#[test]
fn test_agent_session_add_message() {
    let id = SessionId::from_string("test-session".to_string());
    let mut session = AgentSession::new(id);

    assert_eq!(session.messages.len(), 0);

    let message = Message::user("Hello");

    session.add_message(message);
    assert_eq!(session.messages.len(), 1);

    match &session.messages[0] {
        Message::User(user_msg) => {
            assert_eq!(user_msg.message.role, "user");
        }
        _ => panic!("Expected User message"),
    }
}

#[test]
fn test_agent_session_add_multiple_messages() {
    let id = SessionId::from_string("test-session".to_string());
    let mut session = AgentSession::new(id);

    session.add_message(Message::user("First"));
    session.add_message(Message::user("Second"));
    session.add_message(Message::user("Third"));

    assert_eq!(session.messages.len(), 3);
}

#[test]
fn test_agent_session_set_state() {
    let id = SessionId::from_string("test-session".to_string());
    let mut session = AgentSession::new(id);

    // Initial state is Idle
    match session.state {
        SessionState::Idle => {}
        _ => panic!("Expected Idle state"),
    }

    // Change to Running
    session.set_state(SessionState::Running);
    match session.state {
        SessionState::Running => {}
        _ => panic!("Expected Running state"),
    }

    // Change to WaitingForInput
    session.set_state(SessionState::WaitingForInput);
    match session.state {
        SessionState::WaitingForInput => {}
        _ => panic!("Expected WaitingForInput state"),
    }

    // Change to Completed
    session.set_state(SessionState::Completed);
    match session.state {
        SessionState::Completed => {}
        _ => panic!("Expected Completed state"),
    }

    // Change to Error
    session.set_state(SessionState::Error("test error".to_string()));
    match session.state {
        SessionState::Error(msg) => {
            assert_eq!(msg, "test error");
        }
        _ => panic!("Expected Error state"),
    }
}

#[test]
fn test_agent_session_state_transitions() {
    let id = SessionId::from_string("test-session".to_string());
    let mut session = AgentSession::new(id);

    // Simulate a typical session lifecycle

    // Start: Idle
    match session.state {
        SessionState::Idle => {}
        _ => panic!("Expected Idle"),
    }

    // Agent starts processing
    session.set_state(SessionState::Running);

    // Agent needs user input
    session.set_state(SessionState::WaitingForInput);

    // Agent resumes after input
    session.set_state(SessionState::Running);

    // Agent completes
    session.set_state(SessionState::Completed);

    match session.state {
        SessionState::Completed => {}
        _ => panic!("Expected Completed"),
    }
}

#[test]
fn test_agent_session_clone() {
    let id = SessionId::from_string("test-session".to_string());
    let mut session1 = AgentSession::new(id);

    session1.add_message(Message::user("Test"));
    session1.set_state(SessionState::Running);

    let session2 = session1.clone();

    assert_eq!(session2.id, session1.id);
    assert_eq!(session2.messages.len(), session1.messages.len());
}

#[test]
fn test_agent_session_debug() {
    let id = SessionId::from_string("test-session".to_string());
    let session = AgentSession::new(id);

    let debug_str = format!("{:?}", session);
    assert!(debug_str.contains("AgentSession"));
    assert!(debug_str.contains("test-session"));
}

#[test]
fn test_agent_session_with_complex_workflow() {
    let id = SessionId::from_string("workflow-session".to_string());
    let mut session = AgentSession::new(id);

    // User sends initial query
    session.add_message(Message::user("What is the weather?"));
    session.set_state(SessionState::Running);

    // Agent processes and responds (simulated)
    session.set_state(SessionState::Completed);

    // Verify final state
    assert_eq!(session.messages.len(), 1);
    match session.state {
        SessionState::Completed => {}
        _ => panic!("Expected Completed state"),
    }
}

#[test]
fn test_session_id_uniqueness_across_defaults() {
    use std::collections::HashSet;

    let mut ids = HashSet::new();

    // Generate 100 IDs and verify they're all unique
    for _ in 0..100 {
        let id = SessionId::new();
        ids.insert(id.as_str().to_string());
    }

    assert_eq!(ids.len(), 100);
}

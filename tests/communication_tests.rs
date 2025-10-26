//! Tests for inter-agent communication

use periplon_sdk::dsl::{parse_workflow, validate_workflow};

#[test]
fn test_parse_workflow_with_communication() {
    let yaml = r#"
name: "Communication Test"
version: "1.0.0"

agents:
  agent1:
    description: "First agent"
    tools:
      - Read
    permissions:
      mode: "default"

  agent2:
    description: "Second agent"
    tools:
      - Read
    permissions:
      mode: "default"

communication:
  channels:
    test_channel:
      description: "Test communication channel"
      participants:
        - agent1
        - agent2
      message_format: "json"

tasks:
  task1:
    description: "First task"
    agent: "agent1"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    validate_workflow(&workflow).unwrap();

    // Check communication configuration
    assert!(workflow.communication.is_some());
    let comm = workflow.communication.as_ref().unwrap();
    assert_eq!(comm.channels.len(), 1);

    let channel = comm.channels.get("test_channel").unwrap();
    assert_eq!(channel.participants.len(), 2);
    assert_eq!(channel.message_format, "json");
}

#[test]
fn test_parse_collaborative_research() {
    let yaml = std::fs::read_to_string("examples/dsl/collaborative_research.yaml").unwrap();
    let workflow = parse_workflow(&yaml).unwrap();
    validate_workflow(&workflow).unwrap();

    // Verify communication channels
    assert!(workflow.communication.is_some());
    let comm = workflow.communication.as_ref().unwrap();
    assert_eq!(comm.channels.len(), 2);

    // Check research_findings channel
    let research_channel = comm.channels.get("research_findings").unwrap();
    assert_eq!(research_channel.participants.len(), 3);
    assert_eq!(research_channel.message_format, "markdown");

    // Check ml_insights channel
    let ml_channel = comm.channels.get("ml_insights").unwrap();
    assert_eq!(ml_channel.participants.len(), 2);
    assert_eq!(ml_channel.message_format, "json");
}

#[test]
fn test_message_types_schema() {
    let yaml = r#"
name: "Message Types Test"
version: "1.0.0"

agents:
  agent1:
    description: "Test agent"
    tools: [Read]

communication:
  message_types:
    status_update:
      schema:
        type: "object"
        properties:
          status:
            type: "string"
          progress:
            type: "number"

tasks:
  task1:
    description: "Test"
    agent: "agent1"
"#;

    let workflow = parse_workflow(yaml).unwrap();
    validate_workflow(&workflow).unwrap();

    let comm = workflow.communication.as_ref().unwrap();
    assert_eq!(comm.message_types.len(), 1);

    let msg_type = comm.message_types.get("status_update").unwrap();
    assert!(msg_type.schema.is_object());
}

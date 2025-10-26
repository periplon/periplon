use serde::{Deserialize, Serialize};

pub type PermissionMode = String;
pub type PermissionBehavior = String;
pub type PermissionUpdateDestination = String;

#[derive(Debug, Clone)]
pub enum PermissionDecision {
    Allow {
        updated_input: Option<serde_json::Value>,
    },
    Deny {
        reason: String,
    },
    Ask,
}

#[derive(Debug, Clone)]
pub struct ToolPermissionContext {
    pub signal: Option<()>,
    pub suggestions: Vec<PermissionUpdate>,
}

#[derive(Debug, Clone)]
pub enum PermissionResult {
    Allow {
        updated_input: Option<serde_json::Value>,
        updated_permissions: Option<Vec<PermissionUpdate>>,
    },
    Deny {
        message: String,
        interrupt: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PermissionUpdate {
    #[serde(rename = "addRules")]
    AddRules {
        rules: Vec<PermissionRuleValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        behavior: Option<PermissionBehavior>,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<PermissionUpdateDestination>,
    },
    #[serde(rename = "replaceRules")]
    ReplaceRules {
        rules: Vec<PermissionRuleValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        behavior: Option<PermissionBehavior>,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<PermissionUpdateDestination>,
    },
    #[serde(rename = "removeRules")]
    RemoveRules {
        rules: Vec<PermissionRuleValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<PermissionUpdateDestination>,
    },
    #[serde(rename = "setMode")]
    SetMode {
        mode: PermissionMode,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<PermissionUpdateDestination>,
    },
    #[serde(rename = "addDirectories")]
    AddDirectories {
        directories: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<PermissionUpdateDestination>,
    },
    #[serde(rename = "removeDirectories")]
    RemoveDirectories {
        directories: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<PermissionUpdateDestination>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRuleValue {
    #[serde(rename = "toolName")]
    pub tool_name: String,
    #[serde(rename = "ruleContent", skip_serializing_if = "Option::is_none")]
    pub rule_content: Option<String>,
}

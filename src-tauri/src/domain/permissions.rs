use serde::{Deserialize, Serialize};

/// Result of a privileged permission-setup action (install / reload / remove).
///
/// This never carries a password or any authentication secret: the password is
/// supplied by the user, piped to `sudo -S` over stdin for a single command,
/// and only the resulting process output is captured here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionSetupResult {
    pub success: bool,
    pub action: String,
    pub message: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub requires_reconnect: bool,
    pub next_steps: Vec<String>,
    pub warnings: Vec<String>,
}

impl PermissionSetupResult {
    pub fn failure(action: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            action: action.into(),
            message: message.into(),
            stdout: None,
            stderr: None,
            requires_reconnect: false,
            next_steps: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

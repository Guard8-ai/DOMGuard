//! User takeover mode for DOMGuard
//!
//! Allows the agent to pause automation and hand control back to the human user.
//! The user can perform actions manually, and the agent can resume when ready.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Takeover state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TakeoverState {
    /// Normal automation mode
    #[default]
    Automation,
    /// User has taken over control
    UserControl,
    /// Waiting for user to complete an action
    WaitingForUser,
    /// User requested to resume automation
    ResumeRequested,
}

/// Reason for takeover
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TakeoverReason {
    /// CAPTCHA detected
    Captcha,
    /// Sensitive action requires confirmation
    SensitiveAction,
    /// Login/authentication required
    Authentication,
    /// Error occurred that needs human intervention
    Error,
    /// Agent is unsure how to proceed
    Uncertain,
    /// User explicitly requested takeover
    UserRequested,
    /// Complex interaction that automation can't handle
    ComplexInteraction,
    /// Two-factor authentication required
    TwoFactorAuth,
    /// Payment or financial action
    Payment,
    /// Custom reason
    Custom(String),
}

/// Takeover session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeoverSession {
    /// Unique session ID
    pub id: String,

    /// Current state
    pub state: TakeoverState,

    /// Reason for takeover
    pub reason: TakeoverReason,

    /// Human-readable message for the user
    pub message: String,

    /// Instructions for what the user should do
    #[serde(default)]
    pub instructions: Option<String>,

    /// Expected outcome (what should happen after user action)
    #[serde(default)]
    pub expected_outcome: Option<String>,

    /// URL when takeover started
    #[serde(default)]
    pub url: Option<String>,

    /// Timestamp when takeover started
    pub started_at: chrono::DateTime<chrono::Utc>,

    /// Timestamp when takeover ended
    #[serde(default)]
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Duration in seconds
    #[serde(default)]
    pub duration_secs: Option<u64>,

    /// Whether the takeover was successful
    #[serde(default)]
    pub success: Option<bool>,

    /// Notes from user (optional)
    #[serde(default)]
    pub user_notes: Option<String>,
}

impl TakeoverSession {
    /// Create a new takeover session
    pub fn new(reason: TakeoverReason, message: &str) -> Self {
        Self {
            id: generate_id(),
            state: TakeoverState::WaitingForUser,
            reason,
            message: message.to_string(),
            instructions: None,
            expected_outcome: None,
            url: None,
            started_at: chrono::Utc::now(),
            ended_at: None,
            duration_secs: None,
            success: None,
            user_notes: None,
        }
    }

    /// Add instructions for the user
    pub fn with_instructions(mut self, instructions: &str) -> Self {
        self.instructions = Some(instructions.to_string());
        self
    }

    /// Add expected outcome
    pub fn with_expected_outcome(mut self, outcome: &str) -> Self {
        self.expected_outcome = Some(outcome.to_string());
        self
    }

    /// Add URL context
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    /// Mark takeover as complete
    pub fn complete(&mut self, success: bool, notes: Option<String>) {
        self.state = TakeoverState::ResumeRequested;
        self.ended_at = Some(chrono::Utc::now());
        self.duration_secs = Some((chrono::Utc::now() - self.started_at).num_seconds() as u64);
        self.success = Some(success);
        self.user_notes = notes;
    }
}

/// Takeover manager for persisting state
pub struct TakeoverManager {
    state_file: PathBuf,
}

impl TakeoverManager {
    /// Create a new manager
    pub fn new(domguard_dir: PathBuf) -> Self {
        Self {
            state_file: domguard_dir.join("_takeover_state.json"),
        }
    }

    /// Get current takeover session if any
    pub fn get_current(&self) -> Option<TakeoverSession> {
        if self.state_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&self.state_file) {
                if let Ok(session) = serde_json::from_str::<TakeoverSession>(&content) {
                    // Check if still in takeover mode
                    if session.state == TakeoverState::WaitingForUser
                        || session.state == TakeoverState::UserControl
                    {
                        return Some(session);
                    }
                }
            }
        }
        None
    }

    /// Check if currently in takeover mode
    pub fn is_active(&self) -> bool {
        self.get_current().is_some()
    }

    /// Start a new takeover session
    pub fn start(&self, session: TakeoverSession) -> Result<String> {
        let id = session.id.clone();
        let content = serde_json::to_string_pretty(&session)?;
        std::fs::write(&self.state_file, content)?;
        Ok(id)
    }

    /// Complete the takeover and resume automation
    pub fn complete(
        &self,
        success: bool,
        notes: Option<String>,
    ) -> Result<Option<TakeoverSession>> {
        if let Some(mut session) = self.get_current() {
            session.complete(success, notes);

            // Save completed session to history
            self.save_to_history(&session)?;

            // Clear active takeover
            if self.state_file.exists() {
                std::fs::remove_file(&self.state_file)?;
            }

            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    /// Cancel takeover without completing
    pub fn cancel(&self) -> Result<bool> {
        if self.state_file.exists() {
            std::fs::remove_file(&self.state_file)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Save completed session to history
    fn save_to_history(&self, session: &TakeoverSession) -> Result<()> {
        let history_dir = self
            .state_file
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid state file path"))?
            .join("takeover_history");
        std::fs::create_dir_all(&history_dir)?;

        let filename = format!("{}.json", session.id);
        let path = history_dir.join(filename);
        let content = serde_json::to_string_pretty(session)?;
        std::fs::write(path, content)?;

        Ok(())
    }

    /// Get takeover history
    pub fn get_history(&self) -> Result<Vec<TakeoverSession>> {
        let history_dir = self
            .state_file
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid state file path"))?
            .join("takeover_history");

        if !history_dir.exists() {
            return Ok(vec![]);
        }

        let mut sessions = Vec::new();
        for entry in std::fs::read_dir(history_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<TakeoverSession>(&content) {
                        sessions.push(session);
                    }
                }
            }
        }

        // Sort by start time, newest first
        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok(sessions)
    }
}

/// Generate a simple unique ID
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("takeover-{:x}", now)[..20].to_string()
}

/// Format takeover session for display
pub fn format_takeover(session: &TakeoverSession) -> String {
    let mut output = String::new();

    let state_str = match session.state {
        TakeoverState::Automation => "Automation",
        TakeoverState::UserControl => "User Control",
        TakeoverState::WaitingForUser => "Waiting for User",
        TakeoverState::ResumeRequested => "Resume Requested",
    };

    output.push_str(&format!("Takeover Session: {}\n", session.id));
    output.push_str(&format!("  State: {}\n", state_str));
    output.push_str(&format!("  Reason: {:?}\n", session.reason));
    output.push_str(&format!("  Message: {}\n", session.message));

    if let Some(instructions) = &session.instructions {
        output.push_str(&format!("  Instructions: {}\n", instructions));
    }

    if let Some(url) = &session.url {
        output.push_str(&format!("  URL: {}\n", url));
    }

    output.push_str(&format!(
        "  Started: {}\n",
        session.started_at.format("%Y-%m-%d %H:%M:%S")
    ));

    if let Some(duration) = session.duration_secs {
        output.push_str(&format!("  Duration: {}s\n", duration));
    }

    if let Some(success) = session.success {
        output.push_str(&format!(
            "  Success: {}\n",
            if success { "Yes" } else { "No" }
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_takeover_session() {
        let session = TakeoverSession::new(TakeoverReason::Captcha, "CAPTCHA detected");
        assert_eq!(session.state, TakeoverState::WaitingForUser);
        assert!(session.id.starts_with("takeover-"));
    }

    #[test]
    fn test_complete_takeover() {
        let mut session =
            TakeoverSession::new(TakeoverReason::UserRequested, "User requested control");
        session.complete(true, Some("Done".to_string()));

        assert_eq!(session.state, TakeoverState::ResumeRequested);
        assert_eq!(session.success, Some(true));
        assert!(session.ended_at.is_some());
    }

    #[test]
    fn test_with_url() {
        let session =
            TakeoverSession::new(TakeoverReason::Error, "Error").with_url("https://example.com");
        assert_eq!(session.url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_with_instructions() {
        let session = TakeoverSession::new(TakeoverReason::Captcha, "CAPTCHA detected")
            .with_instructions("Please solve the CAPTCHA");
        assert_eq!(
            session.instructions,
            Some("Please solve the CAPTCHA".to_string())
        );
    }
}

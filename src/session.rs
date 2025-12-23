//! Session recording for DOMGuard
//!
//! Records all browser actions for playback, debugging, and workflow creation.

#![allow(dead_code)] // Some methods will be used when integrated with interact commands

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single recorded action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedAction {
    /// Timestamp when action was executed
    pub timestamp: DateTime<Utc>,
    /// Duration of the action in milliseconds
    pub duration_ms: u64,
    /// The command that was executed
    pub command: String,
    /// Command arguments
    pub args: serde_json::Value,
    /// Result status
    pub status: ActionStatus,
    /// Optional screenshot path taken after action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<PathBuf>,
    /// Optional error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Page URL at time of action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_url: Option<String>,
    /// Element selector if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

/// Status of an action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Success,
    Failed,
    Skipped,
    Paused,
}

/// A recorded session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub id: String,
    /// Session name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// When the session started
    pub started_at: DateTime<Utc>,
    /// When the session ended (if finished)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    /// Initial URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_url: Option<String>,
    /// All recorded actions
    pub actions: Vec<RecordedAction>,
    /// Session status
    pub status: SessionStatus,
    /// Metadata
    #[serde(default)]
    pub metadata: SessionMetadata,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Recording,
    Paused,
    Completed,
    Failed,
}

/// Session metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Description of what this session does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// Browser info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_version: Option<String>,
    /// Viewport dimensions at start
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<(u32, u32)>,
}

impl Session {
    /// Create a new session
    pub fn new(name: Option<String>) -> Self {
        Self {
            id: uuid_v4(),
            name,
            started_at: Utc::now(),
            ended_at: None,
            initial_url: None,
            actions: Vec::new(),
            status: SessionStatus::Recording,
            metadata: SessionMetadata::default(),
        }
    }

    /// Add an action to the session
    pub fn add_action(&mut self, action: RecordedAction) {
        self.actions.push(action);
    }

    /// End the session
    pub fn end(&mut self) {
        self.ended_at = Some(Utc::now());
        self.status = SessionStatus::Completed;
    }

    /// Pause the session
    pub fn pause(&mut self) {
        self.status = SessionStatus::Paused;
    }

    /// Resume the session
    pub fn resume(&mut self) {
        self.status = SessionStatus::Recording;
    }

    /// Mark as failed
    pub fn fail(&mut self, reason: &str) {
        self.status = SessionStatus::Failed;
        self.ended_at = Some(Utc::now());
        // Add the failure reason to the last action if exists
        if let Some(last) = self.actions.last_mut() {
            last.error = Some(reason.to_string());
        }
    }

    /// Get total duration in milliseconds
    pub fn total_duration_ms(&self) -> u64 {
        self.actions.iter().map(|a| a.duration_ms).sum()
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.actions.is_empty() {
            return 1.0;
        }
        let successes = self
            .actions
            .iter()
            .filter(|a| a.status == ActionStatus::Success)
            .count();
        successes as f64 / self.actions.len() as f64
    }

    /// Save session to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self).context("Failed to serialize session")?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to save session to {}", path.display()))?;
        Ok(())
    }

    /// Load session from file
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read session from {}", path.display()))?;
        let session: Session =
            serde_json::from_str(&content).context("Failed to parse session file")?;
        Ok(session)
    }

    /// Generate a summary of the session
    pub fn summary(&self) -> SessionSummary {
        let action_counts =
            self.actions
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, action| {
                    *acc.entry(action.command.clone()).or_insert(0) += 1;
                    acc
                });

        SessionSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            status: self.status.clone(),
            total_actions: self.actions.len(),
            successful_actions: self
                .actions
                .iter()
                .filter(|a| a.status == ActionStatus::Success)
                .count(),
            failed_actions: self
                .actions
                .iter()
                .filter(|a| a.status == ActionStatus::Failed)
                .count(),
            total_duration_ms: self.total_duration_ms(),
            action_counts,
            started_at: self.started_at,
            ended_at: self.ended_at,
        }
    }
}

/// Summary of a session for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub name: Option<String>,
    pub status: SessionStatus,
    pub total_actions: usize,
    pub successful_actions: usize,
    pub failed_actions: usize,
    pub total_duration_ms: u64,
    pub action_counts: std::collections::HashMap<String, usize>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

/// Session recorder with file-based persistence
/// Survives between CLI invocations by storing active session in a file
#[derive(Clone)]
pub struct SessionRecorder {
    sessions_dir: PathBuf,
    active_session_path: PathBuf,
}

impl SessionRecorder {
    /// Create a new recorder
    pub fn new(sessions_dir: PathBuf) -> Self {
        let active_session_path = sessions_dir.join("_active_session.json");
        Self {
            sessions_dir,
            active_session_path,
        }
    }

    /// Get the currently active session (if any)
    fn get_active_session(&self) -> Option<Session> {
        if self.active_session_path.exists() {
            Session::load(&self.active_session_path).ok()
        } else {
            None
        }
    }

    /// Save the active session state
    fn save_active_session(&self, session: &Session) -> Result<()> {
        std::fs::create_dir_all(&self.sessions_dir)?;
        session.save(&self.active_session_path)
    }

    /// Clear the active session
    fn clear_active_session(&self) -> Result<()> {
        if self.active_session_path.exists() {
            std::fs::remove_file(&self.active_session_path)?;
        }
        Ok(())
    }

    /// Start a new recording session
    pub fn start(&self, name: Option<String>, initial_url: Option<String>) -> Result<String> {
        // Check if there's already an active session
        if let Some(existing) = self.get_active_session() {
            if existing.status == SessionStatus::Recording
                || existing.status == SessionStatus::Paused
            {
                anyhow::bail!(
                    "A session is already active (ID: {}). Use 'session stop' first.",
                    existing.id
                );
            }
        }

        let mut session = Session::new(name);
        session.initial_url = initial_url;
        let id = session.id.clone();

        self.save_active_session(&session)?;

        Ok(id)
    }

    /// Stop the current recording
    pub fn stop(&self) -> Result<Option<Session>> {
        if let Some(mut session) = self.get_active_session() {
            session.end();

            // Save to permanent storage
            self.save_session(&session)?;

            // Clear active session
            self.clear_active_session()?;

            return Ok(Some(session));
        }

        Ok(None)
    }

    /// Pause the current recording
    pub fn pause(&self) -> Result<()> {
        if let Some(mut session) = self.get_active_session() {
            session.pause();
            self.save_active_session(&session)?;
            Ok(())
        } else {
            anyhow::bail!("No active session to pause")
        }
    }

    /// Resume the current recording
    pub fn resume(&self) -> Result<()> {
        if let Some(mut session) = self.get_active_session() {
            session.resume();
            self.save_active_session(&session)?;
            Ok(())
        } else {
            anyhow::bail!("No active session to resume")
        }
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.get_active_session()
            .map(|s| s.status == SessionStatus::Recording)
            .unwrap_or(false)
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.get_active_session()
            .map(|s| s.status == SessionStatus::Paused)
            .unwrap_or(false)
    }

    /// Record an action
    pub fn record_action(&self, action: RecordedAction) -> Result<()> {
        if let Some(mut session) = self.get_active_session() {
            if session.status == SessionStatus::Recording {
                session.add_action(action);
                self.save_active_session(&session)?;
            }
        }
        Ok(())
    }

    /// Get current session summary
    pub fn get_summary(&self) -> Option<SessionSummary> {
        self.get_active_session().map(|s| s.summary())
    }

    /// Get current session status
    pub fn get_status(&self) -> Option<SessionStatus> {
        self.get_active_session().map(|s| s.status)
    }

    /// Save session to permanent file
    fn save_session(&self, session: &Session) -> Result<()> {
        std::fs::create_dir_all(&self.sessions_dir)?;
        let filename = format!("session_{}.json", session.id);
        let path = self.sessions_dir.join(filename);
        session.save(&path)
    }

    /// List all saved sessions (excluding active)
    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>> {
        let mut sessions = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(sessions);
        }

        for entry in std::fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            // Skip the active session file
            if path
                .file_name()
                .map(|n| n == "_active_session.json")
                .unwrap_or(false)
            {
                continue;
            }
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(session) = Session::load(&path) {
                    sessions.push(session.summary());
                }
            }
        }

        // Sort by start time, newest first
        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok(sessions)
    }

    /// Load a session by ID
    pub fn load_session(&self, id: &str) -> Result<Session> {
        let filename = format!("session_{}.json", id);
        let path = self.sessions_dir.join(filename);
        Session::load(&path)
    }

    /// Delete a session by ID
    pub fn delete_session(&self, id: &str) -> Result<()> {
        let filename = format!("session_{}.json", id);
        let path = self.sessions_dir.join(filename);
        std::fs::remove_file(&path).with_context(|| format!("Failed to delete session {}", id))
    }
}

/// Helper to build recorded actions
pub struct ActionBuilder {
    command: String,
    args: serde_json::Value,
    start_time: std::time::Instant,
    page_url: Option<String>,
    selector: Option<String>,
}

impl ActionBuilder {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            args: serde_json::json!({}),
            start_time: std::time::Instant::now(),
            page_url: None,
            selector: None,
        }
    }

    pub fn with_args(mut self, args: serde_json::Value) -> Self {
        self.args = args;
        self
    }

    pub fn with_page_url(mut self, url: Option<String>) -> Self {
        self.page_url = url;
        self
    }

    pub fn with_selector(mut self, selector: Option<String>) -> Self {
        self.selector = selector;
        self
    }

    pub fn success(self) -> RecordedAction {
        RecordedAction {
            timestamp: Utc::now(),
            duration_ms: self.start_time.elapsed().as_millis() as u64,
            command: self.command,
            args: self.args,
            status: ActionStatus::Success,
            screenshot: None,
            error: None,
            page_url: self.page_url,
            selector: self.selector,
        }
    }

    pub fn failed(self, error: &str) -> RecordedAction {
        RecordedAction {
            timestamp: Utc::now(),
            duration_ms: self.start_time.elapsed().as_millis() as u64,
            command: self.command,
            args: self.args,
            status: ActionStatus::Failed,
            screenshot: None,
            error: Some(error.to_string()),
            page_url: self.page_url,
            selector: self.selector,
        }
    }
}

/// Generate a simple UUID v4
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let timestamp = duration.as_nanos();
    let random: u64 = (timestamp as u64) ^ (std::process::id() as u64 * 0x5DEECE66D);
    format!(
        "{:016x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        timestamp & 0xFFFFFFFFFFFFFFFF,
        (random >> 48) & 0xFFFF,
        (random >> 36) & 0xFFF,
        ((random >> 32) & 0x3FFF) | 0x8000,
        random & 0xFFFFFFFFFFFF
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(Some("Test Session".to_string()));
        assert!(!session.id.is_empty());
        assert_eq!(session.name, Some("Test Session".to_string()));
        assert_eq!(session.status, SessionStatus::Recording);
        assert!(session.actions.is_empty());
    }

    #[test]
    fn test_action_recording() {
        let mut session = Session::new(None);

        let action = ActionBuilder::new("click")
            .with_args(serde_json::json!({"selector": ".button"}))
            .with_selector(Some(".button".to_string()))
            .success();

        session.add_action(action);
        assert_eq!(session.actions.len(), 1);
        assert_eq!(session.actions[0].command, "click");
    }

    #[test]
    fn test_session_summary() {
        let mut session = Session::new(Some("Test".to_string()));

        session.add_action(ActionBuilder::new("click").success());
        session.add_action(ActionBuilder::new("click").success());
        session.add_action(ActionBuilder::new("type").success());
        session.add_action(ActionBuilder::new("type").failed("Element not found"));

        let summary = session.summary();
        assert_eq!(summary.total_actions, 4);
        assert_eq!(summary.successful_actions, 3);
        assert_eq!(summary.failed_actions, 1);
        assert_eq!(summary.action_counts.get("click"), Some(&2));
        assert_eq!(summary.action_counts.get("type"), Some(&2));
    }

    #[test]
    fn test_success_rate() {
        let mut session = Session::new(None);

        session.add_action(ActionBuilder::new("click").success());
        session.add_action(ActionBuilder::new("click").success());
        session.add_action(ActionBuilder::new("click").failed("error"));
        session.add_action(ActionBuilder::new("click").success());

        assert!((session.success_rate() - 0.75).abs() < 0.01);
    }
}

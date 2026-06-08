//! Storage backend module for session persistence.
//!
//! Provides traits and implementations for storing agent session state.

mod file;
mod memory;

pub use file::FileStorage;
pub use memory::MemoryStorage;

use crate::error::NovaResult;
use crate::types::Message;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Session state containing all data for an analysis session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Unique session identifier.
    pub id: String,

    /// Session title or summary.
    pub title: String,

    /// When the session was created.
    pub created_at: DateTime<Utc>,

    /// When the session was last updated.
    pub updated_at: DateTime<Utc>,

    /// Current session status.
    pub status: SessionStatus,

    /// Full LLM message history for context preservation during resume.
    #[serde(default)]
    pub messages: Vec<Message>,

    /// Pending clarification request (when status is Paused).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_clarification: Option<PendingClarification>,

    /// Notes collected during analysis.
    pub notes: Vec<Note>,

    /// Findings and conclusions.
    pub findings: Vec<Finding>,

    /// Arbitrary metadata.
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Information about a pending clarification request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingClarification {
    /// Platform where the question was posted.
    pub platform: String,
    /// Issue/ticket ID where the comment was posted.
    pub issue_id: String,
    /// Comment ID of the posted question.
    pub comment_id: String,
    /// The question that was asked.
    pub question: String,
    /// Options provided (if any).
    pub options: Vec<ClarificationOption>,
    /// Whether multiple options can be selected.
    pub multi_select: bool,
    /// When the clarification was requested.
    pub requested_at: DateTime<Utc>,
}

/// An option in a clarification question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationOption {
    /// Option label.
    pub label: String,
    /// Whether this is the recommended option.
    pub recommended: bool,
}

impl SessionState {
    /// Create a new session with the given ID.
    pub fn new(id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            title: String::new(),
            created_at: now,
            updated_at: now,
            status: SessionStatus::Active,
            messages: Vec::new(),
            pending_clarification: None,
            notes: Vec::new(),
            findings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new session with ID and title.
    pub fn with_title(id: impl Into<String>, title: impl Into<String>) -> Self {
        let mut session = Self::new(id);
        session.title = title.into();
        session
    }

    /// Add a note to the session.
    pub fn add_note(&mut self, content: impl Into<String>, category: Option<String>) {
        self.notes.push(Note {
            content: content.into(),
            category,
            created_at: Utc::now(),
        });
        self.updated_at = Utc::now();
    }

    /// Add a finding to the session.
    pub fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
        self.updated_at = Utc::now();
    }

    /// Set metadata value.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
        self.updated_at = Utc::now();
    }

    /// Mark session as completed.
    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
        self.pending_clarification = None;
        self.updated_at = Utc::now();
    }

    /// Mark session as paused.
    pub fn pause(&mut self) {
        self.status = SessionStatus::Paused;
        self.updated_at = Utc::now();
    }

    /// Pause session with a pending clarification request.
    pub fn pause_for_clarification(&mut self, clarification: PendingClarification) {
        self.status = SessionStatus::Paused;
        self.pending_clarification = Some(clarification);
        self.updated_at = Utc::now();
    }

    /// Resume session from paused state.
    pub fn resume(&mut self) {
        self.status = SessionStatus::Active;
        self.pending_clarification = None;
        self.updated_at = Utc::now();
    }

    /// Set the message history.
    pub fn set_messages(&mut self, messages: Vec<Message>) {
        self.messages = messages;
        self.updated_at = Utc::now();
    }

    /// Add a message to the history.
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }
}

/// Session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    /// Session is actively being worked on.
    Active,
    /// Session is paused (waiting for user input).
    Paused,
    /// Session has been completed.
    Completed,
    /// Session was cancelled.
    Cancelled,
}

/// A note taken during analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Note content.
    pub content: String,
    /// Optional category for organizing notes.
    pub category: Option<String>,
    /// When the note was created.
    pub created_at: DateTime<Utc>,
}

/// A finding or conclusion from the analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Finding title.
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// Severity or importance level.
    pub severity: FindingSeverity,
    /// Source references.
    pub sources: Vec<String>,
    /// When the finding was recorded.
    pub created_at: DateTime<Utc>,
}

impl Finding {
    /// Create a new finding.
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        severity: FindingSeverity,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            severity,
            sources: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Add a source reference.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.sources.push(source.into());
        self
    }
}

/// Severity level for findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    /// Informational finding.
    Info,
    /// Low importance.
    Low,
    /// Medium importance.
    Medium,
    /// High importance.
    High,
    /// Critical importance.
    Critical,
}

/// Brief session information for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID.
    pub id: String,
    /// Session title.
    pub title: String,
    /// Session status.
    pub status: SessionStatus,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// When the session was last updated.
    pub updated_at: DateTime<Utc>,
    /// Number of notes.
    pub note_count: usize,
    /// Number of findings.
    pub finding_count: usize,
}

impl From<&SessionState> for SessionInfo {
    fn from(state: &SessionState) -> Self {
        Self {
            id: state.id.clone(),
            title: state.title.clone(),
            status: state.status,
            created_at: state.created_at,
            updated_at: state.updated_at,
            note_count: state.notes.len(),
            finding_count: state.findings.len(),
        }
    }
}

/// Storage trait for persisting session state.
#[async_trait]
pub trait Storage: Send + Sync {
    /// Save a session state.
    async fn save_session(&self, state: &SessionState) -> NovaResult<()>;

    /// Load a session by ID.
    async fn load_session(&self, id: &str) -> NovaResult<Option<SessionState>>;

    /// List all sessions.
    async fn list_sessions(&self) -> NovaResult<Vec<SessionInfo>>;

    /// Delete a session by ID.
    async fn delete_session(&self, id: &str) -> NovaResult<()>;

    /// Check if a session exists.
    async fn session_exists(&self, id: &str) -> NovaResult<bool> {
        Ok(self.load_session(id).await?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_new() {
        let session = SessionState::new("test-session");
        assert_eq!(session.id, "test-session");
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.notes.is_empty());
        assert!(session.findings.is_empty());
    }

    #[test]
    fn test_session_add_note() {
        let mut session = SessionState::new("test");
        session.add_note("Test note", Some("category".to_string()));

        assert_eq!(session.notes.len(), 1);
        assert_eq!(session.notes[0].content, "Test note");
        assert_eq!(session.notes[0].category, Some("category".to_string()));
    }

    #[test]
    fn test_finding_creation() {
        let finding = Finding::new("Test Finding", "Description", FindingSeverity::High)
            .with_source("source1");

        assert_eq!(finding.title, "Test Finding");
        assert_eq!(finding.severity, FindingSeverity::High);
        assert_eq!(finding.sources.len(), 1);
    }

    #[test]
    fn test_session_info_from_state() {
        let mut session = SessionState::with_title("test", "Test Session");
        session.add_note("Note 1", None);
        session.add_finding(Finding::new("Finding", "Desc", FindingSeverity::Low));

        let info = SessionInfo::from(&session);
        assert_eq!(info.id, "test");
        assert_eq!(info.title, "Test Session");
        assert_eq!(info.note_count, 1);
        assert_eq!(info.finding_count, 1);
    }

    #[test]
    fn test_session_messages() {
        let mut session = SessionState::new("test");
        assert!(session.messages.is_empty());

        session.add_message(Message::user("Hello"));
        session.add_message(Message::assistant("Hi there!"));

        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].content, "Hello");
        assert_eq!(session.messages[1].content, "Hi there!");
    }

    #[test]
    fn test_session_pause_for_clarification() {
        let mut session = SessionState::new("test");
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.pending_clarification.is_none());

        let clarification = PendingClarification {
            platform: "github".to_string(),
            issue_id: "123".to_string(),
            comment_id: "456".to_string(),
            question: "Which option?".to_string(),
            options: vec![
                ClarificationOption {
                    label: "Option A".to_string(),
                    recommended: true,
                },
                ClarificationOption {
                    label: "Option B".to_string(),
                    recommended: false,
                },
            ],
            multi_select: false,
            requested_at: Utc::now(),
        };

        session.pause_for_clarification(clarification);
        assert_eq!(session.status, SessionStatus::Paused);
        assert!(session.pending_clarification.is_some());

        let pending = session.pending_clarification.as_ref().unwrap();
        assert_eq!(pending.platform, "github");
        assert_eq!(pending.options.len(), 2);
        assert!(pending.options[0].recommended);
    }

    #[test]
    fn test_session_resume() {
        let mut session = SessionState::new("test");
        session.pause();
        assert_eq!(session.status, SessionStatus::Paused);

        session.resume();
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.pending_clarification.is_none());
    }
}

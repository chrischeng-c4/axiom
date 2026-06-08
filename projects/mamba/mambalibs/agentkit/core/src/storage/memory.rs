//! In-memory storage backend using HashMap.

use super::{SessionInfo, SessionState, Storage};
use crate::error::NovaResult;
use async_trait::async_trait;
use dashmap::DashMap;

/// In-memory storage backend using a concurrent HashMap.
///
/// This storage is thread-safe and suitable for testing or short-lived sessions.
/// All data is lost when the process exits.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    sessions: DashMap<String, SessionState>,
}

impl MemoryStorage {
    /// Create a new empty memory storage.
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    /// Get the number of stored sessions.
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Check if storage is empty.
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Clear all stored sessions.
    pub fn clear(&self) {
        self.sessions.clear();
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn save_session(&self, state: &SessionState) -> NovaResult<()> {
        self.sessions.insert(state.id.clone(), state.clone());
        Ok(())
    }

    async fn load_session(&self, id: &str) -> NovaResult<Option<SessionState>> {
        Ok(self.sessions.get(id).map(|entry| entry.value().clone()))
    }

    async fn list_sessions(&self) -> NovaResult<Vec<SessionInfo>> {
        let mut sessions: Vec<_> = self
            .sessions
            .iter()
            .map(|entry| SessionInfo::from(entry.value()))
            .collect();

        // Sort by updated_at descending (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    async fn delete_session(&self, id: &str) -> NovaResult<()> {
        self.sessions.remove(id);
        Ok(())
    }

    async fn session_exists(&self, id: &str) -> NovaResult<bool> {
        Ok(self.sessions.contains_key(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{Finding, FindingSeverity};

    #[tokio::test]
    async fn test_memory_storage_save_load() {
        let storage = MemoryStorage::new();
        let session = SessionState::with_title("test-1", "Test Session");

        storage.save_session(&session).await.unwrap();

        let loaded = storage.load_session("test-1").await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, "test-1");
        assert_eq!(loaded.title, "Test Session");
    }

    #[tokio::test]
    async fn test_memory_storage_load_nonexistent() {
        let storage = MemoryStorage::new();
        let loaded = storage.load_session("nonexistent").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_memory_storage_list() {
        let storage = MemoryStorage::new();

        let session1 = SessionState::with_title("session-1", "First");
        let session2 = SessionState::with_title("session-2", "Second");

        storage.save_session(&session1).await.unwrap();
        storage.save_session(&session2).await.unwrap();

        let sessions = storage.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_memory_storage_delete() {
        let storage = MemoryStorage::new();
        let session = SessionState::new("to-delete");

        storage.save_session(&session).await.unwrap();
        assert!(storage.session_exists("to-delete").await.unwrap());

        storage.delete_session("to-delete").await.unwrap();
        assert!(!storage.session_exists("to-delete").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_storage_update() {
        let storage = MemoryStorage::new();
        let mut session = SessionState::with_title("test", "Original Title");

        storage.save_session(&session).await.unwrap();

        // Modify and save again
        session.title = "Updated Title".to_string();
        session.add_note("A note", None);
        session.add_finding(Finding::new("Finding", "Desc", FindingSeverity::Medium));

        storage.save_session(&session).await.unwrap();

        let loaded = storage.load_session("test").await.unwrap().unwrap();
        assert_eq!(loaded.title, "Updated Title");
        assert_eq!(loaded.notes.len(), 1);
        assert_eq!(loaded.findings.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_storage_clear() {
        let storage = MemoryStorage::new();

        storage.save_session(&SessionState::new("1")).await.unwrap();
        storage.save_session(&SessionState::new("2")).await.unwrap();

        assert_eq!(storage.len(), 2);

        storage.clear();

        assert!(storage.is_empty());
    }
}

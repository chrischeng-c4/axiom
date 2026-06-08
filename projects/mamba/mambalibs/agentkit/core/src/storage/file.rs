//! File-based storage backend using JSON files.

use super::{SessionInfo, SessionState, Storage};
use crate::error::{NovaError, NovaResult};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::debug;

/// File-based storage backend storing sessions as JSON files.
///
/// Each session is stored as a separate JSON file in the specified directory.
/// The file name is `{session_id}.json`.
#[derive(Debug, Clone)]
pub struct FileStorage {
    directory: PathBuf,
}

impl FileStorage {
    /// Create a new file storage at the specified directory.
    ///
    /// The directory will be created if it doesn't exist when saving.
    pub fn new(directory: impl AsRef<Path>) -> Self {
        Self {
            directory: directory.as_ref().to_path_buf(),
        }
    }

    /// Get the storage directory path.
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Get the file path for a session ID.
    fn session_path(&self, id: &str) -> PathBuf {
        self.directory.join(format!("{}.json", id))
    }

    /// Ensure the storage directory exists.
    async fn ensure_directory(&self) -> NovaResult<()> {
        if !self.directory.exists() {
            fs::create_dir_all(&self.directory).await?;
            debug!("Created storage directory: {:?}", self.directory);
        }
        Ok(())
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn save_session(&self, state: &SessionState) -> NovaResult<()> {
        self.ensure_directory().await?;

        let path = self.session_path(&state.id);
        let json = serde_json::to_string_pretty(state)?;

        fs::write(&path, json).await?;
        debug!("Saved session {} to {:?}", state.id, path);

        Ok(())
    }

    async fn load_session(&self, id: &str) -> NovaResult<Option<SessionState>> {
        let path = self.session_path(id);

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;
        let state: SessionState = serde_json::from_str(&content).map_err(|e| {
            NovaError::ConfigError(format!("Failed to parse session file {}: {}", id, e))
        })?;

        debug!("Loaded session {} from {:?}", id, path);
        Ok(Some(state))
    }

    async fn list_sessions(&self) -> NovaResult<Vec<SessionInfo>> {
        if !self.directory.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        let mut dir = fs::read_dir(&self.directory).await?;

        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            match fs::read_to_string(&path).await {
                Ok(content) => match serde_json::from_str::<SessionState>(&content) {
                    Ok(state) => {
                        sessions.push(SessionInfo::from(&state));
                    }
                    Err(e) => {
                        debug!("Skipping invalid session file {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    debug!("Failed to read session file {:?}: {}", path, e);
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    async fn delete_session(&self, id: &str) -> NovaResult<()> {
        let path = self.session_path(id);

        if path.exists() {
            fs::remove_file(&path).await?;
            debug!("Deleted session {} from {:?}", id, path);
        }

        Ok(())
    }

    async fn session_exists(&self, id: &str) -> NovaResult<bool> {
        Ok(self.session_path(id).exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{Finding, FindingSeverity, SessionStatus};
    use tempfile::TempDir;

    async fn create_temp_storage() -> (FileStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_file_storage_save_load() {
        let (storage, _temp) = create_temp_storage().await;
        let session = SessionState::with_title("test-1", "Test Session");

        storage.save_session(&session).await.unwrap();

        let loaded = storage.load_session("test-1").await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, "test-1");
        assert_eq!(loaded.title, "Test Session");
    }

    #[tokio::test]
    async fn test_file_storage_load_nonexistent() {
        let (storage, _temp) = create_temp_storage().await;
        let loaded = storage.load_session("nonexistent").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_file_storage_list() {
        let (storage, _temp) = create_temp_storage().await;

        let session1 = SessionState::with_title("session-1", "First");
        let session2 = SessionState::with_title("session-2", "Second");

        storage.save_session(&session1).await.unwrap();
        storage.save_session(&session2).await.unwrap();

        let sessions = storage.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_file_storage_delete() {
        let (storage, _temp) = create_temp_storage().await;
        let session = SessionState::new("to-delete");

        storage.save_session(&session).await.unwrap();
        assert!(storage.session_exists("to-delete").await.unwrap());

        storage.delete_session("to-delete").await.unwrap();
        assert!(!storage.session_exists("to-delete").await.unwrap());
    }

    #[tokio::test]
    async fn test_file_storage_update() {
        let (storage, _temp) = create_temp_storage().await;
        let mut session = SessionState::with_title("test", "Original");

        storage.save_session(&session).await.unwrap();

        // Modify and save again
        session.title = "Updated".to_string();
        session.add_note("A note", None);
        session.add_finding(Finding::new("Finding", "Desc", FindingSeverity::High));

        storage.save_session(&session).await.unwrap();

        let loaded = storage.load_session("test").await.unwrap().unwrap();
        assert_eq!(loaded.title, "Updated");
        assert_eq!(loaded.notes.len(), 1);
        assert_eq!(loaded.findings.len(), 1);
    }

    #[tokio::test]
    async fn test_file_storage_persists_status() {
        let (storage, _temp) = create_temp_storage().await;
        let mut session = SessionState::new("status-test");
        session.complete();

        storage.save_session(&session).await.unwrap();

        let loaded = storage.load_session("status-test").await.unwrap().unwrap();
        assert_eq!(loaded.status, SessionStatus::Completed);
    }

    #[tokio::test]
    async fn test_file_storage_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("storage");
        let storage = FileStorage::new(&nested_path);

        let session = SessionState::new("test");
        storage.save_session(&session).await.unwrap();

        assert!(nested_path.exists());
        assert!(storage.session_exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_file_storage_list_empty_directory() {
        let (storage, _temp) = create_temp_storage().await;
        let sessions = storage.list_sessions().await.unwrap();
        assert!(sessions.is_empty());
    }
}

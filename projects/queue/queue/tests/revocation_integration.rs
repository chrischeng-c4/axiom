// Test to verify RevocationStore integration with Worker

use cclab_queue::{
    WorkerConfig, RevocationStore, TaskId, TaskError,
};
use std::sync::Arc;
use async_trait::async_trait;
use std::collections::HashSet;
use tokio::sync::RwLock;

// Simple in-memory revocation store for testing
#[derive(Clone)]
struct TestRevocationStore {
    revoked: Arc<RwLock<HashSet<TaskId>>>,
}

impl TestRevocationStore {
    fn new() -> Self {
        Self {
            revoked: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    async fn revoke(&self, task_id: TaskId) {
        self.revoked.write().await.insert(task_id);
    }
}

#[async_trait]
impl RevocationStore for TestRevocationStore {
    async fn is_revoked(&self, task_id: &TaskId) -> Result<bool, TaskError> {
        Ok(self.revoked.read().await.contains(task_id))
    }

    async fn revoke(&self, task_id: &TaskId, _terminate: bool) -> Result<(), TaskError> {
        self.revoked.write().await.insert(task_id.clone());
        Ok(())
    }

    async fn revoke_many(&self, task_ids: &[TaskId], _terminate: bool) -> Result<(), TaskError> {
        let mut revoked = self.revoked.write().await;
        for task_id in task_ids {
            revoked.insert(task_id.clone());
        }
        Ok(())
    }

    async fn get_revoked(&self) -> Result<Vec<TaskId>, TaskError> {
        Ok(self.revoked.read().await.iter().cloned().collect())
    }

    async fn cleanup(&self) -> Result<usize, TaskError> {
        let count = self.revoked.read().await.len();
        self.revoked.write().await.clear();
        Ok(count)
    }
}

#[tokio::test]
async fn test_worker_config_with_revocation_store() {
    // This test verifies that:
    // 1. WorkerConfig can be created with a revocation store
    // 2. The field can be set correctly

    let store = TestRevocationStore::new();

    // Test that we can set the revocation store via WorkerConfig
    let mut config = WorkerConfig::default();
    config.revocation_store = Some(Arc::new(store.clone()));

    // Verify the field is set
    assert!(config.revocation_store.is_some());
}

#[tokio::test]
async fn test_revocation_store_functionality() {
    let store = TestRevocationStore::new();
    let task_id = TaskId(uuid::Uuid::now_v7());

    // Initially not revoked
    assert!(!store.is_revoked(&task_id).await.unwrap());

    // Revoke the task
    store.revoke(task_id.clone()).await;

    // Now it should be revoked
    assert!(store.is_revoked(&task_id).await.unwrap());
}

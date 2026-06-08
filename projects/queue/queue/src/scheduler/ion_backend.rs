//! Ion-based scheduler backend
//!
//! Uses cclab-ion for distributed leader election and task state persistence.

use std::time::Duration;

use async_trait::async_trait;
use cclab_kv::client::{KvClient, KvValue};
use tokio::sync::Mutex;

use super::backend::{SchedulerBackend, TaskScheduleState};
use crate::TaskError;

/// Leader lock key prefix
const LEADER_LOCK_KEY: &str = "meteor:scheduler:leader";

/// Task state key prefix
const TASK_STATE_PREFIX: &str = "meteor:schedule:state:";

/// Ion-based scheduler backend
///
/// Provides distributed leader election and task state persistence
/// using cclab-ion as the backend.
pub struct IonSchedulerBackend {
    client: Mutex<KvClient>,
    /// Unique identifier for this scheduler instance
    instance_id: String,
}

impl IonSchedulerBackend {
    /// Create a new Ion scheduler backend
    pub fn new(client: KvClient, instance_id: String) -> Self {
        Self {
            client: Mutex::new(client),
            instance_id,
        }
    }

    /// Create from connection string
    pub async fn connect(addr: &str, instance_id: String) -> Result<Self, TaskError> {
        let client = KvClient::connect(addr)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to connect to Ion: {}", e)))?;
        Ok(Self::new(client, instance_id))
    }

    fn task_state_key(&self, name: &str) -> String {
        format!("{}{}", TASK_STATE_PREFIX, name)
    }
}

#[async_trait]
impl SchedulerBackend for IonSchedulerBackend {
    async fn acquire_leader(&self, ttl: Duration) -> Result<bool, TaskError> {
        let mut client = self.client.lock().await;
        client
            .lock(LEADER_LOCK_KEY, &self.instance_id, ttl)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to acquire leader lock: {}", e)))
    }

    async fn renew_leader(&self, ttl: Duration) -> Result<bool, TaskError> {
        let mut client = self.client.lock().await;
        client
            .extend_lock(LEADER_LOCK_KEY, &self.instance_id, ttl)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to renew leader lock: {}", e)))
    }

    async fn release_leader(&self) -> Result<(), TaskError> {
        let mut client = self.client.lock().await;
        client
            .unlock(LEADER_LOCK_KEY, &self.instance_id)
            .await
            .map(|_| ())
            .map_err(|e| TaskError::Backend(format!("Failed to release leader lock: {}", e)))
    }

    async fn get_task_state(&self, name: &str) -> Result<TaskScheduleState, TaskError> {
        let key = self.task_state_key(name);
        let mut client = self.client.lock().await;

        match client.get(&key).await {
            Ok(Some(value)) => {
                let bytes = match &value {
                    KvValue::Bytes(b) => b.as_slice(),
                    KvValue::String(s) => s.as_bytes(),
                    _ => {
                        return Err(TaskError::Backend(
                            "Invalid task state value type".to_string(),
                        ))
                    }
                };
                serde_json::from_slice(bytes)
                    .map_err(|e| TaskError::Backend(format!("Failed to deserialize task state: {}", e)))
            }
            Ok(None) => Ok(TaskScheduleState::default()),
            Err(e) => Err(TaskError::Backend(format!("Failed to get task state: {}", e))),
        }
    }

    async fn set_task_state(&self, name: &str, state: &TaskScheduleState) -> Result<(), TaskError> {
        let key = self.task_state_key(name);
        let value = serde_json::to_vec(state)
            .map_err(|e| TaskError::Backend(format!("Failed to serialize task state: {}", e)))?;

        let mut client = self.client.lock().await;
        client
            .set(&key, KvValue::Bytes(value), None)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to set task state: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_state_key_format() {
        assert_eq!(
            format!("{}{}", TASK_STATE_PREFIX, "my-task"),
            "meteor:schedule:state:my-task"
        );
    }
}

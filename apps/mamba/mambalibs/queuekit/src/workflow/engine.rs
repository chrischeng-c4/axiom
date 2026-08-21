//! Workflow Engine for chain/chord continuation
//!
//! Handles workflow state management and continuation logic.
//! Can be called by both workers and external executors (K8s Jobs).

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::message::TaskMessage;
use crate::{Broker, ResultBackend, TaskError, TaskId};

/// Chain metadata stored in the backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMeta {
    /// Root task ID (chain ID)
    pub root_id: TaskId,
    /// All task signatures in the chain (serialized)
    pub tasks: Vec<serde_json::Value>,
    /// Current index (next task to execute)
    pub current_index: usize,
    /// Accumulated results from completed tasks
    pub results: Vec<serde_json::Value>,
    /// Queue for publishing next tasks
    pub queue: String,
}

/// Chord metadata stored in the backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordMeta {
    /// Root task ID (chord ID)
    pub root_id: TaskId,
    /// Header task IDs
    pub header_task_ids: Vec<TaskId>,
    /// Collected results from header tasks
    pub results: Vec<Option<serde_json::Value>>,
    /// Callback task signature (serialized)
    pub callback: serde_json::Value,
    /// Queue for publishing callback
    pub queue: String,
    /// Number of completed header tasks
    pub completed_count: usize,
}

/// Workflow engine for managing chains and chords
pub struct WorkflowEngine<R: ResultBackend, B: Broker> {
    backend: Arc<R>,
    broker: Arc<B>,
}

impl<R: ResultBackend, B: Broker> WorkflowEngine<R, B> {
    /// Create a new workflow engine
    pub fn new(backend: Arc<R>, broker: Arc<B>) -> Self {
        Self { backend, broker }
    }

    /// Handle task completion and trigger continuation if needed
    ///
    /// For chains: looks up metadata by root_id (passed via task message)
    /// For chords: uses dedicated on_chord_task_complete method
    pub async fn on_task_complete(
        &self,
        task_id: &TaskId,
        root_id: Option<&TaskId>,
        result: serde_json::Value,
    ) -> Result<(), TaskError> {
        // Use root_id if available, otherwise use task_id (for the first task in chain)
        let lookup_id = root_id.unwrap_or(task_id);

        // Try to find chain metadata by root_id
        let chain_key = format!("chain:{}", lookup_id);
        if let Some(meta) = self.backend.get_metadata(&chain_key).await? {
            let chain_meta: ChainMeta = serde_json::from_value(meta)
                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
            return self.continue_chain(chain_meta, result).await;
        }

        // Check if this task is part of a chord header
        if let Some(root) = root_id {
            let chord_key = format!("chord:{}", root);
            if self.backend.get_metadata(&chord_key).await?.is_some() {
                // Delegate to chord completion handler
                return self.on_chord_task_complete(root, task_id, result).await;
            }
        }

        Ok(())
    }

    /// Continue a chain with the result from the previous task
    async fn continue_chain(
        &self,
        mut meta: ChainMeta,
        result: serde_json::Value,
    ) -> Result<(), TaskError> {
        // Add result to accumulated results
        meta.results.push(result.clone());
        meta.current_index += 1;

        // Always use root_id as the canonical key for chain metadata
        let chain_key = format!("chain:{}", meta.root_id);

        // Check if there's a next task
        if meta.current_index < meta.tasks.len() {
            // Get next task signature
            let next_sig = &meta.tasks[meta.current_index];

            // Build the next task message
            let mut next_msg: TaskMessage = serde_json::from_value(next_sig.clone())
                .map_err(|e| TaskError::Deserialization(e.to_string()))?;

            // Prepend the previous result to args
            if let serde_json::Value::Array(ref mut args) = next_msg.args {
                args.insert(0, result);
            } else {
                next_msg.args = serde_json::json!([result]);
            }

            // Set workflow context
            next_msg.root_id = Some(meta.root_id.clone());
            next_msg.parent_id = None; // Could track parent if needed

            // Save updated chain metadata (always use root_id as key)
            let meta_value =
                serde_json::to_value(&meta).map_err(|e| TaskError::Serialization(e.to_string()))?;
            self.backend
                .set_metadata(&chain_key, meta_value, None)
                .await?;

            // Publish next task
            self.broker.publish(&meta.queue, next_msg).await?;

            tracing::info!(
                root_id = %meta.root_id,
                index = meta.current_index,
                total = meta.tasks.len(),
                "Chain continued to next task"
            );
        } else {
            // Chain complete - clean up metadata
            self.backend.delete_metadata(&chain_key).await?;

            tracing::info!(
                root_id = %meta.root_id,
                total_results = meta.results.len(),
                "Chain completed"
            );
        }

        Ok(())
    }

    /// Record a chord header task result
    pub async fn on_chord_task_complete(
        &self,
        root_id: &TaskId,
        task_id: &TaskId,
        result: serde_json::Value,
    ) -> Result<(), TaskError> {
        let chord_key = format!("chord:{}", root_id);

        let meta_value = self
            .backend
            .get_metadata(&chord_key)
            .await?
            .ok_or_else(|| TaskError::Internal(format!("Chord metadata not found: {}", root_id)))?;

        let mut meta: ChordMeta = serde_json::from_value(meta_value)
            .map_err(|e| TaskError::Deserialization(e.to_string()))?;

        // Find and update the result for this task
        for (i, header_id) in meta.header_task_ids.iter().enumerate() {
            if header_id == task_id && meta.results[i].is_none() {
                meta.results[i] = Some(result.clone());
                meta.completed_count += 1;
                break;
            }
        }

        // Check if all header tasks are complete
        if meta.completed_count == meta.header_task_ids.len() {
            // Dispatch callback with all results
            let all_results: Vec<serde_json::Value> = meta
                .results
                .into_iter()
                .map(|r| r.unwrap_or(serde_json::Value::Null))
                .collect();

            // Build callback message
            let mut callback_msg: TaskMessage = serde_json::from_value(meta.callback.clone())
                .map_err(|e| TaskError::Deserialization(e.to_string()))?;

            // Set args to collected results
            callback_msg.args = serde_json::json!(all_results);
            callback_msg.root_id = Some(root_id.clone());

            // Publish callback
            self.broker.publish(&meta.queue, callback_msg).await?;

            // Clean up chord metadata
            self.backend.delete_metadata(&chord_key).await?;

            tracing::info!(
                root_id = %root_id,
                header_count = meta.header_task_ids.len(),
                "Chord completed, callback dispatched"
            );
        } else {
            // Save updated metadata
            let meta_value =
                serde_json::to_value(&meta).map_err(|e| TaskError::Serialization(e.to_string()))?;
            self.backend
                .set_metadata(&chord_key, meta_value, None)
                .await?;

            tracing::debug!(
                root_id = %root_id,
                completed = meta.completed_count,
                total = meta.header_task_ids.len(),
                "Chord header task completed"
            );
        }

        Ok(())
    }

    /// Initialize chain metadata before starting
    pub async fn init_chain(
        &self,
        root_id: TaskId,
        tasks: Vec<serde_json::Value>,
        queue: String,
    ) -> Result<(), TaskError> {
        let meta = ChainMeta {
            root_id: root_id.clone(),
            tasks,
            current_index: 0,
            results: Vec::new(),
            queue,
        };

        let chain_key = format!("chain:{}", root_id);
        let meta_value =
            serde_json::to_value(&meta).map_err(|e| TaskError::Serialization(e.to_string()))?;
        self.backend
            .set_metadata(&chain_key, meta_value, None)
            .await?;

        Ok(())
    }

    /// Initialize chord metadata before starting
    pub async fn init_chord(
        &self,
        root_id: TaskId,
        header_task_ids: Vec<TaskId>,
        callback: serde_json::Value,
        queue: String,
    ) -> Result<(), TaskError> {
        let results = vec![None; header_task_ids.len()];

        let meta = ChordMeta {
            root_id: root_id.clone(),
            header_task_ids,
            results,
            callback,
            queue,
            completed_count: 0,
        };

        let chord_key = format!("chord:{}", root_id);
        let meta_value =
            serde_json::to_value(&meta).map_err(|e| TaskError::Serialization(e.to_string()))?;
        self.backend
            .set_metadata(&chord_key, meta_value, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_meta_serialization() {
        let meta = ChainMeta {
            root_id: TaskId::new(),
            tasks: vec![serde_json::json!({"task_name": "task1"})],
            current_index: 0,
            results: vec![],
            queue: "default".to_string(),
        };

        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: ChainMeta = serde_json::from_str(&json).unwrap();

        assert_eq!(meta.root_id, deserialized.root_id);
        assert_eq!(meta.current_index, deserialized.current_index);
    }

    #[test]
    fn test_chord_meta_serialization() {
        let meta = ChordMeta {
            root_id: TaskId::new(),
            header_task_ids: vec![TaskId::new(), TaskId::new()],
            results: vec![None, None],
            callback: serde_json::json!({"task_name": "callback"}),
            queue: "default".to_string(),
            completed_count: 0,
        };

        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: ChordMeta = serde_json::from_str(&json).unwrap();

        assert_eq!(meta.root_id, deserialized.root_id);
        assert_eq!(
            meta.header_task_ids.len(),
            deserialized.header_task_ids.len()
        );
    }

    // -----------------------------------------------------------------------
    // Mock broker + real Redis backend for WorkflowEngine integration tests
    // -----------------------------------------------------------------------

    #[cfg(feature = "redis")]
    mod integration {
        use super::*;
        use crate::broker::{BrokerCapabilities, DeliveryModel};
        use crate::{RedisBackend, RedisBackendConfig};
        use async_trait::async_trait;
        use std::sync::Mutex;

        /// Mock broker that records all published messages for assertion.
        /// Implements Broker only (not PullBroker).
        struct MockBroker {
            published: Arc<Mutex<Vec<(String, TaskMessage)>>>,
        }

        impl MockBroker {
            fn new() -> Self {
                Self {
                    published: Arc::new(Mutex::new(Vec::new())),
                }
            }

            fn published_messages(&self) -> Vec<(String, TaskMessage)> {
                self.published.lock().unwrap().clone()
            }
        }

        #[async_trait]
        impl Broker for MockBroker {
            async fn connect(&self) -> Result<(), TaskError> {
                Ok(())
            }
            async fn disconnect(&self) -> Result<(), TaskError> {
                Ok(())
            }
            async fn publish(&self, queue: &str, message: TaskMessage) -> Result<(), TaskError> {
                self.published
                    .lock()
                    .unwrap()
                    .push((queue.to_string(), message));
                Ok(())
            }
            async fn health_check(&self) -> Result<(), TaskError> {
                Ok(())
            }
            fn delivery_model(&self) -> DeliveryModel {
                DeliveryModel::Pull
            }
            fn capabilities(&self) -> BrokerCapabilities {
                BrokerCapabilities::default()
            }
        }

        /// Create a RedisBackend connected to localhost, or None if Redis is unavailable.
        async fn make_redis_backend() -> Option<RedisBackend> {
            RedisBackend::new(RedisBackendConfig::default()).await.ok()
        }

        /// Helper to create a WorkflowEngine with real Redis backend and mock broker.
        async fn make_engine() -> Option<(
            WorkflowEngine<RedisBackend, MockBroker>,
            Arc<RedisBackend>,
            Arc<MockBroker>,
        )> {
            let backend = Arc::new(make_redis_backend().await?);
            let broker = Arc::new(MockBroker::new());
            let engine = WorkflowEngine::new(backend.clone(), broker.clone());
            Some((engine, backend, broker))
        }

        // -------------------------------------------------------------------
        // Chain tests
        // -------------------------------------------------------------------

        #[tokio::test]
        async fn test_init_chain() {
            let Some((engine, backend, _broker)) = make_engine().await else {
                return;
            };
            let root_id = TaskId::new();
            let tasks = vec![
                serde_json::to_value(&TaskMessage::new("task1", serde_json::json!([]))).unwrap(),
                serde_json::to_value(&TaskMessage::new("task2", serde_json::json!([]))).unwrap(),
            ];

            engine
                .init_chain(root_id.clone(), tasks.clone(), "test-queue".to_string())
                .await
                .unwrap();

            // Verify metadata was stored
            let chain_key = format!("chain:{}", root_id);
            let meta_value = backend.get_metadata(&chain_key).await.unwrap();
            assert!(meta_value.is_some(), "chain metadata should exist");

            let meta: ChainMeta = serde_json::from_value(meta_value.unwrap()).unwrap();
            assert_eq!(meta.root_id, root_id);
            assert_eq!(meta.tasks.len(), 2);
            assert_eq!(meta.current_index, 0);
            assert!(meta.results.is_empty());
            assert_eq!(meta.queue, "test-queue");

            // Cleanup
            backend.delete_metadata(&chain_key).await.ok();
        }

        #[tokio::test]
        async fn test_chain_continues_next_step() {
            let Some((engine, backend, broker)) = make_engine().await else {
                return;
            };
            let root_id = TaskId::new();
            let task_id = TaskId::new();

            let tasks = vec![
                serde_json::to_value(&TaskMessage::new("task1", serde_json::json!([]))).unwrap(),
                serde_json::to_value(&TaskMessage::new("task2", serde_json::json!([]))).unwrap(),
                serde_json::to_value(&TaskMessage::new("task3", serde_json::json!([]))).unwrap(),
            ];

            engine
                .init_chain(root_id.clone(), tasks, "work-queue".to_string())
                .await
                .unwrap();

            // Complete the first task
            let result = serde_json::json!({"output": "step1_result"});
            engine
                .on_task_complete(&task_id, Some(&root_id), result.clone())
                .await
                .unwrap();

            // Verify backend metadata was updated
            let chain_key = format!("chain:{}", root_id);
            let meta_value = backend.get_metadata(&chain_key).await.unwrap().unwrap();
            let meta: ChainMeta = serde_json::from_value(meta_value).unwrap();
            assert_eq!(meta.current_index, 1);
            assert_eq!(meta.results.len(), 1);
            assert_eq!(meta.results[0], result);

            // Verify broker published 1 message to the correct queue
            let published = broker.published_messages();
            assert_eq!(published.len(), 1);
            assert_eq!(published[0].0, "work-queue");

            // Verify the published message has the previous result prepended to args
            let next_msg = &published[0].1;
            assert_eq!(next_msg.task_name, "task2");
            if let serde_json::Value::Array(ref args) = next_msg.args {
                assert_eq!(
                    args[0], result,
                    "previous result should be prepended to args"
                );
            } else {
                panic!("expected args to be an array");
            }

            // Cleanup
            backend.delete_metadata(&chain_key).await.ok();
        }

        #[tokio::test]
        async fn test_chain_last_step_cleanup() {
            let Some((engine, backend, broker)) = make_engine().await else {
                return;
            };
            let root_id = TaskId::new();

            let tasks = vec![
                serde_json::to_value(&TaskMessage::new("task1", serde_json::json!([]))).unwrap(),
                serde_json::to_value(&TaskMessage::new("task2", serde_json::json!([]))).unwrap(),
            ];

            engine
                .init_chain(root_id.clone(), tasks, "q".to_string())
                .await
                .unwrap();

            // Complete step 1 -> triggers publish of step 2
            let task1_id = TaskId::new();
            engine
                .on_task_complete(&task1_id, Some(&root_id), serde_json::json!("res1"))
                .await
                .unwrap();

            // Complete step 2 (final) -> should clean up, no publish
            let task2_id = TaskId::new();
            engine
                .on_task_complete(&task2_id, Some(&root_id), serde_json::json!("res2"))
                .await
                .unwrap();

            // Verify metadata was deleted (engine cleans up on final step)
            let chain_key = format!("chain:{}", root_id);
            let meta_value = backend.get_metadata(&chain_key).await.unwrap();
            assert!(
                meta_value.is_none(),
                "chain metadata should be deleted after final step"
            );

            // Verify broker published exactly 1 message (step 1->2 only, not after final step)
            let published = broker.published_messages();
            assert_eq!(
                published.len(),
                1,
                "only one message should be published (for step 1->2)"
            );
        }

        #[tokio::test]
        async fn test_chain_result_accumulation() {
            let Some((engine, backend, _broker)) = make_engine().await else {
                return;
            };
            let root_id = TaskId::new();

            let tasks = vec![
                serde_json::to_value(&TaskMessage::new("task1", serde_json::json!([]))).unwrap(),
                serde_json::to_value(&TaskMessage::new("task2", serde_json::json!([]))).unwrap(),
                serde_json::to_value(&TaskMessage::new("task3", serde_json::json!([]))).unwrap(),
            ];

            engine
                .init_chain(root_id.clone(), tasks, "q".to_string())
                .await
                .unwrap();

            let chain_key = format!("chain:{}", root_id);

            // Complete step 1
            engine
                .on_task_complete(&TaskId::new(), Some(&root_id), serde_json::json!(10))
                .await
                .unwrap();

            // After step 1: results should have 1 entry
            let meta: ChainMeta =
                serde_json::from_value(backend.get_metadata(&chain_key).await.unwrap().unwrap())
                    .unwrap();
            assert_eq!(meta.results, vec![serde_json::json!(10)]);

            // Complete step 2
            engine
                .on_task_complete(&TaskId::new(), Some(&root_id), serde_json::json!(20))
                .await
                .unwrap();

            // After step 2: results should have 2 entries
            let meta: ChainMeta =
                serde_json::from_value(backend.get_metadata(&chain_key).await.unwrap().unwrap())
                    .unwrap();
            assert_eq!(
                meta.results,
                vec![serde_json::json!(10), serde_json::json!(20)]
            );

            // Complete step 3 (final) -> metadata deleted
            engine
                .on_task_complete(&TaskId::new(), Some(&root_id), serde_json::json!(30))
                .await
                .unwrap();

            // After step 3: metadata should be cleaned up (engine deletes on final step)
            let meta_value = backend.get_metadata(&chain_key).await.unwrap();
            assert!(
                meta_value.is_none(),
                "metadata should be deleted after final step"
            );
        }

        #[tokio::test]
        async fn test_on_task_complete_no_chain_or_chord() {
            let Some((engine, _backend, broker)) = make_engine().await else {
                return;
            };
            let task_id = TaskId::new();

            // Call on_task_complete with a task_id that has no chain or chord metadata
            let result = engine
                .on_task_complete(&task_id, None, serde_json::json!("orphan_result"))
                .await;

            assert!(result.is_ok(), "should return Ok(()) for unknown task");

            // Verify no messages were published
            let published = broker.published_messages();
            assert!(published.is_empty(), "no messages should be published");
        }

        // -------------------------------------------------------------------
        // Chord tests
        // -------------------------------------------------------------------

        #[tokio::test]
        async fn test_init_chord() {
            let Some((engine, backend, _broker)) = make_engine().await else {
                return;
            };
            let root_id = TaskId::new();
            let header_ids = vec![TaskId::new(), TaskId::new(), TaskId::new()];
            let callback =
                serde_json::to_value(&TaskMessage::new("aggregate", serde_json::json!([])))
                    .unwrap();

            engine
                .init_chord(
                    root_id.clone(),
                    header_ids.clone(),
                    callback.clone(),
                    "chord-queue".to_string(),
                )
                .await
                .unwrap();

            // Verify metadata was stored
            let chord_key = format!("chord:{}", root_id);
            let meta_value = backend.get_metadata(&chord_key).await.unwrap();
            assert!(meta_value.is_some(), "chord metadata should exist");

            let meta: ChordMeta = serde_json::from_value(meta_value.unwrap()).unwrap();
            assert_eq!(meta.root_id, root_id);
            assert_eq!(meta.header_task_ids.len(), 3);
            assert_eq!(meta.header_task_ids, header_ids);
            assert_eq!(meta.results, vec![None, None, None]);
            assert_eq!(meta.callback, callback);
            assert_eq!(meta.queue, "chord-queue");
            assert_eq!(meta.completed_count, 0);

            // Cleanup
            backend.delete_metadata(&chord_key).await.ok();
        }

        #[tokio::test]
        async fn test_chord_partial_completion() {
            let Some((engine, backend, broker)) = make_engine().await else {
                return;
            };
            let root_id = TaskId::new();
            let header_ids = vec![TaskId::new(), TaskId::new(), TaskId::new()];
            let callback =
                serde_json::to_value(&TaskMessage::new("aggregate", serde_json::json!([])))
                    .unwrap();

            engine
                .init_chord(
                    root_id.clone(),
                    header_ids.clone(),
                    callback,
                    "chord-queue".to_string(),
                )
                .await
                .unwrap();

            // Complete 1 of 3 header tasks
            let result_val = serde_json::json!({"partial": true});
            engine
                .on_chord_task_complete(&root_id, &header_ids[0], result_val.clone())
                .await
                .unwrap();

            // Verify metadata was updated
            let chord_key = format!("chord:{}", root_id);
            let meta_value = backend.get_metadata(&chord_key).await.unwrap().unwrap();
            let meta: ChordMeta = serde_json::from_value(meta_value).unwrap();
            assert_eq!(meta.completed_count, 1);
            assert_eq!(meta.results[0], Some(result_val));
            assert_eq!(meta.results[1], None);
            assert_eq!(meta.results[2], None);

            // Verify no message was published (callback not fired yet)
            let published = broker.published_messages();
            assert!(
                published.is_empty(),
                "callback should not fire until all headers complete"
            );

            // Cleanup
            backend.delete_metadata(&chord_key).await.ok();
        }

        #[tokio::test]
        async fn test_chord_all_complete() {
            let Some((engine, backend, broker)) = make_engine().await else {
                return;
            };
            let root_id = TaskId::new();
            let header_ids = vec![TaskId::new(), TaskId::new()];
            let callback =
                serde_json::to_value(&TaskMessage::new("aggregate", serde_json::json!([])))
                    .unwrap();

            engine
                .init_chord(
                    root_id.clone(),
                    header_ids.clone(),
                    callback,
                    "chord-queue".to_string(),
                )
                .await
                .unwrap();

            // Complete both header tasks
            engine
                .on_chord_task_complete(&root_id, &header_ids[0], serde_json::json!("res_a"))
                .await
                .unwrap();
            engine
                .on_chord_task_complete(&root_id, &header_ids[1], serde_json::json!("res_b"))
                .await
                .unwrap();

            // Verify callback message was published with all results as args
            let published = broker.published_messages();
            assert_eq!(published.len(), 1, "callback should be published once");
            assert_eq!(published[0].0, "chord-queue");

            let callback_msg = &published[0].1;
            assert_eq!(callback_msg.task_name, "aggregate");
            assert_eq!(
                callback_msg.args,
                serde_json::json!(["res_a", "res_b"]),
                "callback args should contain all collected results"
            );
            assert_eq!(
                callback_msg.root_id.as_ref(),
                Some(&root_id),
                "callback root_id should be set"
            );

            // Verify metadata was deleted (engine cleans up on chord completion)
            let chord_key = format!("chord:{}", root_id);
            let meta_value = backend.get_metadata(&chord_key).await.unwrap();
            assert!(
                meta_value.is_none(),
                "chord metadata should be deleted after all complete"
            );
        }

        #[tokio::test]
        async fn test_chord_missing_metadata() {
            let Some((engine, _backend, _broker)) = make_engine().await else {
                return;
            };
            let root_id = TaskId::new();
            let task_id = TaskId::new();

            // Call on_chord_task_complete with a non-existent root_id
            let result = engine
                .on_chord_task_complete(&root_id, &task_id, serde_json::json!("val"))
                .await;

            assert!(
                result.is_err(),
                "should return an error for missing metadata"
            );
            let err = result.unwrap_err();
            match err {
                TaskError::Internal(msg) => {
                    assert!(
                        msg.contains("not found"),
                        "error message should contain 'not found', got: {msg}"
                    );
                }
                other => panic!("expected TaskError::Internal, got: {other:?}"),
            }
        }
    }
}

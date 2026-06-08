//! Workflow primitives
//!
//! Chain, Group, and Chord implementations for composing task workflows.

pub mod signature;
pub mod chain;
pub mod group;
pub mod chord;
pub mod map;
pub mod starmap;
pub mod chunks;
pub mod engine;

pub use signature::{TaskSignature, TaskOptions};
pub use chain::{Chain, AsyncChainResult};
pub use group::{Group, GroupResult};
pub use chord::{Chord, AsyncChordResult};
pub use map::{Map, xmap};
pub use starmap::{Starmap, starmap};
pub use chunks::{Chunks, chunks};
pub use engine::WorkflowEngine;

use serde::{Deserialize, Serialize};
use crate::TaskId;

/// Metadata stored in Redis for chain tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMeta {
    /// Unique chain ID
    pub chain_id: TaskId,
    /// All tasks in the chain
    pub tasks: Vec<TaskSignature>,
    /// Current position in the chain
    pub current_index: usize,
    /// Results from completed tasks
    pub results: Vec<serde_json::Value>,
}

impl ChainMeta {
    /// Create new chain metadata
    pub fn new(chain_id: TaskId, tasks: Vec<TaskSignature>) -> Self {
        Self {
            chain_id,
            tasks,
            current_index: 0,
            results: Vec::new(),
        }
    }

    /// Check if chain is complete
    pub fn is_complete(&self) -> bool {
        self.current_index >= self.tasks.len()
    }

    /// Get next task signature
    pub fn next_task(&self) -> Option<&TaskSignature> {
        self.tasks.get(self.current_index)
    }

    /// Record a result and advance to next task
    pub fn advance_with_result(&mut self, result: serde_json::Value) {
        self.results.push(result);
        self.current_index += 1;
    }
}

/// Metadata stored in Redis for chord tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordMeta {
    /// Unique chord ID
    pub chord_id: TaskId,
    /// Task IDs in the header group
    pub header_ids: Vec<TaskId>,
    /// Callback task signature
    pub callback: TaskSignature,
    /// Number of completed header tasks
    pub completed_count: usize,
    /// Total number of header tasks
    pub total_count: usize,
    /// Results from header tasks (index corresponds to header_ids)
    pub results: Vec<Option<serde_json::Value>>,
}

impl ChordMeta {
    /// Create new chord metadata
    pub fn new(chord_id: TaskId, header_ids: Vec<TaskId>, callback: TaskSignature) -> Self {
        let total_count = header_ids.len();
        Self {
            chord_id,
            header_ids,
            callback,
            completed_count: 0,
            total_count,
            results: vec![None; total_count],
        }
    }

    /// Check if all header tasks are complete
    pub fn is_complete(&self) -> bool {
        self.completed_count >= self.total_count
    }

    /// Record a result for a specific task
    pub fn record_result(&mut self, task_id: &TaskId, result: serde_json::Value) -> bool {
        if let Some(index) = self.header_ids.iter().position(|id| id == task_id) {
            if self.results[index].is_none() {
                self.results[index] = Some(result);
                self.completed_count += 1;
                return true;
            }
        }
        false
    }

    /// Get all results (for callback)
    pub fn collect_results(&self) -> Vec<serde_json::Value> {
        self.results
            .iter()
            .filter_map(|r| r.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_meta() {
        let chain_id = TaskId::new();
        let tasks = vec![
            TaskSignature::new("task1", serde_json::json!([])),
            TaskSignature::new("task2", serde_json::json!([])),
        ];

        let mut meta = ChainMeta::new(chain_id.clone(), tasks);
        assert_eq!(meta.chain_id, chain_id);
        assert_eq!(meta.current_index, 0);
        assert!(!meta.is_complete());

        assert!(meta.next_task().is_some());
        meta.advance_with_result(serde_json::json!({"result": 1}));

        assert_eq!(meta.current_index, 1);
        assert_eq!(meta.results.len(), 1);
        assert!(!meta.is_complete());

        meta.advance_with_result(serde_json::json!({"result": 2}));
        assert!(meta.is_complete());
    }

    #[test]
    fn test_chord_meta() {
        let chord_id = TaskId::new();
        let task1_id = TaskId::new();
        let task2_id = TaskId::new();
        let header_ids = vec![task1_id.clone(), task2_id.clone()];
        let callback = TaskSignature::new("callback", serde_json::json!([]));

        let mut meta = ChordMeta::new(chord_id, header_ids, callback);
        assert_eq!(meta.total_count, 2);
        assert_eq!(meta.completed_count, 0);
        assert!(!meta.is_complete());

        assert!(meta.record_result(&task1_id, serde_json::json!({"result": 1})));
        assert_eq!(meta.completed_count, 1);
        assert!(!meta.is_complete());

        assert!(meta.record_result(&task2_id, serde_json::json!({"result": 2})));
        assert_eq!(meta.completed_count, 2);
        assert!(meta.is_complete());

        let results = meta.collect_results();
        assert_eq!(results.len(), 2);
    }
}

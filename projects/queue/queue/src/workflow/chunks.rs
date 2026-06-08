//! Chunks - split items into batches, each batch as one task

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Broker, TaskError};
use super::{Group, GroupResult, TaskOptions, TaskSignature};

/// Chunks: Split items into batches, each batch processed as one task
///
/// Equivalent to Celery's chunks primitive
///
/// # Example
/// ```ignore
/// let items = vec![json!(1), json!(2), json!(3), json!(4), json!(5)];
/// let chunks = Chunks::new("batch_process", items, 2);
/// // Creates 3 tasks: [1,2], [3,4], [5]
/// let result = chunks.apply_async(&broker).await?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunks {
    /// Task name to apply to each chunk
    pub task_name: String,
    /// Items to process
    pub items: Vec<Value>,
    /// Size of each chunk
    pub chunk_size: usize,
    /// Task options applied to all tasks
    pub options: TaskOptions,
}

impl Chunks {
    /// Create a new Chunks primitive
    pub fn new(task_name: impl Into<String>, items: Vec<Value>, chunk_size: usize) -> Self {
        Self {
            task_name: task_name.into(),
            items,
            chunk_size,
            options: TaskOptions::default(),
        }
    }

    /// Set task options
    pub fn with_options(mut self, options: TaskOptions) -> Self {
        self.options = options;
        self
    }

    /// Get the number of chunks that will be created
    pub fn num_chunks(&self) -> usize {
        if self.chunk_size == 0 {
            return 0;
        }
        self.items.len().div_ceil(self.chunk_size)
    }

    /// Convert to a Group for execution
    pub fn to_group(&self) -> Group {
        if self.chunk_size == 0 {
            return Group::new(vec![]).with_options(self.options.clone());
        }

        let chunks: Vec<Vec<Value>> = self.items
            .chunks(self.chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let tasks: Vec<TaskSignature> = chunks
            .into_iter()
            .map(|chunk| {
                TaskSignature::new(
                    self.task_name.clone(),
                    Value::Array(vec![Value::Array(chunk)])
                ).with_options(self.options.clone())
            })
            .collect();

        Group::new(tasks).with_options(self.options.clone())
    }

    /// Execute the chunks as a group (convenience method)
    pub async fn apply_async<B: Broker>(
        &self,
        broker: &B,
    ) -> Result<GroupResult, TaskError> {
        self.to_group().apply_async(broker).await
    }
}

/// Helper function to create Chunks
///
/// # Example
/// ```ignore
/// let items = vec![json!(1), json!(2), json!(3)];
/// let result = chunks("process", items, 2).apply_async(&broker).await?;
/// ```
pub fn chunks(task_name: &str, items: Vec<Value>, chunk_size: usize) -> Chunks {
    Chunks::new(task_name, items, chunk_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_chunks_basic() {
        let items = vec![json!(1), json!(2), json!(3), json!(4)];
        let chunks = Chunks::new("process", items, 2);

        assert_eq!(chunks.task_name, "process");
        assert_eq!(chunks.items.len(), 4);
        assert_eq!(chunks.chunk_size, 2);
    }

    #[test]
    fn test_chunks_even_split() {
        let items = vec![json!(1), json!(2), json!(3), json!(4)];
        let chunks = Chunks::new("process", items, 2);

        assert_eq!(chunks.num_chunks(), 2);

        let group = chunks.to_group();
        assert_eq!(group.tasks.len(), 2);
        assert_eq!(group.tasks[0].args, json!([[1, 2]]));
        assert_eq!(group.tasks[1].args, json!([[3, 4]]));
    }

    #[test]
    fn test_chunks_uneven_split() {
        let items = vec![json!(1), json!(2), json!(3), json!(4), json!(5)];
        let chunks = Chunks::new("process", items, 2);

        assert_eq!(chunks.num_chunks(), 3);

        let group = chunks.to_group();
        assert_eq!(group.tasks.len(), 3);
        assert_eq!(group.tasks[0].args, json!([[1, 2]]));
        assert_eq!(group.tasks[1].args, json!([[3, 4]]));
        assert_eq!(group.tasks[2].args, json!([[5]]));
    }

    #[test]
    fn test_chunks_larger_than_items() {
        let items = vec![json!(1), json!(2)];
        let chunks = Chunks::new("process", items, 10);

        assert_eq!(chunks.num_chunks(), 1);

        let group = chunks.to_group();
        assert_eq!(group.tasks.len(), 1);
        assert_eq!(group.tasks[0].args, json!([[1, 2]]));
    }

    #[test]
    fn test_chunks_empty() {
        let chunks = Chunks::new("process", vec![], 2);
        assert_eq!(chunks.num_chunks(), 0);

        let group = chunks.to_group();
        assert_eq!(group.tasks.len(), 0);
    }

    #[test]
    fn test_chunks_zero_size() {
        let items = vec![json!(1), json!(2)];
        let chunks = Chunks::new("process", items, 0);

        assert_eq!(chunks.num_chunks(), 0);

        let group = chunks.to_group();
        assert_eq!(group.tasks.len(), 0);
    }

    #[test]
    fn test_num_chunks_calculation() {
        let items = vec![json!(1); 10];

        let chunks = Chunks::new("process", items.clone(), 3);
        assert_eq!(chunks.num_chunks(), 4); // [3, 3, 3, 1]

        let chunks = Chunks::new("process", items.clone(), 5);
        assert_eq!(chunks.num_chunks(), 2); // [5, 5]

        let chunks = Chunks::new("process", items, 10);
        assert_eq!(chunks.num_chunks(), 1); // [10]
    }

    #[test]
    fn test_chunks_with_options() {
        let items = vec![json!(1), json!(2)];
        let options = TaskOptions::new().with_queue("batch");
        let chunks = Chunks::new("process", items, 2).with_options(options);

        assert_eq!(chunks.options.queue, Some("batch".to_string()));

        let group = chunks.to_group();
        assert_eq!(group.options.queue, Some("batch".to_string()));
        assert_eq!(group.tasks[0].options.queue, Some("batch".to_string()));
    }

    #[test]
    fn test_chunks_helper() {
        let items = vec![json!(1), json!(2), json!(3)];
        let c = chunks("task", items, 2);
        assert_eq!(c.task_name, "task");
        assert_eq!(c.chunk_size, 2);
    }

    #[test]
    fn test_chunks_single_item_per_chunk() {
        let items = vec![json!(1), json!(2), json!(3)];
        let chunks = Chunks::new("process", items, 1);

        assert_eq!(chunks.num_chunks(), 3);

        let group = chunks.to_group();
        assert_eq!(group.tasks.len(), 3);
        assert_eq!(group.tasks[0].args, json!([[1]]));
        assert_eq!(group.tasks[1].args, json!([[2]]));
        assert_eq!(group.tasks[2].args, json!([[3]]));
    }

    #[test]
    fn test_chunks_complex_types() {
        let items = vec![
            json!({"id": 1}),
            json!({"id": 2}),
            json!({"id": 3}),
        ];
        let chunks = Chunks::new("batch_process", items, 2);

        let group = chunks.to_group();
        assert_eq!(group.tasks[0].args, json!([[{"id": 1}, {"id": 2}]]));
        assert_eq!(group.tasks[1].args, json!([[{"id": 3}]]));
    }
}

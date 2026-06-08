//! Map - apply a task to each item in an iterable

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Broker, TaskError};
use super::{Group, GroupResult, TaskOptions, TaskSignature};

/// Map: Apply a task to each item in an iterable
///
/// Equivalent to Celery's: `group([task.s(item) for item in items])`
///
/// # Example
/// ```ignore
/// let items = vec![
///     json!(1),
///     json!(2),
///     json!(3),
/// ];
/// let map = Map::new("process_item", items);
/// let result = map.apply_async(&broker).await?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    /// Task name to apply to each item
    pub task_name: String,
    /// Items to process
    pub items: Vec<Value>,
    /// Task options applied to all tasks
    pub options: TaskOptions,
}

impl Map {
    /// Create a new Map primitive
    pub fn new(task_name: impl Into<String>, items: Vec<Value>) -> Self {
        Self {
            task_name: task_name.into(),
            items,
            options: TaskOptions::default(),
        }
    }

    /// Set task options
    pub fn with_options(mut self, options: TaskOptions) -> Self {
        self.options = options;
        self
    }

    /// Convert to a Group for execution
    pub fn to_group(&self) -> Group {
        let tasks: Vec<TaskSignature> = self.items
            .iter()
            .map(|item| {
                TaskSignature::new(
                    self.task_name.clone(),
                    Value::Array(vec![item.clone()])
                ).with_options(self.options.clone())
            })
            .collect();

        Group::new(tasks).with_options(self.options.clone())
    }

    /// Execute the map as a group (convenience method)
    pub async fn apply_async<B: Broker>(
        &self,
        broker: &B,
    ) -> Result<GroupResult, TaskError> {
        self.to_group().apply_async(broker).await
    }
}

/// Helper function to create a Map
///
/// # Example
/// ```ignore
/// let result = xmap("process", vec![json!(1), json!(2)]).apply_async(&broker).await?;
/// ```
pub fn xmap(task_name: &str, items: Vec<Value>) -> Map {
    Map::new(task_name, items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_map_basic() {
        let items = vec![json!(1), json!(2), json!(3)];
        let map = Map::new("process", items.clone());

        assert_eq!(map.task_name, "process");
        assert_eq!(map.items.len(), 3);
        assert_eq!(map.items[0], json!(1));
    }

    #[test]
    fn test_map_empty() {
        let map = Map::new("process", vec![]);
        assert_eq!(map.items.len(), 0);
    }

    #[test]
    fn test_map_to_group() {
        let items = vec![json!({"id": 1}), json!({"id": 2})];
        let map = Map::new("process", items);

        let group = map.to_group();
        assert_eq!(group.tasks.len(), 2);
        assert_eq!(group.tasks[0].task_name, "process");
        assert_eq!(group.tasks[0].args, json!([{"id": 1}]));
        assert_eq!(group.tasks[1].args, json!([{"id": 2}]));
    }

    #[test]
    fn test_map_with_options() {
        let items = vec![json!(1)];
        let options = TaskOptions::new().with_queue("priority");
        let map = Map::new("process", items).with_options(options);

        assert_eq!(map.options.queue, Some("priority".to_string()));

        let group = map.to_group();
        assert_eq!(group.options.queue, Some("priority".to_string()));
        assert_eq!(group.tasks[0].options.queue, Some("priority".to_string()));
    }

    #[test]
    fn test_xmap_helper() {
        let map = xmap("task", vec![json!(1), json!(2)]);
        assert_eq!(map.task_name, "task");
        assert_eq!(map.items.len(), 2);
    }

    #[test]
    fn test_map_preserves_complex_types() {
        let items = vec![
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob", "age": 25}),
        ];
        let map = Map::new("process_user", items.clone());

        let group = map.to_group();
        assert_eq!(group.tasks[0].args, json!([items[0]]));
        assert_eq!(group.tasks[1].args, json!([items[1]]));
    }
}

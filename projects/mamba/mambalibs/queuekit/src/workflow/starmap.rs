//! Starmap - apply a task to each tuple of args in an iterable

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Broker, TaskError};
use super::{Group, GroupResult, TaskOptions, TaskSignature};

/// Starmap: Apply a task to each tuple of args in an iterable
///
/// Equivalent to Celery's: `group([task.s(*args) for args in items])`
///
/// # Example
/// ```ignore
/// let items = vec![
///     vec![json!(1), json!(2)],
///     vec![json!(3), json!(4)],
/// ];
/// let starmap = Starmap::new("add", items);
/// let result = starmap.apply_async(&broker).await?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Starmap {
    /// Task name to apply to each tuple
    pub task_name: String,
    /// Items to process (each inner vec is unpacked as args)
    pub items: Vec<Vec<Value>>,
    /// Task options applied to all tasks
    pub options: TaskOptions,
}

impl Starmap {
    /// Create a new Starmap primitive
    pub fn new(task_name: impl Into<String>, items: Vec<Vec<Value>>) -> Self {
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
            .map(|args| {
                TaskSignature::new(
                    self.task_name.clone(),
                    Value::Array(args.clone())
                ).with_options(self.options.clone())
            })
            .collect();

        Group::new(tasks).with_options(self.options.clone())
    }

    /// Execute the starmap as a group (convenience method)
    pub async fn apply_async<B: Broker>(
        &self,
        broker: &B,
    ) -> Result<GroupResult, TaskError> {
        self.to_group().apply_async(broker).await
    }
}

/// Helper function to create a Starmap
///
/// # Example
/// ```ignore
/// let args = vec![
///     vec![json!(1), json!(2)],
///     vec![json!(3), json!(4)],
/// ];
/// let result = starmap("add", args).apply_async(&broker).await?;
/// ```
pub fn starmap(task_name: &str, items: Vec<Vec<Value>>) -> Starmap {
    Starmap::new(task_name, items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_starmap_basic() {
        let items = vec![
            vec![json!(1), json!(2)],
            vec![json!(3), json!(4)],
        ];
        let starmap = Starmap::new("add", items.clone());

        assert_eq!(starmap.task_name, "add");
        assert_eq!(starmap.items.len(), 2);
        assert_eq!(starmap.items[0], vec![json!(1), json!(2)]);
    }

    #[test]
    fn test_starmap_empty() {
        let starmap = Starmap::new("process", vec![]);
        assert_eq!(starmap.items.len(), 0);
    }

    #[test]
    fn test_starmap_tuples() {
        let items = vec![
            vec![json!("a"), json!(1)],
            vec![json!("b"), json!(2)],
        ];
        let starmap = Starmap::new("process", items);

        let group = starmap.to_group();
        assert_eq!(group.tasks.len(), 2);
        assert_eq!(group.tasks[0].task_name, "process");
        assert_eq!(group.tasks[0].args, json!(["a", 1]));
        assert_eq!(group.tasks[1].args, json!(["b", 2]));
    }

    #[test]
    fn test_starmap_to_group() {
        let items = vec![
            vec![json!(1), json!(2)],
            vec![json!(3), json!(4), json!(5)],
        ];
        let starmap = Starmap::new("multiply", items);

        let group = starmap.to_group();
        assert_eq!(group.tasks.len(), 2);
        assert_eq!(group.tasks[0].args, json!([1, 2]));
        assert_eq!(group.tasks[1].args, json!([3, 4, 5]));
    }

    #[test]
    fn test_starmap_with_options() {
        let items = vec![vec![json!(1)]];
        let options = TaskOptions::new().with_queue("math");
        let starmap = Starmap::new("calc", items).with_options(options);

        assert_eq!(starmap.options.queue, Some("math".to_string()));

        let group = starmap.to_group();
        assert_eq!(group.options.queue, Some("math".to_string()));
        assert_eq!(group.tasks[0].options.queue, Some("math".to_string()));
    }

    #[test]
    fn test_starmap_helper() {
        let items = vec![vec![json!(1), json!(2)]];
        let sm = starmap("task", items);
        assert_eq!(sm.task_name, "task");
        assert_eq!(sm.items.len(), 1);
    }

    #[test]
    fn test_starmap_single_arg() {
        let items = vec![
            vec![json!(1)],
            vec![json!(2)],
        ];
        let starmap = Starmap::new("process", items);

        let group = starmap.to_group();
        assert_eq!(group.tasks[0].args, json!([1]));
        assert_eq!(group.tasks[1].args, json!([2]));
    }

    #[test]
    fn test_starmap_complex_types() {
        let items = vec![
            vec![json!({"id": 1}), json!("create")],
            vec![json!({"id": 2}), json!("update")],
        ];
        let starmap = Starmap::new("process_entity", items.clone());

        let group = starmap.to_group();
        assert_eq!(group.tasks[0].args, json!([{"id": 1}, "create"]));
        assert_eq!(group.tasks[1].args, json!([{"id": 2}, "update"]));
    }
}

// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
// CODEGEN-BEGIN
//! Task dependency graph: DAG construction, topological sort, cycle detection.

use anyhow::Result;
use std::collections::{HashMap, HashSet, VecDeque};

use super::config::TaskDef;

/// Directed acyclic graph of task dependencies.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
#[derive(Debug)]
pub struct TaskGraph {
    /// Adjacency list: task → set of tasks it depends on.
    deps: HashMap<String, Vec<String>>,
    /// All known task names.
    tasks: HashSet<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
impl TaskGraph {
    /// Build a task graph from pipeline configuration.
    pub fn from_config(pipeline: &HashMap<String, TaskDef>) -> Result<Self> {
        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        let tasks: HashSet<String> = pipeline.keys().cloned().collect();

        for (name, def) in pipeline {
            let task_deps: Vec<String> = def
                .depends_on
                .iter()
                .filter_map(|d| {
                    // Strip ^ prefix (cross-package indicator)
                    let dep_name = d.trim_start_matches('^');
                    if tasks.contains(dep_name) {
                        Some(dep_name.to_string())
                    } else {
                        tracing::warn!(
                            "Task '{}' depends on unknown task '{}' (skipped)",
                            name,
                            dep_name
                        );
                        None
                    }
                })
                .collect();
            deps.insert(name.clone(), task_deps);
        }

        let graph = Self { deps, tasks };
        graph.detect_cycles()?;
        Ok(graph)
    }

    /// Get the execution order for a task and all its transitive deps.
    /// Returns tasks in topological order (dependencies first).
    pub fn execution_order(&self, task_name: &str) -> Result<Vec<String>> {
        if !self.tasks.contains(task_name) {
            anyhow::bail!("Task '{}' not found in pipeline", task_name);
        }

        // Collect all reachable tasks via BFS
        let mut needed = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(task_name.to_string());

        while let Some(name) = queue.pop_front() {
            if needed.contains(&name) {
                continue;
            }
            needed.insert(name.clone());
            if let Some(task_deps) = self.deps.get(&name) {
                for dep in task_deps {
                    queue.push_back(dep.clone());
                }
            }
        }

        // Kahn's algorithm on the subset
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for name in &needed {
            in_degree.entry(name.as_str()).or_insert(0);
            if let Some(task_deps) = self.deps.get(name.as_str()) {
                for dep in task_deps {
                    if needed.contains(dep) {
                        let _ = *in_degree.entry(dep.as_str()).or_insert(0);
                    }
                }
            }
        }

        // Count incoming edges
        for name in &needed {
            if let Some(task_deps) = self.deps.get(name.as_str()) {
                for dep in task_deps {
                    if needed.contains(dep) {
                        // name depends on dep → dep has an outgoing edge to name
                        // but we count incoming: name has incoming from dep
                    }
                }
            }
        }

        // Recount properly: for each (name depends on dep), name has in-edge from dep
        let mut in_deg: HashMap<String, usize> = HashMap::new();
        for name in &needed {
            in_deg.entry(name.clone()).or_insert(0);
        }
        for name in &needed {
            if let Some(task_deps) = self.deps.get(name.as_str()) {
                for dep in task_deps {
                    if needed.contains(dep) {
                        *in_deg.entry(name.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut queue: VecDeque<String> = in_deg
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut order = Vec::new();
        while let Some(name) = queue.pop_front() {
            order.push(name.clone());
            // Find tasks that depend on `name`
            for (task, task_deps) in &self.deps {
                if needed.contains(task) && task_deps.contains(&name) {
                    if let Some(deg) = in_deg.get_mut(task) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(task.clone());
                        }
                    }
                }
            }
        }

        Ok(order)
    }

    /// Detect cycles in the task graph.
    fn detect_cycles(&self) -> Result<()> {
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();

        for task in &self.tasks {
            if !visited.contains(task.as_str()) {
                self.dfs_cycle(task, &mut visited, &mut in_stack)?;
            }
        }

        Ok(())
    }

    fn dfs_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
    ) -> Result<()> {
        visited.insert(node.to_string());
        in_stack.insert(node.to_string());

        if let Some(task_deps) = self.deps.get(node) {
            for dep in task_deps {
                if in_stack.contains(dep.as_str()) {
                    anyhow::bail!("Cycle detected in task graph: {} → {}", node, dep);
                }
                if !visited.contains(dep.as_str()) {
                    self.dfs_cycle(dep, visited, in_stack)?;
                }
            }
        }

        in_stack.remove(node);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pipeline(tasks: Vec<(&str, Vec<&str>)>) -> HashMap<String, TaskDef> {
        tasks
            .into_iter()
            .map(|(name, deps)| {
                (
                    name.to_string(),
                    TaskDef {
                        depends_on: deps.iter().map(|s| s.to_string()).collect(),
                        inputs: vec![],
                        outputs: vec![],
                        cache: true,
                        persistent: false,
                        env: vec![],
                        command: None,
                        watch: false,
                    },
                )
            })
            .collect()
    }

    #[test]
    fn test_simple_graph() {
        let pipeline = make_pipeline(vec![
            ("build", vec![]),
            ("test", vec!["build"]),
            ("lint", vec![]),
        ]);

        let graph = TaskGraph::from_config(&pipeline).unwrap();
        let order = graph.execution_order("test").unwrap();

        // build must come before test
        let build_idx = order.iter().position(|t| t == "build").unwrap();
        let test_idx = order.iter().position(|t| t == "test").unwrap();
        assert!(build_idx < test_idx);
    }

    #[test]
    fn test_cycle_detection() {
        let pipeline = make_pipeline(vec![("a", vec!["b"]), ("b", vec!["a"])]);

        let result = TaskGraph::from_config(&pipeline);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Cycle detected"));
    }

    #[test]
    fn test_unknown_task_error() {
        let pipeline = make_pipeline(vec![("build", vec![])]);
        let graph = TaskGraph::from_config(&pipeline).unwrap();
        let result = graph.execution_order("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_cross_package_dep_stripped() {
        let pipeline = make_pipeline(vec![("build", vec!["^build"])]);
        // ^build refers to self after stripping ^
        let graph = TaskGraph::from_config(&pipeline);
        // This creates a self-cycle: build depends on build
        assert!(graph.is_err());
    }

    #[test]
    fn test_diamond_dependency() {
        let pipeline = make_pipeline(vec![
            ("a", vec![]),
            ("b", vec!["a"]),
            ("c", vec!["a"]),
            ("d", vec!["b", "c"]),
        ]);

        let graph = TaskGraph::from_config(&pipeline).unwrap();
        let order = graph.execution_order("d").unwrap();

        assert_eq!(order.len(), 4);
        let a_idx = order.iter().position(|t| t == "a").unwrap();
        let b_idx = order.iter().position(|t| t == "b").unwrap();
        let c_idx = order.iter().position(|t| t == "c").unwrap();
        let d_idx = order.iter().position(|t| t == "d").unwrap();
        assert!(a_idx < b_idx);
        assert!(a_idx < c_idx);
        assert!(b_idx < d_idx);
        assert!(c_idx < d_idx);
    }
}
// CODEGEN-END

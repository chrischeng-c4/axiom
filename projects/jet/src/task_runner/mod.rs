// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
// CODEGEN-BEGIN
//! Task runner: parallel script orchestration with dependency graph and caching.
//!
//! Reads `jet.config.toml` pipeline definitions, builds a task DAG,
//! and executes tasks in topological order with content-hash caching.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub mod cache;
pub mod config;
pub mod graph;
pub mod hash;

use cache::TaskCache;
use config::JetConfig;
use graph::TaskGraph;

/// Result of a single task execution.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_name: String,
    pub package_name: Option<String>,
    pub status: TaskStatus,
    pub duration_ms: u64,
    pub cache_hit: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Task execution status.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Success,
    Failed,
    Cached,
    Skipped,
}

/// Task runner orchestrator.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub struct TaskRunner {
    config: JetConfig,
    graph: TaskGraph,
    cache: TaskCache,
    project_root: PathBuf,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
impl TaskRunner {
    /// Create a task runner from the project root.
    /// Loads jet.config.toml and builds the task graph.
    pub fn new(project_root: &Path) -> Result<Self> {
        let config = JetConfig::load(project_root).context("Failed to load jet.config.toml")?;
        let graph = TaskGraph::from_config(&config.pipeline)?;
        let cache = TaskCache::new(project_root)?;

        Ok(Self {
            config,
            graph,
            cache,
            project_root: project_root.to_path_buf(),
        })
    }

    /// Check if a task name is defined in the pipeline.
    pub fn has_task(&self, name: &str) -> bool {
        self.config.pipeline.contains_key(name)
    }

    /// Run a task and all its dependencies.
    pub async fn run(
        &self,
        task_name: &str,
        filter: Option<&str>,
        dry_run: bool,
    ) -> Result<Vec<TaskResult>> {
        let execution_order = self.graph.execution_order(task_name)?;
        let mut results = Vec::new();

        for name in &execution_order {
            let task_def = match self.config.pipeline.get(name.as_str()) {
                Some(def) => def,
                None => continue,
            };

            // Apply filter
            if let Some(pattern) = filter {
                if !name.contains(pattern) {
                    results.push(TaskResult {
                        task_name: name.clone(),
                        package_name: None,
                        status: TaskStatus::Skipped,
                        duration_ms: 0,
                        cache_hit: false,
                        exit_code: 0,
                        stdout: String::new(),
                        stderr: String::new(),
                    });
                    continue;
                }
            }

            // Dry run: just show what would execute
            if dry_run {
                println!("  [dry] {}", name);
                results.push(TaskResult {
                    task_name: name.clone(),
                    package_name: None,
                    status: TaskStatus::Skipped,
                    duration_ms: 0,
                    cache_hit: false,
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                });
                continue;
            }

            // Persistent tasks (dev servers) are never cached
            if task_def.persistent {
                let result = self.execute_task(name).await?;
                results.push(result);
                continue;
            }

            // Check cache
            if task_def.cache {
                let hash = self.cache.compute_hash(
                    name,
                    &task_def.inputs,
                    &task_def.env,
                    &self.project_root,
                )?;

                if let Some(cached) = self.cache.lookup(&hash)? {
                    tracing::info!("{} → CACHED", name);
                    print!("{}", cached.stdout);
                    eprint!("{}", cached.stderr);
                    results.push(TaskResult {
                        task_name: name.clone(),
                        package_name: None,
                        status: TaskStatus::Cached,
                        duration_ms: 0,
                        cache_hit: true,
                        exit_code: 0,
                        stdout: cached.stdout,
                        stderr: cached.stderr,
                    });
                    continue;
                }

                // Execute and store
                let result = self.execute_task(name).await?;
                if result.status == TaskStatus::Success {
                    self.cache.store(
                        &hash,
                        name,
                        &task_def.outputs,
                        &result.stdout,
                        &result.stderr,
                        &self.project_root,
                    )?;
                }
                results.push(result);
            } else {
                let result = self.execute_task(name).await?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Execute a single task by looking up its script in package.json.
    async fn execute_task(&self, task_name: &str) -> Result<TaskResult> {
        let start = std::time::Instant::now();
        tracing::info!("Running task: {}", task_name);

        let runner = crate::runner::ScriptRunner::new(self.project_root.clone());

        let result = if runner.has_script(task_name) {
            runner.run_script(task_name, &[]).await?
        } else {
            // Fallback: try running as shell command
            runner.exec_command(task_name, &[]).await?
        };

        let duration = start.elapsed();
        let status = if result.exit_code == 0 {
            TaskStatus::Success
        } else {
            TaskStatus::Failed
        };

        print!("{}", result.stdout);
        eprint!("{}", result.stderr);

        Ok(TaskResult {
            task_name: task_name.to_string(),
            package_name: None,
            status,
            duration_ms: duration.as_millis() as u64,
            cache_hit: false,
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
        })
    }

    /// Print execution summary.
    pub fn print_summary(results: &[TaskResult]) {
        let success = results
            .iter()
            .filter(|r| r.status == TaskStatus::Success)
            .count();
        let cached = results
            .iter()
            .filter(|r| r.status == TaskStatus::Cached)
            .count();
        let failed = results
            .iter()
            .filter(|r| r.status == TaskStatus::Failed)
            .count();
        let skipped = results
            .iter()
            .filter(|r| r.status == TaskStatus::Skipped)
            .count();

        println!(
            "\nTasks: {} successful, {} cached, {} failed, {} skipped ({} total)",
            success,
            cached,
            failed,
            skipped,
            results.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_equality() {
        assert_eq!(TaskStatus::Success, TaskStatus::Success);
        assert_ne!(TaskStatus::Success, TaskStatus::Failed);
    }
}
// CODEGEN-END

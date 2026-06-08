// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/workflow/task_graph.md#source
// CODEGEN-BEGIN
//! Task graph helpers for per-task implementation loop.
//!
//! Extracts task blocks from tasks.md, builds topological execution order,
//! tracks completion via review artifacts, and checks codegen eligibility.

use std::path::Path;

use super::helpers::extract_verdict;

/// A parsed task block from tasks.md.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/task_graph.md#schema
#[derive(Debug, Clone)]
pub struct TaskInfo {
    /// Task identifier.
    pub id: String,
    /// Task status (e.g. pending, in_progress, complete).
    pub status: String,
    /// Other task IDs this task depends on.
    pub depends_on: Vec<String>,
    /// Optional spec reference (e.g. "my-spec:R1,R2").
    pub spec_ref: Option<String>,
}
/// Parse task blocks from tasks.md content.
///
/// Extracts id, status, depends_on, and spec_ref from inline YAML blocks like:
/// ```yaml
/// id: 2.1
/// status: pending
/// spec_ref: my-spec:R1,R2
/// depends_on: [3.1]
/// ```
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/task_graph.md#source
pub fn parse_task_blocks(content: &str) -> Vec<TaskInfo> {
    let mut tasks = Vec::new();
    let mut in_yaml = false;
    let mut current_id: Option<String> = None;
    let mut current_status = "pending".to_string();
    let mut current_depends: Vec<String> = Vec::new();
    let mut current_spec_ref: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "```yaml" {
            in_yaml = true;
            current_id = None;
            current_status = "pending".to_string();
            current_depends = Vec::new();
            current_spec_ref = None;
            continue;
        }

        if trimmed == "```" && in_yaml {
            in_yaml = false;
            if let Some(id) = current_id.take() {
                tasks.push(TaskInfo {
                    id,
                    status: std::mem::take(&mut current_status),
                    depends_on: std::mem::take(&mut current_depends),
                    spec_ref: current_spec_ref.take(),
                });
            }
            continue;
        }

        if in_yaml {
            if let Some(val) = trimmed.strip_prefix("id:") {
                current_id = Some(val.trim().to_string());
            } else if let Some(val) = trimmed.strip_prefix("status:") {
                current_status = val.trim().to_string();
            } else if let Some(val) = trimmed.strip_prefix("spec_ref:") {
                current_spec_ref = Some(val.trim().to_string());
            } else if let Some(val) = trimmed.strip_prefix("depends_on:") {
                let deps_str = val.trim();
                if deps_str.starts_with('[') && deps_str.ends_with(']') {
                    let inner = &deps_str[1..deps_str.len() - 1];
                    for dep in inner.split(',') {
                        let dep = dep.trim().trim_matches('"').trim_matches('\'');
                        if !dep.is_empty() {
                            current_depends.push(dep.to_string());
                        }
                    }
                }
            }
        }
    }

    tasks
}

/// Build a deterministic execution order from tasks using topological sort
/// with lexical tie-breaking (R1 from impl-workflow-refactor spec).
///
/// Returns task IDs in execution order. Only includes tasks with
/// status "pending" or "in_progress".
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/task_graph.md#source
pub fn build_task_execution_order(tasks: &[TaskInfo]) -> Vec<String> {
    use std::collections::{BTreeSet, HashMap};

    // Build adjacency: task_id -> set of dependencies
    let task_ids: BTreeSet<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for t in tasks {
        in_degree.entry(t.id.as_str()).or_insert(0);
        for dep in &t.depends_on {
            if task_ids.contains(dep.as_str()) {
                *in_degree.entry(t.id.as_str()).or_insert(0) += 1;
                dependents
                    .entry(dep.as_str())
                    .or_default()
                    .push(t.id.as_str());
            }
        }
    }

    // Kahn's algorithm with BTreeSet for lexical ordering
    let mut ready: BTreeSet<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut order = Vec::new();
    while let Some(&id) = ready.iter().next() {
        ready.remove(id);
        order.push(id.to_string());

        if let Some(deps) = dependents.get(id) {
            for &dep in deps {
                if let Some(deg) = in_degree.get_mut(dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        ready.insert(dep);
                    }
                }
            }
        }
    }

    order
}

/// Find the next pending task to execute, respecting execution order and
/// current_task_id for resumption.
///
/// If `current_task_id` is Some, returns that task if it's still pending/in_progress.
/// Otherwise returns the first pending task in execution order.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/task_graph.md#source
pub fn find_next_task(
    tasks: &[TaskInfo],
    execution_order: &[String],
    current_task_id: Option<&str>,
    completed_tasks: &std::collections::HashSet<String>,
) -> Option<String> {
    // If we have a current task, check if it's still actionable
    if let Some(current) = current_task_id {
        if !completed_tasks.contains(current) {
            // Verify it exists in task list
            if tasks.iter().any(|t| t.id == current) {
                return Some(current.to_string());
            }
        }
    }

    // Find first non-completed task in execution order
    for task_id in execution_order {
        if !completed_tasks.contains(task_id) {
            return Some(task_id.clone());
        }
    }

    None
}

/// Determine which tasks are completed by checking for approved review artifacts.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/task_graph.md#source
pub fn find_completed_tasks(
    change_dir: &Path,
    tasks: &[TaskInfo],
) -> std::collections::HashSet<String> {
    let mut completed = std::collections::HashSet::new();

    for task in tasks {
        // A task is completed if it has status "completed" in tasks.md
        if task.status == "completed" {
            completed.insert(task.id.clone());
            continue;
        }

        // Or if it has an APPROVED review (inline in impl file, or legacy separate file)
        let impl_path = change_dir.join(format!("impl_{}.md", task.id));
        let legacy_path = change_dir.join(format!("review_impl_{}.md", task.id));
        // Try impl_ first, fall back to legacy review_impl_ if no verdict found
        let verdict = extract_verdict(&impl_path).or_else(|| extract_verdict(&legacy_path));
        if let Some(v) = verdict {
            if v == "PASS" || v == "APPROVED" {
                completed.insert(task.id.clone());
            }
        }
    }

    completed
}

/// Check if a task's spec_ref points to a codegen-eligible spec.
///
/// A spec is codegen-eligible if it contains `has_json_schema: true`,
/// `has_api_spec: true`, or `has_semantic_diagrams: true` in its
/// `design_elements` frontmatter.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/task_graph.md#source
pub fn is_codegen_eligible(change_dir: &Path, spec_ref: &str) -> bool {
    // spec_ref format: "spec-name:*" or "spec-name:R1,R2"
    let spec_name = spec_ref.split(':').next().unwrap_or(spec_ref);

    // Search for spec file in specs subdirectories
    let specs_dir = change_dir.join("specs");
    if !specs_dir.exists() {
        return false;
    }

    // Try each subdirectory under specs/
    let entries = match std::fs::read_dir(&specs_dir) {
        Ok(e) => e,
        Err(_) => return false,
    };

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let spec_path = entry.path().join(format!("{}.md", spec_name));
        if spec_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&spec_path) {
                return content.contains("has_json_schema: true")
                    || content.contains("has_api_spec: true")
                    || content.contains("has_semantic_diagrams: true");
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_task_blocks() {
        let content = r#"# Tasks

## Task 2.1

```yaml
id: 2.1
action: CREATE
status: pending
file: src/workflows/impl.rs
```

## Task 3.1

```yaml
id: 3.1
action: CREATE
status: pending
file: src/api/review.rs
depends_on: [2.1]
```
"#;
        let tasks = parse_task_blocks(content);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, "2.1");
        assert_eq!(tasks[0].status, "pending");
        assert!(tasks[0].depends_on.is_empty());
        assert_eq!(tasks[1].id, "3.1");
        assert_eq!(tasks[1].depends_on, vec!["2.1"]);
    }

    #[test]
    fn test_parse_task_blocks_with_spec_ref() {
        let content = r#"# Tasks

## Task 2.1

```yaml
id: 2.1
action: CREATE
status: pending
file: src/model.rs
spec_ref: my-spec:R1,R2
```

## Task 3.1

```yaml
id: 3.1
action: CREATE
status: pending
file: src/api.rs
```
"#;
        let tasks = parse_task_blocks(content);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].spec_ref, Some("my-spec:R1,R2".to_string()));
        assert_eq!(tasks[1].spec_ref, None);
    }

    #[test]
    fn test_build_task_execution_order_with_deps() {
        let tasks = vec![
            TaskInfo {
                id: "C".into(),
                status: "pending".into(),
                depends_on: vec!["B".into()],
                spec_ref: None,
            },
            TaskInfo {
                id: "A".into(),
                status: "pending".into(),
                depends_on: vec![],
                spec_ref: None,
            },
            TaskInfo {
                id: "B".into(),
                status: "pending".into(),
                depends_on: vec!["A".into()],
                spec_ref: None,
            },
        ];
        let order = build_task_execution_order(&tasks);
        assert_eq!(order, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_build_task_execution_order_lexical_tiebreak() {
        let tasks = vec![
            TaskInfo {
                id: "X".into(),
                status: "pending".into(),
                depends_on: vec![],
                spec_ref: None,
            },
            TaskInfo {
                id: "A".into(),
                status: "pending".into(),
                depends_on: vec![],
                spec_ref: None,
            },
            TaskInfo {
                id: "M".into(),
                status: "pending".into(),
                depends_on: vec![],
                spec_ref: None,
            },
        ];
        let order = build_task_execution_order(&tasks);
        assert_eq!(order, vec!["A", "M", "X"]);
    }

    #[test]
    fn test_find_next_task_resumes_current() {
        let tasks = vec![
            TaskInfo {
                id: "A".into(),
                status: "pending".into(),
                depends_on: vec![],
                spec_ref: None,
            },
            TaskInfo {
                id: "B".into(),
                status: "pending".into(),
                depends_on: vec![],
                spec_ref: None,
            },
        ];
        let order = vec!["A".to_string(), "B".to_string()];
        let completed = std::collections::HashSet::new();
        let next = find_next_task(&tasks, &order, Some("B"), &completed);
        assert_eq!(next, Some("B".to_string()));
    }

    #[test]
    fn test_find_next_task_skips_completed() {
        let tasks = vec![
            TaskInfo {
                id: "A".into(),
                status: "pending".into(),
                depends_on: vec![],
                spec_ref: None,
            },
            TaskInfo {
                id: "B".into(),
                status: "pending".into(),
                depends_on: vec![],
                spec_ref: None,
            },
        ];
        let order = vec!["A".to_string(), "B".to_string()];
        let mut completed = std::collections::HashSet::new();
        completed.insert("A".to_string());
        let next = find_next_task(&tasks, &order, None, &completed);
        assert_eq!(next, Some("B".to_string()));
    }

    #[test]
    fn test_find_next_task_all_done() {
        let tasks = vec![TaskInfo {
            id: "A".into(),
            status: "pending".into(),
            depends_on: vec![],
            spec_ref: None,
        }];
        let order = vec!["A".to_string()];
        let mut completed = std::collections::HashSet::new();
        completed.insert("A".to_string());
        let next = find_next_task(&tasks, &order, None, &completed);
        assert_eq!(next, None);
    }

    #[test]
    fn test_find_completed_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        std::fs::write(
            change_dir.join("impl_2.1.md"),
            "---\nverdict: PASS\n---\n# Implementation\n\n# Reviews\nApproved.\n",
        )
        .unwrap();
        let tasks = vec![
            TaskInfo {
                id: "2.1".into(),
                status: "pending".into(),
                depends_on: vec![],
                spec_ref: None,
            },
            TaskInfo {
                id: "3.1".into(),
                status: "pending".into(),
                depends_on: vec![],
                spec_ref: None,
            },
        ];
        let completed = find_completed_tasks(change_dir, &tasks);
        assert!(completed.contains("2.1"));
        assert!(!completed.contains("3.1"));
    }

    #[test]
    fn test_is_codegen_eligible_with_api_spec() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let spec_dir = change_dir.join("specs/my-group");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("my-spec.md"),
            "---\ndesign_elements:\n  has_api_spec: true\n---\n# Spec\n",
        )
        .unwrap();
        assert!(is_codegen_eligible(change_dir, "my-spec:*"));
    }

    #[test]
    fn test_is_codegen_eligible_without_api_spec() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let spec_dir = change_dir.join("specs/my-group");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("my-spec.md"),
            "---\ndesign_elements:\n  has_api_spec: false\n---\n# Spec\n",
        )
        .unwrap();
        assert!(!is_codegen_eligible(change_dir, "my-spec:*"));
    }

    #[test]
    fn test_is_codegen_eligible_no_specs_dir() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!is_codegen_eligible(temp_dir.path(), "missing:*"));
    }
}

// CODEGEN-END

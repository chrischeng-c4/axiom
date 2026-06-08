---
id: sdd-agents-code-agent-tasks
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Agent-facing public interfaces are part of the AW Core client-independent workflow protocol surface."
---

# Code Agent Task Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/agents/code_agent/tasks.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ImplementationTask` | projects/agentic-workflow/src/agents/code_agent/tasks.rs | struct | pub | 48 |  |
| `TaskAction` | projects/agentic-workflow/src/agents/code_agent/tasks.rs | enum | pub | 36 |  |
| `TaskCategory` | projects/agentic-workflow/src/agents/code_agent/tasks.rs | enum | pub | 18 |  |
| `decompose_spec` | projects/agentic-workflow/src/agents/code_agent/tasks.rs | function | pub | 71 | decompose_spec(spec: &str) -> Vec<ImplementationTask> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  TaskCategory:
    type: string
    enum: [DataModel, Logic, Integration, Test]
    description: |
      Execution category that controls the topological sort order.
      Ord is derived so tasks sort Data < Logic < Integration < Test.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize]
      variants:
        - { name: DataModel, doc: "Data model task." }
        - { name: Logic, doc: "Business logic task." }
        - { name: Integration, doc: "Integration task." }
        - { name: Test, doc: "Test task." }

  TaskAction:
    type: string
    enum: [Create, Modify]
    description: |
      Whether the target file should be newly created or modified in place.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      variants:
        - { name: Create, doc: "Create a new file." }
        - { name: Modify, doc: "Modify an existing file." }

  ImplementationTask:
    type: object
    required: [id, description, file_path, category, action]
    description: |
      A single implementation task extracted from a spec.
    properties:
      id:
        type: string
        description: "Sequential identifier (1-based)."
      description:
        type: string
        description: "Short description extracted from the spec bullet."
      file_path:
        type: string
        description: "Relative file path targeted by this task."
      category:
        type: string
        x-rust-type: "TaskCategory"
        description: "Execution category (determines sort position)."
      action:
        type: string
        x-rust-type: "TaskAction"
        description: "Create or modify."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/agents/code_agent/tasks.rs -->
````rust
//! Task decomposition and topological sorting for CodeAgent.
//!
//! Parses the `## Changes` section of a specification and returns
//! implementation tasks ordered Data → Logic → Integration → Test.

// ============================================================
// Types
// ============================================================

use serde::{Deserialize, Serialize};

/// Execution category that controls the topological sort order.
/// Ord is derived so tasks sort Data < Logic < Integration < Test.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/tasks.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskCategory {
    /// Data model task.
    #[serde(rename = "DataModel")]
    DataModel,
    /// Business logic task.
    #[serde(rename = "Logic")]
    Logic,
    /// Integration task.
    #[serde(rename = "Integration")]
    Integration,
    /// Test task.
    #[serde(rename = "Test")]
    Test,
}

/// Whether the target file should be newly created or modified in place.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/tasks.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskAction {
    /// Create a new file.
    #[serde(rename = "Create")]
    Create,
    /// Modify an existing file.
    #[serde(rename = "Modify")]
    Modify,
}

/// A single implementation task extracted from a spec.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/tasks.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationTask {
    /// Sequential identifier (1-based).
    pub id: String,
    /// Short description extracted from the spec bullet.
    pub description: String,
    /// Relative file path targeted by this task.
    pub file_path: String,
    /// Execution category (determines sort position).
    pub category: TaskCategory,
    /// Create or modify.
    pub action: TaskAction,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/tasks.md#source
// ============================================================
// Public API
// ============================================================

/// Parse the `## Changes` section of `spec` into an ordered task list.
///
/// Tasks are sorted by category: Data → Logic → Integration → Test.
/// Within the same category the original spec order is preserved (stable sort).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/tasks.md#source
pub fn decompose_spec(spec: &str) -> Vec<ImplementationTask> {
    let changes = extract_changes_section(spec);
    let mut tasks: Vec<ImplementationTask> = parse_change_entries(&changes)
        .into_iter()
        .enumerate()
        .map(|(idx, entry)| {
            let category = infer_category(&entry.path);
            ImplementationTask {
                id: format!("task-{}", idx + 1),
                description: entry.description,
                file_path: entry.path,
                category,
                action: entry.action,
            }
        })
        .collect();

    tasks.sort_by_key(|t| t.category);
    tasks
}

// ============================================================
// Private helpers
// ============================================================

struct ChangeEntry {
    path: String,
    description: String,
    action: TaskAction,
}

/// Extract everything under `## Changes` up to the next `##` heading.
fn extract_changes_section(spec: &str) -> String {
    let marker = "## Changes";
    if let Some(start) = spec.find(marker) {
        let rest = &spec[start + marker.len()..];
        // Stop at the next top-level section (but not sub-sections like ###).
        if let Some(next) = find_next_h2(rest) {
            rest[..next].to_string()
        } else {
            rest.to_string()
        }
    } else {
        String::new()
    }
}

/// Find the position of the next `\n## ` marker in `s`.
fn find_next_h2(s: &str) -> Option<usize> {
    s.find("\n## ")
}

/// Parse change bullets like:
///
/// ```text
/// - `path/to/file.rs`:
///   - **CREATE**: description
///   - **MODIFY**: description
/// ```
fn parse_change_entries(section: &str) -> Vec<ChangeEntry> {
    let mut entries = Vec::new();
    let mut current_path: Option<String> = None;

    for line in section.lines() {
        let trimmed = line.trim();

        // Detect a path bullet: `- \`path\``
        if let Some(path) = parse_path_bullet(trimmed) {
            current_path = Some(path);
            continue;
        }

        // Detect action bullets nested under a path: `- **CREATE**:` / `- **MODIFY**:`
        if let Some(path) = &current_path {
            if let Some((action, desc)) = parse_action_bullet(trimmed) {
                entries.push(ChangeEntry {
                    path: path.clone(),
                    description: desc,
                    action,
                });
            }
        }
    }

    // Fallback: if no nested action bullets were found, treat each path bullet
    // as a single MODIFY task using the rest of the line as description.
    if entries.is_empty() {
        for line in section.lines() {
            let trimmed = line.trim();
            if let Some(path) = parse_path_bullet(trimmed) {
                let desc = trimmed
                    .trim_start_matches("- ")
                    .trim_start_matches('`')
                    .to_string();
                entries.push(ChangeEntry {
                    path,
                    description: desc,
                    action: TaskAction::Modify,
                });
            }
        }
    }

    entries
}

/// Parse `- \`some/path.rs\`...` → returns the path string.
fn parse_path_bullet(line: &str) -> Option<String> {
    let line = line.strip_prefix("- `")?;
    let end = line.find('`')?;
    let path = line[..end].to_string();
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

/// Parse `- **CREATE**: description` or `- **MODIFY**: description`.
fn parse_action_bullet(line: &str) -> Option<(TaskAction, String)> {
    let inner = line.strip_prefix("- ")?;
    if let Some(rest) = inner
        .strip_prefix("**CREATE**:")
        .or_else(|| inner.strip_prefix("**Create**:"))
    {
        return Some((TaskAction::Create, rest.trim().to_string()));
    }
    if let Some(rest) = inner
        .strip_prefix("**MODIFY**:")
        .or_else(|| inner.strip_prefix("**Modify**:"))
        .or_else(|| inner.strip_prefix("**DO**:"))
        .or_else(|| inner.strip_prefix("**Do**:"))
    {
        return Some((TaskAction::Modify, rest.trim().to_string()));
    }
    None
}

/// Infer category from the file path heuristics.
fn infer_category(path: &str) -> TaskCategory {
    let p = path.to_lowercase();

    if p.ends_with("_test.rs")
        || p.ends_with("_spec.rs")
        || p.contains("/tests/")
        || p.starts_with("tests/")
        || p.contains("test_")
    {
        return TaskCategory::Test;
    }

    if p.contains("integrations/")
        || p.contains("integration/")
        || p.contains("github")
        || p.contains("gitlab")
        || p.contains("jira")
    {
        return TaskCategory::Integration;
    }

    if p.contains("error") || p.contains("types") || p.contains("models") || p.contains("schema") {
        return TaskCategory::DataModel;
    }

    TaskCategory::Logic
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SPEC: &str = r#"## Changes

- `crates/cclab-agent/src/error.rs`:
  - **MODIFY**: Add `MalformedLLMResponse` and `PlatformError` variants.

- `crates/cclab-agent/src/integrations/mod.rs`:
  - **MODIFY**: Add `create_branch`, `create_commit`, `create_pull_request` to the trait.

- `crates/cclab-agent/src/agents/code_agent/mod.rs`:
  - **CREATE**: Introduce the `CodeAgent` struct.

- `crates/cclab-agent/tests/code_agent_test.rs`:
  - **CREATE**: Integration tests for CodeAgent.
"#;

    #[test]
    fn test_decompose_returns_tasks() {
        let tasks = decompose_spec(SAMPLE_SPEC);
        assert!(!tasks.is_empty(), "should return at least one task");
    }

    #[test]
    fn test_topological_order() {
        let tasks = decompose_spec(SAMPLE_SPEC);
        // Verify categories are non-decreasing after sort.
        for w in tasks.windows(2) {
            assert!(
                w[0].category <= w[1].category,
                "tasks out of order: {:?} > {:?}",
                w[0].category,
                w[1].category
            );
        }
    }

    #[test]
    fn test_error_file_is_data_model() {
        let tasks = decompose_spec(SAMPLE_SPEC);
        let error_task = tasks
            .iter()
            .find(|t| t.file_path.contains("error.rs"))
            .expect("error.rs task should be present");
        assert_eq!(error_task.category, TaskCategory::DataModel);
    }

    #[test]
    fn test_integration_file_category() {
        let tasks = decompose_spec(SAMPLE_SPEC);
        let integration_task = tasks
            .iter()
            .find(|t| t.file_path.contains("integrations/mod.rs"))
            .expect("integrations task should be present");
        assert_eq!(integration_task.category, TaskCategory::Integration);
    }

    #[test]
    fn test_test_file_category() {
        let tasks = decompose_spec(SAMPLE_SPEC);
        let test_task = tasks
            .iter()
            .find(|t| t.file_path.contains("test"))
            .expect("test task should be present");
        assert_eq!(test_task.category, TaskCategory::Test);
    }

    #[test]
    fn test_create_action_parsed() {
        let tasks = decompose_spec(SAMPLE_SPEC);
        let create_task = tasks
            .iter()
            .find(|t| t.file_path.contains("code_agent/mod.rs"))
            .expect("code_agent task should be present");
        assert!(matches!(create_task.action, TaskAction::Create));
    }

    #[test]
    fn test_modify_action_parsed() {
        let tasks = decompose_spec(SAMPLE_SPEC);
        let modify_task = tasks
            .iter()
            .find(|t| t.file_path.contains("error.rs"))
            .expect("error task should be present");
        assert!(matches!(modify_task.action, TaskAction::Modify));
    }

    #[test]
    fn test_empty_spec_returns_empty() {
        let tasks = decompose_spec("## Overview\nNo changes section.");
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_infer_category_test() {
        assert_eq!(infer_category("src/foo_test.rs"), TaskCategory::Test);
        assert_eq!(infer_category("tests/integration.rs"), TaskCategory::Test);
    }

    #[test]
    fn test_infer_category_integration() {
        assert_eq!(
            infer_category("src/integrations/github.rs"),
            TaskCategory::Integration
        );
    }

    #[test]
    fn test_infer_category_data_model() {
        assert_eq!(infer_category("src/error.rs"), TaskCategory::DataModel);
        assert_eq!(infer_category("src/types.rs"), TaskCategory::DataModel);
    }

    #[test]
    fn test_infer_category_logic() {
        assert_eq!(
            infer_category("src/agents/code_agent/mod.rs"),
            TaskCategory::Logic
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/agents/code_agent/tasks.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete code-agent task decomposer, including
      schema-derived public types, private helper state, parser helpers,
      category inference, and tests.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Three public types with standard enums + struct shape.
- [schema] All in `required:`; foreign-type fields via x-rust-type. Discriminants safely dropped — declaration order = serde-name-equivalent variant order, and no int casts in the codebase.
- [changes] Standard split with all three public types in `replaces`; private ChangeEntry preserved.

## Review 2
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Promotes the parser and helper algorithm into source-template ownership while preserving schema as the public type contract.
- [source] Uses `strip-managed-markers` to preserve current Rust behavior and remove the mixed CODEGEN/HANDWRITE boundary.
- [changes] Correctly routes the target file through the `source` section with `impl_mode: codegen`.

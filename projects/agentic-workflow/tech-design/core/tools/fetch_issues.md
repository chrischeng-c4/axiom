---
id: projects-sdd-src-tools-fetch-issues-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue and platform-sync tool TDs expose AW Core workflow state through configured external clients."
---

# Standardized projects/agentic-workflow/src/tools/fetch_issues.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/fetch_issues.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/fetch_issues.rs | function | pub | 29 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/fetch_issues.rs | function | pub | 61 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `list_issues_by_labels` | projects/agentic-workflow/src/tools/fetch_issues.rs | function | pub | 657 | list_issues_by_labels(     labels: &[String],     repo: Option<&str>,     project_root: Option<&Path>, ) -> Result<Vec<u64>> |
## Source
<!-- type: source lang: rust -->

````rust
//! Fetch Issues MCP Tool
//!
//! Fetches issue content and dependencies via `gh` (GitHub) or `glab` (GitLab) CLI,
//! stores issue artifacts, and builds a DAG in STATE.yaml.
//! Platform is auto-detected from `[platform] type` in `.aw/config.toml`.

use super::{get_required_string, ToolDefinition};
use crate::models::state::{DagIssue, DagState};
use crate::state::StateManager;
use crate::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::process::Command;

/// Detected platform type from config
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlatformType {
    GitHub,
    GitLab,
}

/// Get the tool definition for fetch_issues
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_fetch_issues".to_string(),
        description: "Fetch issues and build dependency graph in STATE.yaml. \
            Auto-detects platform (GitHub/GitLab) from .aw/config.toml [platform] type. \
            Uses gh or glab CLI for authentication and data retrieval. \
            Parses issue descriptions for dependency links (blockedBy, #NNN)."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "issue_refs"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "description": "Change ID (lowercase, hyphens allowed)"
                },
                "issue_refs": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Issue references: URLs (github.com or gitlab.com) or #NNN"
                }
            }
        }),
    }
}

/// Execute the fetch_issues tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let issue_refs = args
        .get("issue_refs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing required field: issue_refs"))?;

    let change_dir = super::workflow_common::resolve_change_dir(project_root, &change_id);
    if !change_dir.exists() {
        std::fs::create_dir_all(&change_dir)?;
    }

    // Detect platform from config.toml
    let platform = detect_platform(project_root);

    // Parse issue refs into normalized numbers
    let initial_refs: Vec<String> = issue_refs
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    if initial_refs.is_empty() {
        anyhow::bail!("issue_refs must contain at least one issue reference");
    }

    // Detect repo from URL refs or config
    let repo = detect_repo(&initial_refs).or_else(|| detect_repo_from_config(project_root));

    // BFS fetch: start with initial refs, discover dependencies
    let mut fetched: HashMap<u64, FetchedIssue> = HashMap::new();
    let mut queue: VecDeque<u64> = VecDeque::new();
    let mut seen: HashSet<u64> = HashSet::new();

    for r in &initial_refs {
        if let Some(num) = parse_issue_number(r) {
            if seen.insert(num) {
                queue.push_back(num);
            }
        }
    }

    let mut errors: Vec<String> = Vec::new();

    while let Some(issue_num) = queue.pop_front() {
        let result = match platform {
            PlatformType::GitLab => fetch_issue_glab(issue_num, repo.as_deref()),
            PlatformType::GitHub => fetch_issue(issue_num, repo.as_deref()),
        };
        match result {
            Ok(issue) => {
                // Discover dependencies and enqueue
                for dep in &issue.dependencies {
                    if seen.insert(*dep) {
                        queue.push_back(*dep);
                    }
                }
                fetched.insert(issue_num, issue);
            }
            Err(e) => {
                errors.push(format!("#{}: {}", issue_num, e));
            }
        }
    }

    if fetched.is_empty() {
        anyhow::bail!("Failed to fetch any issues. Errors: {}", errors.join("; "));
    }

    // Write issue artifacts into issues/ subdirectory
    let issues_dir = change_dir.join("issues");
    std::fs::create_dir_all(&issues_dir)?;
    for (num, issue) in &fetched {
        let slug = slugify(&issue.title);
        let filename = format!("issue_{}_{}.md", num, slug);
        let content = format_issue_md(issue);
        std::fs::write(issues_dir.join(&filename), &content)?;
    }

    // Build DAG (topological sort)
    let topo_order = topological_sort(&fetched);

    // Update STATE.yaml with dag section
    update_state_dag(&change_dir, &topo_order, &fetched)?;

    // Format response
    let mut response = String::new();
    response.push_str(&format!("## Fetched {} issue(s)\n\n", fetched.len()));

    for num in &topo_order {
        if let Some(issue) = fetched.get(num) {
            let deps_str = if issue.dependencies.is_empty() {
                String::new()
            } else {
                format!(
                    " (blocked by: {})",
                    issue
                        .dependencies
                        .iter()
                        .map(|d| format!("#{}", d))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            response.push_str(&format!("- **#{}** {}{}\n", num, issue.title, deps_str));
        }
    }

    if !errors.is_empty() {
        response.push_str(&format!("\n**Errors**: {}\n", errors.join("; ")));
    }

    response.push_str(&format!(
        "\n**Topological order**: {}\n",
        topo_order
            .iter()
            .map(|n| format!("#{}", n))
            .collect::<Vec<_>>()
            .join(" → ")
    ));

    response.push_str("\n→ Next: call `sdd_run_change` to continue.\n");

    Ok(response)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/fetch_issues.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects-sdd-src-tools-fetch-issues-rs>"
    description: "Fetch issues MCP tool entrypoint and orchestration."
```

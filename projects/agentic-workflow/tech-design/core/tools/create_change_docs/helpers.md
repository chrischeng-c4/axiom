---
id: sdd-tools-create-change-docs-helpers
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change docs helpers

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_docs.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `artifact_definition` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 45 | artifact_definition() -> ToolDefinition |
| `execute_artifact` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 199 | execute_artifact(args: &Value, project_root: &Path) -> Result<String> |
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 94 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_docs.rs | function | pub | 20 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
/// Resolve crates affected by this change from spec files.
///
/// Reads spec files to determine which crates are referenced in the change.
/// Falls back to inspecting the `changes` YAML blocks in spec files.
fn resolve_affected_crates(change_dir: &Path) -> Vec<String> {
    let mut crates = Vec::new();

    // Check groups/*/specs/ and specs/ for spec files
    let spec_dirs: Vec<std::path::PathBuf> = {
        let mut dirs = vec![change_dir.join("specs")];
        let groups_dir = change_dir.join("groups");
        if groups_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&groups_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        dirs.push(entry.path().join("specs"));
                    }
                }
            }
        }
        dirs
    };

    for spec_dir in spec_dirs {
        if !spec_dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&spec_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        // Extract crate names from change paths like "crates/cclab-xxx/..."
                        for line in content.lines() {
                            if let Some(rest) = line.trim().strip_prefix("- path: crates/") {
                                if let Some(crate_name) = rest.split('/').next() {
                                    let name = crate_name.to_string();
                                    if !crates.contains(&name) {
                                        crates.push(name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    crates
}

/// Merge new sections into existing guide content.
///
/// For each section in `sections_content`, if a matching `## section_name`
/// heading exists in the guide, replace that section's content. Otherwise
/// append the new section at the end.
fn merge_guide_sections(existing: &str, sections: &serde_json::Map<String, Value>) -> String {
    let mut result = existing.to_string();

    for (section_name, content) in sections {
        let content_str = content.as_str().unwrap_or("");
        let heading = format!("## {}", section_name);

        if let Some(start) = result.find(&heading) {
            // Find the end of this section (next ## heading or end of file)
            let after_heading = start + heading.len();
            let section_end = result[after_heading..]
                .find("\n## ")
                .map(|pos| after_heading + pos)
                .unwrap_or(result.len());

            // Replace section content
            let new_section = format!("{}\n\n{}\n", heading, content_str.trim());
            result = format!(
                "{}{}{}",
                &result[..start],
                new_section,
                &result[section_end..]
            );
        } else {
            // Append new section
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(&format!("\n{}\n\n{}\n", heading, content_str.trim()));
        }
    }

    result
}

/// Build doc-writer prompt for creating docs.
fn build_create_docs_prompt(
    change_id: &str,
    targets: &[Value],
    group_id: Option<&str>,
    _project_root: &Path,
) -> String {
    let targets_summary: Vec<String> = targets
        .iter()
        .map(|t| {
            let crate_name = t["crate"].as_str().unwrap_or("unknown");
            let guide = t["guide"].as_str().unwrap_or("unknown");
            let audience = t["audience"].as_str().unwrap_or("developer");
            let sections: Vec<&str> = t["sections"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|s| s.as_str()).collect())
                .unwrap_or_default();
            format!(
                "- Crate: `{}`, Guide: `{}`, Audience: {}, Sections: [{}]",
                crate_name,
                guide,
                audience,
                sections.join(", ")
            )
        })
        .collect();

    let spec_path_prefix = match group_id {
        Some(gid) => format!(".aw/changes/{}/groups/{}/specs", change_id, gid),
        None => format!(".aw/changes/{}/specs", change_id),
    };

    format!(
        r#"# Task: Create Docs for Change '{change_id}'

## Instructions

1. Read all change specs in `{spec_path_prefix}/`
2. Read existing guide files (if they exist)
3. For each matched doc target below, generate/update the specified sections
4. Write each target's sections via the artifact CLI command

## Matched Doc Targets

{targets_list}

## Guidelines

- Write clear, accurate documentation based on the change specs
- Match the audience level (developer = technical detail, end-user = usage-focused, admin = deployment/config)
- Preserve existing guide content for sections not being updated
- Include CLI examples where relevant
- Reference actual command names and parameters from the specs

## CLI Commands

```
# Read change specs
Glob pattern: {spec_path_prefix}/*.md

# Write docs artifact (write payload JSON first, then run)
score artifact create-change-docs {change_id} .aw/changes/{change_id}/payloads/create-change-docs.json
```"#,
        targets_list = targets_summary.join("\n"),
    )
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_docs.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "resolve_affected_crates"
      - "merge_guide_sections"
      - "build_create_docs_prompt"
    description: "Helper functions for affected-crate detection, guide section merging, and docs prompts."
```

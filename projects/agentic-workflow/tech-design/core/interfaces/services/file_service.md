---
id: projects-sdd-src-services-file-service-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Standardized projects/agentic-workflow/src/services/file_service.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/file_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `list_specs` | projects/agentic-workflow/src/services/file_service.rs | function | pub | 199 | list_specs(change_id: &str, spec_id: Option<&str>, project_root: &Path) -> Result<String> |
| `read_file` | projects/agentic-workflow/src/services/file_service.rs | function | pub | 26 | read_file(change_id: &str, file: &str, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/services/file_service.rs -->
````rust
//! File service - Business logic for reading change files
//!
//! Supports scope prefixes in the `file` parameter:
//! - `main_spec:group/id` — read from .aw/tech-design/group/id.md
//! - `list:main_specs` — list main specs (optional `list:main_specs:group` to filter)
//! - `list:specs` — list change specs
//! - `requirements` — read all requirements (proposal + tasks + specs)
//! - Unprefixed values — read from change directory (proposal, tasks, specs, contexts, gaps)

use crate::models::SddConfig;
use crate::shared::workspace;
use crate::workflow::scope::resolve_spec_dir_for_root;
use crate::Result;
use std::path::{Path, PathBuf};

/// Unified file reader with scope prefix routing.
///
/// Dispatches based on prefix:
/// - `main_spec:*` → main spec reader
/// - `list:*` → listing functions
/// - `requirements` → all requirements aggregator
/// - else → change-scoped file reader
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/file_service.md#source
pub fn read_file(change_id: &str, file: &str, project_root: &Path) -> Result<String> {
    // Route scope-prefixed requests
    if let Some(ref_str) = file.strip_prefix("main_spec:") {
        return read_main_spec_scoped(ref_str, project_root);
    }
    if let Some(scope) = file.strip_prefix("list:") {
        return read_list_scoped(scope, change_id, project_root);
    }
    if file == "requirements" {
        return read_all_requirements(change_id, project_root);
    }

    // Change-scoped file read (original behavior)
    read_change_file(change_id, file, project_root)
}

/// Read a file from a change directory (original read_file logic)
fn read_change_file(change_id: &str, file: &str, project_root: &Path) -> Result<String> {
    // Check change directory exists
    let change_dir = project_root.join(".aw/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found.", change_id);
    }

    // Determine which file to read
    let file_path = match file {
        "proposal" => change_dir.join("proposal.md"),
        "tasks" => change_dir.join("tasks.md"),
        "spec_context" => change_dir.join("spec_context.md"),
        "knowledge_context" => change_dir.join("knowledge_context.md"),
        "codebase_context" => change_dir.join("codebase_context.md"),
        "gap_codebase_spec" => change_dir.join("gap_codebase_spec.md"),
        "gap_codebase_knowledge" => change_dir.join("gap_codebase_knowledge.md"),
        "gap_spec_knowledge" => change_dir.join("gap_spec_knowledge.md"),
        "reference_context" => change_dir.join("reference_context.md"),
        "pre_clarifications" => change_dir.join("pre_clarifications.md"),
        "context_clarifications" => change_dir.join("pre_clarifications.md"), // legacy alias
        "clarifications" => change_dir.join("pre_clarifications.md"),         // legacy alias
        // Inline review artifacts — extract # Reviews section from original file
        "review_spec_context" => {
            return read_inline_review(&change_dir, "spec_context.md", file);
        }
        "review_knowledge_context" => {
            return read_inline_review(&change_dir, "knowledge_context.md", file);
        }
        "review_codebase_context" => {
            return read_inline_review(&change_dir, "codebase_context.md", file);
        }
        "review_gap_codebase_spec" => {
            return read_inline_review(&change_dir, "gap_codebase_spec.md", file);
        }
        "review_gap_codebase_knowledge" => {
            return read_inline_review(&change_dir, "gap_codebase_knowledge.md", file);
        }
        "review_gap_spec_knowledge" => {
            return read_inline_review(&change_dir, "gap_spec_knowledge.md", file);
        }
        "review_reference_context" => {
            return read_inline_review(&change_dir, "reference_context.md", file);
        }
        "review_pre_clarifications" => {
            return read_inline_review(&change_dir, "pre_clarifications.md", file);
        }
        "review_context_clarifications" => {
            return read_inline_review(&change_dir, "pre_clarifications.md", file);
        }
        "review_spec_clarifications" => {
            return read_inline_review(&change_dir, "spec_clarifications.md", file);
        }
        "review_proposal" => {
            return read_inline_review(&change_dir, "proposal.md", file);
        }
        // Implementation diff artifacts
        "impl" => change_dir.join("impl.md"),
        other_name if other_name.starts_with("impl:") => {
            let task_id = other_name.strip_prefix("impl:").unwrap();
            change_dir.join(format!("impl_{}.md", task_id))
        }
        // Implementation reviews — inline in impl_{task_id}.md
        "review_impl" => {
            return read_inline_review(&change_dir, "impl.md", file);
        }
        other_name if other_name.starts_with("review_impl:") => {
            let task_id = other_name.strip_prefix("review_impl:").unwrap();
            return read_inline_review(&change_dir, &format!("impl_{}.md", task_id), file);
        }
        other_name if other_name.starts_with("review_spec_") => {
            // Inline review: extract # Reviews section from specs/{spec_id}.md
            let spec_id = other_name.strip_prefix("review_spec_").unwrap();
            let spec_file = format!("specs/{}.md", spec_id);
            return read_inline_review(&change_dir, &spec_file, file);
        }
        spec_name => {
            // Try as a spec file
            let spec_path = change_dir.join("specs").join(format!("{}.md", spec_name));
            if spec_path.exists() {
                spec_path
            } else {
                // Maybe they included .md already
                let spec_path_with_ext = change_dir.join("specs").join(spec_name);
                if spec_path_with_ext.exists() {
                    spec_path_with_ext
                } else {
                    anyhow::bail!(
                        "File not found: '{}'. Use 'proposal', 'tasks', or a spec name.",
                        file
                    );
                }
            }
        }
    };

    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    let content = std::fs::read_to_string(&file_path)?;

    Ok(format!(
        "# File: {}\n\n{}",
        file_path
            .strip_prefix(&change_dir)
            .unwrap_or(&file_path)
            .display(),
        content
    ))
}

/// Read inline review (# Reviews section) from an artifact file.
///
/// Falls back to legacy separate review file if the artifact has no inline review.
fn read_inline_review(change_dir: &Path, artifact_file: &str, scope: &str) -> Result<String> {
    use crate::tools::review_helpers::extract_review_section;

    let artifact_path = change_dir.join(artifact_file);
    if !artifact_path.exists() {
        // Fallback: try legacy separate review file
        let legacy_path = change_dir.join(format!("{}.md", scope));
        if legacy_path.exists() {
            let content = std::fs::read_to_string(&legacy_path)?;
            return Ok(format!("# File: {}.md\n\n{}", scope, content));
        }
        anyhow::bail!(
            "Artifact '{}' not found and no legacy review file '{}.md' exists",
            artifact_file,
            scope
        );
    }

    let content = std::fs::read_to_string(&artifact_path)?;
    if let Some(review_section) = extract_review_section(&content) {
        Ok(format!(
            "# File: {} (review section)\n\n{}",
            artifact_file, review_section
        ))
    } else {
        // No inline review section — check legacy separate file
        let legacy_path = change_dir.join(format!("{}.md", scope));
        if legacy_path.exists() {
            let legacy_content = std::fs::read_to_string(&legacy_path)?;
            return Ok(format!("# File: {}.md\n\n{}", scope, legacy_content));
        }
        anyhow::bail!(
            "No review found in '{}' and no legacy file '{}.md'",
            artifact_file,
            scope
        );
    }
}

/// List spec files in a change directory
/// If spec_id is provided, only list its dependency specs from proposal.md
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/file_service.md#source
pub fn list_specs(change_id: &str, spec_id: Option<&str>, project_root: &Path) -> Result<String> {
    // Check change directory exists
    let change_dir = project_root.join(".aw/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found.", change_id);
    }

    // If spec_id is provided, find its dependencies from proposal.md
    let filter_specs: Option<Vec<String>> = if let Some(sid) = spec_id {
        let proposal_path = change_dir.join("proposal.md");
        if proposal_path.exists() {
            let content = std::fs::read_to_string(&proposal_path)?;
            Some(extract_spec_dependencies(&content, sid))
        } else {
            Some(vec![])
        }
    } else {
        None
    };

    let specs_dir = change_dir.join("specs");
    if !specs_dir.exists() {
        return Ok("No specs directory found.".to_string());
    }

    let mut specs = Vec::new();
    for entry in std::fs::read_dir(&specs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "md") {
            if let Some(name) = path.file_stem() {
                let name_str = name.to_string_lossy().to_string();
                // Skip skeleton files
                if name_str.starts_with('_') {
                    continue;
                }
                // Filter by dependencies if spec_id was provided
                if let Some(ref filter) = filter_specs {
                    if filter.contains(&name_str) {
                        specs.push(name_str);
                    }
                } else {
                    specs.push(name_str);
                }
            }
        }
    }

    specs.sort();

    let title = if let Some(sid) = spec_id {
        format!(
            "# Dependencies for spec '{}' in change '{}'\n\n",
            sid, change_id
        )
    } else {
        format!("# Specs for change '{}'\n\n", change_id)
    };

    if specs.is_empty() {
        if spec_id.is_some() {
            return Ok(format!("{}No dependency specs found.", title));
        }
        return Ok("No spec files found.".to_string());
    }

    let mut result = title;
    for spec in &specs {
        result.push_str(&format!("- {}\n", spec));
    }
    result.push_str(&format!("\nTotal: {} spec(s)", specs.len()));

    Ok(result)
}

// ============================================================================
// Scope-prefixed readers
// ============================================================================

/// Read a main spec via `main_spec:group/id` or `main_spec:group/subdir/id` prefix.
///
/// Loads [`SddConfig`] to resolve the group directory via [`resolve_spec_dir`].
/// Falls back to the classic `crates/ → projects/ → root` probe when config is
/// unavailable or `specs.scopes` is empty.
///
/// The `id` component may include subdirectory segments (e.g. `logic/state-machine`).
/// Each segment is validated to prevent path traversal. Validation of individual
/// components via [`validate_path_component`] remains intact (REQ-8).
fn read_main_spec_scoped(ref_str: &str, project_root: &Path) -> Result<String> {
    // Parse "group/id" — group is the first segment; id is the remainder (may contain /)
    let (group, id) = ref_str.split_once('/').ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid main_spec reference '{}'. Expected format: 'main_spec:group/id'",
            ref_str
        )
    })?;

    // Validate group (single component — no slashes allowed)
    validate_path_component(group, "spec_group")?;
    // Validate id (may be a sub-path like "logic/state-machine" — each component validated)
    validate_spec_path(id, "spec_id")?;

    let specs_base = workspace::tech_design_path(project_root);
    let spec_file = format!("{}.md", id);

    // Attempt config-driven resolution; fall back gracefully on any error.
    let config = SddConfig::load(project_root).ok();
    let empty_scopes = std::collections::HashMap::new();
    let scopes = config
        .as_ref()
        .map(|c| &c.specs.scopes)
        .unwrap_or(&empty_scopes);

    let group_dir = resolve_spec_dir_for_root(group, project_root, &specs_base, scopes)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Spec group '{}' not found. \
             Searched configured scopes, crates/{0}, projects/{0}, and root.",
                group
            )
        })?;

    let spec_path = group_dir.join(&spec_file);
    if !spec_path.exists() {
        anyhow::bail!(
            "Spec file '{}' not found in group '{}' (resolved to: {})",
            spec_file,
            group,
            group_dir.display()
        );
    }

    let content = std::fs::read_to_string(&spec_path)?;
    Ok(format!(
        "# Main Spec: {}/{}\n\nPath: `.aw/tech-design/{}/{}.md`\n\n---\n\n{}",
        group, id, group, id, content
    ))
}

/// Validate a spec sub-path (may contain `/` separators for nested specs).
///
/// Rejects traversal sequences (`..`) and validates every path component using
/// [`validate_path_component`]. This preserves the traversal protection
/// required by REQ-8 while allowing multi-level spec paths.
fn validate_spec_path(path: &str, field_name: &str) -> Result<()> {
    if path.is_empty() {
        anyhow::bail!("{} must not be empty", field_name);
    }
    if path.starts_with('/') || path.starts_with('\\') {
        anyhow::bail!("{} must not be an absolute path", field_name);
    }
    for component in path.split('/') {
        validate_path_component(component, field_name)?;
    }
    Ok(())
}

/// Route `list:*` scope prefixes
fn read_list_scoped(scope: &str, change_id: &str, project_root: &Path) -> Result<String> {
    match scope {
        "main_specs" => list_main_specs_scoped(None, project_root),
        s if s.starts_with("main_specs:") => {
            let group = s.strip_prefix("main_specs:").unwrap();
            list_main_specs_scoped(Some(group), project_root)
        }
        "specs" => list_specs(change_id, None, project_root),
        s if s.starts_with("specs:") => {
            let spec_id = s.strip_prefix("specs:").unwrap();
            list_specs(change_id, Some(spec_id), project_root)
        }
        _ => anyhow::bail!(
            "Unknown list scope: '{}'. Use list:main_specs or list:specs.",
            scope
        ),
    }
}

/// List main specs, optionally filtered by group
fn list_main_specs_scoped(group: Option<&str>, project_root: &Path) -> Result<String> {
    let specs_dir = workspace::tech_design_path(project_root);
    let project_td_paths = workspace::project_tech_design_paths(project_root);
    if !specs_dir.exists() && project_td_paths.iter().all(|(_, path)| !path.exists()) {
        anyhow::bail!("Main specs directory not found: {}", specs_dir.display());
    }

    let mut output = String::new();
    output.push_str("# Main Specs\n\n");

    if let Some(group) = group {
        validate_path_component(group, "spec_group")?;
        let config = SddConfig::load(project_root).ok();
        let empty_scopes = std::collections::HashMap::new();
        let scopes = config
            .as_ref()
            .map(|c| &c.specs.scopes)
            .unwrap_or(&empty_scopes);
        let group_dir = resolve_spec_dir_for_root(group, project_root, &specs_dir, scopes)
            .unwrap_or_else(|| specs_dir.join(group));
        if !group_dir.exists() {
            anyhow::bail!(
                "Spec group '{}' not found in tech-design root {}",
                group,
                specs_dir.display()
            );
        }
        output.push_str(&format!("## {}\n\n", group));
        list_specs_in_dir(&group_dir, &mut output)?;
    } else {
        let mut entries: Vec<(String, PathBuf)> = Vec::new();
        if specs_dir.exists() {
            entries.extend(
                std::fs::read_dir(&specs_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .map(|e| (e.file_name().to_string_lossy().to_string(), e.path())),
            );
        }
        entries.extend(
            project_td_paths
                .into_iter()
                .filter(|(_, path)| path.exists()),
        );
        entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        entries.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);

        for (group_name, path) in entries {
            if group_name.starts_with('.') {
                continue;
            }
            output.push_str(&format!("## {}\n\n", group_name));
            list_specs_in_dir(&path, &mut output)?;
        }
    }

    Ok(output)
}

/// List spec files in a directory with titles
fn list_specs_in_dir(dir: &Path, output: &mut String) -> Result<()> {
    let mut files: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        .collect();
    files.sort_by_key(|e| e.path());

    if files.is_empty() {
        output.push_str("(no specs)\n\n");
        return Ok(());
    }

    let mut spec_count = 0;
    for entry in files {
        let path = entry.path();
        let filename = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        if filename.eq_ignore_ascii_case("readme") || filename.eq_ignore_ascii_case("changelog") {
            continue;
        }

        let title = extract_spec_title(&path).unwrap_or_else(|| filename.clone());
        output.push_str(&format!("- `{}` - {}\n", filename, title));
        spec_count += 1;
    }

    if spec_count == 0 {
        output.push_str("(no specs)\n");
    }
    output.push('\n');

    Ok(())
}

/// Extract title from spec frontmatter or first heading
fn extract_spec_title(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;

    if !content.starts_with("---") {
        for line in content.lines() {
            if line.starts_with("# ") {
                return Some(line[2..].trim().to_string());
            }
        }
        return None;
    }

    let rest = &content[3..];
    let end_idx = rest.find("---")?;
    let frontmatter = &rest[..end_idx];

    for line in frontmatter.lines() {
        let line = line.trim();
        if line.starts_with("title:") {
            let value = line[6..].trim();
            let title = value.trim_matches('"').trim_matches('\'');
            return Some(title.to_string());
        }
    }

    None
}

/// Read all requirements (proposal + tasks + specs) for a change
fn read_all_requirements(change_id: &str, project_root: &Path) -> Result<String> {
    let change_dir = project_root.join(".aw/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found at {}",
            change_id,
            change_dir.display()
        );
    }

    let mut output = String::new();
    output.push_str(&format!("# Requirements for Change: {}\n\n", change_id));

    // Read proposal.md (required)
    let proposal_path = change_dir.join("proposal.md");
    if !proposal_path.exists() {
        anyhow::bail!("proposal.md not found for change '{}'", change_id);
    }
    let proposal_content = std::fs::read_to_string(&proposal_path)?;
    output.push_str("## Proposal\n\n");
    output.push_str(&proposal_content);
    output.push_str("\n\n---\n\n");

    // Read tasks.md (required)
    let tasks_path = change_dir.join("tasks.md");
    if !tasks_path.exists() {
        anyhow::bail!("tasks.md not found for change '{}'", change_id);
    }
    let tasks_content = std::fs::read_to_string(&tasks_path)?;
    output.push_str("## Tasks\n\n");
    output.push_str(&tasks_content);
    output.push_str("\n\n---\n\n");

    // Read all specs (optional)
    let specs_dir = change_dir.join("specs");
    let mut spec_count = 0;
    if specs_dir.exists() {
        let mut spec_files = Vec::new();
        for entry in std::fs::read_dir(&specs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(name) = path.file_stem() {
                    let name_str = name.to_string_lossy();
                    if !name_str.starts_with('_') {
                        spec_files.push((name_str.to_string(), path));
                    }
                }
            }
        }

        spec_files.sort_by(|a, b| a.0.cmp(&b.0));

        if !spec_files.is_empty() {
            output.push_str("## Specifications\n\n");
            for (name, path) in spec_files {
                let spec_content = std::fs::read_to_string(&path)?;
                output.push_str(&format!("### Spec: {}\n\n", name));
                output.push_str(&spec_content);
                output.push_str("\n\n");
                spec_count += 1;
            }
            output.push_str("---\n\n");
        }
    }

    output.push_str(&format!(
        "**Total**: 1 proposal, 1 tasks file, {} specification(s)\n",
        spec_count
    ));

    Ok(output)
}

/// Validate a path component (spec_group or spec_id) to prevent traversal
fn validate_path_component(value: &str, field_name: &str) -> Result<()> {
    if value.contains('/') || value.contains('\\') || value.contains("..") {
        anyhow::bail!("{} contains invalid path characters", field_name);
    }
    let valid = !value.is_empty()
        && value
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_lowercase())
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid {
        anyhow::bail!(
            "{} must start with lowercase letter and contain only lowercase alphanumeric with hyphens",
            field_name
        );
    }
    Ok(())
}

/// Extract dependency spec IDs for a given spec from proposal content
/// Parses YAML frontmatter format:
/// ```yaml
/// affected_specs:
///   - id: spec-a
///     depends: []
///   - id: spec-b
///     depends: [spec-a]
/// ```
fn extract_spec_dependencies(content: &str, spec_id: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_frontmatter = false;
    let mut found_spec = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track frontmatter boundaries
        if trimmed == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
                continue;
            } else {
                break; // End of frontmatter
            }
        }

        if !in_frontmatter {
            continue;
        }

        // Look for "- id: spec_id"
        if trimmed.starts_with("- id:") {
            let id = trimmed.strip_prefix("- id:").unwrap().trim();
            found_spec = id == spec_id;
            continue;
        }

        // If we found our spec, look for depends line
        if found_spec && trimmed.starts_with("depends:") {
            let depends_str = trimmed.strip_prefix("depends:").unwrap().trim();
            // Parse [dep1, dep2] format
            if depends_str.starts_with('[') && depends_str.ends_with(']') {
                let inner = &depends_str[1..depends_str.len() - 1];
                for dep in inner.split(',') {
                    let dep = dep.trim();
                    if !dep.is_empty() {
                        deps.push(dep.to_string());
                    }
                }
            }
            found_spec = false; // Done with this spec
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory with proposal
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::write(
            change_dir.join("proposal.md"),
            "# Test Proposal\n\nThis is a test.",
        )
        .unwrap();

        let result = read_file("test-change", "proposal", project_root).unwrap();
        assert!(result.contains("# Test Proposal"));
        assert!(result.contains("This is a test"));
    }

    #[test]
    fn test_read_spec() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory with spec
        let specs_dir = project_root.join(".aw/changes/test-change/specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("my-feature.md"),
            "# My Feature Spec\n\nRequirements here.",
        )
        .unwrap();

        let result = read_file("test-change", "my-feature", project_root).unwrap();
        assert!(result.contains("# My Feature Spec"));
    }

    #[test]
    fn test_list_specs() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory with specs
        let specs_dir = project_root.join(".aw/changes/test-change/specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("spec-a.md"), "# Spec A").unwrap();
        std::fs::write(specs_dir.join("spec-b.md"), "# Spec B").unwrap();
        std::fs::write(specs_dir.join("_skeleton.md"), "# Skeleton").unwrap();

        let result = list_specs("test-change", None, project_root).unwrap();
        assert!(result.contains("spec-a"));
        assert!(result.contains("spec-b"));
        assert!(!result.contains("_skeleton"));
        assert!(result.contains("Total: 2 spec(s)"));
    }

    #[test]
    fn test_read_review_impl_task_scoped() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        // Inline review: impl_1.1.md with # Reviews section
        std::fs::write(
            change_dir.join("impl_1.1.md"),
            "---\nreview_verdict: REVIEWED\n---\n# Implementation Diff — Task 1.1\n\n```diff\n+fn a() {}\n```\n\n# Reviews\n\n## Review (Iteration 1)\n\nNeeds fixes.\n",
        )
        .unwrap();

        let result = read_file("test-change", "review_impl:1.1", project_root).unwrap();
        assert!(result.contains("# Reviews"));
        assert!(result.contains("Needs fixes."));
    }

    #[test]
    fn test_read_review_impl_global() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        // Inline review: impl.md with # Reviews section
        std::fs::write(
            change_dir.join("impl.md"),
            "# Implementation Diff\n\n```diff\n+fn b() {}\n```\n\n# Reviews\n\n## Review (Iteration 1)\n\nGlobal review.\n",
        )
        .unwrap();

        let result = read_file("test-change", "review_impl", project_root).unwrap();
        assert!(result.contains("# Reviews"));
        assert!(result.contains("Global review."));
    }

    // TC_file_service_config — REQ-4: read_main_spec_scoped reads from configured path
    #[test]
    fn test_read_main_spec_with_config_scopes() {
        let temp = TempDir::new().unwrap();
        let project_root = temp.path();

        // Create the spec file at the config-driven path: projects/agentic-workflow/tech-design/core/logic/
        let spec_dir = project_root.join("projects/agentic-workflow/tech-design/core/logic");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("state-machine.md"),
            "# State Machine\n\nTransitions here.\n",
        )
        .unwrap();

        // Write config.toml with [specs.scopes] mapping sdd → crates
        let config_dir = project_root.join("cclab");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            "[specs.scopes]\nsdd = \"crates\"\n",
        )
        .unwrap();

        let result = read_file(
            "any-change",
            "main_spec:sdd/logic/state-machine",
            project_root,
        )
        .unwrap();
        assert!(
            result.contains("State Machine"),
            "should contain spec heading"
        );
        assert!(
            result.contains("Transitions here."),
            "should contain spec content"
        );
    }

    #[test]
    fn test_read_main_spec_with_project_td_path() {
        let temp = TempDir::new().unwrap();
        let project_root = temp.path();

        let spec_dir = project_root.join("projects/agentic-workflow/tech-design/logic");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("state-machine.md"),
            "# State Machine\n\nConfigured TD path.\n",
        )
        .unwrap();
        std::fs::create_dir_all(project_root.join(".aw")).unwrap();
        std::fs::write(
            project_root.join(".aw/config.toml"),
            r#"
[agentic_workflow.tech_design_platform]
path = ".aw/tech-design"

[[projects]]
name = "agentic-workflow"
path = "projects/agentic-workflow"
td_path = "projects/agentic-workflow/tech-design"
"#,
        )
        .unwrap();

        let result = read_file(
            "any-change",
            "main_spec:sdd/logic/state-machine",
            project_root,
        )
        .unwrap();

        assert!(result.contains("Configured TD path."));
    }

    #[test]
    fn test_list_main_specs_with_project_td_path() {
        let temp = TempDir::new().unwrap();
        let project_root = temp.path();

        let spec_dir = project_root.join("projects/agentic-workflow/tech-design");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join("state-machine.md"), "# State Machine\n").unwrap();
        std::fs::create_dir_all(project_root.join(".aw")).unwrap();
        std::fs::write(
            project_root.join(".aw/config.toml"),
            r#"
[agentic_workflow.tech_design_platform]
path = ".aw/tech-design"

[[projects]]
name = "agentic-workflow"
path = "projects/agentic-workflow"
td_path = "projects/agentic-workflow/tech-design"
"#,
        )
        .unwrap();

        let result = read_file("any-change", "list:main_specs:sdd", project_root).unwrap();

        assert!(result.contains("state-machine"));
    }

    // TC_file_service_fallback — REQ-4: read_main_spec_scoped falls back when no config
    #[test]
    fn test_read_main_spec_fallback_no_config() {
        let temp = TempDir::new().unwrap();
        let project_root = temp.path();

        // Spec lives under crates/ (classic fallback path); no config.toml
        let spec_dir = project_root.join("projects/agentic-workflow/tech-design/core");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("workflow.md"),
            "# Workflow Spec\n\nContent.\n",
        )
        .unwrap();

        let result = read_file("any-change", "main_spec:sdd/workflow", project_root).unwrap();
        assert!(result.contains("Workflow Spec"));
    }

    #[test]
    fn test_read_main_spec_subpath() {
        // Spec with a nested path like "logic/state-machine"
        let temp = TempDir::new().unwrap();
        let project_root = temp.path();

        let spec_dir = project_root.join("projects/agentic-workflow/tech-design/core/logic");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join("state-machine.md"), "# State Machine\n").unwrap();

        // Write config.toml so sdd resolves to crates/
        let config_dir = project_root.join("cclab");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            "[specs.scopes]\nsdd = \"crates\"\n",
        )
        .unwrap();

        let result = read_file(
            "any-change",
            "main_spec:sdd/logic/state-machine",
            project_root,
        )
        .unwrap();
        assert!(result.contains("State Machine"));
    }

    #[test]
    fn test_read_main_spec_traversal_rejected() {
        let temp = TempDir::new().unwrap();
        let project_root = temp.path();
        std::fs::create_dir_all(project_root.join(".aw/tech-design")).unwrap();

        // Attempt path traversal via group name
        let err = read_file("any-change", "main_spec:../etc/passwd", project_root).unwrap_err();
        assert!(err.to_string().contains("invalid") || err.to_string().contains("path"));
    }

    #[test]
    fn test_list_specs_with_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory with specs and proposal
        let change_dir = project_root.join(".aw/changes/test-change");
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("spec-a.md"), "# Spec A").unwrap();
        std::fs::write(specs_dir.join("spec-b.md"), "# Spec B").unwrap();
        std::fs::write(specs_dir.join("spec-c.md"), "# Spec C").unwrap();

        // Create proposal with YAML frontmatter affected_specs
        let proposal = r#"---
id: test-change
affected_specs:
  - id: spec-a
    path: specs/spec-a.md
    depends: []
  - id: spec-b
    path: specs/spec-b.md
    depends: [spec-a]
  - id: spec-c
    path: specs/spec-c.md
    depends: [spec-a, spec-b]
---
# Proposal
"#;
        std::fs::write(change_dir.join("proposal.md"), proposal).unwrap();

        // List dependencies for spec-c
        let result = list_specs("test-change", Some("spec-c"), project_root).unwrap();
        assert!(result.contains("- spec-a"));
        assert!(result.contains("- spec-b"));
        assert!(!result.contains("- spec-c")); // spec-c itself should not be in its own deps
        assert!(result.contains("Total: 2 spec(s)"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/file_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete file service module.
```

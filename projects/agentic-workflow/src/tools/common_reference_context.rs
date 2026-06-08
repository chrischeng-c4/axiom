//! Shared helpers for reference context tools.
//!
//! Provides group sub-state resolution, group completion tracking,
//! and spec markdown rendering used across create/review/revise.

use crate::state::StateManager;
use crate::workflow::helpers;
use crate::Result;
use serde_json::Value;
use std::path::Path;

use super::workflow_common;

// ─── Group Sub-State ─────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_reference_context.md#schema
// CODEGEN-BEGIN
/// Per-group sub-state within the reference context lifecycle.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_reference_context.md#schema

pub enum GroupSubState {
    /// No artifact exists — needs creation.
    Create { group_id: String },
    /// Artifact exists with section-loop in progress — fill next section.
    CreateSection { group_id: String, section: String },
    /// Artifact exists, no review verdict — needs review.
    Review { group_id: String },
    /// Reviewed but not approved, revision count < 1 — needs revision.
    Revise { group_id: String },
    /// All groups approved.
    AllDone,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_reference_context/helpers.md#source
// CODEGEN-BEGIN
/// Resolve the next group's sub-state for reference context processing.
///
/// Examines remaining groups (not yet in `groups_progress.reference_context`)
/// and returns the sub-state for the first remaining group.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_reference_context/helpers.md#source
pub fn resolve_next_group(change_dir: &Path) -> Result<GroupSubState> {
    let groups_dir = change_dir.join("groups");
    let all_groups = workflow_common::list_group_ids(&groups_dir)?;

    if all_groups.is_empty() {
        anyhow::bail!("No groups found in {}/groups/", change_dir.display());
    }

    let sm = StateManager::load(change_dir)?;
    let done: Vec<String> = Vec::new();

    let remaining: Vec<&String> = all_groups.iter().filter(|g| !done.contains(g)).collect();

    if remaining.is_empty() {
        return Ok(GroupSubState::AllDone);
    }

    let group_id = remaining[0].clone();
    let artifact_path = groups_dir.join(&group_id).join("reference_context.md");

    if !artifact_path.exists() {
        return Ok(GroupSubState::Create { group_id });
    }

    // Artifact exists — check section-loop sub-state first
    let content = std::fs::read_to_string(&artifact_path).unwrap_or_default();
    let fill_sections = read_fill_sections(&content);

    if !fill_sections.is_empty() && !is_create_complete(&content) {
        // Section-loop in progress — find next unfilled section
        if let Some(next_section) = resolve_next_section(&content) {
            return Ok(GroupSubState::CreateSection {
                group_id,
                section: next_section,
            });
        }
        // All sections filled but create_complete not set — will be handled
        // by the workflow (prune + mark complete)
    }

    // Artifact exists — check revision count for auto-approve (before verdict)
    let rev_key = format!("ref_ctx:{}", group_id);
    let rev_count = sm.revision_count(&rev_key);

    if rev_count >= 1 {
        // Auto-approve: revision limit reached
        return Ok(GroupSubState::AllDone);
    }

    // Check review verdict
    let has_verdict = helpers::has_review_verdict_at(&artifact_path);

    if !has_verdict {
        return Ok(GroupSubState::Review { group_id });
    }

    // Has verdict — check if approved
    let verdict = helpers::extract_verdict_or_approved(&artifact_path);
    match verdict.as_deref() {
        Some("APPROVED") | Some("PASS") => Ok(GroupSubState::AllDone),
        _ => Ok(GroupSubState::Revise { group_id }),
    }
}

// ─── Group Completion ────────────────────────────────────────────────────────

/// Mark a group as done. Group tracking removed — this is now a no-op.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_reference_context/helpers.md#source
pub fn mark_group_done(_change_dir: &Path, _group_id: &str) -> Result<()> {
    Ok(())
}

// ─── Section-Loop Helpers ────────────────────────────────────────────────────

// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R1

/// Read `fill_sections` from reference_context.md frontmatter (YAML list).
///
/// Mirrors `common_change_spec::read_fill_sections` but for reference context.
pub fn read_fill_sections(content: &str) -> Vec<String> {
    parse_yaml_list_field(content, "fill_sections")
}

/// Read `filled_sections` from reference_context.md frontmatter (YAML list).
///
/// Mirrors `common_change_spec::read_filled_sections` but for reference context.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_reference_context/helpers.md#source
pub fn read_filled_sections(content: &str) -> Vec<String> {
    parse_yaml_list_field(content, "filled_sections")
}

/// Read `create_complete` flag from reference_context.md frontmatter.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_reference_context/helpers.md#source
pub fn is_create_complete(content: &str) -> bool {
    if !content.starts_with("---\n") {
        return false;
    }
    let closing = match content[4..].find("\n---") {
        Some(pos) => 4 + pos,
        None => return false,
    };
    let fm = &content[4..closing];
    fm.lines().any(|l| l.trim() == "create_complete: true")
}

/// Resolve the next unfilled section from fill_sections, skipping filled ones.
///
/// Returns `None` if all sections are filled or fill_sections is empty.
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R4
pub fn resolve_next_section(content: &str) -> Option<String> {
    let fill = read_fill_sections(content);
    let filled = read_filled_sections(content);
    fill.into_iter().find(|s| !filled.contains(s))
}

/// Build a summary of already-filled sections for cross-section context.
///
/// Extracts content under each filled section's heading and produces a
/// condensed block for the agent prompt.
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R4
pub fn build_already_filled_summary(content: &str, filled_sections: &[String]) -> String {
    use crate::models::reference_context_sections::section_to_heading;

    if filled_sections.is_empty() {
        return String::new();
    }

    let mut summary = String::from("## Already Filled Sections\n\n");
    summary.push_str("The following sections have already been written. Use this context to avoid duplication and maintain consistency.\n\n");

    for section in filled_sections {
        let heading = section_to_heading(section);
        if let Some(section_content) = extract_section_content(content, &heading) {
            let trimmed = section_content.trim();
            if !trimmed.is_empty() {
                summary.push_str(&format!("### {}\n", section));
                // Truncate to first 20 lines for summary
                let lines: Vec<&str> = trimmed.lines().take(20).collect();
                summary.push_str(&lines.join("\n"));
                if trimmed.lines().count() > 20 {
                    summary.push_str("\n... (truncated)");
                }
                summary.push_str("\n\n");
            }
        }
    }

    summary
}

/// Extract content under a specific heading from markdown.
fn extract_section_content(content: &str, heading: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let heading_level = heading.chars().take_while(|c| *c == '#').count();
    let mut in_section = false;
    let mut section_lines = Vec::new();

    for line in &lines {
        if line.starts_with('#') {
            let level = line.chars().take_while(|c| *c == '#').count();
            if line.trim().eq_ignore_ascii_case(heading) {
                in_section = true;
                continue;
            } else if in_section && level <= heading_level {
                break;
            }
        }
        if in_section {
            section_lines.push(*line);
        }
    }

    if section_lines.is_empty() {
        None
    } else {
        Some(section_lines.join("\n"))
    }
}

/// Generate a skeleton reference_context.md with section-loop frontmatter.
///
/// Creates a file with fill_sections set to the provided sections list,
/// empty filled_sections, and create_complete: false.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_reference_context/helpers.md#source
pub fn generate_section_loop_skeleton(
    change_id: &str,
    group_id: &str,
    sections: &[&str],
) -> String {
    let fill_sections_str = sections
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "---\nchange: {change_id}\ngroup: {group_id}\nfill_sections: [{fill_sections_str}]\nfilled_sections: []\ncreate_complete: false\n---\n\n# Reference Context\n\n{sections_body}",
        sections_body = sections
            .iter()
            .map(|s| {
                let heading = crate::models::reference_context_sections::section_to_heading(s);
                format!("{heading}\n\n<!-- TODO: Fill this section -->\n")
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// Replace a section's content in reference_context.md.
///
/// Finds the heading matching `section` and replaces everything up to the
/// next heading of the same or higher level. If the heading is not found,
/// inserts before `# Reviews` or appends at end.
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R5
pub fn replace_ref_ctx_section(content: &str, section: &str, new_content: &str) -> String {
    let heading = crate::models::reference_context_sections::section_to_heading(section);
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut in_target = false;
    let mut target_level = 0;
    let mut found = false;

    for line in &lines {
        if line.starts_with('#') {
            let level = line.chars().take_while(|c| *c == '#').count();
            if line.trim().eq_ignore_ascii_case(&heading) {
                in_target = true;
                found = true;
                target_level = level;
                result.push(line.to_string());
                result.push(String::new());
                for new_line in new_content.lines() {
                    result.push(new_line.to_string());
                }
                continue;
            } else if in_target && level <= target_level {
                in_target = false;
            }
        }
        if !in_target {
            result.push(line.to_string());
        }
    }

    if !found {
        result.push(String::new());
        result.push(heading);
        result.push(String::new());
        for l in new_content.lines() {
            result.push(l.to_string());
        }
        result.push(String::new());
    }

    result.join("\n")
}

/// Parse a YAML list field from frontmatter.
///
/// Handles both inline `[a, b, c]` and block list formats.
fn parse_yaml_list_field(content: &str, field_name: &str) -> Vec<String> {
    if !content.starts_with("---\n") {
        return vec![];
    }
    let closing = match content[4..].find("\n---") {
        Some(pos) => 4 + pos,
        None => return vec![],
    };
    let fm = &content[4..closing];

    let prefix = format!("{}:", field_name);
    let mut in_field = false;
    let mut items = Vec::new();

    for line in fm.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&prefix) {
            in_field = true;
            let after = trimmed[prefix.len()..].trim();
            if after.starts_with('[') && after.ends_with(']') {
                let inner = &after[1..after.len() - 1];
                for item in inner.split(',') {
                    let s = item.trim().trim_matches('"').trim_matches('\'');
                    if !s.is_empty() {
                        items.push(s.to_string());
                    }
                }
                return items;
            }
            continue;
        }
        if in_field {
            if trimmed.starts_with("- ") {
                let item = trimmed
                    .trim_start_matches("- ")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                items.push(item.to_string());
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
        }
    }
    items
}

// ─── Spec Rendering ──────────────────────────────────────────────────────────

/// Check if a group's reference_context.md was written via the artifact CLI.
///
/// The artifact CLI always writes `written_by: artifact_cli` in frontmatter.
/// If the agent wrote the file directly (bypassing the CLI), this marker is absent.
pub fn verify_artifact_written(change_dir: &Path, group_id: &str) -> bool {
    let path = change_dir
        .join("groups")
        .join(group_id)
        .join("reference_context.md");
    if !path.exists() {
        return false;
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    content.contains("written_by: artifact_cli")
}

/// Extract spec references from agent's prose content (mainthread fallback).
///
/// Parses the prose looking for spec names, paths, and relevance levels.
/// Returns a JSON array suitable for `execute_artifact()`.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_reference_context/helpers.md#source
pub fn extract_specs_from_prose(prose: &str) -> Vec<serde_json::Value> {
    use serde_json::json;

    let mut specs = Vec::new();
    let mut current_spec_id = String::new();
    let mut current_relevance = "medium".to_string();
    let mut current_reqs = Vec::new();

    for line in prose.lines() {
        let trimmed = line.trim();

        // Match markdown table rows: | spec_name | group | relevance | requirements |
        if trimmed.starts_with('|')
            && !trimmed.contains("---")
            && !trimmed.to_lowercase().contains("spec")
        {
            let cols: Vec<&str> = trimmed
                .split('|')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if cols.len() >= 3 {
                let spec_id = cols[0]
                    .trim_matches(|c: char| c == '*' || c == '`')
                    .to_string();
                if spec_id.is_empty() || spec_id.to_lowercase() == "spec" {
                    continue;
                }
                let group = cols.get(1).unwrap_or(&"").to_string();
                let relevance = cols
                    .get(2)
                    .map(|r| r.to_lowercase())
                    .unwrap_or_else(|| "medium".to_string());
                let relevance = match relevance.as_str() {
                    r if r.contains("high") => "high",
                    r if r.contains("low") => "low",
                    _ => "medium",
                };
                let reqs: Vec<String> = cols
                    .get(3)
                    .map(|r| {
                        r.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();

                specs.push(json!({
                    "spec_id": spec_id,
                    "spec_group": group,
                    "relevance": relevance,
                    "key_requirements": reqs,
                }));
                continue;
            }
        }

        // Match headers like "### 1. spec_name (HIGH)" or "### spec_name (high)"
        if trimmed.starts_with("###") || trimmed.starts_with("## ") {
            // Save previous spec if any
            if !current_spec_id.is_empty() {
                specs.push(json!({
                    "spec_id": current_spec_id,
                    "spec_group": "",
                    "relevance": current_relevance,
                    "key_requirements": current_reqs,
                }));
                current_reqs = Vec::new();
            }

            // Extract spec name and relevance from header
            let header = trimmed.trim_start_matches('#').trim();
            let header = header
                .trim_start_matches(|c: char| c.is_numeric() || c == '.')
                .trim();

            if header.to_lowercase().contains("(high)") {
                current_relevance = "high".to_string();
            } else if header.to_lowercase().contains("(low)") {
                current_relevance = "low".to_string();
            } else if header.to_lowercase().contains("(medium)") {
                current_relevance = "medium".to_string();
            } else {
                current_relevance = "medium".to_string();
            }

            current_spec_id = header
                .split('(')
                .next()
                .unwrap_or(header)
                .trim()
                .to_string();
            continue;
        }

        // Match requirement lines like "- R1: ..." or "- **R1**: ..."
        if (trimmed.starts_with("- R") || trimmed.starts_with("- **R"))
            && !current_spec_id.is_empty()
        {
            let req = trimmed
                .trim_start_matches("- ")
                .trim_matches('*')
                .to_string();
            current_reqs.push(req);
        }
    }

    // Save last spec
    if !current_spec_id.is_empty() {
        specs.push(json!({
            "spec_id": current_spec_id,
            "spec_group": "",
            "relevance": current_relevance,
            "key_requirements": current_reqs,
        }));
    }

    specs
}

/// Render structured spec references into markdown.
///
/// If `spec_plan` is provided, it is rendered as an additional section
/// after the specs table.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_reference_context/helpers.md#source
pub fn render_specs_markdown(
    change_id: &str,
    group_id: &str,
    date: &str,
    specs: &[Value],
    spec_plan: Option<&Vec<Value>>,
) -> String {
    let mut md = format!(
        "---\nchange: {}\ngroup: {}\ndate: {}\nwritten_by: artifact_cli\n---\n\n# Reference Context\n\n",
        change_id, group_id, date
    );

    md.push_str("| Spec | Group | Relevance | Key Requirements |\n");
    md.push_str("|------|-------|-----------|------------------|\n");

    for spec in specs {
        let spec_id = spec.get("spec_id").and_then(|v| v.as_str()).unwrap_or("?");
        let spec_group = spec
            .get("spec_group")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("-");
        let relevance = spec
            .get("relevance")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");
        let key_reqs: Vec<&str> = spec
            .get("key_requirements")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        let reqs_str = if key_reqs.is_empty() {
            "\u{2014}".to_string()
        } else {
            key_reqs.join(", ")
        };

        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            spec_id, spec_group, relevance, reqs_str
        ));
    }

    md.push('\n');

    // Render spec_plan section if provided
    if let Some(plan) = spec_plan {
        if !plan.is_empty() {
            md.push_str("## Spec Plan\n\n");
            md.push_str("| Spec ID | Action | Main Spec Ref | Sections |\n");
            md.push_str("|---------|--------|---------------|----------|\n");

            for entry in plan {
                let spec_id = entry.get("spec_id").and_then(|v| v.as_str()).unwrap_or("?");
                let action = entry.get("action").and_then(|v| v.as_str()).unwrap_or("?");
                let main_ref = entry
                    .get("main_spec_ref")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let sections: Vec<&str> = entry
                    .get("sections")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();
                let sections_str = if sections.is_empty() {
                    "\u{2014}".to_string()
                } else {
                    sections.join(", ")
                };

                md.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    spec_id, action, main_ref, sections_str
                ));
            }

            md.push('\n');
        }
    }

    md
}

/// Write spec_plan entries to a YAML file in the group directory.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_reference_context/helpers.md#source
pub fn write_spec_plan_yaml(group_dir: &Path, plan_entries: &[Value]) -> crate::Result<()> {
    #[derive(serde::Serialize)]
    struct SpecPlanEntry {
        spec_id: String,
        action: String,
        main_spec_ref: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        sections: Vec<String>,
    }

    let entries: Vec<SpecPlanEntry> = plan_entries
        .iter()
        .map(|v| SpecPlanEntry {
            spec_id: v
                .get("spec_id")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string(),
            action: v
                .get("action")
                .and_then(|s| s.as_str())
                .unwrap_or("create")
                .to_string(),
            main_spec_ref: v
                .get("main_spec_ref")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string(),
            source: v
                .get("source")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string()),
            sections: v
                .get("sections")
                .and_then(|s| s.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect();

    let yaml = serde_yaml::to_string(&entries)?;
    let plan_path = group_dir.join("spec_plan.yaml");
    std::fs::write(&plan_path, yaml)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Set up a project root with `.aw/changes/test/` change dir, a backing
    /// issue, and groups. Returns (TempDir, change_dir).
    fn setup_change(groups: &[&str]) -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join(".aw/changes/test");
        std::fs::create_dir_all(&change_dir).unwrap();
        crate::test_util::write_minimal_issue(tmp.path(), "test");
        let groups_dir = change_dir.join("groups");
        for g in groups {
            std::fs::create_dir_all(groups_dir.join(g)).unwrap();
        }
        (tmp, change_dir)
    }

    /// Set up a project root with `.aw/changes/test/` change dir, a backing
    /// issue, groups, and a specific revision count in state.
    fn setup_change_with_revision(
        groups: &[&str],
        rev_key: &str,
        rev_count: u32,
    ) -> (TempDir, std::path::PathBuf) {
        let (tmp, change_dir) = setup_change(groups);
        let mut sm = crate::state::StateManager::load(&change_dir).unwrap();
        for _ in 0..rev_count {
            sm.increment_revision_count(rev_key);
        }
        sm.save().unwrap();
        (tmp, change_dir)
    }

    fn write_artifact(change_dir: &Path, group_id: &str, content: &str) {
        let path = change_dir
            .join("groups")
            .join(group_id)
            .join("reference_context.md");
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn test_resolve_create_when_no_artifact() {
        let (_tmp, change_dir) = setup_change(&["g1"]);
        let result = resolve_next_group(&change_dir).unwrap();
        assert!(matches!(result, GroupSubState::Create { group_id } if group_id == "g1"));
    }

    #[test]
    fn test_resolve_review_when_artifact_exists_no_verdict() {
        let (_tmp, change_dir) = setup_change(&["g1"]);
        write_artifact(&change_dir, "g1", "---\nchange: test\n---\n# Ref\n");
        let result = resolve_next_group(&change_dir).unwrap();
        assert!(matches!(result, GroupSubState::Review { group_id } if group_id == "g1"));
    }

    #[test]
    fn test_resolve_auto_approve_when_revision_count_reached() {
        // Bug #872: after revise clears verdict, revision_count >= 1 should auto-approve
        let (_tmp, change_dir) = setup_change_with_revision(&["g1"], "ref_ctx:g1", 1);
        // Artifact exists but NO verdict (simulates revise clearing it)
        write_artifact(&change_dir, "g1", "---\nchange: test\n---\n# Ref\n");
        let result = resolve_next_group(&change_dir).unwrap();
        assert!(matches!(result, GroupSubState::AllDone));
    }

    #[test]
    fn test_resolve_revise_when_reviewed_not_approved() {
        let (_tmp, change_dir) = setup_change(&["g1"]);
        write_artifact(
            &change_dir,
            "g1",
            "---\nchange: test\nreview_verdict: REVIEWED\n---\n# Ref\n",
        );
        let result = resolve_next_group(&change_dir).unwrap();
        assert!(matches!(result, GroupSubState::Revise { group_id } if group_id == "g1"));
    }

    #[test]
    #[ignore = "groups_progress removed — test uses obsolete field"]
    fn test_resolve_all_done_when_all_groups_complete() {
        // Intentionally stubbed out after groups removal.
    }

    #[test]
    fn test_resolve_approved_verdict_marks_done() {
        let (_tmp, change_dir) = setup_change(&["g1"]);
        write_artifact(
            &change_dir,
            "g1",
            "---\nchange: test\nreview_verdict: APPROVED\n---\n# Ref\n",
        );
        let result = resolve_next_group(&change_dir).unwrap();
        assert!(matches!(result, GroupSubState::AllDone));
    }

    #[test]
    fn test_verify_artifact_written_via_cli() {
        let (_tmp, change_dir) = setup_change(&["g1"]);
        // Written via artifact CLI (has marker)
        write_artifact(
            &change_dir,
            "g1",
            "---\nchange: test\ngroup: g1\ndate: 2026-03-16\nwritten_by: artifact_cli\n---\n\n# Reference Context\n",
        );
        assert!(verify_artifact_written(&change_dir, "g1"));
    }

    #[test]
    fn test_verify_artifact_written_directly_by_agent() {
        let (_tmp, change_dir) = setup_change(&["g1"]);
        // Written directly by agent (no marker)
        write_artifact(
            &change_dir,
            "g1",
            "---\nchange: test\ngroup: g1\ndate: 2026-03-16\n---\n\n# Reference Context\n\nProse content...\n",
        );
        assert!(!verify_artifact_written(&change_dir, "g1"));
    }

    #[test]
    fn test_verify_artifact_not_written() {
        let (_tmp, change_dir) = setup_change(&["g1"]);
        // No file at all
        assert!(!verify_artifact_written(&change_dir, "g1"));
    }

    #[test]
    fn test_extract_specs_from_prose_table() {
        let prose = "\
---
change: test
group: g1
date: 2026-03-17
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| parser/patterns.md | mamba | high | R1, R2, R3 |
| types/checker.md | mamba | medium | R3 |
";
        let specs = extract_specs_from_prose(prose);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0]["spec_id"], "parser/patterns.md");
        assert_eq!(specs[0]["relevance"], "high");
        assert_eq!(specs[1]["spec_id"], "types/checker.md");
        assert_eq!(specs[1]["relevance"], "medium");
    }

    #[test]
    fn test_extract_specs_from_prose_headers() {
        let prose = "\
### 1. Parser Patterns (HIGH)
- R1: Literal patterns
- R2: Capture patterns

### 2. Type Checker (MEDIUM)
- R3: Flow narrowing
";
        let specs = extract_specs_from_prose(prose);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0]["spec_id"], "Parser Patterns");
        assert_eq!(specs[0]["relevance"], "high");
        assert_eq!(specs[1]["spec_id"], "Type Checker");
        assert_eq!(specs[1]["relevance"], "medium");
    }

    #[test]
    fn test_extract_specs_from_prose_empty() {
        let prose = "Just some random text with no specs.";
        let specs = extract_specs_from_prose(prose);
        assert!(specs.is_empty());
    }
}
// CODEGEN-END

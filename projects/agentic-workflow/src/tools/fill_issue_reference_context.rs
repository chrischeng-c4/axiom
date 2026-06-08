// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/fill_issue_reference_context.md#source
// CODEGEN-BEGIN
//! Section-loop tool for filling issue Reference Context.
//!
//! Mirrors `create_reference_context.rs` section-loop but operates on the
//! temp-backed issue working copy instead of change artifacts.
//!
//! Two tools:
//! - `sdd_workflow_fill_issue_reference_context` — resolve next section, return prompt
//! - `sdd_artifact_fill_issue_reference_context` — write one section, advance filled_sections

use crate::models::reference_context_sections::{
    self, is_valid_section, REFERENCE_CONTEXT_SECTIONS,
};
use crate::tools::{get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

// ─── Tool Definitions ────────────────────────────────────────────────────────

/// MCP tool definition for sdd_workflow_fill_issue_reference_context
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R2
pub fn workflow_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_workflow_fill_issue_reference_context".to_string(),
        description: "Resolve next Reference Context section for an issue (section-loop dispatch)"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "slug"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "slug": {
                    "type": "string",
                    "description": "Issue slug"
                }
            }
        }),
    }
}

/// MCP tool definition for sdd_artifact_fill_issue_reference_context
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R2
pub fn artifact_definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_artifact_fill_issue_reference_context".to_string(),
        description: "Write one Reference Context section into an issue file".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "slug", "section", "content"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "slug": {
                    "type": "string",
                    "description": "Issue slug"
                },
                "section": {
                    "type": "string",
                    "description": "Section name (one of: source_refs, related_specs, reproductions, related_issues, first_fix)"
                },
                "content": {
                    "type": "string",
                    "description": "Section content to write"
                }
            }
        }),
    }
}

// ──�� Workflow ────────────────────────────────────────────────────────────────

/// Execute sdd_workflow_fill_issue_reference_context.
///
/// Reads the issue file, checks fill_sections/filled_sections in the body,
/// and returns a prompt for the next unfilled section — or signals completion.
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R2
pub fn execute_workflow(args: &Value, project_root: &Path) -> Result<String> {
    let slug = get_required_string(args, "slug")?;

    let issue_path = resolve_issue_path(project_root, &slug)?;
    let body = std::fs::read_to_string(&issue_path)?;

    // Parse fill state from the Reference Context section
    let fill_sections = read_issue_fill_sections(&body);
    let filled_sections = read_issue_filled_sections(&body);

    // If no fill_sections found, initialize them
    if fill_sections.is_empty() {
        // Initialize: inject fill_sections into the issue body
        let updated = inject_fill_sections_into_issue(&body, REFERENCE_CONTEXT_SECTIONS);
        std::fs::write(&issue_path, &updated)?;

        // Return prompt for first section
        let first_section = REFERENCE_CONTEXT_SECTIONS[0];
        return build_issue_section_prompt(&slug, first_section, &[], project_root);
    }

    // Find next unfilled section
    let next_section = fill_sections.iter().find(|s| !filled_sections.contains(s));

    if let Some(section) = next_section {
        build_issue_section_prompt(&slug, section, &filled_sections, project_root)
    } else {
        // All sections filled — mark complete
        let result = json!({
            "status": "ok",
            "reference_context_complete": true,
            "message": "All Reference Context sections filled for issue.",
            "slug": slug
        });
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

// ─── Artifact ────────────────────────────────────────────────────────────────

/// Execute sdd_artifact_fill_issue_reference_context.
///
/// Writes one section into the issue's Reference Context, updates
/// filled_sections tracking, and returns next_actions.
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R5
pub fn execute_artifact(args: &Value, project_root: &Path) -> Result<String> {
    let slug = get_required_string(args, "slug")?;
    let section = get_required_string(args, "section")?;
    let content = get_required_string(args, "content")?;

    if !is_valid_section(&section) {
        anyhow::bail!(
            "Invalid reference context section '{}'. Valid: {:?}",
            section,
            REFERENCE_CONTEXT_SECTIONS
        );
    }

    let issue_path = resolve_issue_path(project_root, &slug)?;
    let body = std::fs::read_to_string(&issue_path)?;

    // Replace section content in the Reference Context area
    let updated = replace_issue_ref_ctx_section(&body, &section, &content);

    // Update filled_sections tracking
    let mut filled = read_issue_filled_sections(&updated);
    if !filled.contains(&section) {
        filled.push(section.clone());
    }
    let final_body = update_issue_filled_sections(&updated, &filled);

    std::fs::write(&issue_path, &final_body)?;

    // Check if all sections are now filled
    let fill_sections = read_issue_fill_sections(&final_body);
    let all_filled = !fill_sections.is_empty() && fill_sections.iter().all(|s| filled.contains(s));

    let result = json!({
        "status": "ok",
        "section_filled": section,
        "filled_sections": filled,
        "reference_context_complete": all_filled,
        "slug": slug
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Find an issue file by slug, checking both open/ and closed/ directories.
fn resolve_issue_path(project_root: &Path, slug: &str) -> Result<std::path::PathBuf> {
    let issues_base = crate::shared::workspace::issues_path(project_root);
    for subdir in &["open", "closed"] {
        let candidate = issues_base.join(subdir).join(format!("{}.md", slug));
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    anyhow::bail!(
        "Issue not found: {}.md (searched temp issue store at {})",
        slug,
        issues_base.display()
    )
}

/// Read fill_sections from the issue body.
///
/// Looks for a YAML-like comment in the Reference Context section:
/// `<!-- fill_sections: [source_refs, related_specs, ...] -->`
fn read_issue_fill_sections(body: &str) -> Vec<String> {
    parse_issue_tracking_field(body, "fill_sections")
}

/// Read filled_sections from the issue body.
///
/// Looks for: `<!-- filled_sections: [source_refs, ...] -->`
fn read_issue_filled_sections(body: &str) -> Vec<String> {
    parse_issue_tracking_field(body, "filled_sections")
}

/// Parse a tracking field from HTML comments in the issue body.
fn parse_issue_tracking_field(body: &str, field: &str) -> Vec<String> {
    let prefix = format!("<!-- {}: [", field);
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            if let Some(end) = rest.find(']') {
                let inner = &rest[..end];
                return inner
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}

/// Inject fill_sections tracking into an issue's Reference Context section.
fn inject_fill_sections_into_issue(body: &str, sections: &[&str]) -> String {
    let sections_str = sections.join(", ");
    let tracking = format!(
        "<!-- fill_sections: [{}] -->\n<!-- filled_sections: [] -->",
        sections_str
    );

    // If there's a ## Reference Context section, inject after it
    if let Some(pos) = body.find("## Reference Context") {
        let after_heading = body[pos..]
            .find('\n')
            .map(|p| pos + p + 1)
            .unwrap_or(body.len());
        let mut result = String::new();
        result.push_str(&body[..after_heading]);
        result.push('\n');
        result.push_str(&tracking);
        result.push('\n');

        // Add section headings
        for section in sections {
            let heading = reference_context_sections::section_to_heading(section);
            result.push('\n');
            result.push_str(&heading);
            result.push_str("\n\n<!-- TODO: Fill this section -->\n");
        }

        // Append remaining body content after Reference Context
        // (skip to next ## heading at same or higher level)
        let remaining = &body[after_heading..];
        let next_h2 = remaining.find("\n## ").map(|p| after_heading + p);
        if let Some(h2_pos) = next_h2 {
            result.push_str(&body[h2_pos..]);
        }

        result
    } else {
        // No Reference Context section — append one
        let mut result = body.to_string();
        result.push_str("\n## Reference Context\n\n");
        result.push_str(&tracking);
        result.push('\n');
        for section in sections {
            let heading = reference_context_sections::section_to_heading(section);
            result.push('\n');
            result.push_str(&heading);
            result.push_str("\n\n<!-- TODO: Fill this section -->\n");
        }
        result
    }
}

/// Replace a section's content within the Reference Context area of an issue.
fn replace_issue_ref_ctx_section(body: &str, section: &str, new_content: &str) -> String {
    let heading = reference_context_sections::section_to_heading(section);
    let lines: Vec<&str> = body.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut in_target = false;
    let target_level = heading.chars().take_while(|c| *c == '#').count();
    let mut found = false;

    for line in &lines {
        if line.starts_with('#') {
            let level = line.chars().take_while(|c| *c == '#').count();
            if line.trim().eq_ignore_ascii_case(&heading) {
                in_target = true;
                found = true;
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
        // Insert before the next ## heading or at end
        result.push(String::new());
        result.push(heading);
        result.push(String::new());
        for l in new_content.lines() {
            result.push(l.to_string());
        }
    }

    result.join("\n")
}

/// Update the filled_sections tracking comment in an issue body.
fn update_issue_filled_sections(body: &str, filled: &[String]) -> String {
    let new_comment = format!("<!-- filled_sections: [{}] -->", filled.join(", "));
    let mut result = String::new();
    let mut found = false;

    for line in body.lines() {
        if line.trim().starts_with("<!-- filled_sections:") {
            result.push_str(&new_comment);
            found = true;
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    if !found {
        // Append after fill_sections comment or at end
        result.push_str(&new_comment);
        result.push('\n');
    }

    result
}

/// Build a per-section prompt for issue Reference Context filling.
// @spec projects/agentic-workflow/tech-design/core/logic/reference-context.md#R4
fn build_issue_section_prompt(
    slug: &str,
    section: &str,
    filled_sections: &[String],
    project_root: &Path,
) -> Result<String> {
    let project_path = project_root.display();
    let section_heading = reference_context_sections::section_to_heading(section);

    let section_guidance = match section {
        "source_refs" => "Identify source code files and functions relevant to this issue.",
        "related_specs" => "Find specs under .aw/tech-design/ that relate to this issue. Build a spec table and spec_plan.",
        "reproductions" => "Document reproduction steps for the problem described in the issue.",
        "related_issues" => "List related issues from the temp issue store that touch the same area.",
        "first_fix" => "Analyze the most likely approach to fix/implement this issue.",
        _ => "Fill this section with relevant context.",
    };

    let already_filled = if filled_sections.is_empty() {
        String::new()
    } else {
        format!(
            "## Already Filled Sections\n\nPreviously filled: {}. Review those sections in the issue file for context.\n",
            filled_sections.join(", ")
        )
    };

    let prompt = format!(
        r#"# Task: Fill Section '{section}' — Reference Context for Issue '{slug}'

## Your Task

Fill ONLY the **{section_heading}** section in the issue's Reference Context. Do NOT fill other sections.

## Section Guidance: {section}

{section_guidance}

{already_filled}
## Context

1. Read the temp issue working copy for `{slug}`
2. Explore specs under `{project_path}/.aw/tech-design/`
3. Explore source code as needed

## CLI Commands

```
# Write section via artifact CLI
score artifact fill-issue-reference-context {slug} --section {section} --content "..."
```

Or use the MCP tool:
```json
{{
  "slug": "{slug}",
  "section": "{section}",
  "content": "... your section content ..."
}}
```"#,
    );

    let result = json!({
        "status": "ok",
        "slug": slug,
        "section": section,
        "reference_context_complete": false,
        "prompt": prompt
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_issue(slug: &str, body: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let issues_dir = crate::shared::workspace::issues_path(tmp.path()).join("open");
        std::fs::create_dir_all(&issues_dir).unwrap();
        std::fs::write(issues_dir.join(format!("{}.md", slug)), body).unwrap();
        // Create .aw directory marker
        std::fs::create_dir_all(tmp.path().join(".aw")).unwrap();
        tmp
    }

    // REQ: R2
    #[test]
    fn test_issue_frontmatter_carries_fill_sections() {
        // T7: Verify fill_sections/filled_sections are parsed from issue body
        let body = "---\nid: test-issue\ntitle: Test\nstate: open\nlabels: []\n---\n\n## Problem\n\nA bug.\n\n## Requirements\n\n- R1: Fix it\n\n## Scope\n\n### In-scope\n- crate: sdd\n\n## Reference Context\n\n<!-- fill_sections: [source_refs, related_specs, first_fix] -->\n<!-- filled_sections: [source_refs] -->\n\n## Source References\n\nSome refs here.\n";
        let fill = read_issue_fill_sections(body);
        let filled = read_issue_filled_sections(body);
        assert_eq!(fill, vec!["source_refs", "related_specs", "first_fix"]);
        assert_eq!(filled, vec!["source_refs"]);
    }

    // REQ: R2
    #[test]
    fn test_issue_fill_loop_terminates_on_complete() {
        // T9: All fill_sections in filled_sections → complete
        let tmp = setup_issue(
            "complete-issue",
            "---\nid: complete-issue\ntitle: Test\nstate: open\nlabels: []\n---\n\n## Problem\n\nDone.\n\n## Requirements\n\n- R1: Test\n\n## Scope\n\n### In-scope\n- crate: sdd\n\n## Reference Context\n\n<!-- fill_sections: [source_refs, related_specs] -->\n<!-- filled_sections: [source_refs, related_specs] -->\n\n## Source References\n\nDone.\n\n## Related Specs\n\nDone.\n",
        );

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "slug": "complete-issue"
        });
        let result = execute_workflow(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["reference_context_complete"], true);
    }

    // REQ: R4
    #[test]
    fn test_workflow_returns_section_prompt() {
        let tmp = setup_issue(
            "prompt-issue",
            "---\nid: prompt-issue\ntitle: Test\nstate: open\nlabels: []\n---\n\n## Problem\n\nBug.\n\n## Requirements\n\n- R1: Fix\n\n## Scope\n\n### In-scope\n- crate: sdd\n\n## Reference Context\n\n<!-- fill_sections: [source_refs, related_specs, first_fix] -->\n<!-- filled_sections: [] -->\n",
        );

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "slug": "prompt-issue"
        });
        let result = execute_workflow(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["section"], "source_refs");
        assert_eq!(parsed["reference_context_complete"], false);
        let prompt = parsed["prompt"].as_str().unwrap();
        assert!(prompt.contains("source_refs"));
    }

    // REQ: R5
    #[test]
    fn test_partial_fill_overwrites_not_appends() {
        // T6: crashed agent left partial content, next write overwrites
        let tmp = setup_issue(
            "partial-issue",
            "---\nid: partial-issue\ntitle: Test\nstate: open\nlabels: []\n---\n\n## Problem\n\nBug.\n\n## Requirements\n\n- R1: Fix\n\n## Scope\n\n### In-scope\n- crate: sdd\n\n## Reference Context\n\n<!-- fill_sections: [source_refs, related_specs] -->\n<!-- filled_sections: [] -->\n\n## Source References\n\nPARTIAL CRASHED CONTENT\n\n## Related Specs\n\n<!-- TODO -->\n",
        );

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "slug": "partial-issue",
            "section": "source_refs",
            "content": "Complete new content for source refs."
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["section_filled"], "source_refs");

        // Verify the old partial content is gone
        let issue_path = crate::shared::workspace::issues_path(tmp.path())
            .join("open")
            .join("partial-issue.md");
        let updated = std::fs::read_to_string(&issue_path).unwrap();
        assert!(!updated.contains("PARTIAL CRASHED CONTENT"));
        assert!(updated.contains("Complete new content for source refs."));
        // filled_sections should now include source_refs
        let filled = read_issue_filled_sections(&updated);
        assert!(filled.contains(&"source_refs".to_string()));
    }

    // REQ: R2
    #[test]
    fn test_artifact_advances_filled_sections() {
        // T4: writing a section advances filled_sections
        let tmp = setup_issue(
            "advance-issue",
            "---\nid: advance-issue\ntitle: Test\nstate: open\nlabels: []\n---\n\n## Problem\n\nBug.\n\n## Requirements\n\n- R1: Fix\n\n## Scope\n\n### In-scope\n- crate: sdd\n\n## Reference Context\n\n<!-- fill_sections: [source_refs, related_specs] -->\n<!-- filled_sections: [source_refs] -->\n\n## Source References\n\nDone.\n\n## Related Specs\n\n<!-- TODO -->\n",
        );

        let args = json!({
            "project_path": tmp.path().to_str().unwrap(),
            "slug": "advance-issue",
            "section": "related_specs",
            "content": "| Spec | Relevance |\n|------|----------|\n| foo.md | high |"
        });
        let result = execute_artifact(&args, tmp.path()).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert!(parsed["filled_sections"].as_array().unwrap().len() == 2);
        assert_eq!(parsed["reference_context_complete"], true);
    }
}

// CODEGEN-END

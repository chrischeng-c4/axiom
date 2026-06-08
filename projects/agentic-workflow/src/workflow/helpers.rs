// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
// CODEGEN-BEGIN
//! Shared helpers for run_change flow modules.
//!
//! Extracted from decide.rs, plan.rs, implement.rs, and merge.rs
//! to avoid duplication across flow-based files.

use crate::models::change::SddInterface;
use crate::models::state::StatePhase;
use std::path::Path;

// ---------------------------------------------------------------------------
// Tool name constants
// ---------------------------------------------------------------------------

/// MCP tool name for run-change (used in instructions to hint "call again")
pub const RUN_CHANGE_TOOL: &str = "sdd_run_change";

// ---------------------------------------------------------------------------
// next_action builder (interface-aware: CLI-only or MCP-only)
// ---------------------------------------------------------------------------

/// Build a `next_actions` entry in CLI format.
///
/// `{ "cli": "score ...", "args": {...} }`
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn next_action(
    _interface: SddInterface,
    tool: &str,
    args: serde_json::Value,
) -> serde_json::Value {
    let cli = format_cli_command(tool, &args);
    serde_json::json!({
        "cli": cli,
        "args": args,
    })
}

/// Format an MCP tool name as a natural CLI command string.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn format_cli_command(tool: &str, args: &serde_json::Value) -> String {
    if tool == RUN_CHANGE_TOOL {
        let mut parts = vec!["score".to_string(), "run-change".to_string()];
        if let Some(id) = args.get("change_id").and_then(|v| v.as_str()) {
            parts.push("--change-id".to_string());
            parts.push(id.to_string());
        }
        if let Some(desc) = args.get("description").and_then(|v| v.as_str()) {
            parts.push("--description".to_string());
            parts.push(format!("\"{}\"", desc));
        }
        parts.join(" ")
    } else if let Some(action_name) = tool.strip_prefix("sdd_workflow_") {
        // sdd_workflow_* → score workflow <action-name> <change_id> [extra_args_json]
        let action_kebab = action_name.replace('_', "-");
        let mut parts = vec!["score".to_string(), "workflow".to_string(), action_kebab];
        if let Some(id) = args.get("change_id").and_then(|v| v.as_str()) {
            parts.push(id.to_string());
        }
        // Pass remaining args as JSON (same pattern as artifact)
        let mut extra = args.clone();
        if let Some(obj) = extra.as_object_mut() {
            obj.remove("change_id");
            obj.remove("project_path");
            if !obj.is_empty() {
                let json_str = serde_json::to_string(&serde_json::Value::Object(obj.clone()))
                    .unwrap_or_default();
                parts.push(format!("'{}'", json_str));
            }
        }
        parts.join(" ")
    } else if let Some(action_name) = tool.strip_prefix("sdd_artifact_") {
        // sdd_artifact_* → score artifact <action-name> <change_id> <payload_path>
        let action_kebab = action_name.replace('_', "-");
        let change_id = args
            .get("change_id")
            .and_then(|v| v.as_str())
            .unwrap_or("CHANGE_ID");
        let group_id = args.get("group_id").and_then(|v| v.as_str());
        let payload_path = match group_id {
            Some(gid) => format!(
                ".aw/changes/{}/groups/{}/payloads/{}.json",
                change_id, gid, action_kebab
            ),
            None => format!(".aw/changes/{}/payloads/{}.json", change_id, action_kebab),
        };
        format!(
            "score artifact {} {} {}",
            action_kebab, change_id, payload_path
        )
    } else {
        // Fallback: route unmatched tools through workflow
        let action_kebab = tool.strip_prefix("sdd_").unwrap_or(tool).replace('_', "-");
        let args_json = serde_json::to_string(args).unwrap_or_else(|_| "{}".to_string());
        format!("score workflow {} '{}'", action_kebab, args_json)
    }
}

// ---------------------------------------------------------------------------
// Payload file helpers (for payload_path convention)
// ---------------------------------------------------------------------------

/// Write an artifact payload JSON file and return its relative path.
///
/// When `group_id` is `Some`, the payload is written to
/// `.aw/changes/{change_id}/groups/{gid}/payloads/{action_kebab}.json`.
/// Otherwise it falls back to the change-level `payloads/` directory.
///
/// Returns the relative path (from project root) to the written file.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn write_artifact_payload(
    project_root: &Path,
    change_id: &str,
    group_id: Option<&str>,
    action_kebab: &str,
    payload: &serde_json::Value,
) -> crate::Result<String> {
    let change_base = project_root.join(".aw/changes").join(change_id);
    let payloads_dir = match group_id {
        Some(gid) => change_base.join("groups").join(gid).join("payloads"),
        None => change_base.join("payloads"),
    };
    std::fs::create_dir_all(&payloads_dir)?;
    let rel_path = match group_id {
        Some(gid) => format!(
            ".aw/changes/{}/groups/{}/payloads/{}.json",
            change_id, gid, action_kebab
        ),
        None => format!(".aw/changes/{}/payloads/{}.json", change_id, action_kebab),
    };
    let abs_path = project_root.join(&rel_path);
    let content = serde_json::to_string_pretty(payload)?;
    std::fs::write(&abs_path, content)?;
    Ok(rel_path)
}

// ---------------------------------------------------------------------------
// Phase helpers
// ---------------------------------------------------------------------------

/// Convert StatePhase to string for JSON output
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn phase_to_string(phase: &StatePhase) -> &'static str {
    crate::tools::workflow_common::phase_to_string(phase)
}

// ---------------------------------------------------------------------------
// Schema version
// ---------------------------------------------------------------------------

/// Get schema_version from STATE.yaml (default 3 for new changes)
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn get_schema_version(change_dir: &Path) -> u32 {
    let state_path = change_dir.join("STATE.yaml");
    if !state_path.exists() {
        return 3;
    }
    if let Ok(content) = std::fs::read_to_string(&state_path) {
        for line in content.lines() {
            if let Some(v) = line.strip_prefix("schema_version:") {
                let v = v.trim().trim_matches('"');
                if let Some(major) = v.split('.').next() {
                    if let Ok(n) = major.parse::<u32>() {
                        return n;
                    }
                }
            }
        }
    }
    2
}

// ---------------------------------------------------------------------------
// Complexity
// ---------------------------------------------------------------------------

/// Read complexity from STATE.yaml
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn read_complexity_from_state(change_dir: &Path) -> Option<String> {
    let state_path = change_dir.join("STATE.yaml");
    if let Ok(content) = std::fs::read_to_string(&state_path) {
        for line in content.lines() {
            if let Some(v) = line.strip_prefix("complexity:") {
                let v = v.trim().trim_matches('"');
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

/// Check if exploration.md has needs_clarification: true in frontmatter
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn check_exploration_needs_clarification(path: &Path) -> bool {
    if let Ok(content) = std::fs::read_to_string(path) {
        if content.starts_with("---") {
            if let Some(end) = content[3..].find("---") {
                let frontmatter = &content[3..3 + end];
                return frontmatter.contains("needs_clarification: true");
            }
        }
    }
    false
}

/// Suggest question topics based on description (delegated to scope module).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn suggest_topics(description: &str) -> Vec<&'static str> {
    super::scope::suggest_topics(description)
}

// ---------------------------------------------------------------------------
// Three-verdict escalation
// ---------------------------------------------------------------------------

/// Build a "mainthread must fix" response for REJECTED verdict with high revision count.
///
/// Per spec: when revision_count >= 1 after REJECTED, mainthread MUST intervene.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn mainthread_must_fix(
    change_id: &str,
    phase: &StatePhase,
    artifact_type: &str,
    review_file: &str,
    interface: SddInterface,
) -> serde_json::Value {
    serde_json::json!({
        "change_id": change_id,
        "workflow_version": 2,
        "current_phase": phase_to_string(phase),
        "action": "mainthread_fix",
        "severity": "REJECTED",
        "message": format!(
            "Artifact '{}' has been REJECTED after 4+ revision attempts. Mainthread must intervene.",
            artifact_type
        ),
        "prompt": format!(
            "# ESCALATION: Mainthread Must Fix '{}' for Change '{}'\n\n\
             The {} artifact has been REJECTED after multiple revision attempts.\n\
             The automated agent has exhausted its revision budget.\n\n\
             ## Instructions\n\n\
             1. Read the artifact and its latest review for context\n\
             2. Identify the fundamental issues that agents couldn't resolve\n\
             3. Fix the artifact directly\n\
             4. Call `score run-change` to continue the workflow\n\n\
             ## Files to Read\n\n\
             - `{}` — the artifact\n\
             - `{}` — the latest review with rejection reasons",
            artifact_type, change_id, artifact_type,
            artifact_type, review_file
        ),
        "executor": ["mainthread"],
        "next_actions": [
            next_action(interface, RUN_CHANGE_TOOL, serde_json::json!({"change_id": change_id})),
        ],
    })
}

// ---------------------------------------------------------------------------
// Spec analysis (from plan.rs)
// ---------------------------------------------------------------------------

/// Spec metadata extracted from a spec file.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#schema
#[derive(Debug, Clone)]
pub struct SpecInfo {
    /// Spec identifier.
    pub id: String,
    /// Dependencies on other specs.
    pub depends: Vec<String>,
}
/// Count spec files in the specs directory
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn count_spec_files(specs_dir: &Path) -> usize {
    if !specs_dir.exists() {
        return 0;
    }
    std::fs::read_dir(specs_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
                .count()
        })
        .unwrap_or(0)
}

/// Analyze specs: find missing specs and pending review
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn analyze_specs(
    proposal_path: &Path,
    specs_dir: &Path,
) -> crate::Result<(Vec<SpecInfo>, Option<String>)> {
    let content = std::fs::read_to_string(proposal_path)?;
    let change_dir = proposal_path.parent().unwrap_or(Path::new("."));

    let affected_specs = parse_affected_specs(&content);

    let mut missing_specs = Vec::new();
    let mut pending_review = None;

    for spec in affected_specs {
        if !spec
            .id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            continue;
        }

        let spec_path = specs_dir.join(format!("{}.md", spec.id));
        let review_new = change_dir.join(format!("review_spec_{}.md", spec.id));
        let review_legacy = change_dir.join(format!("REVIEW_SPEC_{}.md", spec.id));

        if !spec_path.exists() {
            missing_specs.push(spec);
        } else {
            // Check inline review first (review_verdict in spec frontmatter)
            let inline_verdict = extract_verdict(&spec_path);
            if let Some(ref v) = inline_verdict {
                if v == "REVIEWED" || v == "NEEDS_REVISION" || v == "REJECTED" {
                    pending_review = Some(spec.id);
                }
                // APPROVED or other → spec is reviewed and approved, skip
            } else if !review_new.exists() && !review_legacy.exists() {
                // No inline review and no separate review file → needs review
                pending_review = Some(spec.id);
            } else {
                // Legacy: separate review file exists — check verdict
                let review_path = if review_new.exists() {
                    review_new
                } else {
                    review_legacy
                };
                if let Some(verdict) = extract_verdict(&review_path) {
                    if verdict == "REVIEWED" || verdict == "NEEDS_REVISION" || verdict == "REJECTED"
                    {
                        pending_review = Some(spec.id);
                    }
                }
            }
        }
    }

    Ok((missing_specs, pending_review))
}

/// Parse affected_specs or spec_plan from proposal frontmatter.
///
/// Supports both legacy `affected_specs:` and v2 `spec_plan:` formats.
/// The `spec_plan:` format includes extra fields (title, context_refs,
/// gap_repairs, affected_code) which are ignored — only `id` and `depends`
/// are extracted.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn parse_affected_specs(content: &str) -> Vec<SpecInfo> {
    let mut specs = Vec::new();

    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let frontmatter = &content[3..3 + end];

            let mut in_affected_specs = false;
            let mut current_spec: Option<String> = None;
            let mut current_depends: Vec<String> = Vec::new();

            for line in frontmatter.lines() {
                let trimmed = line.trim();

                // Support both legacy affected_specs: and v2 spec_plan:
                if trimmed.starts_with("affected_specs:") || trimmed.starts_with("spec_plan:") {
                    in_affected_specs = true;
                    continue;
                }

                if in_affected_specs {
                    if trimmed.starts_with("- id:") {
                        if let Some(id) = current_spec.take() {
                            specs.push(SpecInfo {
                                id,
                                depends: std::mem::take(&mut current_depends),
                            });
                        }
                        current_spec = Some(trimmed.trim_start_matches("- id:").trim().to_string());
                    } else if trimmed.starts_with("id:") {
                        if let Some(id) = current_spec.take() {
                            specs.push(SpecInfo {
                                id,
                                depends: std::mem::take(&mut current_depends),
                            });
                        }
                        current_spec = Some(trimmed.trim_start_matches("id:").trim().to_string());
                    } else if trimmed.starts_with("depends:") {
                        let deps_str = trimmed.trim_start_matches("depends:").trim();
                        if deps_str.starts_with('[') && deps_str.ends_with(']') {
                            let inner = &deps_str[1..deps_str.len() - 1];
                            for dep in inner.split(',') {
                                let dep = dep.trim().trim_matches('"').trim_matches('\'');
                                if !dep.is_empty() {
                                    current_depends.push(dep.to_string());
                                }
                            }
                        }
                    } else if trimmed.starts_with("- ") && current_spec.is_some() {
                        let item = trimmed.trim_start_matches("- ").trim();
                        if !item.contains(':') {
                            current_depends.push(item.to_string());
                        }
                    } else if !trimmed.starts_with('-')
                        && !trimmed.is_empty()
                        && !trimmed.contains(':')
                    {
                        break;
                    }
                }
            }

            if let Some(id) = current_spec {
                specs.push(SpecInfo {
                    id,
                    depends: current_depends,
                });
            }
        }
    }

    specs
}

/// Get the last review verdict from inline reviews or legacy review files.
///
/// Checks (in order):
/// 1. Inline reviews: `review_verdict:` in spec files (`specs/*.md`)
/// 2. Legacy separate files: `review_spec_*.md`, `REVIEW_SPEC_*.md`
/// 3. Inline review in `proposal.md`
/// 4. Legacy separate files: `review_proposal.md`, `REVIEW_PROPOSAL.md`
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn get_last_review_verdict(change_dir: &Path) -> Option<String> {
    // Check inline spec reviews first (most recent by modification time)
    let specs_dir = change_dir.join("specs");
    if specs_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&specs_dir) {
            let mut spec_files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
                .collect();
            spec_files.sort_by(|a, b| {
                let time_a = a.metadata().and_then(|m| m.modified()).ok();
                let time_b = b.metadata().and_then(|m| m.modified()).ok();
                time_b.cmp(&time_a)
            });
            for entry in &spec_files {
                if let Some(verdict) = extract_verdict(&entry.path()) {
                    return Some(verdict);
                }
            }
        }
    }

    // Fallback: check legacy separate spec review files
    if let Ok(entries) = std::fs::read_dir(change_dir) {
        let mut spec_reviews: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                (name.starts_with("REVIEW_SPEC_") || name.starts_with("review_spec_"))
                    && name.ends_with(".md")
            })
            .collect();

        spec_reviews.sort_by(|a, b| {
            let time_a = a.metadata().and_then(|m| m.modified()).ok();
            let time_b = b.metadata().and_then(|m| m.modified()).ok();
            time_b.cmp(&time_a)
        });

        if let Some(entry) = spec_reviews.first() {
            if let Some(verdict) = extract_verdict(&entry.path()) {
                return Some(verdict);
            }
        }
    }

    // Check inline review in proposal.md
    let proposal_path = change_dir.join("proposal.md");
    if proposal_path.exists() {
        if let Some(verdict) = extract_verdict(&proposal_path) {
            return Some(verdict);
        }
    }

    // Fallback: check legacy separate proposal review files
    for name in &["review_proposal.md", "REVIEW_PROPOSAL.md"] {
        let path = change_dir.join(name);
        if path.exists() {
            if let Some(verdict) = extract_verdict(&path) {
                return Some(verdict);
            }
        }
    }

    None
}

/// Check if an artifact has a review_verdict in its frontmatter.
///
/// Used by reference_context.rs to detect mainthread-path: artifact exists
/// and was reviewed (has verdict) but phase not yet advanced.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn has_review_verdict(change_dir: &Path, artifact: &str) -> bool {
    let path = change_dir.join(format!("{}.md", artifact));
    extract_verdict(&path).is_some()
}

/// Check if a specific file path has a review_verdict in its frontmatter.
///
/// Unlike `has_review_verdict` which takes change_dir + artifact name,
/// this takes the full path to the artifact file.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn has_review_verdict_at(path: &Path) -> bool {
    extract_verdict(path).is_some()
}

/// Extract verdict, treating missing verdict in an existing file as APPROVED.
///
/// `review_service` removes `review_verdict` for APPROVED verdicts, so if the
/// file exists but has no verdict field, it was approved.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn extract_verdict_or_approved(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }
    match extract_verdict(path) {
        Some(v) => Some(v),
        None => Some("APPROVED".to_string()),
    }
}

/// Extract verdict from a review file or artifact with inline review.
///
/// Supports three formats (checked in order):
/// 1. YAML frontmatter: `review_verdict: APPROVED` (inline review in artifact)
/// 2. YAML frontmatter: `verdict: APPROVED` (separate review file)
/// 3. Checkbox markdown: `- [x] REVIEWED` (legacy sdd_review_proposal/spec)
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn extract_verdict(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    extract_verdict_from_content(&content)
}

/// Extract verdict from content string (testable without filesystem).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn extract_verdict_from_content(content: &str) -> Option<String> {
    // Try YAML frontmatter first
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let frontmatter = &content[3..3 + end];
            // Check review_verdict first (inline review in artifact file)
            for line in frontmatter.lines() {
                if line.trim().starts_with("review_verdict:") {
                    return Some(
                        line.trim()
                            .trim_start_matches("review_verdict:")
                            .trim()
                            .to_string(),
                    );
                }
            }
            // Fallback: verdict (separate review file)
            for line in frontmatter.lines() {
                if line.trim().starts_with("verdict:") {
                    return Some(
                        line.trim()
                            .trim_start_matches("verdict:")
                            .trim()
                            .to_string(),
                    );
                }
            }
        }
    }

    // Fallback: checkbox format (legacy review tools)
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [x]") {
            let verdict_part = trimmed.trim_start_matches("- [x]").trim();
            // Extract first word as verdict (e.g., "APPROVED - description" -> "APPROVED")
            if let Some(v) = verdict_part.split_whitespace().next() {
                let v = v.trim_end_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_');
                match v {
                    "APPROVED" | "REVIEWED" | "NEEDS_REVISION" | "REJECTED" | "PASS" => {
                        return Some(v.to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Review info extraction (from implement.rs / merge.rs)
// ---------------------------------------------------------------------------

/// Extract verdict and issues from a REVIEW_*.md file (review_impl.md, review_merge.md)
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn extract_review_info(path: &Path) -> (Option<String>, Vec<String>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (None, vec![]),
    };

    let mut verdict = None;
    let mut issues = Vec::new();

    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let frontmatter = &content[3..3 + end];
            for line in frontmatter.lines() {
                let trimmed = line.trim();
                // Support both "verdict:" (legacy separate file) and "review_verdict:" (inline)
                if trimmed.starts_with("review_verdict:") {
                    verdict = Some(
                        trimmed
                            .trim_start_matches("review_verdict:")
                            .trim()
                            .to_string(),
                    );
                } else if trimmed.starts_with("verdict:") && verdict.is_none() {
                    verdict = Some(trimmed.trim_start_matches("verdict:").trim().to_string());
                }
            }
        }
    }

    let mut in_issues = false;
    for line in content.lines() {
        if line.starts_with("## Issues") || line.starts_with("### Issues") {
            in_issues = true;
            continue;
        }
        if in_issues && line.starts_with("## ") {
            break;
        }
        if in_issues && line.starts_with("- ") {
            issues.push(line.trim_start_matches("- ").to_string());
        }
    }

    (verdict, issues)
}

// ---------------------------------------------------------------------------
// Merge helpers (from merge.rs)
// ---------------------------------------------------------------------------

/// Find specs that need to be merged.
///
/// Checks `groups/*/specs/` first (new structure), then falls back to `specs/`
/// (legacy structure) for backward compatibility. Returns full paths.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn find_specs_to_merge(change_dir: &Path) -> Vec<std::path::PathBuf> {
    // New structure: groups/*/specs/
    let groups_dir = change_dir.join("groups");
    if groups_dir.is_dir() {
        let mut paths = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&groups_dir) {
            let mut group_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            group_entries.sort_by_key(|e| e.file_name());
            for entry in group_entries {
                if entry.path().is_dir() {
                    let group_specs = entry.path().join("specs");
                    if group_specs.is_dir() {
                        collect_spec_paths_into(&group_specs, &mut paths);
                    }
                }
            }
        }
        if !paths.is_empty() {
            return paths;
        }
    }
    // Legacy fallback: specs/ directory
    let specs_dir = change_dir.join("specs");
    let mut paths = Vec::new();
    collect_spec_paths_into(&specs_dir, &mut paths);
    paths
}

/// Collect .md spec file paths from a directory into `out` (no symlinks, sorted).
fn collect_spec_paths_into(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut file_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        file_entries.sort_by_key(|e| e.file_name());
        for entry in file_entries {
            let path = entry.path();
            let meta = match std::fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                collect_spec_paths_into(&path, out);
            } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                // Skip .base.md files — they are 3-way merge artifacts, not content specs
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.ends_with(".base.md") {
                    continue;
                }
                out.push(path);
            }
        }
    }
}

/// Recursively collect spec files from a directory.
/// Uses symlink_metadata to avoid following symlinks (directory traversal prevention).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn collect_spec_files(dir: &Path) -> Vec<String> {
    let mut specs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let metadata = match std::fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                specs.extend(collect_spec_files(&path));
            } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Some(name) = path.file_stem() {
                    specs.push(name.to_string_lossy().to_string());
                }
            }
        }
    }

    specs
}

// ---------------------------------------------------------------------------
// Alignment warnings (Phase 3: run-change integration)
// ---------------------------------------------------------------------------

/// Collect alignment violations from the current group's spec files.
///
/// Loads STATE.yaml to determine the current group, globs `*.md` files in
/// the group's `specs/` directory, and runs `spec_alignment::check()` on each.
///
/// Returns `Some(vec)` if violations found, `None` if clean, empty, or on error.
/// Errors are caught and logged via `tracing::warn!` — never propagated.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/helpers.md#source
pub fn collect_alignment_warnings(change_dir: &Path) -> Option<Vec<serde_json::Value>> {
    let result = collect_alignment_warnings_inner(change_dir);
    match result {
        Ok(warnings) => warnings,
        Err(e) => {
            tracing::warn!(
                change_dir = %change_dir.display(),
                error = %e,
                "alignment check failed — returning null"
            );
            None
        }
    }
}

/// Inner implementation for `collect_alignment_warnings` — returns Result for
/// error isolation (outer function catches all errors).
fn collect_alignment_warnings_inner(
    change_dir: &Path,
) -> std::result::Result<Option<Vec<serde_json::Value>>, Box<dyn std::error::Error>> {
    // Collect spec files using the same logic as find_specs_to_merge:
    // groups/*/specs/*.md first, then fallback to specs/
    let groups_dir = change_dir.join("groups");
    if groups_dir.is_dir() {
        let mut all_warnings = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&groups_dir) {
            let mut group_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            group_entries.sort_by_key(|e| e.file_name());
            for entry in group_entries {
                if entry.path().is_dir() {
                    let specs_dir = entry.path().join("specs");
                    if specs_dir.is_dir() {
                        if let Ok(Some(warnings)) = collect_violations_from_dir(&specs_dir) {
                            all_warnings.extend(warnings);
                        }
                    }
                }
            }
        }
        if !all_warnings.is_empty() {
            return Ok(Some(all_warnings));
        }
        // No violations found in groups — don't fall through to legacy
        return Ok(None);
    }

    // Legacy fallback: specs/ directory
    let specs_dir = change_dir.join("specs");
    if !specs_dir.is_dir() {
        return Ok(None);
    }
    collect_violations_from_dir(&specs_dir)
}

/// Collect violations from all `.md` files in a directory.
fn collect_violations_from_dir(
    specs_dir: &Path,
) -> std::result::Result<Option<Vec<serde_json::Value>>, Box<dyn std::error::Error>> {
    let entries = std::fs::read_dir(specs_dir)?;
    let mut md_files: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .map(|e| e.path())
        .collect();
    md_files.sort();

    if md_files.is_empty() {
        return Ok(None);
    }

    let mut all_warnings = Vec::new();

    for file_path in &md_files {
        let check_result = crate::spec_alignment::check(file_path);
        for file_result in &check_result.files {
            for violation in &file_result.violations {
                all_warnings.push(serde_json::json!({
                    "kind": violation.kind.to_string(),
                    "message": &violation.message,
                    "heading": violation.heading.as_deref(),
                    "line": violation.line,
                    "file": &file_result.path,
                }));
            }
        }
    }

    if all_warnings.is_empty() {
        Ok(None)
    } else {
        Ok(Some(all_warnings))
    }
}

// ---------------------------------------------------------------------------
// Task graph helpers — re-exported from task_graph module
// ---------------------------------------------------------------------------

pub use super::task_graph::{
    build_task_execution_order, find_completed_tasks, find_next_task, is_codegen_eligible,
    parse_task_blocks, TaskInfo,
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_suggest_topics_auth() {
        let topics = suggest_topics("Add OAuth authentication with Google");
        assert!(topics.iter().any(|t| t.contains("Authentication")));
    }

    #[test]
    fn test_suggest_topics_default() {
        let topics = suggest_topics("Implement generic feature");
        assert!(topics.iter().any(|t| t.contains("Implementation")));
    }

    #[test]
    fn test_get_schema_version_new() {
        let temp_dir = TempDir::new().unwrap();
        let v = get_schema_version(temp_dir.path());
        assert_eq!(v, 3);
    }

    #[test]
    fn test_get_schema_version_legacy() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("STATE.yaml"),
            "schema_version: \"2.0\"\n",
        )
        .unwrap();
        let v = get_schema_version(temp_dir.path());
        assert_eq!(v, 2);
    }

    #[test]
    fn test_parse_affected_specs() {
        let content = r#"---
id: test
affected_specs:
  - id: auth-flow
    depends: []
  - id: user-model
    depends: [auth-flow]
---
# Proposal
"#;
        let specs = parse_affected_specs(content);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].id, "auth-flow");
        assert_eq!(specs[1].id, "user-model");
        assert_eq!(specs[1].depends, vec!["auth-flow"]);
    }

    #[test]
    fn test_extract_verdict_from_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("REVIEW.md");
        std::fs::write(
            &path,
            "---\nverdict: APPROVED\niteration: 1\n---\n# Review\n",
        )
        .unwrap();
        assert_eq!(extract_verdict(&path), Some("APPROVED".to_string()));
    }

    #[test]
    fn test_extract_verdict_from_review_verdict_field() {
        // Inline review: review_verdict in artifact frontmatter takes priority
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("proposal.md");
        std::fs::write(
            &path,
            "---\nid: test\ntype: proposal\nreview_verdict: REVIEWED\nreview_iteration: 2\n---\n\n# Proposal\n",
        )
        .unwrap();
        assert_eq!(extract_verdict(&path), Some("REVIEWED".to_string()));
    }

    #[test]
    fn test_extract_verdict_from_content_review_verdict_priority() {
        // review_verdict takes priority over verdict
        let content = "---\nreview_verdict: REJECTED\nverdict: APPROVED\n---\n# Content\n";
        assert_eq!(
            extract_verdict_from_content(content),
            Some("REJECTED".to_string())
        );
    }

    #[test]
    fn test_extract_verdict_from_checkbox() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("REVIEW_PROPOSAL.md");
        std::fs::write(
            &path,
            "# Review\n\n## Verdict\n\n- [ ] APPROVED - Ready\n- [x] NEEDS_REVISION - Has issues\n- [ ] REJECTED - Bad\n",
        )
        .unwrap();
        assert_eq!(extract_verdict(&path), Some("NEEDS_REVISION".to_string()));
    }

    #[test]
    fn test_extract_review_info() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("review_impl.md");
        std::fs::write(
            &path,
            "---\nverdict: NEEDS_REVISION\n---\n# Review\n## Issues\n- Fix bug\n- Add tests\n",
        )
        .unwrap();
        let (verdict, issues) = extract_review_info(&path);
        assert_eq!(verdict, Some("NEEDS_REVISION".to_string()));
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn test_parse_affected_specs_legacy_format() {
        let content = "---\naffected_specs:\n  - id: my-spec\n    depends: []\n---\n# Proposal\n";
        let specs = parse_affected_specs(content);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id, "my-spec");
        assert!(specs[0].depends.is_empty());
    }

    #[test]
    fn test_parse_affected_specs_v2_spec_plan() {
        let content = r#"---
spec_plan:
  - id: fix-parsing
    title: "Fix parsing"
    depends: []
    context_refs:
    affected_code: ["src/helpers.rs"]
  - id: add-feature
    title: "Add feature"
    depends: [fix-parsing]
    context_refs:
    affected_code: ["src/main.rs"]
---

# Proposal
"#;
        let specs = parse_affected_specs(content);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].id, "fix-parsing");
        assert!(specs[0].depends.is_empty());
        assert_eq!(specs[1].id, "add-feature");
        assert_eq!(specs[1].depends, vec!["fix-parsing"]);
    }

    #[test]
    fn test_parse_affected_specs_v2_with_gap_repairs() {
        let content = r#"---
spec_plan:
  - id: my-spec
    title: "My Spec"
    depends: [dep-a, dep-b]
    gap_repairs:
      - { source: unknown, gap_index: 0 }
    affected_code: ["src/lib.rs"]
---
"#;
        let specs = parse_affected_specs(content);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id, "my-spec");
        assert_eq!(specs[0].depends, vec!["dep-a", "dep-b"]);
    }

    #[test]
    fn test_parse_affected_specs_empty_returns_empty() {
        let content = "---\nid: test\ntype: proposal\n---\n# No specs\n";
        let specs = parse_affected_specs(content);
        assert!(specs.is_empty());
    }

    // ─── Phase 3: Alignment Warnings Tests (R25, R26, R27) ─────────────────

    #[test]
    fn test_collect_alignment_warnings_io_error() {
        // R27: Non-existent change directory → returns None, no panic
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join("nonexistent-change");
        // change_dir doesn't exist — no groups/ or specs/ directory
        let result = collect_alignment_warnings(&change_dir);
        assert!(
            result.is_none(),
            "Non-existent change dir should return None"
        );
    }

    #[test]
    fn test_collect_alignment_warnings_empty_dir() {
        // R25: Empty specs directory (no .md files) → returns None
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join("change-empty");
        let group_dir = change_dir.join("groups/test-group/specs");
        std::fs::create_dir_all(&group_dir).unwrap();
        // Empty specs dir — no .md files

        let result = collect_alignment_warnings(&change_dir);
        assert!(result.is_none(), "Empty specs dir should return None");
    }

    #[test]
    fn test_alignment_warnings_json_schema() {
        // R26: Known violations → each element has kind, message, file, optional heading/line
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join("change-schema");
        let specs_dir = change_dir.join("groups/g1/specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        // Write a spec with duplicate sections → known violations
        std::fs::write(
            specs_dir.join("dup-spec.md"),
            "---\nid: dup-spec\n---\n# Spec\n\n\
             ## Overview\n<!-- type: overview lang: markdown -->\n\nText A.\n\n\
             ## Overview\n<!-- type: overview lang: markdown -->\n\nText B.\n",
        )
        .unwrap();

        let result = collect_alignment_warnings(&change_dir);
        assert!(result.is_some(), "Should have violations");

        let warnings = result.unwrap();
        assert!(!warnings.is_empty(), "Should have at least one warning");

        // Verify JSON schema of each warning element (R26)
        for w in &warnings {
            assert!(
                w.get("kind").and_then(|v| v.as_str()).is_some(),
                "Each warning must have 'kind' as string"
            );
            assert!(
                w.get("message").and_then(|v| v.as_str()).is_some(),
                "Each warning must have 'message' as string"
            );
            assert!(
                w.get("file").and_then(|v| v.as_str()).is_some(),
                "Each warning must have 'file' as string"
            );
            // heading and line are optional (may be null)
            assert!(
                w.get("heading").is_some(),
                "Each warning must have 'heading' field (may be null)"
            );
            assert!(
                w.get("line").is_some(),
                "Each warning must have 'line' field (may be null)"
            );
        }

        // Verify at least one is a duplicate_section violation
        let has_dup = warnings
            .iter()
            .any(|w| w.get("kind").and_then(|v| v.as_str()) == Some("duplicate_section"));
        assert!(has_dup, "Should contain a duplicate_section violation");
    }

    #[test]
    fn test_collect_alignment_warnings_clean_specs() {
        // R25: All specs clean → returns None
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join("change-clean");
        let specs_dir = change_dir.join("groups/g1/specs");
        std::fs::create_dir_all(&specs_dir).unwrap();

        // Write a clean spec (no sections → no violations)
        std::fs::write(
            specs_dir.join("clean-spec.md"),
            "---\nid: clean-spec\n---\n# Clean Spec\n",
        )
        .unwrap();

        let result = collect_alignment_warnings(&change_dir);
        assert!(result.is_none(), "Clean specs should return None");
    }

    #[test]
    fn test_collect_alignment_warnings_multiple_groups() {
        // R25: Multiple groups with violations → collects from all
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path().join("change-multi");
        let specs1 = change_dir.join("groups/g1/specs");
        let specs2 = change_dir.join("groups/g2/specs");
        std::fs::create_dir_all(&specs1).unwrap();
        std::fs::create_dir_all(&specs2).unwrap();

        // g1: clean spec
        std::fs::write(specs1.join("clean.md"), "---\nid: clean\n---\n# Clean\n").unwrap();

        // g2: spec with violations
        std::fs::write(
            specs2.join("bad.md"),
            "---\nid: bad\n---\n# Bad\n\n\
             ## Overview\n<!-- type: overview lang: markdown -->\n\nA.\n\n\
             ## Overview\n<!-- type: overview lang: markdown -->\n\nB.\n",
        )
        .unwrap();

        let result = collect_alignment_warnings(&change_dir);
        assert!(result.is_some(), "Should have violations from g2");

        let warnings = result.unwrap();
        assert!(!warnings.is_empty());
    }
}

// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
// CODEGEN-BEGIN
//! `aw cb fill` — Phase 3 marker-fill workflow.
//!
//! Two modes:
//! - **Brief** (no `--apply`): walk the current checkout source tree and emit a
//!   marker-list dispatch envelope for mainthread,
//!   or fast-path-dispatch directly to `aw td merge` when zero markers
//!   are present (R11).
//! - **Apply** (`--apply --marker <id>`): merge the expected marker payload
//!   into the HANDWRITE block matching `<id>`, commit that marker with WI
//!   projection trailers, then lock the next marker or dispatch
//!   `aw cb check`.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md

use crate::generate::audit::parse_handwrite_markers;
use crate::issues::{IssueBackend, IssuePatch, LocalBackend};
use anyhow::{Context, Result};
use globset::{Glob, GlobSetBuilder};
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::cli::cb::CbFillArgs;
use crate::cli::remote_push::maybe_push_remote;

// A single open HANDWRITE block discovered in the worktree.
///
// Spec name: `HandwriteMarkerEntry`.
// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct HandwriteMarkerEntry {
    /// Marker identifier — derived from the `gap` attribute (canonical) or
    /// from the `reason:` body when only the legacy reason-style begin
    /// comment is present.
    pub id: String,
    /// Repo-root-relative path to the source file.
    pub source_path: String,
    /// 1-indexed line of the XML or comment-style begin marker.
    pub start_line: usize,
    /// 1-indexed line of the XML or comment-style end marker.
    pub end_line: usize,
    /// Reason string from the marker.
    pub reason: String,
    /// Optional `@spec` reference associated with this block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_ref: Option<String>,
}

// Walk the worktree source tree (under `crates/`, `projects/`, `src/`,
// `tests/`) and return every open HANDWRITE block.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
pub fn enumerate_worktree_markers(worktree: &Path) -> Vec<HandwriteMarkerEntry> {
    let mut out: Vec<HandwriteMarkerEntry> = Vec::new();
    let candidate_subdirs = ["crates", "projects", "src", "tests"];

    let mut roots: Vec<PathBuf> = Vec::new();
    for sub in candidate_subdirs {
        let p = worktree.join(sub);
        if p.exists() {
            roots.push(p);
        }
    }
    if roots.is_empty() {
        roots.push(worktree.to_path_buf());
    }

    for root in roots {
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "rs" | "py" | "ts" | "tsx" | "md") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(path) else {
                continue;
            };

            // Form 1: <HANDWRITE>...</HANDWRITE> (canonical, parsed by
            // crate::generate::audit::parse_handwrite_markers).
            let path_str = path.to_string_lossy().to_string();
            if let Ok(markers) = parse_handwrite_markers(&content, &path_str) {
                for m in markers {
                    let rel = path
                        .strip_prefix(worktree)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    out.push(HandwriteMarkerEntry {
                        id: m.gap.clone(),
                        source_path: rel,
                        start_line: m.line_start,
                        end_line: m.line_end,
                        reason: m.reason,
                        spec_ref: None,
                    });
                }
            }

            // Form 2: comment-style begin/end markers emitted by
            // `crate::generate::apply::scaffold_handwrite_file`.
            for m in parse_handwrite_begin_end(&content) {
                let rel = path
                    .strip_prefix(worktree)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                out.push(HandwriteMarkerEntry {
                    id: m.id,
                    source_path: rel,
                    start_line: m.start_line,
                    end_line: m.end_line,
                    reason: m.reason,
                    spec_ref: m.spec_ref,
                });
            }
        }
    }

    out
}

// Lightweight count of HANDWRITE markers in the worktree. Used by
// `td.rs::run_gen_code` for the post-codegen R8/R11 dispatch decision.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
pub fn count_worktree_handwrite_markers(worktree: &Path) -> usize {
    enumerate_worktree_markers(worktree).len()
}

fn cb_marker_payload_rel(slug: &str, marker_id: &str) -> String {
    format!(".aw/payloads/{}/{}.md", slug, marker_id)
}

fn cb_fill_apply_command(slug: &str, marker_id: &str) -> String {
    format!("aw cb fill {} --apply --marker {}", slug, marker_id)
}

fn td_merge_command(slug: &str, spec_path: &str) -> String {
    if spec_path.is_empty() {
        format!("aw td merge {}", slug)
    } else {
        format!("aw td merge {} --spec-path {}", slug, spec_path)
    }
}

fn marker_payload_template(marker: &HandwriteMarkerEntry) -> String {
    format!(
        "(fill)\n\n<!-- marker: {} path: {} reason: {} -->\n",
        marker.id, marker.source_path, marker.reason
    )
}

fn initialize_marker_payload(
    worktree: &Path,
    slug: &str,
    marker: &HandwriteMarkerEntry,
) -> Result<(String, bool)> {
    let rel = cb_marker_payload_rel(slug, &marker.id);
    let abs = worktree.join(&rel);
    if abs.exists() {
        return Ok((rel, false));
    }
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create payload directory {}", parent.display()))?;
    }
    std::fs::write(&abs, marker_payload_template(marker))
        .with_context(|| format!("failed to write payload {}", abs.display()))?;
    Ok((rel, true))
}

fn next_for_marker(
    slug: &str,
    marker: &HandwriteMarkerEntry,
    payload_path: &str,
) -> serde_json::Value {
    serde_json::json!({
        "kind": "dispatch",
        "command": cb_fill_apply_command(slug, &marker.id),
        "reason": "fill the next HANDWRITE marker payload and apply it",
        "requires_hitl": false,
        "payload_path": payload_path,
    })
}

fn next_for_td_merge(slug: &str, spec_path: &str) -> serde_json::Value {
    serde_json::json!({
        "kind": "dispatch",
        "command": td_merge_command(slug, spec_path),
        "reason": "all HANDWRITE markers are filled",
        "requires_hitl": false,
        "payload_path": null,
    })
}

fn print_compact_json(value: &serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}

// Marker discovered by the comment-style scanner.
struct BeginEndMarker {
    id: String,
    start_line: usize,
    end_line: usize,
    reason: String,
    spec_ref: Option<String>,
}

const HANDWRITE_BEGIN_TOKEN: &str = concat!("HANDWRITE-", "BEGIN");
const HANDWRITE_END_TOKEN: &str = concat!("HANDWRITE-", "END");

// Scan `content` for comment-style begin/end marker blocks
// (also `# ` and `<!-- -->` variants). Tolerant of extra prose between
// the keyword and attribute soup.
fn parse_handwrite_begin_end(content: &str) -> Vec<BeginEndMarker> {
    let mut out: Vec<BeginEndMarker> = Vec::new();
    let mut open: Option<(usize, String, String, Option<String>, String)> = None;
    // Counter for synthetic id fallback — each block gets a unique slug if
    // the BEGIN line has neither `gap=` nor a `reason:` keyword.
    let mut synth_idx: usize = 0;

    for (idx, raw) in content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim_start();
        let body = strip_lead(trimmed);

        if body.contains(HANDWRITE_BEGIN_TOKEN) {
            // Already inside a block — skip nested/duplicate.
            if open.is_some() {
                continue;
            }
            // Prefer attribute-style `gap="..." reason="..."`. Fall back to
            // the freeform `reason: <text>` style used by hand-written
            // markers in cb.rs / td.rs.
            let id_attr = extract_xml_attr(body, "gap");
            let reason_attr = extract_xml_attr(body, "reason");
            let tracker = extract_xml_attr(body, "tracker");
            let (id, reason) = match (id_attr.clone(), reason_attr.clone()) {
                (Some(g), Some(r)) => (g, r),
                _ => {
                    // freeform `reason: <text>` form
                    if let Some(rest) = body.split_once("reason:") {
                        let r = rest.1.trim().to_string();
                        let id = match (id_attr, tracker) {
                            (Some(g), _) => g,
                            (None, Some(t)) => t,
                            (None, None) => {
                                synth_idx += 1;
                                slugify_short(&r)
                                    .unwrap_or_else(|| format!("handwrite-{}", synth_idx))
                            }
                        };
                        (id, r)
                    } else {
                        synth_idx += 1;
                        let id = id_attr.unwrap_or_else(|| format!("handwrite-{}", synth_idx));
                        (id, String::new())
                    }
                }
            };
            open = Some((line_no, id, reason, None, raw.to_string()));
            continue;
        }
        if body.contains(HANDWRITE_END_TOKEN) {
            if let Some((start, id, reason, spec_ref, _open_line)) = open.take() {
                out.push(BeginEndMarker {
                    id,
                    start_line: start,
                    end_line: line_no,
                    reason,
                    spec_ref,
                });
            }
            continue;
        }
    }
    out
}

// Strip leading comment markers used in Rust / Python / Markdown so we
// can pattern-match the body uniformly.
fn strip_lead(line: &str) -> &str {
    let s = line.trim_start();
    for prefix in ["///", "//!", "//", "# ", "#", "<!--"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            return rest.trim_start();
        }
    }
    s
}

// Extract `name="value"` (XML-ish). Returns None if absent.
fn extract_xml_attr(body: &str, name: &str) -> Option<String> {
    let needle = format!("{}=\"", name);
    let i = body.find(&needle)? + needle.len();
    let rest = &body[i..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

// Slugify a phrase down to ~40 chars, lowercase, dash-separated. Returns
// None when the result would be empty.
fn slugify_short(text: &str) -> Option<String> {
    let mut out = String::new();
    let mut last_dash = true;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
        if out.len() >= 40 {
            break;
        }
    }
    let s = out.trim_matches('-').to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

// Top-level dispatch for `aw cb fill`.
pub async fn run(args: CbFillArgs) -> Result<()> {
    if args.apply {
        run_apply(args).await
    } else {
        run_brief(args).await
    }
}

// Brief mode (default): enumerate markers, emit dispatch envelope.
async fn run_brief(args: CbFillArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let slug = args.slug.clone();
    crate::cli::td::td_activate_inplace_if_present(&project_root, &slug)?;
    let worktree_abs = crate::cli::td::td_workspace_path(&project_root, &slug);
    if !worktree_abs.exists() {
        emit_error(
            &slug,
            &format!("workspace not found: {}", worktree_abs.display()),
        )?;
        std::process::exit(2);
    }

    let all_markers = enumerate_worktree_markers(&worktree_abs);

    // Look up the spec_path from the explicit CLI arg, issue frontmatter, or
    // the unique TD spec touched by this branch. If none is available, preserve
    // the legacy all-marker behavior.
    let backend = LocalBackend::from_project_root(&worktree_abs);
    let issue = backend.get(&slug).await.ok().flatten();
    let spec_path = resolve_active_spec_path(&args, issue.as_ref(), &worktree_abs);
    let markers = match spec_path.as_deref().filter(|p| !p.is_empty()) {
        Some(path) => {
            let spec_abs = worktree_abs.join(path);
            let spec_content = match std::fs::read_to_string(&spec_abs) {
                Ok(content) => content,
                Err(e) => {
                    emit_error(
                        &slug,
                        &format!("spec_path not readable at {}: {}", spec_abs.display(), e),
                    )?;
                    std::process::exit(2);
                }
            };
            let change_paths = extract_change_paths_from_spec(&spec_content);
            scope_markers_for_change_paths(&all_markers, Some(&change_paths))
        }
        None => scope_markers_for_change_paths(&all_markers, None),
    };
    let spec_path = spec_path.unwrap_or_default();

    if markers.is_empty() {
        // 0-marker fast-path (R11): dispatch directly to td merge.
        let merge_args = if spec_path.is_empty() {
            serde_json::json!({ "slug": slug })
        } else {
            serde_json::json!({ "slug": slug, "spec_path": spec_path })
        };
        let env = serde_json::json!({
            "action": "dispatch",
            "agent": serde_json::Value::Null,
            "slug": slug,
            "next": next_for_td_merge(&slug, &spec_path),
            "invoke": {
                "command": "aw td merge",
                "args": merge_args,
            },
        });
        print_compact_json(&env)?;
        let _ = args.json;
        let _ = args.force;
        return Ok(());
    }

    let first = &markers[0];
    let (first_payload, first_payload_created) =
        initialize_marker_payload(&worktree_abs, &slug, first)?;
    let already_locked = issue
        .as_ref()
        .and_then(|i| crate::cli::workflow_guard::parse_projection(&i.body))
        .map(|p| p.locked)
        .unwrap_or(false);
    if !already_locked {
        crate::cli::workflow_guard::create_issue_lock(
            &worktree_abs,
            &crate::cli::workflow_guard::TransitionLock::new(
                &slug,
                "cb",
                cb_fill_apply_command(&slug, &first.id),
            )
            .with_expected_payload(first_payload.clone())
            .with_phase_from("cb_genned")
            .with_active_phase("cb_fill_in_progress")
            .with_current_section(first.id.clone())
            .with_remaining_sections(markers.iter().skip(1).map(|m| m.id.clone()))
            .with_dirty_paths([first.source_path.clone()]),
        )
        .await?;
        let issue_path_s = issue
            .as_ref()
            .map(|issue| backend.issue_path(issue).to_string_lossy().into_owned())
            .ok_or_else(|| anyhow::anyhow!("issue '{}' not found in current checkout", slug))?;
        if let Err(e) =
            stage_and_commit_cb_queue_start(&worktree_abs, &slug, &issue_path_s, &first.id)
        {
            emit_error(&slug, &format!("git commit failed: {}", e))?;
            std::process::exit(1);
        }
    }

    // Build the dispatch envelope (mainthread runs invoke.command directly under
    // the mainthread-only execution model; agent is null).
    let env = serde_json::json!({
        "action": "dispatch",
        "agent": null,
        "slug": slug,
        "next": next_for_marker(&slug, first, &first_payload),
        "payload_initialized": first_payload_created,
        "invoke": {
            "command": "aw cb fill",
            "args": {
                "slug": slug,
                "marker_list": markers,
                "spec_path": spec_path,
            },
        },
    });
    print_compact_json(&env)?;
    let _ = args.json;
    let _ = args.force;
    Ok(())
}

fn resolve_active_spec_path(
    args: &CbFillArgs,
    issue: Option<&crate::issues::Issue>,
    worktree_abs: &Path,
) -> Option<String> {
    args.spec_path
        .clone()
        .filter(|p| !p.is_empty())
        .or_else(|| issue.and_then(derive_spec_path_from_implements))
        .or_else(|| crate::cli::td::discover_worktree_spec(worktree_abs))
}

// Resolve a worktree-relative spec path from `Issue.implements` (best
// effort — agents may also rely on the worktree's tech_design tree).
fn derive_spec_path_from_implements(issue: &crate::issues::Issue) -> Option<String> {
    issue
        .implements
        .iter()
        .find(|s| s.ends_with(".md"))
        .cloned()
}

// Extract repo-relative path entries from a TD `## Changes` YAML block.
///
// Supports both `changes:` and `files:` sequence keys and accepts either
// `path:` or `file:` per entry for compatibility with older specs.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
pub fn extract_change_paths_from_spec(spec_content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut in_changes = false;
    let mut in_yaml = false;
    let mut yaml_content = String::new();

    for line in spec_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") && trimmed.to_lowercase().contains("changes") {
            in_changes = true;
            continue;
        }
        if in_changes && trimmed.starts_with("## ") {
            break;
        }
        if in_changes && trimmed == "```yaml" {
            in_yaml = true;
            yaml_content.clear();
            continue;
        }
        if in_yaml && trimmed == "```" {
            append_change_paths_from_yaml(&yaml_content, &mut paths);
            in_yaml = false;
            continue;
        }
        if in_yaml {
            yaml_content.push_str(line);
            yaml_content.push('\n');
        }
    }

    paths.sort();
    paths.dedup();
    paths
}

fn append_change_paths_from_yaml(yaml_content: &str, paths: &mut Vec<String>) {
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(yaml_content) else {
        return;
    };
    let entries = value.get("changes").or_else(|| value.get("files"));
    let Some(serde_yaml::Value::Sequence(entries)) = entries else {
        return;
    };
    for entry in entries {
        let path = entry
            .get("path")
            .or_else(|| entry.get("file"))
            .and_then(|v| v.as_str());
        if let Some(path) = path {
            let path = normalize_rel_path(path);
            if !path.is_empty() {
                paths.push(path);
            }
        }
    }
}

// Filter markers to those owned by the TD's `## Changes` paths.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
pub fn filter_markers_to_change_paths(
    markers: &[HandwriteMarkerEntry],
    change_paths: &[String],
) -> Vec<HandwriteMarkerEntry> {
    if change_paths.is_empty() {
        return Vec::new();
    }

    let mut glob_builder = GlobSetBuilder::new();
    let mut exact_or_prefix = Vec::new();
    for raw in change_paths {
        let path = normalize_rel_path(raw);
        if path.contains('*') || path.contains('?') || path.contains('[') {
            if let Ok(glob) = Glob::new(&path) {
                glob_builder.add(glob);
            }
        } else {
            exact_or_prefix.push(path);
        }
    }
    let glob_set = glob_builder.build().ok();

    markers
        .iter()
        .filter(|marker| {
            let source = normalize_rel_path(&marker.source_path);
            exact_or_prefix
                .iter()
                .any(|path| path_matches(&source, path))
                || glob_set
                    .as_ref()
                    .is_some_and(|set| set.is_match(source.as_str()))
        })
        .cloned()
        .collect()
}

// Apply optional TD Changes scoping to a marker list.
///
// `None` preserves the legacy all-marker behavior for callers that cannot
// resolve an active TD spec. `Some(paths)` scopes to the TD's Changes block.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
pub fn scope_markers_for_change_paths(
    markers: &[HandwriteMarkerEntry],
    change_paths: Option<&[String]>,
) -> Vec<HandwriteMarkerEntry> {
    match change_paths {
        Some(paths) => filter_markers_to_change_paths(markers, paths),
        None => markers.to_vec(),
    }
}

fn path_matches(source: &str, change_path: &str) -> bool {
    source == change_path || source.starts_with(&format!("{}/", change_path.trim_end_matches('/')))
}

fn normalize_rel_path(path: &str) -> String {
    path.trim()
        .trim_start_matches("./")
        .replace(std::path::MAIN_SEPARATOR, "/")
}

// Apply mode: merge a single marker payload, then either continue
// (partial-progress envelope) or run the cb check gate.
async fn run_apply(args: CbFillArgs) -> Result<()> {
    let slug = args.slug.clone();
    let project_root = crate::find_project_root()?;
    crate::cli::td::td_activate_inplace_if_present(&project_root, &slug)?;
    let worktree_abs = crate::cli::td::td_workspace_path(&project_root, &slug);

    let marker_id = match args.marker.as_deref() {
        Some(m) if !m.is_empty() => m.to_string(),
        _ => {
            emit_error(&slug, "--apply requires --marker <id>")?;
            std::process::exit(2);
        }
    };

    if !worktree_abs.exists() {
        emit_error(
            &slug,
            &format!("workspace not found: {}", worktree_abs.display()),
        )?;
        std::process::exit(2);
    }

    // Locate the marker in the worktree source tree. R5
    // (bug-cb-fill-payload-routes-by-marker-id-alone-collides): when
    // multiple markers share an id (e.g. legacy markers emitted before
    // the R1 scaffold disambiguator landed), surface the collision as a
    // hard error instead of silently routing to the alphabetically-first
    // match. Callers must rebuild the marker list (which now uses the
    // R1-disambiguated ids) and re-dispatch with the correct id.
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#logic-resolve_marker_file
    let markers = enumerate_worktree_markers(&worktree_abs);
    let matches: Vec<&HandwriteMarkerEntry> =
        markers.iter().filter(|m| m.id == marker_id).collect();
    let target = match matches.as_slice() {
        [] => {
            emit_error(
                &slug,
                &format!("marker id '{}' not found in current checkout", marker_id),
            )?;
            std::process::exit(2);
        }
        [only] => (*only).clone(),
        many => {
            let paths: Vec<String> = many.iter().map(|m| m.source_path.clone()).collect();
            emit_error(
                &slug,
                &format!(
                    "marker id '{}' is ambiguous — {} files match: {}. \
                     Re-run `aw cb fill` (no --apply) to get the disambiguated marker list.",
                    marker_id,
                    many.len(),
                    paths.join(", "),
                ),
            )?;
            std::process::exit(2);
        }
    };

    // Read the payload.
    let payload_rel = format!(".aw/payloads/{}/{}.md", slug, marker_id);
    let payload_abs = worktree_abs.join(&payload_rel);
    let payload_body = match std::fs::read_to_string(&payload_abs) {
        Ok(s) => s,
        Err(e) => {
            emit_error(
                &slug,
                &format!("payload not readable at {}: {}", payload_abs.display(), e),
            )?;
            std::process::exit(2);
        }
    };

    // Replace the targeted block.
    let source_abs = worktree_abs.join(&target.source_path);
    let original = std::fs::read_to_string(&source_abs)
        .with_context(|| format!("reading source {}", source_abs.display()))?;
    let new_content =
        replace_block_body(&original, target.start_line, target.end_line, &payload_body)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "could not locate marker block at lines {}..{} in {}",
                    target.start_line,
                    target.end_line,
                    source_abs.display()
                )
            })?;
    std::fs::write(&source_abs, &new_content)
        .with_context(|| format!("writing source {}", source_abs.display()))?;
    // Re-enumerate.
    let remaining = enumerate_worktree_markers(&worktree_abs);

    if !remaining.is_empty() {
        let next = &remaining[0];
        let (next_payload, next_payload_created) =
            initialize_marker_payload(&worktree_abs, &slug, next)?;
        crate::cli::workflow_guard::create_issue_lock(
            &worktree_abs,
            &crate::cli::workflow_guard::TransitionLock::new(
                &slug,
                "cb",
                cb_fill_apply_command(&slug, &next.id),
            )
            .with_expected_payload(next_payload.clone())
            .with_phase_from("cb_genned")
            .with_active_phase("cb_fill_in_progress")
            .with_current_section(next.id.clone())
            .with_remaining_sections(remaining.iter().skip(1).map(|m| m.id.clone()))
            .with_dirty_paths([next.source_path.clone()]),
        )
        .await?;
        let backend = LocalBackend::from_project_root(&worktree_abs);
        let issue = backend
            .get(&slug)
            .await?
            .ok_or_else(|| anyhow::anyhow!("issue '{}' not found in current checkout", slug))?;
        let issue_path_s = backend.issue_path(&issue).to_string_lossy().into_owned();
        if let Err(e) = stage_and_commit_cb_marker(
            &worktree_abs,
            &slug,
            &issue_path_s,
            &target.source_path,
            &target.id,
            &next.id,
        ) {
            emit_error(&slug, &format!("git commit failed: {}", e))?;
            std::process::exit(1);
        }
        // Partial-progress envelope (agent: null); mainthread continues.
        let env = serde_json::json!({
            "action": "dispatch",
            "agent": serde_json::Value::Null,
            "slug": slug,
            "next": next_for_marker(&slug, next, &next_payload),
            "payload_initialized": next_payload_created,
            "invoke": {
                "command": "aw cb fill",
                "args": {
                    "slug": slug,
                    "apply": true,
                    "marker": next.id,
                },
            },
        });
        print_compact_json(&env)?;
        let _ = args.json;
        let _ = args.force;
        return Ok(());
    }

    // All markers applied — run cb check as gate.
    let check_ok = run_cb_check_gate(&worktree_abs).await;
    if !check_ok.is_ok() {
        let msg = check_ok
            .err()
            .unwrap_or_else(|| "cb check failed".to_string());
        emit_error(&slug, &format!("cb check gate failed: {}", msg))?;
        std::process::exit(1);
    }

    // Commit Cb-Fill trailer + advance phase.
    let backend = LocalBackend::from_project_root(&worktree_abs);
    let issue = backend
        .get(&slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found in current checkout", slug))?;
    let patch = IssuePatch {
        phase: Some(crate::issues::types::td_phase::CB_FILLED.to_string()),
        ..Default::default()
    };
    backend.update(&slug, &patch).await?;

    // Stage source files + issue + commit.
    let issue_path = backend.issue_path(&issue);
    let issue_path_s = issue_path.to_string_lossy().into_owned();
    maybe_push_remote(&worktree_abs, &issue_path, &slug).await?;
    if let Err(e) = stage_and_commit_cb_fill(&worktree_abs, &slug, &issue_path_s) {
        emit_error(&slug, &format!("git commit failed: {}", e))?;
        std::process::exit(1);
    }
    crate::cli::workflow_guard::complete_issue_lock(&worktree_abs, &slug, "cb").await?;

    // Dispatch next verb.
    // Default: dispatch `aw cb review` so filled handwrite bodies pass
    // through the CB CRRR loop. Backward-compat: `--no-review` short-circuits
    // straight to `aw td merge` for callers that don't need review (e.g.,
    // greenfield slugs with no novel handwrite content).
    // @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli
    let env = if args.no_review {
        serde_json::json!({
            "action": "dispatch",
            "agent": serde_json::Value::Null,
            "slug": slug,
            "next": next_for_td_merge(&slug, ""),
            "invoke": {
                "command": "aw td merge",
                "args": { "slug": slug },
            },
        })
    } else {
        serde_json::json!({
            "action": "dispatch",
            "agent": serde_json::Value::Null,
            "slug": slug,
            "next": {
                "kind": "dispatch",
                "command": format!("aw cb review {}", slug),
                "reason": "filled HANDWRITE markers require CB review",
                "requires_hitl": false,
                "payload_path": null,
            },
            "invoke": {
                "command": "aw cb review",
                "args": { "slug": slug },
            },
        })
    };
    print_compact_json(&env)?;
    let _ = args.json;
    let _ = args.force;
    Ok(())
}

// Replace lines `[start_line, end_line]` (inclusive, 1-indexed) of `src`
// with the BEGIN line + payload body + END line, preserving the BEGIN/END
// marker lines themselves so the block can be re-filled if needed.
fn replace_block_body(
    src: &str,
    start_line: usize,
    end_line: usize,
    payload: &str,
) -> Option<String> {
    if start_line == 0 || end_line < start_line {
        return None;
    }
    let lines: Vec<&str> = src.lines().collect();
    if end_line > lines.len() {
        return None;
    }

    let before = &lines[..start_line]; // includes the BEGIN line
    let after = &lines[end_line - 1..]; // starts at the END line (1-indexed → idx end_line-1)
    let mut out = String::new();
    for l in before {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str(payload.trim_end_matches('\n'));
    out.push('\n');
    for l in after {
        out.push_str(l);
        out.push('\n');
    }
    Some(out)
}

// Resolve the base branch for slug-scoped marker checking.
///
// Resolution order: `SCORE_CB_FILL_BASE_BRANCH` env var → `"main"` fallback.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#logic
fn resolve_base_branch() -> String {
    std::env::var("SCORE_CB_FILL_BASE_BRANCH").unwrap_or_else(|_| "main".to_string())
}

// Files changed by the worktree branch relative to its base. Returns
// repo-root-relative paths (matching `HandwriteMarkerEntry.source_path`).
///
// Empty result on git failure — the caller treats that as "no changes
// to gate against" and the gate falls through to the legacy
// whole-worktree check, preserving the prior behaviour for non-branch
// invocations (e.g. detached HEAD or first commit).
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#logic
pub fn branch_changed_files(worktree: &Path, base_branch: &str) -> HashSet<String> {
    let git_bin = match crate::git::find_git_bin() {
        Some(g) => g,
        None => return HashSet::new(),
    };
    let out = match std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree)
        .args(["diff", "--name-only", &format!("{base_branch}...HEAD")])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return HashSet::new(),
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect()
}

// Run `aw cb check` against the worktree as a gate. Returns Ok(())
// when no slug-introduced markers remain, Err(msg) on findings or
// invocation error.
///
// Slug-scoping (R1, R2, R4): only HANDWRITE markers in files modified
// between the worktree branch and its base count toward the gate.
// Markers inherited from `main` (other unmerged refactors) do not fail
// this gate even though they remain in the worktree. Greenfield
// worktrees with no diff against base trivially pass (R5).
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#logic
async fn run_cb_check_gate(worktree_abs: &Path) -> std::result::Result<(), String> {
    let remaining = enumerate_worktree_markers(worktree_abs);
    if remaining.is_empty() {
        return Ok(());
    }

    let base = resolve_base_branch();
    let changed = branch_changed_files(worktree_abs, &base);
    if changed.is_empty() {
        // Could not compute a branch diff (detached HEAD, missing base,
        // or git error). Fall through to the legacy global check rather
        // than silently passing on a worktree that may have real issues.
        return Err(format!(
            "{} HANDWRITE marker(s) still present after fill",
            remaining.len()
        ));
    }

    let slug_markers: Vec<&HandwriteMarkerEntry> = remaining
        .iter()
        .filter(|m| changed.contains(m.source_path.as_str()))
        .collect();
    if !slug_markers.is_empty() {
        return Err(format!(
            "{} HANDWRITE marker(s) introduced by this branch still present after fill (\
             {} inherited markers ignored)",
            slug_markers.len(),
            remaining.len() - slug_markers.len()
        ));
    }
    Ok(())
}

fn should_stage_lifecycle_path(worktree: &Path, path: &str) -> bool {
    let path = Path::new(path);
    !path.is_absolute() || path.starts_with(worktree)
}

// Stage files and create the `Lifecycle-Stage: Cb-Fill` commit.
fn stage_and_commit_cb_fill(worktree: &Path, slug: &str, issue_path: &str) -> Result<()> {
    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;

    // Add everything that changed (source files + issue file).
    let _ = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree)
        .args(["add", "-A"])
        .output()
        .context("git add -A")?;
    if should_stage_lifecycle_path(worktree, issue_path) {
        // Make sure issue file is staged too (-A should cover it but be explicit).
        let _ = std::process::Command::new(&git_bin)
            .arg("-C")
            .arg(worktree)
            .args(["add", issue_path])
            .output();
    }

    let msg = format!(
        "cb({slug}) \u{2014} markers filled\n\n\
         Lifecycle-Slug: {slug}\n\
         Work-Item: {slug}\n\
         Lifecycle-Stage: Cb-Fill",
    );
    let out = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree)
        .args(["commit", "--allow-empty", "-m", &msg])
        .output()
        .context("git commit")?;
    if !out.status.success() {
        anyhow::bail!(
            "{}",
            String::from_utf8_lossy(&out.stderr).trim().to_string()
        );
    }
    Ok(())
}

fn stage_and_commit_cb_marker(
    worktree: &Path,
    slug: &str,
    rel_issue: &str,
    source_path: &str,
    marker_id: &str,
    next_marker_id: &str,
) -> Result<()> {
    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    for path in [source_path, rel_issue] {
        if !should_stage_lifecycle_path(worktree, path) {
            continue;
        }
        let add = std::process::Command::new(&git_bin)
            .arg("-C")
            .arg(worktree)
            .args(["add", path])
            .output()
            .context("git add")?;
        if !add.status.success() {
            anyhow::bail!(
                "git add '{}' failed: {}",
                path,
                String::from_utf8_lossy(&add.stderr).trim()
            );
        }
    }
    let msg = format!(
        "cb({slug}) \u{2014} marker filled: {marker_id}\n\n\
         Work-Item: {slug}\n\
         Lifecycle-Stage: Cb-Fill-Section\n\
         Lifecycle-Phase: cb_fill_in_progress\n\
         Lifecycle-Pass: fill\n\
         CB-Marker: {marker_id}\n\
         Next-Command: aw cb fill {slug} --apply --marker {next_marker_id}",
    );
    let out = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree)
        .args(["commit", "-m", &msg])
        .output()
        .context("git commit")?;
    if !out.status.success() {
        anyhow::bail!(
            "{}",
            String::from_utf8_lossy(&out.stderr).trim().to_string()
        );
    }
    Ok(())
}

fn stage_and_commit_cb_queue_start(
    worktree: &Path,
    slug: &str,
    rel_issue: &str,
    first_marker_id: &str,
) -> Result<()> {
    let git_bin = crate::git::find_git_bin()
        .ok_or_else(|| anyhow::anyhow!("git binary not found on PATH"))?;
    if should_stage_lifecycle_path(worktree, rel_issue) {
        let add = std::process::Command::new(&git_bin)
            .arg("-C")
            .arg(worktree)
            .args(["add", rel_issue])
            .output()
            .context("git add")?;
        if !add.status.success() {
            anyhow::bail!(
                "git add '{}' failed: {}",
                rel_issue,
                String::from_utf8_lossy(&add.stderr).trim()
            );
        }
    }
    let msg = format!(
        "cb({slug}) \u{2014} fill queue started\n\n\
         Work-Item: {slug}\n\
         Lifecycle-Stage: Cb-Fill-Start\n\
         Lifecycle-Phase: cb_fill_in_progress\n\
         Lifecycle-Pass: fill\n\
         Next-Command: aw cb fill {slug} --apply --marker {first_marker_id}",
    );
    let out = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree)
        .args(["commit", "--allow-empty", "-m", &msg])
        .output()
        .context("git commit")?;
    if !out.status.success() {
        anyhow::bail!(
            "{}",
            String::from_utf8_lossy(&out.stderr).trim().to_string()
        );
    }
    Ok(())
}

fn emit_error(slug: &str, message: &str) -> Result<()> {
    let env = serde_json::json!({
        "action": "error",
        "slug": slug,
        "message": message,
        "next": {
            "kind": "none",
            "command": null,
            "reason": "error requires resolution before continuing",
            "requires_hitl": false,
            "payload_path": null,
        },
    });
    print_compact_json(&env)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handwrite_begin(attrs: &str) -> String {
        format!("// HANDWRITE-{} {}", "BEGIN", attrs)
    }

    fn handwrite_end() -> &'static str {
        concat!("// HANDWRITE-", "END")
    }

    fn marker(id: &str) -> HandwriteMarkerEntry {
        HandwriteMarkerEntry {
            id: id.to_string(),
            source_path: "src/demo.rs".to_string(),
            start_line: 10,
            end_line: 14,
            reason: "missing deterministic generator".to_string(),
            spec_ref: Some("spec.md#logic".to_string()),
        }
    }

    #[test]
    fn slugify_short_basic() {
        assert_eq!(
            slugify_short("Hello World"),
            Some("hello-world".to_string())
        );
        assert_eq!(slugify_short(""), None);
    }

    #[test]
    fn parse_begin_end_with_reason_keyword() {
        let src = format!(
            "{}\npub fn x() {{}}\n{}\n",
            handwrite_begin("reason: phase-1-namespace-split - top-level cli"),
            handwrite_end()
        );
        let m = parse_handwrite_begin_end(&src);
        assert_eq!(m.len(), 1);
        assert!(m[0].id.starts_with("phase-1"));
    }

    #[test]
    fn parse_begin_end_with_xml_attrs() {
        let src = format!(
            "{}\nfoo\n{}\n",
            handwrite_begin("gap=\"missing-generator:cli\" tracker=\"none\" reason=\"the why\""),
            handwrite_end()
        );
        let m = parse_handwrite_begin_end(&src);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].id, "missing-generator:cli");
        assert_eq!(m[0].reason, "the why");
    }

    #[test]
    fn cb_fill_next_command_omits_legacy_json() {
        let marker = marker("missing-generator-cli");
        let next = next_for_marker(
            "4124",
            &marker,
            ".aw/payloads/4124/missing-generator-cli.md",
        );

        assert_eq!(
            next["command"],
            "aw cb fill 4124 --apply --marker missing-generator-cli"
        );
        assert!(!next["command"].as_str().unwrap().contains("--json"));
        assert_eq!(
            next["payload_path"],
            ".aw/payloads/4124/missing-generator-cli.md"
        );
    }

    #[test]
    fn cb_fill_initializes_marker_payload_without_overwrite() {
        let tmp = tempfile::tempdir().unwrap();
        let marker = marker("missing-generator-cli");

        let (rel, created) = initialize_marker_payload(tmp.path(), "4124", &marker).unwrap();
        assert_eq!(rel, ".aw/payloads/4124/missing-generator-cli.md");
        assert!(created);
        let abs = tmp.path().join(&rel);
        let content = std::fs::read_to_string(&abs).unwrap();
        assert!(content.contains("(fill)"));
        assert!(content.contains("missing deterministic generator"));

        std::fs::write(&abs, "custom\n").unwrap();
        let (_, created_again) = initialize_marker_payload(tmp.path(), "4124", &marker).unwrap();
        assert!(!created_again);
        assert_eq!(std::fs::read_to_string(&abs).unwrap(), "custom\n");
    }

    #[test]
    fn td_merge_next_command_uses_positional_slug() {
        assert_eq!(
            td_merge_command("4124", ".aw/tech-design/demo.md"),
            "aw td merge 4124 --spec-path .aw/tech-design/demo.md"
        );
        assert!(!td_merge_command("4124", "").contains("--json"));
    }

    #[test]
    fn replace_block_body_preserves_markers() {
        let src = format!(
            "fn before() {{}}\n{}\nstub\n{}\nfn after() {{}}\n",
            handwrite_begin("reason: x"),
            handwrite_end()
        );
        let out = replace_block_body(&src, 2, 4, "FILLED").unwrap();
        assert!(out.contains(&format!("HANDWRITE-{}", "BEGIN")));
        assert!(out.contains(&format!("HANDWRITE-{}", "END")));
        assert!(out.contains("FILLED"));
        assert!(!out.contains("stub"));
        assert!(out.contains("fn before"));
        assert!(out.contains("fn after"));
    }
}

// CODEGEN-END

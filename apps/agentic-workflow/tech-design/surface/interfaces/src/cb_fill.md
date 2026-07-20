---
id: projects-score-src-cb-fill-rs
fill_sections: [overview, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "TD/CB CLI surface manifests cover lifecycle dispatch, review, fill, and merge command behavior."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: scoped-td-fill-marker-completion
    claim: scoped-td-fill-marker-completion
    coverage: full
    rationale: "Brief and apply continuations enumerate only the active TD Changes paths, including the post-apply queue that selects code-check versus another marker."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: project-qualified-terminal-td-resolution
    claim: project-qualified-terminal-td-resolution
    coverage: full
    rationale: "Marker fill uses the same issue-owned TD resolver as generation and terminal code-check, with global discovery restricted to no-issue utility mode."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: app-lib-handwrite-marker-discovery
    claim: app-lib-handwrite-marker-discovery
    coverage: full
    rationale: "Whole-worktree marker counting treats apps, libs, and crates as independent roots, while active-TD fill enumerates exact app/lib Changes paths."
---

# Standardized apps/agentic-workflow/src/cli/cb_fill.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/cli/cb_fill.rs` generated from AST during Score force-regeneration standardization.

### Active TD marker queue

Both `aw td fill` brief mode and its `--apply` continuation derive the marker
queue from the active TD's `## Changes` paths. After an apply, re-enumeration
uses that same queue; unresolved markers owned by another app or library are
not eligible to delay this work item's code-check.

When the workflow issue exists, fill resolves its active spec from exact
`implements` ownership or the configured project-qualified default. Legacy
checkout discovery is limited to explicit no-issue utility operation.

Whole-worktree marker counting always visits app and library roots even when a
root crates tree exists, so the post-generation decision cannot report zero
markers prematurely. Active-TD fill then enumerates the exact app/lib Changes
paths and advances only after both queues are exhausted.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `HandwriteMarkerEntry` | apps/agentic-workflow/src/cli/cb_fill.rs | struct | pub | 33 |  |
| `branch_changed_files` | apps/agentic-workflow/src/cli/cb_fill.rs | function | pub | 1023 | branch_changed_files(worktree: &Path, base_branch: &str) -> HashSet<String> |
| `count_worktree_handwrite_markers` | apps/agentic-workflow/src/cli/cb_fill.rs | function | pub | 229 | count_worktree_handwrite_markers(worktree: &Path) -> usize |
| `enumerate_markers_for_scope` | apps/agentic-workflow/src/cli/cb_fill.rs | function | pub | 164 | enumerate_markers_for_scope(     worktree: &Path,     scope_paths: &[String], ) -> Vec<HandwriteMarkerEntry> |
| `enumerate_worktree_markers` | apps/agentic-workflow/src/cli/cb_fill.rs | function | pub | 119 | enumerate_worktree_markers(worktree: &Path) -> Vec<HandwriteMarkerEntry> |
| `extract_change_paths_from_spec` | apps/agentic-workflow/src/cli/cb_fill.rs | function | pub | 586 | extract_change_paths_from_spec(spec_content: &str) -> Vec<String> |
| `filter_markers_to_change_paths` | apps/agentic-workflow/src/cli/cb_fill.rs | function | pub | 646 | filter_markers_to_change_paths(     markers: &[HandwriteMarkerEntry],     change_paths: &[String], ) -> Vec<HandwriteMarkerEntry> |
| `run` | apps/agentic-workflow/src/cli/cb_fill.rs | function | pub | 427 | run(args: CbFillArgs) -> Result<()> |
| `scope_markers_for_change_paths` | apps/agentic-workflow/src/cli/cb_fill.rs | function | pub | 688 | scope_markers_for_change_paths(     markers: &[HandwriteMarkerEntry],     change_paths: Option<&[String]>, ) -> Vec<HandwriteMarkerEntry> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/src/cli/cb_fill.rs -->
````rust
// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
// CODEGEN-BEGIN
//! `aw td fill` — Phase 3 marker-fill workflow.
//!
//! Two modes:
//! - **Brief** (no `--apply`): walk the current checkout source tree and emit a
//!   marker-list dispatch envelope for mainthread,
//!   or fast-path-dispatch directly to `aw td code-check` when zero markers
//!   are present (R11).
//! - **Apply** (`--apply --marker <id>`): merge the expected marker payload
//!   into the HANDWRITE block matching `<id>`, commit that marker with WI
//!   projection trailers, then lock the next marker or dispatch
//!   `aw td code-check`.
//!
//! @spec apps/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md

use crate::generate::audit::parse_handwrite_markers;
use crate::issues::{IssueBackend, IssuePatch, LocalBackend};
use anyhow::{Context, Result};
use globset::{Glob, GlobSetBuilder};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use crate::cli::cb::CbFillArgs;
use crate::cli::remote_push::maybe_push_remote;

const ADOPT_EXISTING_PAYLOAD: &str = "<!-- aw:adopt-existing -->";

// A single open HANDWRITE block discovered in the worktree.
///
// Spec name: `HandwriteMarkerEntry`.
// @spec apps/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#schema
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
    /// True only for an XML marker generator-scaffolded around a real,
    /// pre-existing implementation. Its fill payload adopts that body instead
    /// of replacing it with a generic placeholder.
    #[serde(skip)]
    pub adopt_existing: bool,
}

// Extract every unfilled HANDWRITE marker from a single file's content,
// pushing repo-root-relative entries onto `out`. Shared by
// `enumerate_worktree_markers` (whole-worktree walk) and
// `enumerate_markers_for_scope` (issue #859 part a — walk only the
// caller's scope paths) so both enumerate identically per file.
fn collect_markers_from_file(worktree: &Path, path: &Path, out: &mut Vec<HandwriteMarkerEntry>) {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if !matches!(
        ext,
        "rs" | "py"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "css"
            | "scss"
            | "md"
            | "html"
            | "toml"
            | "json"
            | "yaml"
            | "yml"
    ) && file_name != "Dockerfile"
    {
        return;
    }
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };

    // Form 1: <HANDWRITE>...</HANDWRITE> (canonical, parsed by
    // crate::generate::audit::parse_handwrite_markers).
    let path_str = path.to_string_lossy().to_string();
    if let Ok(markers) = parse_handwrite_markers(&content, &path_str) {
        for m in markers {
            let adopt_existing = m.tracker == crate::generate::handwrite_scaffold::PENDING_TRACKER
                && !marker_body_is_unfilled(&content, m.line_start, m.line_end);
            if !adopt_existing && !marker_body_is_unfilled(&content, m.line_start, m.line_end) {
                continue;
            }
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
                adopt_existing,
            });
        }
    }

    // Form 2: comment-style begin/end markers emitted by
    // `crate::generate::apply::scaffold_handwrite_file`.
    for m in parse_handwrite_begin_end(&content) {
        if !marker_body_is_unfilled(&content, m.start_line, m.end_line) {
            continue;
        }
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
            adopt_existing: false,
        });
    }
}

// Walk the worktree source tree (under `apps/`, `libs/`, `crates/`,
// `projects/`, `src/`, `tests/`) and return every open HANDWRITE block.
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
pub fn enumerate_worktree_markers(worktree: &Path) -> Vec<HandwriteMarkerEntry> {
    let mut out: Vec<HandwriteMarkerEntry> = Vec::new();
    let candidate_subdirs = ["apps", "libs", "crates", "projects", "src", "tests"];

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
            collect_markers_from_file(worktree, entry.path(), &mut out);
        }
    }

    out
}

// Enumerate HANDWRITE markers restricted to `scope_paths` (issue #859 part
// a): each entry may name a single file (only that file is read) or a
// directory (only that subtree is walked), repo-root-relative to
// `worktree`. Falls back to the whole-worktree walk
// (`enumerate_worktree_markers`) when any scope entry is a glob pattern
// (`*`, `?`, `[`) — bounding a directory walk to an arbitrary glob isn't a
// well-defined subset, and glob scope entries are rare in practice (a
// branch diff's file list and a TD's literal `## Changes` paths are both
// ordinarily plain paths).
///
// This replaces the ~53k-file monorepo-wide walk `run_cb_check_gate_scoped`
// used to perform (over `crates/`, `projects/`, `src/`, `tests/`) before
// intersecting the result with the branch diff / WI scope — the walk itself
// is now bounded to exactly the caller's scope paths, not just the result.
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
pub fn enumerate_markers_for_scope(
    worktree: &Path,
    scope_paths: &[String],
) -> Vec<HandwriteMarkerEntry> {
    if scope_paths
        .iter()
        .any(|p| p.contains('*') || p.contains('?') || p.contains('['))
    {
        return enumerate_worktree_markers(worktree);
    }

    let mut out: Vec<HandwriteMarkerEntry> = Vec::new();
    let mut visited: HashSet<PathBuf> = HashSet::new();
    for raw in scope_paths {
        let normalized = normalize_rel_path(raw);
        if normalized.is_empty() {
            continue;
        }
        let abs = worktree.join(&normalized);
        if abs.is_file() {
            if visited.insert(abs.clone()) {
                collect_markers_from_file(worktree, &abs, &mut out);
            }
        } else if abs.is_dir() {
            for entry in walkdir::WalkDir::new(&abs)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path().to_path_buf();
                if visited.insert(path.clone()) {
                    collect_markers_from_file(worktree, &path, &mut out);
                }
            }
        }
        // Else: scope entry doesn't exist in this worktree (deleted file,
        // stale scope entry, ...) — nothing to scan for it.
    }
    out
}

fn marker_body_is_unfilled(content: &str, start_line: usize, end_line: usize) -> bool {
    if start_line == 0 || end_line <= start_line {
        return true;
    }
    let lines: Vec<&str> = content.lines().collect();
    if end_line > lines.len() {
        return true;
    }
    let body = lines[start_line..end_line - 1].join("\n");
    let body = body.trim();
    if body.is_empty() {
        return true;
    }
    let lower = body.to_ascii_lowercase();
    lower.contains("todo: hand-write content")
        || lower.contains("todo hand-write content")
        || lower == "(fill)"
}

// Lightweight count of HANDWRITE markers in the worktree. Used by
// `td.rs::run_gen_code` for the post-codegen R8/R11 dispatch decision.
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
pub fn count_worktree_handwrite_markers(worktree: &Path) -> usize {
    enumerate_worktree_markers(worktree).len()
}

fn cb_marker_payload_path(project_root: &Path, slug: &str, marker_id: &str) -> PathBuf {
    crate::shared::workspace::payloads_path(project_root)
        .join(slug)
        .join(format!("{marker_id}.md"))
}

fn cb_fill_apply_command(slug: &str, marker_id: &str) -> String {
    format!("aw td fill {} --apply --marker {}", slug, marker_id)
}

fn td_code_check_command(slug: &str) -> String {
    format!("aw td code-check {slug}")
}

fn marker_payload_template(marker: &HandwriteMarkerEntry) -> String {
    if marker.adopt_existing {
        return format!("{ADOPT_EXISTING_PAYLOAD}\n");
    }
    format!(
        "(fill)\n\n<!-- marker: {} path: {} reason: {} -->\n",
        marker.id, marker.source_path, marker.reason
    )
}

fn initialize_marker_payload(
    project_root: &Path,
    slug: &str,
    marker: &HandwriteMarkerEntry,
) -> Result<(String, bool)> {
    let abs = cb_marker_payload_path(project_root, slug, &marker.id);
    if abs.exists() {
        return Ok((abs.to_string_lossy().into_owned(), false));
    }
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create payload directory {}", parent.display()))?;
    }
    std::fs::write(&abs, marker_payload_template(marker))
        .with_context(|| format!("failed to write payload {}", abs.display()))?;
    Ok((abs.to_string_lossy().into_owned(), true))
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

fn next_for_td_code_check(slug: &str) -> serde_json::Value {
    serde_json::json!({
        "kind": "dispatch",
        "command": td_code_check_command(slug),
        "reason": "all HANDWRITE markers are filled; run the terminal code-check action",
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

        // A marker token inside a Rust/TypeScript/Python string literal is
        // fixture data, not a source-ownership marker. `strip_lead` removes
        // supported comment prefixes, so a real comment-style marker starts
        // with the token while `format!("// HANDWRITE-BEGIN ...")` does not.
        if body.starts_with(HANDWRITE_BEGIN_TOKEN) {
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
        if body.starts_with(HANDWRITE_END_TOKEN) {
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
    for prefix in ["///", "//!", "//", "# ", "#", "<!--", "/*"] {
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
    let worktree_abs = crate::cli::td::td_workspace_path(&project_root, &slug);
    if !worktree_abs.exists() {
        emit_error(
            &slug,
            &format!("workspace not found: {}", worktree_abs.display()),
        )?;
        std::process::exit(2);
    }

    // Look up the spec_path from the explicit CLI arg, issue frontmatter, or
    // the unique TD spec touched by this branch. If none is available, preserve
    // the legacy all-marker behavior.
    let backend = LocalBackend::from_project_root(&worktree_abs);
    let issue = backend.get(&slug).await.ok().flatten();
    let (markers, change_paths, spec_path) =
        match markers_for_active_td(&args, issue.as_ref(), &worktree_abs) {
            Ok(queue) => queue,
            Err(e) => {
                emit_error(&slug, &e.to_string())?;
                std::process::exit(2);
            }
        };
    let spec_path = spec_path.unwrap_or_default();
    let allowed_dirty_paths = brief_allowed_dirty_paths(&markers, change_paths.as_deref());
    crate::cli::td::td_activate_inplace_allowing_dirty_lifecycle_paths(
        &project_root,
        &slug,
        &allowed_dirty_paths,
    )?;

    if markers.is_empty() {
        // A TD may legitimately generate no HANDWRITE blocks, or a caller may
        // re-enter after the blocks were already filled but before the phase
        // transition was recorded. Verify the scoped marker gate and record
        // the normal post-fill phase rather than stranding the WI at
        // `cb_genned` with no legal `--apply --marker` command.
        if issue
            .as_ref()
            .is_some_and(|issue| marker_free_fill_can_commit_evidence(issue.phase.as_deref()))
        {
            let scope = change_paths.as_deref().unwrap_or_default();
            if let Err(message) = run_cb_check_gate_scoped(&worktree_abs, scope).await {
                emit_error(
                    &slug,
                    &format!("cannot reconcile filled markers: {message}"),
                )?;
                std::process::exit(1);
            }
            let issue = issue
                .as_ref()
                .expect("post-gen phase requires an issue projection");
            backend
                .update(
                    &slug,
                    &IssuePatch {
                        phase: Some(crate::issues::types::td_phase::CB_FILLED.to_string()),
                        ..Default::default()
                    },
                )
                .await?;
            let issue_path = backend.issue_path(issue);
            let issue_path_s = issue_path.to_string_lossy().into_owned();
            if let Err(error) = stage_and_commit_cb_fill(&worktree_abs, &slug, &issue_path_s) {
                emit_error(&slug, &format!("git commit failed: {error}"))?;
                std::process::exit(1);
            }
            crate::cli::workflow_guard::complete_issue_lock(&worktree_abs, &slug, "td").await?;
        }
        // 0-marker fast-path: dispatch directly to terminal code-check.
        let env = serde_json::json!({
            "action": "dispatch",
            "agent": serde_json::Value::Null,
            "slug": slug,
            "next": next_for_td_code_check(&slug),
            "invoke": {
                "command": "aw td code-check",
                "args": { "target": slug },
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
                "td",
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
            "command": "aw td fill",
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

/// While a marker is pending, brief mode must begin from a clean tree so its
/// payload dispatch names an unambiguous source snapshot. Once the active TD
/// has no markers left, its declared Changes paths are the only legitimate
/// marker-free evidence that may be staged by the terminal Cb-Fill commit.
fn brief_allowed_dirty_paths<'a>(
    markers: &[HandwriteMarkerEntry],
    change_paths: Option<&'a [String]>,
) -> Vec<&'a str> {
    if markers.is_empty() {
        return change_paths
            .unwrap_or_default()
            .iter()
            .map(String::as_str)
            .collect();
    }
    Vec::new()
}

fn marker_free_fill_can_commit_evidence(phase: Option<&str>) -> bool {
    phase.is_some_and(|phase| {
        crate::issues::types::td_phase::is_post_gen(phase)
            || phase == crate::issues::types::td_phase::CB_FILLED
    })
}

fn resolve_active_spec_path(
    args: &CbFillArgs,
    issue: Option<&crate::issues::Issue>,
    worktree_abs: &Path,
) -> Result<Option<String>> {
    if let Some(explicit) = args.spec_path.clone().filter(|p| !p.is_empty()) {
        return Ok(Some(explicit));
    }
    if let Some(issue) = issue {
        let paths = crate::cli::td::resolve_issue_td_spec_paths(worktree_abs, issue, &args.slug)?;
        return match paths.as_slice() {
            [path] => Ok(Some(path.clone())),
            _ => anyhow::bail!(
                "issue '{}' owns multiple TD specs ({}); rerun aw td fill with --spec-path",
                args.slug,
                paths.join(", ")
            ),
        };
    }
    // Legacy no-issue utility mode only. An existing WI must never inherit a
    // foreign project TD from checkout-global branch discovery (#1679).
    Ok(crate::cli::td::discover_worktree_spec(worktree_abs))
}

// Resolve the active TD and enumerate only the HANDWRITE markers it owns.
// Both brief and apply mode use this queue so that applying a local marker
// cannot reintroduce an unrelated app or library marker on the next step.
fn markers_for_active_td(
    args: &CbFillArgs,
    issue: Option<&crate::issues::Issue>,
    worktree_abs: &Path,
) -> Result<(
    Vec<HandwriteMarkerEntry>,
    Option<Vec<String>>,
    Option<String>,
)> {
    let spec_path = resolve_active_spec_path(args, issue, worktree_abs)?;
    let markers_and_changes = match spec_path.as_deref().filter(|path| !path.is_empty()) {
        Some(path) => {
            let spec_abs = worktree_abs.join(path);
            let spec_content = std::fs::read_to_string(&spec_abs)
                .with_context(|| format!("spec_path not readable at {}", spec_abs.display()))?;
            let change_paths = extract_change_paths_from_spec(&spec_content);
            (
                markers_for_td_changes(worktree_abs, Some(&change_paths)),
                Some(change_paths),
            )
        }
        None => (markers_for_td_changes(worktree_abs, None), None),
    };
    Ok((
        disambiguate_marker_ids(markers_and_changes.0),
        markers_and_changes.1,
        spec_path,
    ))
}

/// Fill payload paths are keyed by marker id, while the gap remains the shared
/// generator-taxonomy value. Scope-local duplicate gaps therefore need a
/// deterministic location suffix before the queue can route them one at a
/// time.
fn disambiguate_marker_ids(mut markers: Vec<HandwriteMarkerEntry>) -> Vec<HandwriteMarkerEntry> {
    let mut counts = BTreeMap::<String, usize>::new();
    for marker in &markers {
        *counts.entry(marker.id.clone()).or_default() += 1;
    }
    for marker in &mut markers {
        if counts.get(&marker.id).copied().unwrap_or_default() > 1 {
            marker.id = format!("{}--{}", marker.id, marker_location_hash(marker));
        }
    }
    markers
}

fn marker_location_hash(marker: &HandwriteMarkerEntry) -> String {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in marker
        .source_path
        .bytes()
        .chain(b":".iter().copied())
        .chain(marker.start_line.to_string().bytes())
    {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    format!("{hash:08x}")
}

// Extract repo-relative path entries from a TD `## Changes` YAML block.
///
// Supports both `changes:` and `files:` sequence keys and accepts either
// `path:` or `file:` per entry for compatibility with older specs.
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
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
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
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
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md#source
pub fn scope_markers_for_change_paths(
    markers: &[HandwriteMarkerEntry],
    change_paths: Option<&[String]>,
) -> Vec<HandwriteMarkerEntry> {
    match change_paths {
        Some(paths) => filter_markers_to_change_paths(markers, paths),
        None => markers.to_vec(),
    }
}

/// Select unfilled markers for a resolved TD Changes plan. Canonical TDs use
/// monorepo paths such as `apps/tape/src/push.rs`; enumerate those paths
/// directly instead of first walking the legacy root shortlist and filtering
/// its result. This keeps app/lib targets both discoverable and bounded.
fn markers_for_td_changes(
    worktree: &Path,
    change_paths: Option<&[String]>,
) -> Vec<HandwriteMarkerEntry> {
    match change_paths {
        Some(paths) => enumerate_markers_for_scope(worktree, paths),
        None => enumerate_worktree_markers(worktree),
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
    let marker_id = match args.marker.as_deref() {
        Some(m) if !m.is_empty() => m.to_string(),
        _ => {
            emit_error(&slug, "--apply requires --marker <id>")?;
            std::process::exit(2);
        }
    };
    let project_root = crate::find_project_root()?;
    let worktree_abs = crate::cli::td::td_workspace_path(&project_root, &slug);

    if !worktree_abs.exists() {
        emit_error(
            &slug,
            &format!("workspace not found: {}", worktree_abs.display()),
        )?;
        std::process::exit(2);
    }

    let backend = LocalBackend::from_project_root(&worktree_abs);
    let issue = backend.get(&slug).await?;

    // Locate the marker in the active TD's source scope. R5
    // (bug-cb-fill-payload-routes-by-marker-id-alone-collides): when
    // multiple markers share an id (e.g. legacy markers emitted before
    // the R1 scaffold disambiguator landed), surface the collision as a
    // hard error instead of silently routing to the alphabetically-first
    // match. Callers must rebuild the marker list (which now uses the
    // R1-disambiguated ids) and re-dispatch with the correct id.
    // @spec apps/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#logic-resolve_marker_file
    let (markers, _, _) = match markers_for_active_td(&args, issue.as_ref(), &worktree_abs) {
        Ok(queue) => queue,
        Err(e) => {
            emit_error(&slug, &e.to_string())?;
            std::process::exit(2);
        }
    };
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
                     Re-run `aw td fill` (no --apply) to get the disambiguated marker list.",
                    marker_id,
                    many.len(),
                    paths.join(", "),
                ),
            )?;
            std::process::exit(2);
        }
    };

    // The active marker's file is the only source path a fill application may
    // carry: an adoption payload must preserve its existing body while
    // committing the author's bounded implementation edit and tracker update
    // together. All unrelated dirty paths remain a hard preflight failure.
    crate::cli::td::td_activate_inplace_allowing_dirty_lifecycle_paths(
        &project_root,
        &slug,
        &[target.source_path.as_str()],
    )?;

    // Read the payload.
    let payload_abs = cb_marker_payload_path(&project_root, &slug, &marker_id);
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

    // Replace a generated empty marker, or explicitly adopt a generated XML
    // marker that already wraps existing implementation.
    let source_abs = worktree_abs.join(&target.source_path);
    let original = std::fs::read_to_string(&source_abs)
        .with_context(|| format!("reading source {}", source_abs.display()))?;
    let new_content = apply_marker_payload(&original, &target, &payload_body, &slug)?;
    std::fs::write(&source_abs, &new_content)
        .with_context(|| format!("writing source {}", source_abs.display()))?;
    // Re-enumerate against the same active TD scope. A foreign marker must
    // not block this work item's terminal code-check.
    let (remaining, _, _) = match markers_for_active_td(&args, issue.as_ref(), &worktree_abs) {
        Ok(queue) => queue,
        Err(e) => {
            emit_error(&slug, &e.to_string())?;
            std::process::exit(2);
        }
    };

    if !remaining.is_empty() {
        let next = &remaining[0];
        let (next_payload, next_payload_created) =
            initialize_marker_payload(&worktree_abs, &slug, next)?;
        crate::cli::workflow_guard::create_issue_lock(
            &worktree_abs,
            &crate::cli::workflow_guard::TransitionLock::new(
                &slug,
                "td",
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
        let issue = issue
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("issue '{}' not found in current checkout", slug))?;
        let issue_path_s = backend.issue_path(issue).to_string_lossy().into_owned();
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
                "command": "aw td fill",
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

    // All active-TD markers are filled. `remaining` above has already
    // re-enumerated that TD's declared Changes paths, so terminal code-check
    // can validate the same bounded scope without a foreign marker changing
    // this work item's lifecycle.

    // Commit Cb-Fill trailer + advance phase.
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
    crate::cli::workflow_guard::complete_issue_lock(&worktree_abs, &slug, "td").await?;

    // Dispatch to terminal code-check after the local code gate.
    // Capability/EC/health gates decide whether another iteration is needed.
    let env = serde_json::json!({
        "action": "dispatch",
        "agent": serde_json::Value::Null,
        "slug": slug,
        "next": next_for_td_code_check(&slug),
        "invoke": {
            "command": "aw td code-check",
            "args": { "target": slug },
        },
    });
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

fn replace_block_body_for_path(
    src: &str,
    start_line: usize,
    end_line: usize,
    payload: &str,
    source_path: &str,
) -> Option<String> {
    if should_preserve_handwrite_markers(source_path) {
        return replace_block_body(src, start_line, end_line, payload);
    }
    replace_block_and_markers(src, start_line, end_line, payload)
}

/// XML-form markers scaffolded around existing source carry the pending
/// sentinel until their first fill. Promote that sentinel to the work-item
/// reference after a successful body replacement so the marker leaves the
/// pending queue and becomes valid managed-source ownership.
fn mark_pending_xml_marker_filled(src: &str, start_line: usize, slug: &str) -> String {
    let mut lines: Vec<String> = src.lines().map(str::to_string).collect();
    let Some(line) = lines.get_mut(start_line.saturating_sub(1)) else {
        return src.to_string();
    };
    if line.contains("<HANDWRITE")
        && line.contains(&format!(
            "tracker=\"{}\"",
            crate::generate::handwrite_scaffold::PENDING_TRACKER
        ))
    {
        *line = line.replacen(
            &format!(
                "tracker=\"{}\"",
                crate::generate::handwrite_scaffold::PENDING_TRACKER
            ),
            &format!("tracker=\"#{slug}\""),
            1,
        );
        let mut normalized = lines.join("\n");
        if src.ends_with('\n') {
            normalized.push('\n');
        }
        return normalized;
    }
    src.to_string()
}

fn pending_xml_marker_has_existing_body(src: &str, start_line: usize, end_line: usize) -> bool {
    let Some(line) = src.lines().nth(start_line.saturating_sub(1)) else {
        return false;
    };
    line.contains("<HANDWRITE")
        && line.contains(&format!(
            "tracker=\"{}\"",
            crate::generate::handwrite_scaffold::PENDING_TRACKER
        ))
        && !marker_body_is_unfilled(src, start_line, end_line)
}

fn apply_marker_payload(
    original: &str,
    target: &HandwriteMarkerEntry,
    payload: &str,
    slug: &str,
) -> Result<String> {
    if payload.trim() == ADOPT_EXISTING_PAYLOAD {
        if !pending_xml_marker_has_existing_body(original, target.start_line, target.end_line) {
            anyhow::bail!(
                "{} may only adopt a pending XML marker that already contains implementation",
                ADOPT_EXISTING_PAYLOAD
            );
        }
        return Ok(mark_pending_xml_marker_filled(
            original,
            target.start_line,
            slug,
        ));
    }
    let replaced = replace_block_body_for_path(
        original,
        target.start_line,
        target.end_line,
        payload,
        &target.source_path,
    )
    .ok_or_else(|| {
        anyhow::anyhow!(
            "could not locate marker block at lines {}..{} in {}",
            target.start_line,
            target.end_line,
            target.source_path
        )
    })?;
    Ok(mark_pending_xml_marker_filled(
        &replaced,
        target.start_line,
        slug,
    ))
}

fn should_preserve_handwrite_markers(source_path: &str) -> bool {
    let path = Path::new(source_path);
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name == "Dockerfile" {
        return false;
    }
    !matches!(
        path.extension().and_then(|e| e.to_str()).unwrap_or(""),
        "html" | "json" | "toml" | "yaml" | "yml"
    )
}

fn replace_block_and_markers(
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

    let before = &lines[..start_line - 1];
    let after = &lines[end_line..];
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
// @spec apps/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#logic
fn resolve_base_branch() -> String {
    std::env::var("SCORE_CB_FILL_BASE_BRANCH").unwrap_or_else(|_| "main".to_string())
}

// Resolve the concrete ref to diff `HEAD` against for a given base branch
// name. The rebase-landing recipe (squash-merge to `origin/<base>` +
// `git fetch origin <base>` + `git rebase origin/<base>`) advances the
// remote-tracking ref but leaves the local `<base_branch>` ref exactly
// where it was before the fetch — only an explicit `git pull`/checkout of
// that local branch would move it, and the recipe never does that on a
// long-lived work-area branch. A three-dot diff against that stale local
// ref then re-walks every commit that landed on `<base_branch>` since the
// last local sync and misattributes it to this branch (issue #1423).
// Prefer the remote-tracking ref (`origin/<base_branch>`) whenever it
// resolves, since the rebase-landing recipe always fetches it immediately
// before rebasing; fall back to the bare local branch name for repos with
// no `origin` remote (fixtures, detached/standalone clones), preserving
// prior behaviour there.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#logic
fn resolve_diff_base_ref(git_bin: &Path, worktree: &Path, base_branch: &str) -> String {
    let remote_ref = format!("origin/{base_branch}");
    let remote_resolves = std::process::Command::new(git_bin)
        .arg("-C")
        .arg(worktree)
        .args([
            "rev-parse",
            "--verify",
            "-q",
            &format!("refs/remotes/{remote_ref}"),
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if remote_resolves {
        remote_ref
    } else {
        base_branch.to_string()
    }
}

// Files changed by the worktree branch relative to its base. Returns
// repo-root-relative paths (matching `HandwriteMarkerEntry.source_path`).
///
// Empty result on git failure — the caller treats that as "no changes
// to gate against" and the gate falls through to the legacy
// whole-worktree check, preserving the prior behaviour for non-branch
// invocations (e.g. detached HEAD or first commit).
///
// @spec apps/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#logic
pub fn branch_changed_files(worktree: &Path, base_branch: &str) -> HashSet<String> {
    let git_bin = match crate::git::find_git_bin() {
        Some(g) => g,
        None => return HashSet::new(),
    };
    let diff_base = resolve_diff_base_ref(&git_bin, worktree, base_branch);
    let out = match std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree)
        .args(["diff", "--name-only", &format!("{diff_base}...HEAD")])
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

// Resolve the WI's touched file set: the union of the branch diff against
// base (`branch_changed_files`) and `extra_scope` (typically a WI's own TD
// `## Changes` paths), sorted and deduplicated. Repo-root-relative,
// matching both `HandwriteMarkerEntry.source_path` and the paths a
// managed-inventory scan reports.
///
// Issue #932 (touched-scope standardization gate): extracted from
// `run_cb_check_gate_scoped` so a second caller (the terminal
// standardization check in `cb.rs`) can consume the same touched-file set
// the marker gate computes, without re-deriving the branch-diff ∪
// Changes-paths union itself. `run_cb_check_gate_scoped` below now calls
// this instead of inlining the union.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#logic
pub(crate) fn resolve_touched_scope(worktree_abs: &Path, extra_scope: &[String]) -> Vec<String> {
    let base = resolve_base_branch();
    let changed = branch_changed_files(worktree_abs, &base);

    let mut scope: Vec<String> = changed.into_iter().collect();
    scope.extend(extra_scope.iter().cloned());
    scope.sort();
    scope.dedup();
    scope
}

// Run code-check semantics against the worktree as a gate. Returns Ok(())
// when no slug-introduced markers remain, Err(msg) on findings or
// invocation error.
///
// Slug-scoping (R1, R2, R4): only HANDWRITE markers in files modified
// between the worktree branch and its base count toward the gate.
// Markers inherited from `main` (other unmerged refactors) do not fail
// this gate even though they remain in the worktree. Greenfield
// worktrees with no diff against base trivially pass (R5).
///
// Callers always go through this scoped entry point (with an empty
// `extra_scope` when no additional WI-scope paths apply) — issue #859
// removed the last caller of an unscoped `run_cb_check_gate` wrapper, since
// a wrapper that hard-codes `extra_scope: &[]` added no behavior
// `run_cb_check_gate_scoped(w, &[])` doesn't already provide directly.
///
// The gate's scope is the union of the branch diff (`changed`) and
// `extra_scope`. When that union is empty — no branch diff AND no WI Changes
// entries to check against — the gate passes vacuously: a docs-only WI (or
// one whose branch diff is unresolvable) must not be blocked by markers
// inherited from unrelated, unmerged work elsewhere in the tree.
///
// Issue #859 part a: the scope union is computed *before* any marker
// enumeration, and when it's empty the gate returns without walking the
// worktree at all. When non-empty, only `enumerate_markers_for_scope` (a
// walk bounded to the scope paths themselves) runs — not the whole-tree
// `enumerate_worktree_markers` walk this function used to unconditionally
// perform up front, which then discarded everything outside `scope` anyway.
///
// @spec apps/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#logic
pub(crate) async fn run_cb_check_gate_scoped(
    worktree_abs: &Path,
    extra_scope: &[String],
) -> std::result::Result<(), String> {
    let scope = resolve_touched_scope(worktree_abs, extra_scope);

    if scope.is_empty() {
        // No branch diff against base and no WI-scope file list to check
        // against — nothing identifies this worktree's own markers, so
        // markers inherited from other unmerged work must not block
        // (issue #854). Nothing to enumerate either (issue #859 part a).
        return Ok(());
    }

    let remaining = enumerate_markers_for_scope(worktree_abs, &scope);
    if remaining.is_empty() {
        return Ok(());
    }

    let slug_markers = filter_markers_to_change_paths(&remaining, &scope);
    if !slug_markers.is_empty() {
        let names: Vec<&str> = slug_markers
            .iter()
            .map(|m| m.source_path.as_str())
            .collect();
        return Err(format!(
            "{} HANDWRITE marker(s) introduced by this branch still present after fill (\
             {} inherited markers ignored): {}",
            slug_markers.len(),
            remaining.len() - slug_markers.len(),
            names.join(", "),
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
         Next-Command: aw td fill {slug} --apply --marker {next_marker_id}",
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
         Next-Command: aw td fill {slug} --apply --marker {first_marker_id}",
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
            adopt_existing: false,
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
    fn marker_free_brief_allows_only_declared_evidence_paths() {
        let paths = vec![
            "apps/pgpool/tests/connection_discovery.rs".to_string(),
            "apps/pgpool/tech-design/semantic/discovery.md".to_string(),
        ];
        assert_eq!(
            brief_allowed_dirty_paths(&[], Some(&paths)),
            vec![
                "apps/pgpool/tests/connection_discovery.rs",
                "apps/pgpool/tech-design/semantic/discovery.md",
            ]
        );
        assert!(brief_allowed_dirty_paths(&[marker("pending")], Some(&paths)).is_empty());
    }

    #[test]
    fn marker_free_fill_reenters_for_the_terminal_cb_filled_phase() {
        assert!(marker_free_fill_can_commit_evidence(Some("cb_genned")));
        assert!(marker_free_fill_can_commit_evidence(Some("cb_filled")));
        assert!(!marker_free_fill_can_commit_evidence(Some(
            "td_contract_in_progress"
        )));
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
    fn parse_begin_end_ignores_marker_tokens_inside_string_literals() {
        let src = concat!(
            "let fixture = \"// HANDWRITE-BEGIN gap=\\\"fixture\\\" reason=\\\"unfilled\\\"\\n\",\n",
            "let end = \"// HANDWRITE-END\\n\";\n",
        );
        assert!(parse_handwrite_begin_end(src).is_empty());
    }

    #[test]
    fn enumerate_worktree_markers_skips_filled_handwrite_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let filled = format!(
            "{}\nexport function App() {{ return null; }}\n{}\n",
            handwrite_begin("gap=\"missing-generator:component\" reason=\"fill app\""),
            handwrite_end(),
        );
        std::fs::write(src_dir.join("App.tsx"), filled).unwrap();

        assert!(enumerate_worktree_markers(tmp.path()).is_empty());

        let unfilled = format!(
            "{}\n// TODO: hand-write content for `src/App.tsx`.\n{}\n",
            handwrite_begin("gap=\"missing-generator:component\" reason=\"fill app\""),
            handwrite_end(),
        );
        std::fs::write(src_dir.join("App.tsx"), unfilled).unwrap();
        let markers = enumerate_worktree_markers(tmp.path());
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].id, "missing-generator:component");
    }

    #[test]
    fn pending_xml_handwrite_marker_is_queued_despite_existing_body() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let source = "// <HANDWRITE gap=\"missing-generator:logic\" tracker=\"pending-tracker\" reason=\"fixture\">\n\
pub fn existing() {}\n\
// </HANDWRITE>\n";
        let source_path = src_dir.join("demo.rs");
        std::fs::write(&source_path, source).unwrap();

        let markers = enumerate_worktree_markers(tmp.path());
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].id, "missing-generator:logic");

        let filled = mark_pending_xml_marker_filled(source, 1, "1882");
        std::fs::write(&source_path, filled).unwrap();
        assert!(enumerate_worktree_markers(tmp.path()).is_empty());
    }

    #[test]
    fn pending_xml_existing_body_initializes_an_adopt_payload() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("demo.rs"),
            "// <HANDWRITE gap=\"missing-generator:logic\" tracker=\"pending-tracker\" reason=\"fixture\">\n\
pub fn existing() {}\n\
// </HANDWRITE>\n",
        )
        .unwrap();

        let marker = enumerate_worktree_markers(tmp.path()).pop().unwrap();
        assert!(marker.adopt_existing);
        let (path, created) = initialize_marker_payload(tmp.path(), "4124", &marker).unwrap();
        assert!(created);
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            format!("{ADOPT_EXISTING_PAYLOAD}\n")
        );
    }

    #[test]
    fn adopt_existing_payload_preserves_body_and_binds_tracker() {
        let src = "// <HANDWRITE gap=\"missing-generator:logic\" tracker=\"pending-tracker\" reason=\"fixture\">\n\
pub fn existing() { 42; }\n\
// </HANDWRITE>\n";
        let target = HandwriteMarkerEntry {
            id: "missing-generator:logic".to_string(),
            source_path: "src/demo.rs".to_string(),
            start_line: 1,
            end_line: 3,
            reason: "fixture".to_string(),
            spec_ref: None,
            adopt_existing: true,
        };
        let filled = apply_marker_payload(src, &target, ADOPT_EXISTING_PAYLOAD, "1882").unwrap();
        assert!(filled.contains("pub fn existing() { 42; }"));
        assert!(filled.contains("tracker=\"#1882\""));
        assert!(filled.contains("</HANDWRITE>"));
    }

    #[test]
    fn xml_handwrite_marker_ids_are_disambiguated_within_one_queue() {
        let markers = vec![
            HandwriteMarkerEntry {
                id: "missing-generator:logic".to_string(),
                source_path: "apps/pgpool/src/a.rs".to_string(),
                start_line: 10,
                end_line: 12,
                reason: "a".to_string(),
                spec_ref: None,
                adopt_existing: false,
            },
            HandwriteMarkerEntry {
                id: "missing-generator:logic".to_string(),
                source_path: "apps/pgpool/src/b.rs".to_string(),
                start_line: 20,
                end_line: 22,
                reason: "b".to_string(),
                spec_ref: None,
                adopt_existing: false,
            },
        ];

        let ids = disambiguate_marker_ids(markers)
            .into_iter()
            .map(|marker| marker.id)
            .collect::<Vec<_>>();
        assert_ne!(ids[0], ids[1]);
        assert!(ids
            .iter()
            .all(|id| id.starts_with("missing-generator:logic--")));
    }

    #[test]
    fn enumerate_worktree_markers_includes_config_artifact_files() {
        let tmp = tempfile::tempdir().unwrap();
        let files = [
            "frontend/index.html",
            "frontend/package.json",
            "backend/pyproject.toml",
            "k8s/base/backend-deployment.yaml",
            "backend/Dockerfile",
        ];
        for file in files {
            let path = tmp.path().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let body = format!(
                "{}\n// TODO: hand-write content for `{}`.\n{}\n",
                handwrite_begin("gap=\"missing-generator:config\" reason=\"fill config\""),
                file,
                handwrite_end(),
            );
            std::fs::write(path, body).unwrap();
        }

        let markers = enumerate_worktree_markers(tmp.path());
        let paths: HashSet<String> = markers.into_iter().map(|m| m.source_path).collect();
        assert_eq!(paths.len(), files.len());
        for file in files {
            assert!(paths.contains(file), "missing marker for {file}");
        }
    }

    #[test]
    fn canonical_td_changes_path_queues_app_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("apps/tape/src/push.rs");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(
            &source,
            format!(
                "{}\n// TODO: hand-write content for `apps/tape/src/push.rs`.\n{}\n",
                handwrite_begin("gap=\"missing-generator:push\" reason=\"push worker\""),
                handwrite_end(),
            ),
        )
        .unwrap();

        let changes = vec!["apps/tape/src/push.rs".to_string()];
        let markers = markers_for_td_changes(tmp.path(), Some(&changes));
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].source_path, "apps/tape/src/push.rs");
        assert_eq!(markers[0].id, "missing-generator:push");
    }

    #[test]
    fn cb_fill_apply_scopes_remaining_markers_to_active_changes() {
        let tmp = tempfile::tempdir().unwrap();
        let local_path = "apps/local/src/peer_tls.rs";
        let foreign_path = "apps/foreign/src/fixture.rs";
        for (path, gap) in [
            (local_path, "missing-generator:local"),
            (foreign_path, "missing-generator:foreign"),
        ] {
            let source = tmp.path().join(path);
            std::fs::create_dir_all(source.parent().unwrap()).unwrap();
            std::fs::write(
                source,
                format!(
                    "{}\n// TODO: hand-write content for `{path}`.\n{}\n",
                    handwrite_begin(&format!("gap=\"{gap}\" reason=\"scope test\"")),
                    handwrite_end(),
                ),
            )
            .unwrap();
        }
        let spec_path = "apps/local/tech-design/logic/local-marker.md";
        let spec = tmp.path().join(spec_path);
        std::fs::create_dir_all(spec.parent().unwrap()).unwrap();
        std::fs::write(
            &spec,
            format!("## Changes\n```yaml\nchanges:\n  - path: {local_path}\n```\n"),
        )
        .unwrap();
        let args = CbFillArgs {
            slug: "1717".to_string(),
            spec_path: Some(spec_path.to_string()),
            apply: true,
            marker: Some("missing-generator:local".to_string()),
            json: false,
            force: false,
        };

        let (markers, changes, resolved_spec) =
            markers_for_active_td(&args, None, tmp.path()).unwrap();

        assert_eq!(resolved_spec.as_deref(), Some(spec_path));
        assert_eq!(changes.unwrap(), vec![local_path.to_string()]);
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].source_path, local_path);
        assert_eq!(markers[0].id, "missing-generator:local");
    }

    #[test]
    fn whole_worktree_walk_includes_apps_and_libs() {
        let tmp = tempfile::tempdir().unwrap();
        let existing_crate = tmp.path().join("crates/existing/src/lib.rs");
        std::fs::create_dir_all(existing_crate.parent().unwrap()).unwrap();
        std::fs::write(existing_crate, "pub fn existing_crate() {}\n").unwrap();
        for path in ["apps/tape/src/push.rs", "libs/tape-core/src/lib.rs"] {
            let source = tmp.path().join(path);
            std::fs::create_dir_all(source.parent().unwrap()).unwrap();
            std::fs::write(
                source,
                format!(
                    "{}\n// TODO: hand-write content for `{path}`.\n{}\n",
                    handwrite_begin("gap=\"missing-generator:scope\" reason=\"scope walk\""),
                    handwrite_end(),
                ),
            )
            .unwrap();
        }

        let paths: HashSet<String> = enumerate_worktree_markers(tmp.path())
            .into_iter()
            .map(|marker| marker.source_path)
            .collect();
        assert!(paths.contains("apps/tape/src/push.rs"));
        assert!(paths.contains("libs/tape-core/src/lib.rs"));
        assert_eq!(count_worktree_handwrite_markers(tmp.path()), 2);
    }

    #[test]
    fn cb_fill_next_command_omits_legacy_json() {
        let marker = marker("missing-generator-cli");
        let next = next_for_marker(
            "4124",
            &marker,
            "/tmp/aw/workspaces/example/payloads/4124/missing-generator-cli.md",
        );

        assert_eq!(
            next["command"],
            "aw td fill 4124 --apply --marker missing-generator-cli"
        );
        assert!(!next["command"].as_str().unwrap().contains("--json"));
        assert_eq!(
            next["payload_path"],
            "/tmp/aw/workspaces/example/payloads/4124/missing-generator-cli.md"
        );
    }

    #[test]
    fn cb_fill_initializes_marker_payload_without_overwrite() {
        let tmp = tempfile::tempdir().unwrap();
        let marker = marker("missing-generator-cli");

        let (abs_s, created) = initialize_marker_payload(tmp.path(), "4124", &marker).unwrap();
        let expected_abs = crate::shared::workspace::payloads_path(tmp.path())
            .join("4124")
            .join("missing-generator-cli.md");
        assert_eq!(abs_s, expected_abs.to_string_lossy());
        assert!(created);
        let content = std::fs::read_to_string(&expected_abs).unwrap();
        assert!(content.contains("(fill)"));
        assert!(content.contains("missing deterministic generator"));

        std::fs::write(&expected_abs, "custom\n").unwrap();
        let (_, created_again) = initialize_marker_payload(tmp.path(), "4124", &marker).unwrap();
        assert!(!created_again);
        assert_eq!(std::fs::read_to_string(&expected_abs).unwrap(), "custom\n");
    }

    #[test]
    fn td_code_check_next_command_uses_positional_slug() {
        assert_eq!(td_code_check_command("4124"), "aw td code-check 4124");
        assert!(!td_code_check_command("4124").contains("--json"));
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

    #[test]
    fn xml_handwrite_marker_fill_preserves_pair_and_binds_tracker() {
        let src = "// <HANDWRITE gap=\"missing-generator:logic\" tracker=\"pending-tracker\" reason=\"fixture\">\n\
pub fn before() {}\n\
// </HANDWRITE>\n";
        let replaced = replace_block_body(src, 1, 3, "pub fn after() {}").unwrap();
        let filled = mark_pending_xml_marker_filled(&replaced, 1, "1882");
        assert!(filled.contains("<HANDWRITE"));
        assert!(filled.contains("</HANDWRITE>"));
        assert!(filled.contains("tracker=\"#1882\""));
        assert!(filled.contains("pub fn after() {}"));
        assert!(!filled.contains("pub fn before() {}"));
    }

    #[test]
    fn gen_to_fill_transition_from_scaffold_handwrite_over_pre_existing_source() {
        // Issue #1898 end-to-end: drive the real `aw td gen` scaffold
        // (`crate::generate::handwrite_scaffold::scaffold_handwrite`) over a
        // pre-existing function, then confirm `aw td fill`'s enumerator
        // offers it (AC1), its payload adopts the existing body (AC1), and
        // applying that payload resolves the pending tracker and drops the
        // marker out of the next enumeration (AC4 in the issue's R4 sense).
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let target = src_dir.join("demo.rs");
        std::fs::write(&target, "pub fn existing() {\n    42;\n}\n").unwrap();

        let entry = crate::generate::handwrite::HandwriteEntry::default();
        let outcome = crate::generate::handwrite_scaffold::scaffold_handwrite(
            &entry,
            &target,
            "existing",
            Some("logic"),
        )
        .unwrap();
        assert_eq!(
            outcome,
            crate::generate::handwrite_scaffold::ScaffoldOutcome::Inserted
        );

        // The scaffolder wrote a real XML marker with the pending sentinel.
        let scaffolded = std::fs::read_to_string(&target).unwrap();
        assert!(scaffolded.contains("tracker=\"pending-tracker\""));
        assert!(scaffolded.contains("pub fn existing()"));

        // AC1: still queued despite surrounding non-empty existing source.
        let markers = enumerate_worktree_markers(tmp.path());
        assert_eq!(markers.len(), 1);
        let marker = &markers[0];
        assert!(marker.adopt_existing);
        assert_eq!(marker.id, "missing-generator:logic");

        let (payload_path, created) =
            initialize_marker_payload(tmp.path(), "1898", marker).unwrap();
        assert!(created);
        assert_eq!(
            std::fs::read_to_string(&payload_path).unwrap(),
            format!("{ADOPT_EXISTING_PAYLOAD}\n")
        );

        // Apply the adopt-existing payload and write the result back, as
        // `run_apply` would.
        let filled = apply_marker_payload(&scaffolded, marker, ADOPT_EXISTING_PAYLOAD, "1898")
            .expect("adopt-existing payload should apply to a genuinely pending XML marker");
        std::fs::write(&target, &filled).unwrap();
        assert!(filled.contains("pub fn existing()"));
        assert!(filled.contains("tracker=\"#1898\""));
        assert!(filled.contains("</HANDWRITE>"));

        // The next enumeration no longer offers it (R4 / AC3's "next queue"
        // half — the resolved marker leaves the queue).
        assert!(enumerate_worktree_markers(tmp.path()).is_empty());
    }

    #[test]
    fn adopt_existing_payload_rejects_an_empty_or_comment_marker() {
        let src = format!(
            "{}\n// TODO: hand-write content for `src/demo.rs`.\n{}\n",
            handwrite_begin("gap=\"missing-generator:logic\" reason=\"fixture\""),
            handwrite_end(),
        );
        let target = HandwriteMarkerEntry {
            id: "missing-generator:logic".to_string(),
            source_path: "src/demo.rs".to_string(),
            start_line: 1,
            end_line: 3,
            reason: "fixture".to_string(),
            spec_ref: None,
            adopt_existing: false,
        };
        let error = apply_marker_payload(&src, &target, ADOPT_EXISTING_PAYLOAD, "1882")
            .unwrap_err()
            .to_string();
        assert!(error.contains("may only adopt"));
    }

    #[test]
    fn replace_block_body_for_config_paths_removes_invalid_markers() {
        let src = format!(
            "{{\n{}\n// TODO: hand-write content for `frontend/package.json`.\n{}\n}}\n",
            handwrite_begin("gap=\"missing-generator:config\" reason=\"package metadata\""),
            handwrite_end(),
        );
        let out =
            replace_block_body_for_path(&src, 2, 4, "\"scripts\": {}", "frontend/package.json")
                .unwrap();
        assert!(!out.contains(HANDWRITE_BEGIN_TOKEN));
        assert!(!out.contains(HANDWRITE_END_TOKEN));
        assert!(out.contains("\"scripts\": {}"));
        assert!(out.starts_with("{\n"));
        assert!(out.ends_with("}\n"));

        let html = format!(
            "{}\n// TODO: hand-write content for `frontend/index.html`.\n{}\n",
            handwrite_begin("gap=\"missing-generator:html\" reason=\"bootstrap document\""),
            handwrite_end(),
        );
        let out = replace_block_body_for_path(
            &html,
            1,
            3,
            "<!doctype html><title>Workbench</title>",
            "frontend/index.html",
        )
        .unwrap();
        assert!(!out.contains(HANDWRITE_BEGIN_TOKEN));
        assert!(!out.contains(HANDWRITE_END_TOKEN));
        assert_eq!(out, "<!doctype html><title>Workbench</title>\n");
    }

    #[test]
    fn css_and_javascript_markers_enumerate_and_css_fill_stays_valid() {
        let tmp = tempfile::tempdir().unwrap();
        let ui = tmp.path().join("apps/workbench/ui");
        std::fs::create_dir_all(&ui).unwrap();
        std::fs::write(
            ui.join("shell.css"),
            "/* HANDWRITE-BEGIN gap=\"missing-generator:css\" reason=\"shell styles\" */\n\
             /* TODO: hand-write content for shell CSS. */\n\
             /* HANDWRITE-END */\n",
        )
        .unwrap();
        std::fs::write(
            ui.join("shell.js"),
            "// HANDWRITE-BEGIN gap=\"missing-generator:js\" reason=\"shell behavior\"\n\
             // TODO: hand-write content for shell JavaScript.\n\
             // HANDWRITE-END\n",
        )
        .unwrap();

        let markers = enumerate_worktree_markers(tmp.path());
        assert_eq!(markers.len(), 2);
        assert!(markers
            .iter()
            .any(|marker| marker.id == "missing-generator:css"));
        assert!(markers
            .iter()
            .any(|marker| marker.id == "missing-generator:js"));

        let css = std::fs::read_to_string(ui.join("shell.css")).unwrap();
        let marker = markers
            .iter()
            .find(|marker| marker.id == "missing-generator:css")
            .unwrap();
        let filled = apply_marker_payload(&css, marker, ":root { color: #fff; }", "2210").unwrap();
        assert!(filled.contains("/* HANDWRITE-BEGIN"));
        assert!(filled.contains("/* HANDWRITE-END */"));
        assert!(filled.contains(":root { color: #fff; }"));
        assert!(!filled.contains("// HANDWRITE"));
    }

    #[test]
    fn replace_block_body_for_source_paths_preserves_markers() {
        let src = format!(
            "{}\nstub\n{}\n",
            handwrite_begin("gap=\"missing-generator:component\" reason=\"component\""),
            handwrite_end(),
        );
        let out = replace_block_body_for_path(
            &src,
            1,
            3,
            "export const value = 1;",
            "frontend/src/demo.tsx",
        )
        .unwrap();
        assert!(out.contains(HANDWRITE_BEGIN_TOKEN));
        assert!(out.contains(HANDWRITE_END_TOKEN));
        assert!(out.contains("export const value = 1;"));
        assert!(!out.contains("stub"));
    }

    /// Issue #1423 repro: the rebase-landing recipe (squash-merge to
    /// `origin/main` + `git fetch origin main` + `git rebase origin/main`)
    /// advances the remote-tracking `origin/main` ref but leaves the local
    /// `main` branch ref exactly where it was before the fetch. A diff
    /// against that stale local ref must not misattribute already-landed
    /// main-side work (from a completely unrelated branch) to this branch;
    /// only this branch's own commit must show up.
    #[test]
    fn branch_changed_files_survives_rebase_landing_stale_local_main() {
        let Some(git) = crate::git::find_git_bin() else {
            eprintln!("skipping: git binary not on PATH");
            return;
        };
        let tmp = tempfile::tempdir().unwrap();
        let origin = tmp.path().join("origin.git");
        let work = tmp.path().join("work");
        let other = tmp.path().join("other");

        let run = |dir: &Path, args: &[&str]| {
            let out = std::process::Command::new(&git)
                .arg("-C")
                .arg(dir)
                .args(args)
                .output()
                .unwrap_or_else(|e| panic!("git {args:?} failed to spawn: {e}"));
            assert!(
                out.status.success(),
                "git {args:?} failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        };
        let configure_identity = |dir: &Path| {
            for (k, v) in [
                ("user.email", "test@test"),
                ("user.name", "test"),
                ("commit.gpgsign", "false"),
            ] {
                run(dir, &["config", k, v]);
            }
        };

        std::process::Command::new(&git)
            .args(["init", "--bare", "-q"])
            .arg(&origin)
            .status()
            .unwrap();
        std::process::Command::new(&git)
            .args(["clone", "-q"])
            .arg(&origin)
            .arg(&work)
            .status()
            .unwrap();
        configure_identity(&work);
        std::fs::write(work.join("README.md"), "seed\n").unwrap();
        run(&work, &["checkout", "-q", "-b", "main"]);
        run(&work, &["add", "-A"]);
        run(&work, &["commit", "-q", "-m", "seed"]);
        run(&work, &["push", "-q", "origin", "main"]);
        run(&work, &["checkout", "-q", "-b", "feature"]);
        std::fs::write(work.join("feature.txt"), "feature work\n").unwrap();
        run(&work, &["add", "-A"]);
        run(&work, &["commit", "-q", "-m", "feature work"]);

        // An independent PR lands directly on origin/main via a second
        // clone — unrelated to `feature`, and never fetched into `work`
        // until the explicit `git fetch` below.
        std::process::Command::new(&git)
            .args(["clone", "-q"])
            .arg(&origin)
            .arg(&other)
            .status()
            .unwrap();
        configure_identity(&other);
        std::fs::write(other.join("unrelated.rs"), "unrelated\n").unwrap();
        run(&other, &["add", "-A"]);
        run(&other, &["commit", "-q", "-m", "unrelated landed work"]);
        run(&other, &["push", "-q", "origin", "main"]);

        // Rebase-landing recipe: fetch advances `origin/main` (remote-
        // tracking) but `work`'s local `main` branch stays at `seed`.
        run(&work, &["fetch", "-q", "origin", "main"]);
        run(&work, &["rebase", "-q", "origin/main"]);

        let changed = branch_changed_files(&work, "main");
        assert!(
            changed.contains("feature.txt"),
            "this branch's own commit must still be detected, got: {changed:?}"
        );
        assert!(
            !changed.contains("unrelated.rs"),
            "already-landed main-side work from an unrelated branch must not \
             be attributed to this branch after rebase-landing, got: {changed:?}"
        );
    }
}

// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/cb_fill.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
      Updates CB fill to resolve and re-enumerate only active-TD Changes paths,
      write WI workflow projection locks for owned marker payloads, and commit
      per-marker progress before dispatching the next marker.
```

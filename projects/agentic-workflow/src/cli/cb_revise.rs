// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/cb_revise.md#source
// CODEGEN-BEGIN
//! `aw cb revise` — fourth-of-four CB CRRR verbs.
//!
//! Brief mode reads the prior review payload's flagged markers and dispatches
//! `score-cb-reviser` to re-fill them. Apply mode validates that the reviser
//! produced a `cb_revise.md` payload covering all flagged markers, commits a
//! `Lifecycle-Stage: Cb-Revise` trailer, advances phase to `cb_revised`, and
//! dispatches `aw cb review` for round 2.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli

use std::path::Path;

use crate::issues::{IssueBackend, IssuePatch, LocalBackend};
use anyhow::{Context, Result};
use clap::Args;

use crate::cli::cb_review::extract_flagged_markers_from;
use crate::cli::remote_push::maybe_push_remote;

// Args for `aw cb revise <slug>`.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli
#[derive(Debug, Args)]
pub struct CbReviseArgs {
    /// Issue slug identifying the current checkout branch.
    pub slug: String,
    /// Apply mode: merge `.aw/payloads/<slug>/cb_revise.md`, commit
    /// `Lifecycle-Stage: Cb-Revise`, advance phase, dispatch cb review.
    #[arg(long)]
    pub apply: bool,
    /// Emit envelope as JSON.
    #[arg(long, hide = true)]
    pub json: bool,
    /// Pretty-print the JSON envelope.
    #[arg(long)]
    pub pretty: bool,
}

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_revise.md#source
pub async fn run_revise(args: CbReviseArgs) -> Result<()> {
    if args.apply {
        run_revise_apply(args).await
    } else {
        run_revise_brief(args).await
    }
}

fn worktree_path(slug: &str) -> Result<std::path::PathBuf> {
    let project_root = crate::find_project_root()?;
    let payload_rel = cb_revise_payload_rel(slug);
    crate::cli::td::td_activate_inplace_allowing_dirty_lifecycle_paths(
        &project_root,
        slug,
        &[payload_rel.as_str()],
    )?;
    let path = crate::cli::td::td_workspace_path(&project_root, slug);
    if !path.exists() {
        anyhow::bail!("workspace not found: {}", path.display());
    }
    Ok(path)
}

fn cb_revise_payload_rel(slug: &str) -> String {
    format!(".aw/payloads/{}/cb_revise.md", slug)
}

fn cb_revise_apply_command(slug: &str) -> String {
    format!("aw cb revise {} --apply", slug)
}

fn cb_review_command(slug: &str) -> String {
    format!("aw cb review {}", slug)
}

fn next_for_revise_apply(slug: &str, payload_path: &str) -> serde_json::Value {
    serde_json::json!({
        "kind": "dispatch",
        "command": cb_revise_apply_command(slug),
        "reason": "complete the CB revision payload and apply it",
        "requires_hitl": false,
        "payload_path": payload_path,
    })
}

fn next_for_cb_review(slug: &str) -> serde_json::Value {
    serde_json::json!({
        "kind": "dispatch",
        "command": cb_review_command(slug),
        "reason": "CB revision is ready for another review round",
        "requires_hitl": false,
        "payload_path": null,
    })
}

fn print_json(value: &serde_json::Value, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", serde_json::to_string(value)?);
    }
    Ok(())
}

fn emit_error(slug: &str, message: &str, pretty: bool) -> Result<()> {
    let env = serde_json::json!({
        "action": "error",
        "slug": slug,
        "message": message,
        "next": {
            "kind": "none",
            "command": null,
            "reason": message,
            "requires_hitl": false,
            "payload_path": null,
        },
    });
    print_json(&env, pretty)
}

fn cb_revise_payload_template(flagged: &[String], spec_path: &str) -> String {
    let markers = flagged
        .iter()
        .map(|marker| format!("- {}\n", marker))
        .collect::<String>();

    format!(
        "# CB Revise\n\n\
         Spec: {spec_path}\n\n\
         ## Flagged Markers\n\n{markers}\n\
         ## Revision Payload\n\n\
         Edit the affected source first, then replace this placeholder with one section per revised marker:\n\n\
         ## marker: <marker-id>\n\
         - Summary: (fill)\n\
         - Files: (fill)\n",
    )
}

fn initialize_cb_revise_payload(
    worktree: &Path,
    slug: &str,
    flagged: &[String],
    spec_path: &str,
) -> Result<(String, bool)> {
    let rel = cb_revise_payload_rel(slug);
    let abs = worktree.join(&rel);
    if abs.exists() {
        return Ok((rel, false));
    }
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create payload directory {}", parent.display()))?;
    }
    std::fs::write(&abs, cb_revise_payload_template(flagged, spec_path))
        .with_context(|| format!("failed to write payload {}", abs.display()))?;
    Ok((rel, true))
}

fn read_flagged_markers(worktree: &Path, slug: &str) -> Result<Vec<String>> {
    let path = worktree.join(format!(".aw/payloads/{}/cb_review.md", slug));
    let content = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "cb_review.md not readable at {}: cb revise requires the prior review payload",
            path.display()
        )
    })?;
    Ok(extract_flagged_markers_from(&content))
}

async fn run_revise_brief(args: CbReviseArgs) -> Result<()> {
    let slug = args.slug.clone();
    let worktree = worktree_path(&slug)?;
    let backend = LocalBackend::from_project_root(&worktree);
    let issue = backend
        .get(&slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", slug))?;

    let phase = issue.phase.as_deref().unwrap_or("");
    if phase != "cb_reviewed" {
        emit_error(
            &slug,
            &format!(
                "phase '{}' is not eligible for cb revise (expected cb_reviewed)",
                phase
            ),
            args.pretty,
        )?;
        std::process::exit(2);
    }

    let flagged = read_flagged_markers(&worktree, &slug)?;
    if flagged.is_empty() {
        emit_error(
            &slug,
            "no flagged markers found in cb_review.md — cannot dispatch reviser",
            args.pretty,
        )?;
        std::process::exit(2);
    }

    let spec_path = issue
        .implements
        .iter()
        .find(|s| s.ends_with(".md"))
        .cloned()
        .unwrap_or_default();

    let (payload_path, payload_created) =
        initialize_cb_revise_payload(&worktree, &slug, &flagged, &spec_path)?;
    let env = serde_json::json!({
        "action": "dispatch",
        "agent": null,
        "slug": slug,
        "next": next_for_revise_apply(&slug, &payload_path),
        "payload_initialized": payload_created,
        "invoke": {
            "command": "aw cb revise",
            "args": {
                "slug": slug,
                "flagged_markers": flagged,
                "spec_path": spec_path,
                "payload_path": payload_path,
            },
        },
    });
    print_json(&env, args.pretty)?;
    let _ = args.json;
    Ok(())
}

// Extract revised marker IDs from a cb_revise.md payload. The reviser writes
// one section per re-filled marker as `## marker: <id>` headings.
fn extract_revised_markers(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("## marker:") {
            let id = rest.trim().to_string();
            if !id.is_empty() && !out.contains(&id) {
                out.push(id);
            }
        } else if let Some(rest) = trimmed.strip_prefix("## marker ") {
            let id = rest.trim().to_string();
            if !id.is_empty() && !out.contains(&id) {
                out.push(id);
            }
        }
    }
    out
}

async fn run_revise_apply(args: CbReviseArgs) -> Result<()> {
    let slug = args.slug.clone();
    let worktree = worktree_path(&slug)?;
    let backend = LocalBackend::from_project_root(&worktree);
    let issue = backend
        .get(&slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", slug))?;

    let flagged = read_flagged_markers(&worktree, &slug)?;
    if flagged.is_empty() {
        emit_error(
            &slug,
            "cb_review.md has no flagged markers — nothing to revise",
            args.pretty,
        )?;
        std::process::exit(1);
    }

    let revise_rel = cb_revise_payload_rel(&slug);
    let revise_abs = worktree.join(&revise_rel);
    let revise_body = std::fs::read_to_string(&revise_abs).with_context(|| {
        format!(
            "cb_revise.md not readable at {}: reviser must produce per-marker re-fill bodies first",
            revise_abs.display()
        )
    })?;

    let revised = extract_revised_markers(&revise_body);
    let flagged_set: std::collections::HashSet<&String> = flagged.iter().collect();
    let revised_set: std::collections::HashSet<&String> = revised.iter().collect();

    let unrevised: Vec<&String> = flagged_set.difference(&revised_set).copied().collect();
    if !unrevised.is_empty() {
        let list = unrevised
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        emit_error(
            &slug,
            &format!("flagged markers not re-filled in cb_revise.md: {}", list),
            args.pretty,
        )?;
        std::process::exit(1);
    }

    let patch = IssuePatch {
        phase: Some(crate::issues::types::td_phase::CB_REVISED.to_string()),
        ..Default::default()
    };
    backend.update(&slug, &patch).await?;

    let issue_path = backend.issue_path(&issue);
    let issue_path_s = issue_path.to_string_lossy().into_owned();
    maybe_push_remote(&worktree, &issue_path, &slug).await?;
    crate::cli::cb_review::stage_and_commit_for_revise(
        &worktree,
        &slug,
        crate::issues::types::lifecycle_trailer::CB_REVISE,
        &format!("revised {} marker(s)", revised.len()),
        &[issue_path_s.as_str(), &revise_rel],
    )?;

    let env = serde_json::json!({
        "action": "dispatch",
        "agent": serde_json::Value::Null,
        "slug": slug,
        "next": next_for_cb_review(&slug),
        "invoke": {
            "command": "aw cb review",
            "args": { "slug": slug },
        },
    });
    print_json(&env, args.pretty)?;
    let _ = args.json;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cb_revise_next_command_omits_legacy_json() {
        let next = next_for_revise_apply("4124", ".aw/payloads/4124/cb_revise.md");

        assert_eq!(next["command"], "aw cb revise 4124 --apply");
        assert!(!next["command"].as_str().unwrap().contains("--json"));
        assert_eq!(next["payload_path"], ".aw/payloads/4124/cb_revise.md");
    }

    #[test]
    fn cb_revise_payload_template_does_not_preapprove_flagged_markers() {
        let flagged = vec!["missing-demo".to_string()];
        let template = cb_revise_payload_template(&flagged, "td/demo.md");
        let revised = extract_revised_markers(&template);

        assert!(template.contains("- missing-demo"));
        assert!(template.contains("## marker: <marker-id>"));
        assert!(!revised.contains(&"missing-demo".to_string()));
    }

    #[test]
    fn initialize_cb_revise_payload_preserves_existing_content() {
        let tmp = tempfile::tempdir().unwrap();
        let flagged = vec!["missing-demo".to_string()];

        let (rel, created) =
            initialize_cb_revise_payload(tmp.path(), "4124", &flagged, "td/demo.md").unwrap();
        assert!(created);
        assert_eq!(rel, ".aw/payloads/4124/cb_revise.md");

        let abs = tmp.path().join(&rel);
        std::fs::write(&abs, "custom\n").unwrap();
        let (_, created_again) =
            initialize_cb_revise_payload(tmp.path(), "4124", &flagged, "td/demo.md").unwrap();
        assert!(!created_again);
        assert_eq!(std::fs::read_to_string(abs).unwrap(), "custom\n");
    }
}

// CODEGEN-END

---
id: projects-score-src-cb-revise-rs
fill_sections: [overview, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "TD/CB CLI surface manifests cover lifecycle dispatch, review, fill, and merge command behavior."
---

# Standardized projects/agentic-workflow/src/cli/cb_revise.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/cb_revise.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CbReviseArgs` | projects/agentic-workflow/src/cli/cb_revise.rs | struct | pub | 26 |  |
| `run_revise` | projects/agentic-workflow/src/cli/cb_revise.rs | function | pub | 42 | run_revise(args: CbReviseArgs) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/cb_revise.rs -->
```rust
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

use anyhow::{Context, Result};
use clap::Args;
use agentic_workflow::issues::{IssueBackend, IssuePatch, LocalBackend};

use crate::cb_review::extract_flagged_markers_from;
use crate::remote_push::maybe_push_remote;

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
    #[arg(long)]
    pub json: bool,
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
    crate::td::td_activate_inplace_if_present(&project_root, slug)?;
    let path = crate::td::td_workspace_path(&project_root, slug);
    if !path.exists() {
        anyhow::bail!("workspace not found: {}", path.display());
    }
    Ok(path)
}

fn emit_error(slug: &str, message: &str) -> Result<()> {
    let env = serde_json::json!({
        "action": "error",
        "slug": slug,
        "message": message,
    });
    println!("{}", serde_json::to_string_pretty(&env)?);
    Ok(())
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
    let worktree = worktree_path(&args.slug)?;
    let backend = LocalBackend::from_project_root(&worktree);
    let issue = backend
        .get(&args.slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", args.slug))?;

    let phase = issue.phase.as_deref().unwrap_or("");
    if phase != "cb_reviewed" {
        emit_error(
            &args.slug,
            &format!(
                "phase '{}' is not eligible for cb revise (expected cb_reviewed)",
                phase
            ),
        )?;
        std::process::exit(2);
    }

    let flagged = read_flagged_markers(&worktree, &args.slug)?;
    if flagged.is_empty() {
        emit_error(
            &args.slug,
            "no flagged markers found in cb_review.md — cannot dispatch reviser",
        )?;
        std::process::exit(2);
    }

    let spec_path = issue
        .implements
        .iter()
        .find(|s| s.ends_with(".md"))
        .cloned()
        .unwrap_or_default();

    let env = serde_json::json!({
        "action": "dispatch",
        "agent": null,
        "slug": args.slug,
        "invoke": {
            "command": "aw cb revise",
            "args": {
                "slug": args.slug,
                "flagged_markers": flagged,
                "spec_path": spec_path,
            },
        },
    });
    println!("{}", serde_json::to_string_pretty(&env)?);
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
    let worktree = worktree_path(&args.slug)?;
    let backend = LocalBackend::from_project_root(&worktree);
    let _issue = backend
        .get(&args.slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", args.slug))?;

    let flagged = read_flagged_markers(&worktree, &args.slug)?;
    if flagged.is_empty() {
        emit_error(
            &args.slug,
            "cb_review.md has no flagged markers — nothing to revise",
        )?;
        std::process::exit(1);
    }

    let revise_rel = format!(".aw/payloads/{}/cb_revise.md", args.slug);
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
            &args.slug,
            &format!("flagged markers not re-filled in cb_revise.md: {}", list),
        )?;
        std::process::exit(1);
    }

    let patch = IssuePatch {
        phase: Some(agentic_workflow::issues::types::td_phase::CB_REVISED.to_string()),
        ..Default::default()
    };
    backend.update(&args.slug, &patch).await?;

    let issue_rel = format!(".aw/issues/open/{}.md", args.slug);
    maybe_push_remote(&worktree, &worktree.join(&issue_rel), &args.slug).await?;
    crate::cb_review::stage_and_commit_for_revise(
        &worktree,
        &args.slug,
        agentic_workflow::issues::types::lifecycle_trailer::CB_REVISE,
        &format!("revised {} marker(s)", revised.len()),
        &[&issue_rel, &revise_rel],
    )?;

    let env = serde_json::json!({
        "action": "dispatch",
        "agent": serde_json::Value::Null,
        "slug": args.slug,
        "invoke": {
            "command": "aw cb review",
            "args": { "slug": args.slug },
        },
    });
    println!("{}", serde_json::to_string_pretty(&env)?);
    let _ = args.json;
    Ok(())
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/cb_revise.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```

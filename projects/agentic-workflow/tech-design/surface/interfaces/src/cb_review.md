---
id: projects-score-src-cb-review-rs
fill_sections: [overview, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "TD/CB CLI surface manifests cover lifecycle dispatch, review, fill, and merge command behavior."
---

# Standardized projects/agentic-workflow/src/cli/cb_review.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/cb_review.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CbReviewArgs` | projects/agentic-workflow/src/cli/cb_review.rs | struct | pub | 31 |  |
| `extract_flagged_markers_from` | projects/agentic-workflow/src/cli/cb_review.rs | function | pub | 338 | extract_flagged_markers_from(text: &str) -> Vec<String> |
| `run_review` | projects/agentic-workflow/src/cli/cb_review.rs | function | pub | 48 | run_review(args: CbReviewArgs) -> Result<()> |
| `stage_and_commit_for_revise` | projects/agentic-workflow/src/cli/cb_review.rs | function | pub | 530 | stage_and_commit_for_revise(     worktree: &Path,     slug: &str,     trailer: &str,     detail: &str,     paths: &[&str], ) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/cb_review.rs -->
```rust
//! `aw cb review` — third-of-four CB CRRR verbs.
//!
//! Brief mode dispatches `score-cb-reviewer` with the list of slug-introduced
//! files and the filled HANDWRITE markers within them. Apply mode reads
//! `.aw/payloads/<slug>/cb_review.md`, validates the verdict, commits a
//! `Lifecycle-Stage: Cb-Review` trailer, advances phase to `cb_reviewed`, and
//! emits the next dispatch (td merge / cb revise / cb arbitrate).
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;
use agentic_workflow::issues::{IssueBackend, IssuePatch, LocalBackend};

use crate::cb_fill::{branch_changed_files, enumerate_worktree_markers};
use crate::remote_push::maybe_push_remote;

// Args for `aw cb review <slug>`.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli
#[derive(Debug, Args)]
pub struct CbReviewArgs {
    /// Issue slug identifying the current checkout branch.
    pub slug: String,
    /// Apply mode: merge `.aw/payloads/<slug>/cb_review.md`, commit
    /// `Lifecycle-Stage: Cb-Review`, advance phase, dispatch next verb.
    #[arg(long)]
    pub apply: bool,
    /// Emit envelope as JSON.
    #[arg(long)]
    pub json: bool,
}

// Top-level dispatch.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_review.md#source
pub async fn run_review(args: CbReviewArgs) -> Result<()> {
    if args.apply {
        run_review_apply(args).await
    } else {
        run_review_brief(args).await
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

// Brief mode: list slug-introduced files + filled markers, emit dispatch.
async fn run_review_brief(args: CbReviewArgs) -> Result<()> {
    let worktree = worktree_path(&args.slug)?;
    let backend = LocalBackend::from_project_root(&worktree);
    let issue = backend
        .get(&args.slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", args.slug))?;

    let phase = issue.phase.as_deref().unwrap_or("");
    if phase != "cb_filled" && phase != "cb_revised" {
        emit_error(
            &args.slug,
            &format!(
                "phase '{}' is not eligible for cb review (expected cb_filled or cb_revised)",
                phase
            ),
        )?;
        std::process::exit(2);
    }

    let base_branch =
        std::env::var("SCORE_CB_FILL_BASE_BRANCH").unwrap_or_else(|_| "main".to_string());
    let changed: HashSet<String> = branch_changed_files(&worktree, &base_branch);

    // List filled markers in slug-introduced files (post-fill, so this should
    // be empty if the slug introduced markers and they were all filled). We
    // surface code_paths so the reviewer agent can read what changed.
    let all_markers = enumerate_worktree_markers(&worktree);
    let unfilled_in_slug: Vec<String> = all_markers
        .iter()
        .filter(|m| changed.contains(m.source_path.as_str()))
        .map(|m| m.id.clone())
        .collect();

    let spec_path = issue
        .implements
        .iter()
        .find(|s| s.ends_with(".md"))
        .cloned()
        .unwrap_or_default();

    let round = issue.review_count.unwrap_or(0) + 1;
    let env = serde_json::json!({
        "action": "dispatch",
        "agent": null,
        "slug": args.slug,
        "invoke": {
            "command": "aw cb review",
            "args": {
                "slug": args.slug,
                "round": round,
                "code_paths": changed.into_iter().collect::<Vec<_>>(),
                "unfilled_markers": unfilled_in_slug,
                "spec_path": spec_path,
            },
        },
    });
    println!("{}", serde_json::to_string_pretty(&env)?);
    let _ = args.json;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Approved,
    NeedsRevision,
}

fn parse_verdict(text: &str) -> Option<Verdict> {
    if text.contains("**Verdict:** approved")
        || text.contains("**Verdict**: approved")
        || text.contains("Verdict: approved")
    {
        Some(Verdict::Approved)
    } else if text.contains("**Verdict:** needs-revision")
        || text.contains("**Verdict**: needs-revision")
        || text.contains("Verdict: needs-revision")
    {
        Some(Verdict::NeedsRevision)
    } else {
        None
    }
}

// Extract flagged marker IDs from review findings. Format is `- [<marker-id>]
// <finding>` mirroring the TD review shape.
///
// Public so `cb_revise` can read the same `cb_review.md` payload.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_review.md#source
pub fn extract_flagged_markers_from(text: &str) -> Vec<String> {
    extract_flagged_markers(text)
}

fn extract_flagged_markers(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        let rest = if let Some(r) = trimmed.strip_prefix("- [") {
            r
        } else if let Some(r) = trimmed.strip_prefix("* [") {
            r
        } else {
            continue;
        };
        if let Some(close) = rest.find(']') {
            let id = rest[..close].trim().to_string();
            if !id.is_empty() && !out.contains(&id) {
                out.push(id);
            }
        }
    }
    out
}

// Public wrapper for `cb_revise` to commit `Lifecycle-Stage: Cb-Revise`.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_review.md#source
pub fn stage_and_commit_for_revise(
    worktree: &Path,
    slug: &str,
    trailer: &str,
    detail: &str,
    paths: &[&str],
) -> Result<()> {
    stage_and_commit(worktree, slug, trailer, detail, paths)
}

fn stage_and_commit(
    worktree: &Path,
    slug: &str,
    trailer: &str,
    detail: &str,
    paths: &[&str],
) -> Result<()> {
    let git_bin =
        agentic_workflow::git::find_git_bin().ok_or_else(|| anyhow::anyhow!("git binary not found"))?;
    for p in paths {
        let _ = std::process::Command::new(&git_bin)
            .arg("-C")
            .arg(worktree)
            .args(["add", p])
            .output();
    }
    let msg = format!(
        "{trailer_kebab}({slug}) — {detail}\n\nWork-Item: {slug}\nLifecycle-Stage: {trailer}\n",
        trailer_kebab = trailer.to_lowercase(),
        slug = slug,
        detail = detail,
        trailer = trailer,
    );
    let out = std::process::Command::new(&git_bin)
        .arg("-C")
        .arg(worktree)
        .args(["commit", "-m", &msg])
        .output()
        .context("git commit failed")?;
    if !out.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

// Apply mode: validate review payload, commit trailer, dispatch next verb.
async fn run_review_apply(args: CbReviewArgs) -> Result<()> {
    let worktree = worktree_path(&args.slug)?;
    let backend = LocalBackend::from_project_root(&worktree);
    let issue = backend
        .get(&args.slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", args.slug))?;

    let payload_rel = format!(".aw/payloads/{}/cb_review.md", args.slug);
    let payload_abs = worktree.join(&payload_rel);
    let payload = std::fs::read_to_string(&payload_abs).with_context(|| {
        format!(
            "payload not readable at {}: write the review with verdict + flagged markers first",
            payload_abs.display()
        )
    })?;

    let verdict = match parse_verdict(&payload) {
        Some(v) => v,
        None => {
            emit_error(
                &args.slug,
                "review payload missing verdict line — expected '**Verdict:** approved' or '**Verdict:** needs-revision'",
            )?;
            std::process::exit(1);
        }
    };

    let flagged = extract_flagged_markers(&payload);
    if matches!(verdict, Verdict::NeedsRevision) && flagged.is_empty() {
        emit_error(
            &args.slug,
            "needs-revision verdict requires at least one [marker-id] finding",
        )?;
        std::process::exit(1);
    }

    // Snapshot the payload into the issue body's `# Reviews` section by
    // appending the file (parallels TD-side reviewer template).
    let issue_rel = format!(".aw/issues/open/{}.md", args.slug);
    let issue_abs = worktree.join(&issue_rel);
    if issue_abs.exists() {
        let mut body = std::fs::read_to_string(&issue_abs)?;
        if !body.contains("# Reviews") {
            body.push_str("\n# Reviews\n");
        }
        let new_count = issue.review_count.unwrap_or(0) + 1;
        body.push_str(&format!("\n## Cb-Review {}\n\n", new_count));
        body.push_str(&payload);
        if !body.ends_with('\n') {
            body.push('\n');
        }
        std::fs::write(&issue_abs, body)?;
    }

    let new_count = issue.review_count.unwrap_or(0) + 1;
    let detail = match verdict {
        Verdict::Approved => format!("approved (cb-review #{})", new_count),
        Verdict::NeedsRevision => {
            format!("needs-revision (cb-review #{})", new_count)
        }
    };

    let patch = IssuePatch {
        phase: Some(agentic_workflow::issues::types::td_phase::CB_REVIEWED.to_string()),
        review_count: Some(new_count),
        ..Default::default()
    };
    backend.update(&args.slug, &patch).await?;

    maybe_push_remote(&worktree, &issue_abs, &args.slug).await?;

    stage_and_commit(
        &worktree,
        &args.slug,
        agentic_workflow::issues::types::lifecycle_trailer::CB_REVIEW,
        &detail,
        &[&issue_rel],
    )?;

    // Routing
    let env = match verdict {
        Verdict::Approved => serde_json::json!({
            "action": "dispatch",
            "agent": serde_json::Value::Null,
            "slug": args.slug,
            "invoke": {
                "command": "aw td merge",
                "args": { "slug": args.slug },
            },
        }),
        Verdict::NeedsRevision if new_count < 2 => serde_json::json!({
            "action": "dispatch",
            "agent": null,
            "slug": args.slug,
            "invoke": {
                "command": "aw cb revise",
                "args": { "slug": args.slug, "flagged_markers": flagged },
            },
        }),
        Verdict::NeedsRevision => serde_json::json!({
            "action": "dispatch",
            "agent": serde_json::Value::Null,
            "slug": args.slug,
            "invoke": {
                "command": "aw cb arbitrate",
                "args": { "slug": args.slug },
            },
        }),
    };
    println!("{}", serde_json::to_string_pretty(&env)?);
    let _ = args.json;
    Ok(())
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/cb_review.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```

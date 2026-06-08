---
id: projects-score-src-cb-arbitrate-rs
fill_sections: [overview, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "TD/CB CLI surface manifests cover lifecycle dispatch, review, fill, and merge command behavior."
---

# Standardized projects/agentic-workflow/src/cli/cb_arbitrate.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/cb_arbitrate.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CbArbitrateArgs` | projects/agentic-workflow/src/cli/cb_arbitrate.rs | struct | pub | 22 |  |
| `run_arbitrate` | projects/agentic-workflow/src/cli/cb_arbitrate.rs | function | pub | 51 | run_arbitrate(args: CbArbitrateArgs) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/cb_arbitrate.rs -->
```rust
//! `aw cb arbitrate` — terminal escalation verb for the CB CRRR loop.
//!
//! Invoked when `aw cb review --apply` records a second `needs-revision`
//! verdict. Advances phase to `cb_arbitrated`, commits a
//! `Lifecycle-Stage: Cb-Arbitrate` trailer, and prints human guidance for
//! the two recovery paths (force-merge or send-back).
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli

use anyhow::{Context, Result};
use clap::Args;
use agentic_workflow::issues::{IssueBackend, IssuePatch, LocalBackend};

use crate::remote_push::maybe_push_remote;

// Args for `aw cb arbitrate <slug>`.
///
// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#cli
#[derive(Debug, Args)]
pub struct CbArbitrateArgs {
    /// Issue slug identifying the current checkout branch.
    pub slug: String,
    /// Emit envelope as JSON.
    #[arg(long)]
    pub json: bool,
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

// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/cb_arbitrate.md#source
pub async fn run_arbitrate(args: CbArbitrateArgs) -> Result<()> {
    let worktree = worktree_path(&args.slug)?;
    let backend = LocalBackend::from_project_root(&worktree);
    let issue = backend
        .get(&args.slug)
        .await?
        .ok_or_else(|| anyhow::anyhow!("issue '{}' not found", args.slug))?;

    let phase = issue.phase.as_deref().unwrap_or("");
    if phase != agentic_workflow::issues::types::td_phase::CB_REVIEWED {
        emit_error(
            &args.slug,
            &format!(
                "phase '{}' is not eligible for cb arbitrate (expected cb_reviewed)",
                phase
            ),
        )?;
        std::process::exit(2);
    }
    let count = issue.review_count.unwrap_or(0);
    if count < 2 {
        emit_error(
            &args.slug,
            &format!(
                "cb arbitrate requires cb_review_count >= 2 (got {}); resolve via cb revise instead",
                count
            ),
        )?;
        std::process::exit(2);
    }

    let patch = IssuePatch {
        phase: Some(agentic_workflow::issues::types::td_phase::CB_ARBITRATED.to_string()),
        ..Default::default()
    };
    backend.update(&args.slug, &patch).await?;

    let issue_rel = format!(".aw/issues/open/{}.md", args.slug);
    maybe_push_remote(&worktree, &worktree.join(&issue_rel), &args.slug).await?;
    if let Err(e) = stage_and_commit(
        &worktree,
        &args.slug,
        agentic_workflow::issues::types::lifecycle_trailer::CB_ARBITRATE,
        &format!("escalated to human arbitration (cb-review #{})", count),
        &[&issue_rel],
    ) {
        emit_error(&args.slug, &format!("git commit failed: {}", e))?;
        std::process::exit(1);
    }

    eprintln!(
        "\u{26a0} CB review for '{}' needs human arbitration.",
        args.slug
    );
    eprintln!("  Issue: {}", issue.title);
    eprintln!("  Checkout: {}", worktree.display());
    eprintln!("  {} review rounds completed without approval.", count);
    eprintln!("  Read the # Reviews section in the issue, then either:");
    eprintln!("    - Force-merge:  aw td merge {}", args.slug);
    eprintln!(
        "    - Send-back:    edit the flagged HANDWRITE blocks back to stubs and re-run `aw cb fill {}`",
        args.slug
    );

    let env = serde_json::json!({
        "action": "done",
        "slug": args.slug,
        "message": format!(
            "escalated to human arbitration \u{2014} {} cb-review rounds exhausted",
            count
        ),
    });
    println!("{}", serde_json::to_string_pretty(&env)?);
    let _ = args.json;
    Ok(())
}

fn stage_and_commit(
    worktree: &std::path::Path,
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
        "{trailer_kebab}({slug}) \u{2014} {detail}\n\nWork-Item: {slug}\nLifecycle-Stage: {trailer}\n",
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/cb_arbitrate.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```

---
id: projects-score-src-td-migrate-rs
fill_sections: [overview, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "TD/CB CLI surface manifests cover lifecycle dispatch, review, fill, and merge command behavior."
---

# Standardized projects/agentic-workflow/src/cli/td_migrate.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/td_migrate.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MigrateMermaidArgs` | projects/agentic-workflow/src/cli/td_migrate.rs | struct | pub | 29 |  |
| `run` | projects/agentic-workflow/src/cli/td_migrate.rs | function | pub | 50 | run(args: MigrateMermaidArgs) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/td_migrate.rs -->
```rust
//! `aw td migrate-mermaid` — convert legacy mermaid blocks via envelope dispatch.
//!
//! Two modes:
//!
//! - **Enumerate** (default): scan the file, print one JSON dispatch envelope per
//!   legacy mermaid block on stdout. Caller authors the YAML payload externally.
//! - **Apply** (`--apply --block-id <id>`): read the payload from disk, render +
//!   verify equivalence + atomic-write the converted block.
//!
//! No embedded LLM call lives here.
//
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate-envelope.md

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

use agentic_workflow::generate::diagrams::mermaid_plus::migrate::{
    apply_block_payload, enumerate_envelopes, MigrationOptions,
};

// Arguments for `aw td migrate-mermaid`.
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#cli
#[derive(Debug, Args)]
pub struct MigrateMermaidArgs {
    /// Path to a TD spec file.
    pub path: PathBuf,

    /// Apply mode: render + verify + atomic-write the payload for `--block-id`.
    #[arg(long)]
    pub apply: bool,

    /// Block id (`<line_open>-<line_close>`) of a previously-enumerated envelope.
    /// Required with `--apply`.
    #[arg(long = "block-id")]
    pub block_id: Option<String>,

    /// Override the default payload path
    /// (`<project_root>/.aw/payloads/migrate-mermaid/<basename>-<block_id>.yaml`).
    #[arg(long = "payload-path")]
    pub payload_path: Option<PathBuf>,
}

// Entry point dispatched from `aw td migrate-mermaid`.
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#cli
pub async fn run(args: MigrateMermaidArgs) -> Result<()> {
    let project_root = crate::find_project_root()?;
    let opts = MigrationOptions {
        path: Some(args.path.clone()),
        apply: args.apply,
        block_id: args.block_id.clone(),
        payload_path: args.payload_path.clone(),
        project_root: project_root.clone(),
    };

    if args.apply {
        let block_id = args
            .block_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("--apply requires --block-id"))?;
        let payload_path = match &args.payload_path {
            Some(p) => p.clone(),
            None => {
                let base = args
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("block");
                project_root
                    .join(".aw/payloads/migrate-mermaid")
                    .join(format!("{}-{}.yaml", base, block_id))
            }
        };
        let payload = std::fs::read_to_string(&payload_path)
            .with_context(|| format!("read payload: {}", payload_path.display()))?;
        let result = apply_block_payload(&args.path, block_id, &payload, &opts).await?;
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        let envelopes = enumerate_envelopes(&args.path, &opts)?;
        for env in &envelopes {
            println!("{}", serde_json::to_string_pretty(env)?);
        }
    }
    Ok(())
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/td_migrate.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```

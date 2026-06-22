---
id: projects-score-tests-recovery-flow-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/recovery_flow_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/recovery_flow_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/recovery_flow_test.rs -->
```rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/recovery_flow_test.md#source
// CODEGEN-BEGIN
//! End-to-end recovery-flow tests (B1, B2, B3).
//!
//! These flows require live infrastructure (real GitHub for B1, real
//! fillback pipeline + tree-sitter for B3) and are marked #[ignore] for
//! manual / CI invocation only. The non-ignored test asserts the
//! flow-level wiring at the CLI surface (subcommand registration only).
//!
//! Recovery-flow CLI surface tests.

use agentic_workflow::cli::Commands;
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Smoke: recovery claim verbs remain registered on the public CLI.
#[test]
fn test_recovery_verbs_present() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td namespace");
    td.find_subcommand("claim").expect("td claim");
    assert!(
        td.find_subcommand("idle").is_none(),
        "td idle was removed with the old .aw/worktrees recovery model"
    );
    assert!(
        cmd.find_subcommand("cb").is_none(),
        "cb namespace is retired into td"
    );
    td.find_subcommand("code-claim").expect("td code-claim");
    assert!(
        cmd.find_subcommand("idle").is_none(),
        "cb idle was removed with the old .aw/worktrees recovery model"
    );
}

/// B1 e2e: `aw init` + `aw wi sync` adopts existing GitHub
/// issues into local frontmatter; phase remains unset (td_inited only
/// after `aw td create`). Requires a real GitHub repo with at least
/// one open issue.
#[test]
#[ignore = "requires real GitHub repo + auth; run manually with --ignored"]
fn flow_b1_e2e_init_and_sync() {
    // Reserved for end-to-end: bootstrap .aw/, run sync, assert
    // local issue files appear.
}

/// B2 e2e: `aw td claim --from-path <spec.md>` advances phase to
/// td_reviewed and emits a dispatch envelope to `aw td gen`.
/// Requires a temp git repo with the spec on disk.
#[test]
#[ignore = "requires temp git repo + git binary; run manually with --ignored"]
fn flow_b2_e2e_td_claim_from_path() {
    // Reserved for e2e: stage a spec.md outside .aw/, run
    // `aw td claim slug --from-path spec.md`, assert phase advance
    // and dispatch envelope.
}

/// B3 e2e: `aw td code-claim <code-path>` followed by
/// `aw td claim <slug>` reaches td_reviewed. Requires fillback
/// infrastructure (tree-sitter, codebase fixture).
#[test]
#[ignore = "requires fillback pipeline + tree-sitter fixtures; run manually with --ignored"]
fn flow_b3_e2e_cb_then_td_claim() {
    // Reserved for e2e: td code-claim creates a spec from code, then td
    // claim --from-path on that spec lands at td_reviewed.
}

// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/recovery_flow_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```

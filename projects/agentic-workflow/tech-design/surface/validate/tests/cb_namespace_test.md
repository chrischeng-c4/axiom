---
id: projects-score-tests-cb-namespace-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/cb_namespace_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/cb_namespace_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/cb_namespace_test.rs -->
```rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/cb_namespace_test.md#source
// CODEGEN-BEGIN
//! Integration tests for code-artifact verbs inherited by `aw td`.
//!
//! Phase 1 contract: registration, phase advance to `cb_genned`, trailer
//! `Cb-Gen`, dispatch envelope on stdout, and `--group-by` flag for
//! `td code-check`.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#test-plan

use agentic_workflow::cli::Commands;
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// R1: the retired top-level `aw cb` namespace is gone and TD inherits codegen.
#[test]
fn test_cb_gen_registered() {
    let cmd = Cli::command();
    assert!(cmd.find_subcommand("cb").is_none());
    let td = cmd.find_subcommand("td").expect("td namespace registered");
    td.find_subcommand("gen").expect("td gen registered");
}

#[test]
fn test_cb_check_registered() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td namespace registered");
    td.find_subcommand("code-check")
        .expect("td code-check registered");
}

/// R2: phase-advance verification at the *string* level — `aw td gen`
/// must write canonical `cb_genned`, not the legacy `td_gen_coded`.
#[test]
fn test_cb_gen_phase_advance() {
    use agentic_workflow::issues::types::td_phase;
    assert_eq!(td_phase::CB_GENNED, "cb_genned");
    assert!(td_phase::is_post_gen("cb_genned"));
    assert!(td_phase::is_post_gen("td_gen_coded"));
    assert!(!td_phase::is_post_gen("td_reviewed"));
}

/// R2: trailer constant is `Cb-Gen`, not `Td-GenCode`.
#[test]
fn test_cb_gen_trailer() {
    use agentic_workflow::issues::types::lifecycle_trailer;
    assert_eq!(lifecycle_trailer::CB_GEN, "Cb-Gen");
    assert_eq!(lifecycle_trailer::LEGACY_TD_GEN_CODE, "Td-GenCode");
    assert_eq!(lifecycle_trailer::normalize("Td-GenCode"), "Cb-Gen");
    assert_eq!(lifecycle_trailer::normalize("Cb-Gen"), "Cb-Gen");
}

/// R2: dispatch envelope shape is preserved by virtue of `cb::run_gen`
/// delegating to `td::run_gen_code`. Verify the `td gen` arg shape.
#[test]
fn test_cb_gen_envelope() {
    let cmd = Cli::command();
    let cb_gen = cmd
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("gen"))
        .expect("td gen present");
    let positionals: Vec<String> = cb_gen
        .get_positionals()
        .map(|p: &clap::Arg| p.get_id().as_str().to_string())
        .collect();
    assert!(
        positionals.iter().any(|p| p == "slug"),
        "td gen has slug arg, got {:?}",
        positionals
    );
}

/// R3: `td code-check --group-by` accepts gap | file | status.
#[test]
fn test_cb_check_group_by() {
    let cmd = Cli::command();
    let cb_check = cmd
        .find_subcommand("td")
        .and_then(|c| c.find_subcommand("code-check"))
        .expect("td code-check present");
    let gb = cb_check
        .get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "group_by")
        .expect("--group-by flag present");
    let possible: Vec<String> = gb
        .get_possible_values()
        .iter()
        .map(|v: &clap::builder::PossibleValue| v.get_name().to_string())
        .collect();
    for variant in ["gap", "file", "status"] {
        assert!(
            possible.iter().any(|p| p == variant),
            "--group-by missing '{}', got {:?}",
            variant,
            possible
        );
    }
}

// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/cb_namespace_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```

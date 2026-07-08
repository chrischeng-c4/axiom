---
id: projects-score-tests-standardize-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized apps/agentic-workflow/tests/cli/tests/standardize_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/tests/cli/tests/standardize_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/tests/cli/tests/standardize_test.rs -->
````rust
//! Integration tests for `aw standardize`.

use agentic_workflow::cli::Commands;
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[test]
fn standardize_subcommands_registered() {
    let cmd = Cli::command();
    let standardize = cmd
        .find_subcommand("standardize")
        .expect("standardize namespace");
    let audit = standardize.find_subcommand("audit").expect("audit");
    audit.find_subcommand("check").expect("audit check");
    audit.find_subcommand("record").expect("audit record");
    // #920 (epic #914 slice F): `managed`/`semantic`/`traceability` layer
    // `report`/`next`/`run` drivers are retired -- `aw health` absorbs the
    // read (same inventory/coverage library code) and slice-E worker verbs
    // (`aw td promote`, `aw td code-claim`, `aw wi create`, ...) absorb the
    // mutating remediation. Only `audit` remains under `aw standardize`.
    assert!(standardize.find_subcommand("managed").is_none());
    assert!(standardize.find_subcommand("semantic").is_none());
    assert!(standardize.find_subcommand("traceability").is_none());
    assert!(standardize.find_subcommand("capability").is_none());
    assert!(standardize.find_subcommand("regenerable").is_none());
    // Legacy top-level aliases (`report`/`codegen`/`next`/`run`) have been
    // removed; only the canonical takeover subcommands remain.
    assert!(standardize.find_subcommand("report").is_none());
    assert!(standardize.find_subcommand("codegen").is_none());
    assert!(standardize.find_subcommand("next").is_none());
    assert!(standardize.find_subcommand("run").is_none());
}

#[test]
fn standardize_requires_an_explicit_subcommand() {
    // #920: with only `audit` left, a bare `aw standardize --project <p>`
    // would just be a second, narrower copy of `aw health --project <p>`.
    // `StandardizeArgs::command` is required (not `Option`), so parsing
    // without a subcommand must fail rather than fall through to a removed
    // parent-workflow shorthand.
    // `Commands` intentionally has no `Debug` impl, so this matches on the
    // `Result` directly instead of `.expect_err(..)` (which would require
    // `Cli: Debug` to format an unexpected `Ok` value).
    let err = match Cli::try_parse_from(["aw", "standardize", "--project", "cap"]) {
        Ok(_) => panic!("standardize without a subcommand must not parse"),
        Err(err) => err,
    };
    let rendered = err.to_string();
    assert!(
        rendered.contains("subcommand") || rendered.contains("required"),
        "unexpected clap error: {rendered}"
    );
}

#[test]
fn standardize_project_option_propagates_to_audit_subcommand() {
    let parsed = Cli::try_parse_from(["aw", "standardize", "--project", "cap", "audit", "check"])
        .expect("standardize audit check with leading --project parses");
    let Commands::Standardize(args) = parsed.command else {
        panic!("expected standardize command");
    };
    assert_eq!(args.project.as_deref(), Some("cap"));
    assert!(matches!(
        args.command,
        agentic_workflow::cli::standardize::StandardizeCommand::Audit(_)
    ));

    let parsed = Cli::try_parse_from(["aw", "standardize", "audit", "check", "--project", "cap"])
        .expect("standardize audit check with trailing --project parses");
    let Commands::Standardize(args) = parsed.command else {
        panic!("expected standardize command");
    };
    assert_eq!(args.project.as_deref(), Some("cap"));
    assert!(matches!(
        args.command,
        agentic_workflow::cli::standardize::StandardizeCommand::Audit(_)
    ));
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/tests/cli/tests/standardize_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
      #920 (epic #914 slice F): `aw standardize` is retired down to `audit`
      only. Parser coverage now protects the reduced `StandardizeCommand`
      surface (`audit` is the sole variant), asserts the removed
      `managed`/`semantic`/`traceability`/legacy-alias subcommands are gone
      from the clap tree, and asserts a bare `aw standardize --project <p>`
      fails to parse because `StandardizeArgs::command` is required (not
      `Option`) -- `aw health --project <p>` is the read-only successor for
      that shorthand. The former parent-workflow-shorthand and layer-driver
      integration tests (which invoked the removed `managed`/`traceability`
      `report`/`next`/`run` verbs against real fixtures) are deleted; `audit`
      already has its own unit coverage in `src/cli/standardize_audit.rs`.
```

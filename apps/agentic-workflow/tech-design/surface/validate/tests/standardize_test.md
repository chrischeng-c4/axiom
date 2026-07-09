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
// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/validate/tests/standardize_test.md#source
// CODEGEN-BEGIN
//! Integration tests for the retired `aw standardize` namespace (#1278, epic
//! #1270 R7): `aw standardize audit check`'s reporting folded into the `aw
//! health` takeover-audit axis, `aw standardize audit record` rehomed as
//! `aw td audit-record`, and the `aw standardize` namespace itself removed
//! from the clap tree.

use agentic_workflow::cli::Commands;
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Test name kept for the `existing-project-standardization-brownfield-takeover-surface`
// AW-EC gate (see
// tests/behavior_existing_project_standardization_brownfield_takeover_surface.rs),
// which shells out to this exact test name -- renaming it would silently
// stop exercising that EC contract instead of failing loudly. #1278 repoints
// its assertion from "the audit subcommand is registered under standardize"
// to "the whole namespace is retired", matching the namespace's actual
// disposition.
#[test]
fn standardize_subcommands_registered() {
    let cmd = Cli::command();
    assert!(
        cmd.find_subcommand("standardize").is_none(),
        "`aw standardize` must be fully retired (#1278): reporting folded into \
         `aw health`'s takeover-audit axis, `audit record` rehomed to `aw td audit-record`"
    );
}

#[test]
fn standardize_top_level_and_audit_forms_fail_to_parse() {
    for argv in [
        vec!["aw", "standardize", "--project", "cap"],
        vec!["aw", "standardize", "audit", "check", "--project", "cap"],
        vec!["aw", "standardize", "audit", "record", "--project", "cap"],
    ] {
        let err = match Cli::try_parse_from(&argv) {
            Ok(_) => panic!("{argv:?} should not parse: `aw standardize` is retired (#1278)"),
            Err(err) => err,
        };
        let rendered = err.to_string();
        assert!(
            rendered.contains("unrecognized subcommand") || rendered.contains("error"),
            "unexpected clap error for {argv:?}: {rendered}"
        );
    }
}

#[test]
fn audit_record_is_rehomed_under_td() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td namespace");
    assert!(
        td.find_subcommand("audit-record").is_some(),
        "`aw standardize audit record` must be rehomed as `aw td audit-record` (#1278)"
    );
}

#[test]
fn audit_record_project_option_propagates() {
    let parsed = Cli::try_parse_from(["aw", "td", "--project", "cap", "audit-record"])
        .expect("td audit-record with leading --project parses");
    let Commands::Td(args) = parsed.command else {
        panic!("expected td command");
    };
    assert_eq!(args.project.as_deref(), Some("cap"));
    assert!(matches!(
        args.command,
        agentic_workflow::cli::td::TdCommand::AuditRecord(_)
    ));

    let parsed = Cli::try_parse_from(["aw", "td", "audit-record", "--project", "cap"])
        .expect("td audit-record with trailing --project parses");
    let Commands::Td(args) = parsed.command else {
        panic!("expected td command");
    };
    assert_eq!(args.project.as_deref(), Some("cap"));
    assert!(matches!(
        args.command,
        agentic_workflow::cli::td::TdCommand::AuditRecord(_)
    ));
}
// CODEGEN-END
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
      only.
      #1278 (epic #1270 R7): `aw standardize` is retired entirely. Parser
      coverage now asserts the `standardize` subcommand no longer exists on
      the clap tree at all, that top-level and `audit check`/`audit record`
      forms fail to parse, and that the former `audit record` mutating verb
      is rehomed as `td audit-record` (with the same leading/trailing
      `--project` propagation coverage the old standardize test carried).
      `standardize_subcommands_registered`'s name is kept unchanged because
      the `existing-project-standardization-brownfield-takeover-surface`
      AW-EC behavior gate shells out to that exact test name; its assertion
      body is repointed to namespace retirement instead of subcommand
      presence. `audit check`'s reporting moved to `aw health`'s
      takeover-audit axis (covered by `project_health_test.rs`) rather than a
      sibling read-only command; `audit` fixture-evaluation logic itself is
      unchanged and still covered by `src/cli/standardize_audit.rs`'s unit
      tests.
```

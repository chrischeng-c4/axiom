---
id: projects-score-tests-legacy-cli-removal-test-rs
type: claim
fill_sections: [changes]
related:
  - ../specs/score-cli-surface-cleanup.md
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs

## Overview
<!-- type: overview lang: markdown -->

Claim TD for the regression test that enforces hard removal of legacy Score
CLI commands and deprecated `aw td` aliases.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Cli` | projects/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | struct | private | 5 |  |
| `legacy_top_level_commands_are_removed` | projects/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 13 | legacy_top_level_commands_are_removed() |
| `workflow_protocol_commands_remain_registered` | projects/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 47 | workflow_protocol_commands_remain_registered() |
| `deprecated_td_aliases_are_removed` | projects/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 74 | deprecated_td_aliases_are_removed() |
| `canonical_cb_commands_remain_registered` | projects/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 82 | canonical_cb_commands_remain_registered() |
| `public_aggregation_points_remain_registered` | projects/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 102 | public_aggregation_points_remain_registered() |

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs
    action: add
    impl_mode: hand-written
    section: source
    description: |
      Hand-written negative Clap registration tests for hard-removed legacy
      commands and td aliases, covered semantically by
      score-cli-surface-cleanup.
```

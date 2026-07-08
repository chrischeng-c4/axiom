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

# Standardized apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs

## Overview
<!-- type: overview lang: markdown -->

Claim TD for the regression test that enforces hard removal of legacy Score
CLI commands and deprecated `aw td` aliases.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Cli` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | struct | private | 10 |  |
| `aw_bin` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 15 | aw_bin() -> Option<String> |
| `collect_markdown_files` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 19 | collect_markdown_files(root: &Path, out: &mut Vec<PathBuf>) |
| `legacy_top_level_commands_are_removed` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 34 | legacy_top_level_commands_are_removed() |
| `workflow_protocol_commands_remain_registered` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 88 | workflow_protocol_commands_remain_registered() |
| `deprecated_capability_alias_is_rejected_by_parser` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 109 | deprecated_capability_alias_is_rejected_by_parser() |
| `deleted_top_level_commands_fail_as_unknown_commands` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 121 | deleted_top_level_commands_fail_as_unknown_commands() |
| `active_docs_and_templates_do_not_reference_deleted_commands` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 166 | active_docs_and_templates_do_not_reference_deleted_commands() |
| `deprecated_td_aliases_are_removed` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 228 | deprecated_td_aliases_are_removed() |
| `test_td_merge_subcommand_is_removed` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 240 | test_td_merge_subcommand_is_removed() |
| `test_td_merge_parse_fails` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 250 | test_td_merge_parse_fails() |
| `code_artifact_commands_are_inherited_by_td` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 262 | code_artifact_commands_are_inherited_by_td() |
| `public_aggregation_points_remain_registered` | apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs | function | private | 281 | public_aggregation_points_remain_registered() |

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs
    action: add
    impl_mode: hand-written
    section: source
    description: |
      Hand-written negative Clap registration tests for hard-removed legacy
      commands and td aliases, covered semantically by
      score-cli-surface-cleanup.
  - path: apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs
    action: modify
    impl_mode: hand-written
    section: source
    description: |
      Issue #848: renumbered the Symbols table (line numbers had drifted
      8-16 lines since b424851c1/68a33689a) and added the five test fns that
      had accumulated without a mirror update: aw_bin,
      collect_markdown_files, deprecated_capability_alias_is_rejected_by_parser,
      deleted_top_level_commands_fail_as_unknown_commands, and
      active_docs_and_templates_do_not_reference_deleted_commands.
  - path: apps/agentic-workflow/tests/cli/tests/legacy_cli_removal_test.rs
    action: modify
    impl_mode: hand-written
    section: source
    description: |
      Issue #985 (init-projector slice 2/3): the local `deleted` list inside
      `active_docs_and_templates_do_not_reference_deleted_commands` used a
      bare `"aw iss"` substring to catch a long-retired abbreviated legacy
      command, which also false-positives on the current, active `aw issue`
      verb now that the CLI-table template renders `` `aw issue` `` verbatim.
      Changed to `"aw iss "` (trailing space), mirroring the same
      false-positive-avoidance technique `standardize.rs`'s
      `DELETED_COMMAND_PATHS` already uses for `"aw cb "`. Renumbered the
      Symbols table for the five lines this added (166..end all shift +5).
```

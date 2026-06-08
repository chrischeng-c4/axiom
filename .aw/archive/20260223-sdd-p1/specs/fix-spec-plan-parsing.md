---
id: fix-spec-plan-parsing
type: spec
title: "Fix Spec Plan Parsing in sdd_run_change"
version: 1
spec_type: algorithm
created_at: 2026-02-23T00:00:00+00:00
updated_at: 2026-02-23T00:00:00+00:00
requirements:
  total: 3
  ids: [R1, R2, R3]
---

# Fix Spec Plan Parsing in sdd_run_change

## Overview

Fix #463: `sdd_run_change` skips spec creation after `proposal_approved` because `parse_affected_specs()` only parses YAML frontmatter `affected_specs:` key. When proposals use `spec_plan` entries (v2 format), the parser returns empty vec and routing falls through to `generate_tasks`.

## Requirements

### R1 - Parse spec_plan from frontmatter

Extend `parse_affected_specs()` in `helpers.rs` to also parse `spec_plan:` entries from proposal YAML frontmatter. The v2 proposal format uses `spec_plan:` instead of `affected_specs:`.

**WHEN** proposal.md has `spec_plan:` entries in YAML frontmatter
**THEN** `parse_affected_specs()` returns them as `AffectedSpec` structs with id, depends, affected_code

### R2 - Prevent silent fallthrough

In `spec.rs`, when phase is `ProposalApproved` and both `missing_specs` is empty AND `spec_count` is 0, log a warning and check if the proposal actually has spec_plan entries before falling through to task generation.

**WHEN** `missing_specs` is empty AND `spec_count == 0` AND proposal has `spec_plan` entries
**THEN** re-parse the proposal and return `CreateSpec` for the first entry

### R3 - Backwards compatibility

Existing proposals with `affected_specs:` format must continue to work.

**WHEN** proposal.md uses legacy `affected_specs:` format
**THEN** behavior is unchanged

## Affected Files

- `crates/cclab-sdd/src/mcp/tools/run_change/helpers.rs` — `parse_affected_specs()`, `analyze_specs()`
- `crates/cclab-sdd/src/mcp/tools/run_change/spec.rs` — routing logic lines 36-47

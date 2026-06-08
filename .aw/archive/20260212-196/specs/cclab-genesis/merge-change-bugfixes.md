---
id: merge-change-bugfixes
type: spec
title: "Merge-Change Bug Fixes"
version: 1
spec_type: utility
spec_group: cclab-genesis
main_spec_ref: merge-change
merge_strategy: patch
created_at: 2026-02-12T08:04:43.971890+00:00
updated_at: 2026-02-12T08:04:43.971890+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-12T08:04:43.971890+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Merge-Change Bug Fixes

## Overview

Fix 4 known bugs in the merge-change workflow: (1) verdict mismatch where merge.rs accepts both NEEDS_REVISION and REVIEWED, (2) REVIEW_MERGE.md written without YAML frontmatter causing parser failure, (3) producer-less phases merged/merge_approved that exist in routing but are never produced by tools, (4) REVIEW_MERGE.md uppercase naming inconsistent with v2 lowercase convention.

## Requirements

### R1 - Unify merge verdict routing

```yaml
id: R1
priority: high
status: draft
```

In merge.rs, remove acceptance of NEEDS_REVISION and NEEDS_FIX. Only accept APPROVED, REVIEWED, and REJECTED as valid verdicts from REVIEW_MERGE.md. This aligns with the unified verdict standard from #193.

### R2 - Add YAML frontmatter to REVIEW_MERGE.md

```yaml
id: R2
priority: high
status: draft
```

Update implementation_service.rs create_merge_review to write YAML frontmatter (verdict, iteration, change_id) before the review content. This ensures extract_review_info can parse the verdict correctly.

### R3 - Document phase producers for merged/merge_approved

```yaml
id: R3
priority: medium
status: draft
```

Update merge-change.md to clarify: merged phase is set by genesis_review_merge tool after writing REVIEW_MERGE.md (or can be skipped since merge.rs routes from merging directly). merge_approved is a terminal review state before archive. Remove or annotate producer-less phases.

### R4 - Rename REVIEW_MERGE.md to review_merge.md

```yaml
id: R4
priority: low
status: draft
```

Rename the review artifact from REVIEW_MERGE.md to review_merge.md in both code (merge.rs, helpers.rs, implementation_service.rs) and spec (merge-change.md) to follow v2 lowercase naming convention.

### R5 - Remove resolved Implementation Notes

```yaml
id: R5
priority: low
status: draft
```

After fixing the bugs, remove or update the Implementation Notes section in merge-change.md that documents these known issues.

## Acceptance Criteria

### Scenario: Merge review REVIEWED verdict routes to fix_merge

- **GIVEN** review_merge.md has verdict: REVIEWED in frontmatter
- **WHEN** merge.rs extract_review_info parses the file
- **THEN** Returns Some("REVIEWED") and routes to Action::FixMerge

### Scenario: Merge review APPROVED verdict archives

- **GIVEN** review_merge.md has verdict: APPROVED in frontmatter
- **WHEN** merge.rs checks verdict at merging phase
- **THEN** Routes to Action::MergeComplete

### Scenario: Legacy NEEDS_REVISION verdict rejected

- **WHEN** review_merge.md has verdict: NEEDS_REVISION
- **THEN** Falls through to default case (not matched as REVIEWED)

</spec>

# Task: Fill Section 'overview' for Spec 'grid-crate-structure' (Change 'grid-consolidate')

**group_id**: `consolidate-grid-crates` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/grid-consolidate/groups/consolidate-grid-crates/specs/grid-crate-structure.md`
2. Read relevant context if needed
3. Write content for the **overview** section

## Section Guidance

Write a comprehensive overview (>= 50 chars) describing what this spec covers.
Begin with `<!-- type: overview lang: markdown -->` on its own line.

## Action

Run `score artifact create-change-spec` with section="overview" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/grid-consolidate/groups/consolidate-grid-crates/specs/grid-crate-structure.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec grid-consolidate .score/changes/grid-consolidate/groups/consolidate-grid-crates/payloads/create-change-spec.json
```
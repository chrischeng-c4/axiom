# Task: Fill Section 'overview' for Spec 'mamba-all-support-spec' (Change 'mamba-all-support')

**group_id**: `all-support` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md`
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
Read file: .score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec mamba-all-support .score/changes/mamba-all-support/groups/all-support/payloads/create-change-spec.json
```
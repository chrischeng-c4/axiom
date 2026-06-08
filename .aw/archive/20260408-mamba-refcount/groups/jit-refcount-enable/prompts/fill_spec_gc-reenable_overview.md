# Task: Fill Section 'overview' for Spec 'gc-reenable' (Change 'mamba-refcount')

**group_id**: `jit-refcount-enable` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/mamba-refcount/groups/jit-refcount-enable/specs/gc-reenable.md`
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
Read file: .score/changes/mamba-refcount/groups/jit-refcount-enable/specs/gc-reenable.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec mamba-refcount .score/changes/mamba-refcount/groups/jit-refcount-enable/payloads/create-change-spec.json
```
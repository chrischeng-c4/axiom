# Task: Fill Section 'requirements' for Spec 'sdd-fill-order-topdown' (Change 'sdd-spec-format-unify')

**group_id**: `sdd-spec-format-unify` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-fill-order-topdown.md`
2. Read relevant context if needed
3. Write content for the **requirements** section

## Section Guidance

Write requirements in markdown:
### R1: Title

Description.

**Priority**: high/medium/low
Begin with `<!-- type: requirements lang: markdown -->`.

## Action

Run `score artifact create-change-spec` with section="requirements" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-fill-order-topdown.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec sdd-spec-format-unify .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/payloads/create-change-spec.json
```
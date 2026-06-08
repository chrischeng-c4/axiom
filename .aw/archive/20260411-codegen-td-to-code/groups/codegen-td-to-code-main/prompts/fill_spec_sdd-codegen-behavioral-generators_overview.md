# Task: Fill Section 'overview' for Spec 'sdd-codegen-behavioral-generators' (Change 'codegen-td-to-code')

**group_id**: `codegen-td-to-code-main` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md`
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
Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec codegen-td-to-code .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/payloads/create-change-spec.json
```
# Task: Fill Section 'requirements' for Spec 'sdd-codegen-structural-generators' (Change 'codegen-td-to-code')

**group_id**: `codegen-td-to-code-main` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-structural-generators.md`
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
Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-structural-generators.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec codegen-td-to-code .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/payloads/create-change-spec.json
```
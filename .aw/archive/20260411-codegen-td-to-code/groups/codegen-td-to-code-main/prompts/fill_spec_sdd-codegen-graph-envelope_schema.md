# Task: Fill Section 'schema' for Spec 'sdd-codegen-graph-envelope' (Change 'codegen-td-to-code')

**group_id**: `codegen-td-to-code-main` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md`
2. Read relevant context if needed
3. Write content for the **schema** section

## Section Guidance

Write JSON Schema for interface/data models. Begin with `<!-- type: schema lang: json -->`.

## Action

Run `score artifact create-change-spec` with section="schema" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-graph-envelope.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec codegen-td-to-code .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/payloads/create-change-spec.json
```
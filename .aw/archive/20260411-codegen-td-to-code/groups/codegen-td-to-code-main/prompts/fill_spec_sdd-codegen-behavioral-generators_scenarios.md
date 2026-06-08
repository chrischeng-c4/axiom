# Task: Fill Section 'scenarios' for Spec 'sdd-codegen-behavioral-generators' (Change 'codegen-td-to-code')

**group_id**: `codegen-td-to-code-main` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md`
2. Read relevant context if needed
3. Write content for the **scenarios** section

## Section Guidance

Write acceptance scenarios:
### Scenario: Name
**GIVEN** precondition
**WHEN** action
**THEN** outcome
Begin with `<!-- type: scenarios lang: markdown -->`.

## Action

Run `score artifact create-change-spec` with section="scenarios" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-behavioral-generators.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec codegen-td-to-code .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/payloads/create-change-spec.json
```
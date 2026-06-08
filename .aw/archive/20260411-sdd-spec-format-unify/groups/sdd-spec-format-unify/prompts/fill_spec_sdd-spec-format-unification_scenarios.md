# Task: Fill Section 'scenarios' for Spec 'sdd-spec-format-unification' (Change 'sdd-spec-format-unify')

**group_id**: `sdd-spec-format-unify` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md`
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
Read file: .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec sdd-spec-format-unify .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/payloads/create-change-spec.json
```
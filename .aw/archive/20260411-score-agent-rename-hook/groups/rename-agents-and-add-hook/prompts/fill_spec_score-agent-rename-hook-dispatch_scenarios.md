# Task: Fill Section 'scenarios' for Spec 'score-agent-rename-hook-dispatch' (Change 'score-agent-rename-hook')

**group_id**: `rename-agents-and-add-hook` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/score-agent-rename-hook/groups/rename-agents-and-add-hook/specs/score-agent-rename-hook-dispatch.md`
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
Read file: .score/changes/score-agent-rename-hook/groups/rename-agents-and-add-hook/specs/score-agent-rename-hook-dispatch.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec score-agent-rename-hook .score/changes/score-agent-rename-hook/groups/rename-agents-and-add-hook/payloads/create-change-spec.json
```
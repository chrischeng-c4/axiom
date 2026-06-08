# Task: Fill Section 'requirements' for Spec 'score-init-command' (Change 'score-init-bootstrap')

**group_id**: `default` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/score-init-bootstrap/groups/default/specs/score-init-command.md`
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
Read file: .score/changes/score-init-bootstrap/groups/default/specs/score-init-command.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec score-init-bootstrap .score/changes/score-init-bootstrap/groups/default/payloads/create-change-spec.json
```
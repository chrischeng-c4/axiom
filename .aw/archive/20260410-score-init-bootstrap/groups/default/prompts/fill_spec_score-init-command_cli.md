# Task: Fill Section 'cli' for Spec 'score-init-command' (Change 'score-init-bootstrap')

**group_id**: `default` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/score-init-bootstrap/groups/default/specs/score-init-command.md`
2. Read relevant context if needed
3. Write content for the **cli** section

## Section Guidance

Define CLI command tree in YAML. Begin with `<!-- type: cli lang: yaml -->`.

## Action

Run `score artifact create-change-spec` with section="cli" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/score-init-bootstrap/groups/default/specs/score-init-command.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec score-init-bootstrap .score/changes/score-init-bootstrap/groups/default/payloads/create-change-spec.json
```
# Task: Fill Section 'cli' for Spec 'score-handoff-takeoff-spec' (Change 'score-handoff-takeoff')

**group_id**: `default` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md`
2. Read relevant context if needed
3. Write content for the **cli** section

## Section Guidance

Define CLI command tree in YAML. Begin with `<!-- type: cli lang: yaml -->`.

## Action

Run `score artifact create-change-spec` with section="cli" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec score-handoff-takeoff .score/changes/score-handoff-takeoff/groups/default/payloads/create-change-spec.json
```
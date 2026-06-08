# Task: Fill Section 'schema' for Spec 'jet-console-error-relay' (Change 'jet-browser-console-errors')

**group_id**: `browser-console-error-relay` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md`
2. Read relevant context if needed
3. Write content for the **schema** section

## Section Guidance

Write JSON Schema for interface/data models. Begin with `<!-- type: schema lang: json -->`.

## Action

Run `score artifact create-change-spec` with section="schema" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec jet-browser-console-errors .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/payloads/create-change-spec.json
```
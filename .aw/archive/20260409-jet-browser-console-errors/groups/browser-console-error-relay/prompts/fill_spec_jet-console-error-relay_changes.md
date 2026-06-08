# Task: Fill Section 'changes' for Spec 'jet-console-error-relay' (Change 'jet-browser-console-errors')

**group_id**: `browser-console-error-relay` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md`
2. Read relevant context if needed
3. Write content for the **changes** section

## Section Guidance

List files that will change. For MODIFY entries, include function/type-level `targets`:
```yaml
changes:
  - path: foo.rs
    action: CREATE
    description: new file
- path: bar.rs
    action: MODIFY
    targets:
- type: function
        name: handle_request
        change: add error handling
- type: struct
        name: Config
        change: add timeout field
do_not_touch: [validate_input, parse_args]
```
Target type values: function, struct, enum, trait, impl, method.
`targets` is required for MODIFY, optional for CREATE/DELETE.
`do_not_touch` lists functions/types the agent must NOT modify.
Begin with `<!-- type: changes lang: yaml -->`.

## Action

Run `score artifact create-change-spec` with section="changes" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec jet-browser-console-errors .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/payloads/create-change-spec.json
```
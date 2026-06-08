# Task: Fill Section 'schema' for Spec 'sdd-structured-issue' (Change 'sdd-structured-issue')

**group_id**: `structured-issue-format` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/sdd-structured-issue/groups/structured-issue-format/specs/sdd-structured-issue.md`
2. Read relevant context if needed
3. Write content for the **schema** section

## Section Guidance

Write JSON Schema for interface/data models. Begin with `<!-- type: schema lang: json -->`.

## Action

Run `score artifact create-change-spec` with section="schema" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/sdd-structured-issue/groups/structured-issue-format/specs/sdd-structured-issue.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec sdd-structured-issue .score/changes/sdd-structured-issue/groups/structured-issue-format/payloads/create-change-spec.json
```
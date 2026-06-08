# Task: Fill Section 'overview' for Spec 'sdd-structured-issue' (Change 'sdd-structured-issue')

**group_id**: `structured-issue-format` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/sdd-structured-issue/groups/structured-issue-format/specs/sdd-structured-issue.md`
2. Read relevant context if needed
3. Write content for the **overview** section

## Section Guidance

Write a comprehensive overview (>= 50 chars) describing what this spec covers.
Begin with `<!-- type: overview lang: markdown -->` on its own line.

## Action

Run `score artifact create-change-spec` with section="overview" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/sdd-structured-issue/groups/structured-issue-format/specs/sdd-structured-issue.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec sdd-structured-issue .score/changes/sdd-structured-issue/groups/structured-issue-format/payloads/create-change-spec.json
```
# Task: Fill Section 'schema' for Spec 'sdd-issue-centric' (Change 'sdd-issue-centric')

**group_id**: `issue-centric-workflow` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/sdd-issue-centric/groups/issue-centric-workflow/specs/sdd-issue-centric.md`
2. Read relevant context if needed
3. Write content for the **schema** section

## Section Guidance

Write JSON Schema for interface/data models. Begin with `<!-- type: schema lang: json -->`.

## Action

Run `score artifact create-change-spec` with section="schema" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/sdd-issue-centric/groups/issue-centric-workflow/specs/sdd-issue-centric.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec sdd-issue-centric .score/changes/sdd-issue-centric/groups/issue-centric-workflow/payloads/create-change-spec.json
```
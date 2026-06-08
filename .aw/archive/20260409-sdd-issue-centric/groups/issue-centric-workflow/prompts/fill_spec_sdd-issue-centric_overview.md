# Task: Fill Section 'overview' for Spec 'sdd-issue-centric' (Change 'sdd-issue-centric')

**group_id**: `issue-centric-workflow` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/sdd-issue-centric/groups/issue-centric-workflow/specs/sdd-issue-centric.md`
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
Read file: .score/changes/sdd-issue-centric/groups/issue-centric-workflow/specs/sdd-issue-centric.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec sdd-issue-centric .score/changes/sdd-issue-centric/groups/issue-centric-workflow/payloads/create-change-spec.json
```
# Task: Fill Section 'logic' for Spec 'mamba-all-support-spec' (Change 'mamba-all-support')

**group_id**: `all-support` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md`
2. Read relevant context if needed
3. Write content for the **logic** section

## Section Guidance

Draw a Mermaid flowchart. Begin with `<!-- type: logic lang: mermaid -->`.

## Action

Run `score artifact create-change-spec` with section="logic" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec mamba-all-support .score/changes/mamba-all-support/groups/all-support/payloads/create-change-spec.json
```
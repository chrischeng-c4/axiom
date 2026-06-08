# Task: Fill Section 'diagrams' for Spec '1134-mamba-dual-config-spec' (Change '1134-mamba-dual-config')

**group_id**: `unify-mamba-config` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md`
2. Read relevant context if needed
3. Write content for the **diagrams** section

## Section Guidance

Fill in relevant diagram sub-sections (Sequence, Flowchart, Class, State, ERD). Use Mermaid syntax in fenced code blocks. Remove sub-sections that don't apply.

## Action

Run `score artifact create-change-spec` with section="diagrams" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec 1134-mamba-dual-config .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/payloads/create-change-spec.json
```
# Task: Fill Section 'acceptance_criteria' for Spec 'mamba-stdlib-builtins-spec' (Change 'mamba-stdlib-builtins')

**group_id**: `stdlib-builtins-module` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/mamba-stdlib-builtins/groups/stdlib-builtins-module/specs/mamba-stdlib-builtins-spec.md`
2. Read relevant context if needed
3. Write content for the **acceptance_criteria** section

## Section Guidance

Fill in this section with appropriate content.

## Action

Run `score artifact create-change-spec` with section="acceptance_criteria" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-stdlib-builtins/groups/stdlib-builtins-module/specs/mamba-stdlib-builtins-spec.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec mamba-stdlib-builtins .score/changes/mamba-stdlib-builtins/groups/stdlib-builtins-module/payloads/create-change-spec.json
```
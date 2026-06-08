# Task: Fill Section 'test-plan' for Spec 'jit-refcount-enable' (Change 'mamba-refcount')

**group_id**: `jit-refcount-enable` (pass this to the artifact CLI)

## Instructions

1. Read the current spec: `.score/changes/mamba-refcount/groups/jit-refcount-enable/specs/jit-refcount-enable.md`
2. Read relevant context if needed
3. Write content for the **test-plan** section

## Section Guidance

Define test cases using BDD Given/When/Then. Use sdd_generate_requirement_plus tool.
Begin with `<!-- type: test-plan lang: markdown -->`.

## Action

Run `score artifact create-change-spec` with section="test-plan" and your content.

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-refcount/groups/jit-refcount-enable/specs/jit-refcount-enable.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec mamba-refcount .score/changes/mamba-refcount/groups/jit-refcount-enable/payloads/create-change-spec.json
```
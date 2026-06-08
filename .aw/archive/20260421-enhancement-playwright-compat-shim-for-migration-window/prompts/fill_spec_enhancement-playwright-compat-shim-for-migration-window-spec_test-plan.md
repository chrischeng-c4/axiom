# Task: Fill Section 'test-plan' for Spec 'enhancement-playwright-compat-shim-for-migration-window-spec' (Change 'enhancement-playwright-compat-shim-for-migration-window')

## Instructions

1. Read the current spec: `.score/changes/enhancement-playwright-compat-shim-for-migration-window/specs/enhancement-playwright-compat-shim-for-migration-window-spec.md`
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
Read file: .score/changes/enhancement-playwright-compat-shim-for-migration-window/specs/enhancement-playwright-compat-shim-for-migration-window-spec.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec enhancement-playwright-compat-shim-for-migration-window .score/changes/enhancement-playwright-compat-shim-for-migration-window/payloads/create-change-spec.json
```
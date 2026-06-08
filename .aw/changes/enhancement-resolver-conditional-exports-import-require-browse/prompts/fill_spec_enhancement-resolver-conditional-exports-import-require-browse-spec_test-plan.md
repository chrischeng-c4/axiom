# Task: Fill Section 'test-plan' for Spec 'enhancement-resolver-conditional-exports-import-require-browse-spec' (Change 'enhancement-resolver-conditional-exports-import-require-browse')

## Instructions

1. Read the current spec: `.aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md`
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
Read file: .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec enhancement-resolver-conditional-exports-import-require-browse .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/payloads/create-change-spec.json
```
# Task: Fill Section 'test-plan' for Spec 'bug-init-change-phase-mapping-conflates-crrr-terminal-spec' (Change 'bug-init-change-phase-mapping-conflates-crrr-terminal')

## Instructions

1. Read the current spec: `.score/changes/bug-init-change-phase-mapping-conflates-crrr-terminal/specs/bug-init-change-phase-mapping-conflates-crrr-terminal-spec.md`
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
Read file: .score/changes/bug-init-change-phase-mapping-conflates-crrr-terminal/specs/bug-init-change-phase-mapping-conflates-crrr-terminal-spec.md

# Write section (write payload JSON to EXACT path below, do NOT write elsewhere)
score artifact create-change-spec bug-init-change-phase-mapping-conflates-crrr-terminal .score/changes/bug-init-change-phase-mapping-conflates-crrr-terminal/payloads/create-change-spec.json
```
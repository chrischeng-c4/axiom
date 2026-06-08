# Task: Review Implementation of Spec 'enhancement-playwright-compat-shim-for-migration-window-spec' for Change 'enhancement-playwright-compat-shim-for-migration-window'

## Pre-Review Step (MANDATORY)

Before evaluating any checklist items:
1. Read spec: `.score/changes/enhancement-playwright-compat-shim-for-migration-window/specs/enhancement-playwright-compat-shim-for-migration-window-spec.md`
2. Find the `## Test Plan` section (if present) and note whether it exists and how many test cases it defines.

## Instructions

3. Read implementation diff: `.score/changes/enhancement-playwright-compat-shim-for-migration-window/implementation.md`
4. List changed files via `score workflow list-changed-files enhancement-playwright-compat-shim-for-migration-window`
5. Review code changes against spec requirements
6. Evaluate ALL checklist items below
7. **CRITICAL: List ALL issues in a single pass.** You only get ONE review round before auto-approve. Report every problem NOW.
8. Write review via the artifact CLI command

## Checklist

### Hard Checklist (MUST ALL PASS for APPROVED)

- [HARD] Code matches all spec requirements
- [HARD] If spec has `## Test Plan` section: diff contains at least one `#[test]` function
- [HARD] Existing tests still pass (no regressions introduced)

### Soft Checklist (Issues → REVIEWED verdict)

- Code quality and readability
- Error handling completeness
- Performance considerations
- Documentation where needed

## HARD REJECT RULE

**IF** the spec has a `## Test Plan` section
**AND** the implementation diff contains zero `#[test]` or `#[cfg(test)]` blocks
**THEN** verdict MUST be `REJECTED` — no exceptions, regardless of other checklist results.

This rule overrides all other considerations.

## Verdict Guidelines

- **APPROVED**: All hard checklist items pass, code matches spec, tests pass
- **REVIEWED**: Hard checklist passes but has fixable soft issues
- **REJECTED**: Any hard checklist item fails (including the hard reject rule above)

## CLI Commands

```
# Read spec and implementation
Read file: .score/changes/enhancement-playwright-compat-shim-for-migration-window/specs/enhancement-playwright-compat-shim-for-migration-window-spec.md
Read file: .score/changes/enhancement-playwright-compat-shim-for-migration-window/implementation.md

# List changed files
score workflow list-changed-files enhancement-playwright-compat-shim-for-migration-window

# Write review (write payload JSON first, then run)
score artifact review-change-implementation enhancement-playwright-compat-shim-for-migration-window .score/changes/enhancement-playwright-compat-shim-for-migration-window/payloads/review-change-implementation.json
```
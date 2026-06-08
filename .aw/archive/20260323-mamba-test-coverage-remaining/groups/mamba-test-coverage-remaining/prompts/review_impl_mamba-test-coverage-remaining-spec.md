# Task: Review Implementation of Spec 'mamba-test-coverage-remaining-spec' for Change 'mamba-test-coverage-remaining'

## Pre-Review Step (MANDATORY)

Before evaluating any checklist items:
1. Read spec: `cclab/changes/mamba-test-coverage-remaining/groups/mamba-test-coverage-remaining/specs/mamba-test-coverage-remaining-spec.md`
2. Find the `## Test Plan` section (if present) and note whether it exists and how many test cases it defines.

## Instructions

3. Read implementation diff: `cclab/changes/mamba-test-coverage-remaining/implementation.md`
4. List changed files via `cclab sdd workflow list-changed-files mamba-test-coverage-remaining`
5. Review code changes against spec requirements
6. Evaluate ALL checklist items below
7. Write review via the artifact CLI command

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
Read file: cclab/changes/mamba-test-coverage-remaining/groups/mamba-test-coverage-remaining/specs/mamba-test-coverage-remaining-spec.md
Read file: cclab/changes/mamba-test-coverage-remaining/implementation.md

# List changed files
cclab sdd workflow list-changed-files mamba-test-coverage-remaining

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation mamba-test-coverage-remaining cclab/changes/mamba-test-coverage-remaining/payloads/review-change-implementation.json
```
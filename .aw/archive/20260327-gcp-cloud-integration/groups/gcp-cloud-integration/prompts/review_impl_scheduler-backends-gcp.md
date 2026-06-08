# Task: Review Implementation of Spec 'scheduler-backends-gcp' for Change 'gcp-cloud-integration'

## Pre-Review Step (MANDATORY)

Before evaluating any checklist items:
1. Read spec: `cclab/changes/gcp-cloud-integration/groups/gcp-cloud-integration/specs/scheduler-backends-gcp.md`
2. Find the `## Test Plan` section (if present) and note whether it exists and how many test cases it defines.

## Instructions

3. Read implementation diff: `cclab/changes/gcp-cloud-integration/implementation.md`
4. List changed files via `cclab sdd workflow list-changed-files gcp-cloud-integration`
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
Read file: cclab/changes/gcp-cloud-integration/groups/gcp-cloud-integration/specs/scheduler-backends-gcp.md
Read file: cclab/changes/gcp-cloud-integration/implementation.md

# List changed files
cclab sdd workflow list-changed-files gcp-cloud-integration

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation gcp-cloud-integration cclab/changes/gcp-cloud-integration/payloads/review-change-implementation.json
```
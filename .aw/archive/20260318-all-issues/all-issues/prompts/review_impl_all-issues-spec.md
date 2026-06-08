# Task: Review Implementation of Spec 'all-issues-spec' for Change 'all-issues'

## Instructions

1. Read spec: `cclab/changes/all-issues/specs/all-issues-spec.md`
2. Read implementation diff: `cclab/changes/all-issues/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files all-issues`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/all-issues/specs/all-issues-spec.md
Read file: cclab/changes/all-issues/implementation.md

# List changed files
cclab sdd workflow list-changed-files all-issues

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation all-issues cclab/changes/all-issues/payloads/review-change-implementation.json
```
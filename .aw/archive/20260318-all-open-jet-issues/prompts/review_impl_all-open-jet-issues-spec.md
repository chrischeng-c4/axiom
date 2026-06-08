# Task: Review Implementation of Spec 'all-open-jet-issues-spec' for Change 'all-open-jet-issues'

## Instructions

1. Read spec: `cclab/changes/all-open-jet-issues/specs/all-open-jet-issues-spec.md`
2. Read implementation diff: `cclab/changes/all-open-jet-issues/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files all-open-jet-issues`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/all-open-jet-issues/specs/all-open-jet-issues-spec.md
Read file: cclab/changes/all-open-jet-issues/implementation.md

# List changed files
cclab sdd workflow list-changed-files all-open-jet-issues

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation all-open-jet-issues cclab/changes/all-open-jet-issues/payloads/review-change-implementation.json
```
# Task: Review Implementation of Spec 'jet-remaining-issues-spec' for Change 'jet-remaining-issues'

## Instructions

1. Read spec: `cclab/changes/jet-remaining-issues/specs/jet-remaining-issues-spec.md`
2. Read implementation diff: `cclab/changes/jet-remaining-issues/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files jet-remaining-issues`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/jet-remaining-issues/specs/jet-remaining-issues-spec.md
Read file: cclab/changes/jet-remaining-issues/implementation.md

# List changed files
cclab sdd workflow list-changed-files jet-remaining-issues

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation jet-remaining-issues cclab/changes/jet-remaining-issues/payloads/review-change-implementation.json
```
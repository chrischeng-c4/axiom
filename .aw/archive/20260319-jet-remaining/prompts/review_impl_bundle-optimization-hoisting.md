# Task: Review Implementation of Spec 'bundle-optimization-hoisting' for Change 'jet-remaining'

## Instructions

1. Read spec: `cclab/changes/jet-remaining/specs/bundle-optimization-hoisting.md`
2. Read implementation diff: `cclab/changes/jet-remaining/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files jet-remaining`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/jet-remaining/specs/bundle-optimization-hoisting.md
Read file: cclab/changes/jet-remaining/implementation.md

# List changed files
cclab sdd workflow list-changed-files jet-remaining

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation jet-remaining cclab/changes/jet-remaining/payloads/review-change-implementation.json
```
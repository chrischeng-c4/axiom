# Task: Review Implementation of Spec 'jet-nx-support-spec' for Change 'jet-nx-support'

## Instructions

1. Read spec: `cclab/changes/jet-nx-support/specs/jet-nx-support-spec.md`
2. Read implementation diff: `cclab/changes/jet-nx-support/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files jet-nx-support`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/jet-nx-support/specs/jet-nx-support-spec.md
Read file: cclab/changes/jet-nx-support/implementation.md

# List changed files
cclab sdd workflow list-changed-files jet-nx-support

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation jet-nx-support cclab/changes/jet-nx-support/payloads/review-change-implementation.json
```
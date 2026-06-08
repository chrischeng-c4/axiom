# Task: Review Implementation of Spec 'merge-lens-into-sdd-spec' for Change 'merge-lens-into-sdd'

## Instructions

1. Read spec: `cclab/changes/merge-lens-into-sdd/specs/merge-lens-into-sdd-spec.md`
2. Read implementation diff: `cclab/changes/merge-lens-into-sdd/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files merge-lens-into-sdd`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/merge-lens-into-sdd/specs/merge-lens-into-sdd-spec.md
Read file: cclab/changes/merge-lens-into-sdd/implementation.md

# List changed files
cclab sdd workflow list-changed-files merge-lens-into-sdd

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation merge-lens-into-sdd cclab/changes/merge-lens-into-sdd/payloads/review-change-implementation.json
```
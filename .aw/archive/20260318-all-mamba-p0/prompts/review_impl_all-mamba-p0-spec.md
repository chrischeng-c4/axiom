# Task: Review Implementation of Spec 'all-mamba-p0-spec' for Change 'all-mamba-p0'

## Instructions

1. Read spec: `cclab/changes/all-mamba-p0/specs/all-mamba-p0-spec.md`
2. Read implementation diff: `cclab/changes/all-mamba-p0/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files all-mamba-p0`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/all-mamba-p0/specs/all-mamba-p0-spec.md
Read file: cclab/changes/all-mamba-p0/implementation.md

# List changed files
cclab sdd workflow list-changed-files all-mamba-p0

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation all-mamba-p0 cclab/changes/all-mamba-p0/payloads/review-change-implementation.json
```
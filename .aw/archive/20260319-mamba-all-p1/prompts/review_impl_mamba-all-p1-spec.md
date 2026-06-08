# Task: Review Implementation of Spec 'mamba-all-p1-spec' for Change 'mamba-all-p1'

## Instructions

1. Read spec: `cclab/changes/mamba-all-p1/specs/mamba-all-p1-spec.md`
2. Read implementation diff: `cclab/changes/mamba-all-p1/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files mamba-all-p1`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/mamba-all-p1/specs/mamba-all-p1-spec.md
Read file: cclab/changes/mamba-all-p1/implementation.md

# List changed files
cclab sdd workflow list-changed-files mamba-all-p1

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation mamba-all-p1 cclab/changes/mamba-all-p1/payloads/review-change-implementation.json
```
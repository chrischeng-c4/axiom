# Task: Review Implementation of Spec 'mamba-binding-tests-spec' for Change 'mamba-binding-tests'

## Instructions

1. Read spec: `cclab/changes/mamba-binding-tests/specs/mamba-binding-tests-spec.md`
2. Read implementation diff: `cclab/changes/mamba-binding-tests/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files mamba-binding-tests`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/mamba-binding-tests/specs/mamba-binding-tests-spec.md
Read file: cclab/changes/mamba-binding-tests/implementation.md

# List changed files
cclab sdd workflow list-changed-files mamba-binding-tests

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation mamba-binding-tests cclab/changes/mamba-binding-tests/payloads/review-change-implementation.json
```
# Task: Review Implementation of Spec 'sdd-impl-test-split-spec' for Change 'sdd-impl-test-split'

## Instructions

1. Read spec: `cclab/changes/sdd-impl-test-split/specs/sdd-impl-test-split-spec.md`
2. Read implementation diff: `cclab/changes/sdd-impl-test-split/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files sdd-impl-test-split`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/sdd-impl-test-split/specs/sdd-impl-test-split-spec.md
Read file: cclab/changes/sdd-impl-test-split/implementation.md

# List changed files
cclab sdd workflow list-changed-files sdd-impl-test-split

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation sdd-impl-test-split cclab/changes/sdd-impl-test-split/payloads/review-change-implementation.json
```
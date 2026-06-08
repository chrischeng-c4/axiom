# Task: Review Implementation of Spec 'sdd-frontend-doc-support-spec' for Change 'sdd-frontend-doc-support'

## Instructions

1. Read spec: `cclab/changes/sdd-frontend-doc-support/specs/sdd-frontend-doc-support-spec.md`
2. Read implementation diff: `cclab/changes/sdd-frontend-doc-support/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files sdd-frontend-doc-support`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/sdd-frontend-doc-support/specs/sdd-frontend-doc-support-spec.md
Read file: cclab/changes/sdd-frontend-doc-support/implementation.md

# List changed files
cclab sdd workflow list-changed-files sdd-frontend-doc-support

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation sdd-frontend-doc-support cclab/changes/sdd-frontend-doc-support/payloads/review-change-implementation.json
```
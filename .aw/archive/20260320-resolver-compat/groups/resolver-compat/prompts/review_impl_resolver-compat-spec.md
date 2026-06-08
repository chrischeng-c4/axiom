# Task: Review Implementation of Spec 'resolver-compat-spec' for Change 'resolver-compat'

## Instructions

1. Read spec: `cclab/changes/resolver-compat/specs/resolver-compat-spec.md`
2. Read implementation diff: `cclab/changes/resolver-compat/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files resolver-compat`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/resolver-compat/specs/resolver-compat-spec.md
Read file: cclab/changes/resolver-compat/implementation.md

# List changed files
cclab sdd workflow list-changed-files resolver-compat

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation resolver-compat cclab/changes/resolver-compat/payloads/review-change-implementation.json
```
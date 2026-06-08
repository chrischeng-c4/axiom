# Task: Review Implementation of Spec 'sdd-codegen-testgen-spec' for Change 'sdd-codegen-testgen'

## Instructions

1. Read spec: `cclab/changes/sdd-codegen-testgen/specs/sdd-codegen-testgen-spec.md`
2. Read implementation diff: `cclab/changes/sdd-codegen-testgen/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files sdd-codegen-testgen`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/sdd-codegen-testgen/specs/sdd-codegen-testgen-spec.md
Read file: cclab/changes/sdd-codegen-testgen/implementation.md

# List changed files
cclab sdd workflow list-changed-files sdd-codegen-testgen

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation sdd-codegen-testgen cclab/changes/sdd-codegen-testgen/payloads/review-change-implementation.json
```
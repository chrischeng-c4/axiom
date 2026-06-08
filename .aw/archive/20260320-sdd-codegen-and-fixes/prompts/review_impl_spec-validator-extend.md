# Task: Review Implementation of Spec 'spec-validator-extend' for Change 'sdd-codegen-and-fixes'

## Instructions

1. Read spec: `cclab/changes/sdd-codegen-and-fixes/specs/spec-validator-extend.md`
2. Read implementation diff: `cclab/changes/sdd-codegen-and-fixes/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files sdd-codegen-and-fixes`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/sdd-codegen-and-fixes/specs/spec-validator-extend.md
Read file: cclab/changes/sdd-codegen-and-fixes/implementation.md

# List changed files
cclab sdd workflow list-changed-files sdd-codegen-and-fixes

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation sdd-codegen-and-fixes cclab/changes/sdd-codegen-and-fixes/payloads/review-change-implementation.json
```
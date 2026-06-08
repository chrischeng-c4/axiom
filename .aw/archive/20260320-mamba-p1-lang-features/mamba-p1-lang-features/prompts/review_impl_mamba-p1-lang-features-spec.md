# Task: Review Implementation of Spec 'mamba-p1-lang-features-spec' for Change 'mamba-p1-lang-features'

## Instructions

1. Read spec: `cclab/changes/mamba-p1-lang-features/specs/mamba-p1-lang-features-spec.md`
2. Read implementation diff: `cclab/changes/mamba-p1-lang-features/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files mamba-p1-lang-features`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/mamba-p1-lang-features/specs/mamba-p1-lang-features-spec.md
Read file: cclab/changes/mamba-p1-lang-features/implementation.md

# List changed files
cclab sdd workflow list-changed-files mamba-p1-lang-features

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation mamba-p1-lang-features cclab/changes/mamba-p1-lang-features/payloads/review-change-implementation.json
```
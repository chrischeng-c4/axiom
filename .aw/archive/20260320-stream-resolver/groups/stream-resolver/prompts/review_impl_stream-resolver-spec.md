# Task: Review Implementation of Spec 'stream-resolver-spec' for Change 'stream-resolver'

## Instructions

1. Read spec: `cclab/changes/stream-resolver/specs/stream-resolver-spec.md`
2. Read implementation diff: `cclab/changes/stream-resolver/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files stream-resolver`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/stream-resolver/specs/stream-resolver-spec.md
Read file: cclab/changes/stream-resolver/implementation.md

# List changed files
cclab sdd workflow list-changed-files stream-resolver

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation stream-resolver cclab/changes/stream-resolver/payloads/review-change-implementation.json
```
# Task: Review Implementation of Spec 'restructure-agent-spec' for Change 'restructure-agent'

## Instructions

1. Read spec: `cclab/changes/restructure-agent/specs/restructure-agent-spec.md`
2. Read implementation diff: `cclab/changes/restructure-agent/implementation.md`
3. List changed files via `cclab sdd workflow list-changed-files restructure-agent`
4. Review code changes against spec requirements
5. Write review via the artifact CLI command

## Verdict Guidelines

- **APPROVED**: Code matches spec, tests pass
- **REVIEWED**: Has fixable issues
- **REJECTED**: Fundamental implementation problems

## CLI Commands

```
# Read spec and implementation
Read file: cclab/changes/restructure-agent/specs/restructure-agent-spec.md
Read file: cclab/changes/restructure-agent/implementation.md

# List changed files
cclab sdd workflow list-changed-files restructure-agent

# Write review (write payload JSON first, then run)
cclab sdd artifact review-change-implementation restructure-agent cclab/changes/restructure-agent/payloads/review-change-implementation.json
```
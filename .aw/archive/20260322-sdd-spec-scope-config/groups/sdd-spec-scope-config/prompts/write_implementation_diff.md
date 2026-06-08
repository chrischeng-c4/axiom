# Task: Write Implementation Diff for Change 'sdd-spec-scope-config'

## Instructions

1. Run `git diff` (or `git diff HEAD~N..HEAD` if already committed) to get the full diff
2. Write `implementation.md` via the artifact CLI command
3. The artifact tool will redirect back to the workflow router automatically

## CLI Commands

```
# Write implementation artifact (write payload JSON first, then run)
cclab sdd artifact create-change-implementation sdd-spec-scope-config cclab/changes/sdd-spec-scope-config/payloads/create-change-implementation.json
```
# Task: Write Implementation Diff for Change 'cclab-api-asgi-dispatch'

## Instructions

1. Run `git diff` (or `git diff HEAD~N..HEAD` if already committed) to get the full diff
2. Write `implementation.md` via the artifact CLI command
3. The artifact tool will redirect back to the workflow router automatically

## CLI Commands

```
# Write implementation artifact (write payload JSON first, then run)
cclab sdd artifact create-change-implementation cclab-api-asgi-dispatch cclab/changes/cclab-api-asgi-dispatch/payloads/create-change-implementation.json
```
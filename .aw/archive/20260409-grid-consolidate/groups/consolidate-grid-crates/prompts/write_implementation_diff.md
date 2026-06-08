# Task: Write Implementation Diff for Change 'grid-consolidate'

## Instructions

1. Run `git diff` (or `git diff HEAD~N..HEAD` if already committed) to get the full diff
2. Write `implementation.md` via the artifact CLI command
3. The artifact tool will redirect back to the workflow router automatically

## CLI Commands

```
# Write implementation artifact (write payload JSON first, then run)
score artifact create-change-implementation grid-consolidate .score/changes/grid-consolidate/payloads/create-change-implementation.json
```
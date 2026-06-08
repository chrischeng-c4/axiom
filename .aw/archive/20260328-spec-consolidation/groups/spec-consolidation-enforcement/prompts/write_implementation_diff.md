# Task: Write Implementation Diff for Change 'spec-consolidation'

## Instructions

1. Run `git diff` (or `git diff HEAD~N..HEAD` if already committed) to get the full diff
2. Write `implementation.md` via the artifact CLI command
3. The artifact tool will redirect back to the workflow router automatically

## CLI Commands

```
# Write implementation artifact (write payload JSON first, then run)
cclab sdd artifact create-change-implementation spec-consolidation cclab/changes/spec-consolidation/payloads/create-change-implementation.json
```
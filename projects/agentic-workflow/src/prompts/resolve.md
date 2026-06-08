# Task: Fix Review Issues

All cclab CLI commands operate on the current working directory.

## Change ID
{{change_id}}

## Instructions

1. **Read REVIEW.md** to understand issues

2. **Fix all issues**:
   - Fix all HIGH severity issues
   - Fix MEDIUM issues if feasible
   - Update IMPLEMENTATION.md with notes

3. **Ensure tests pass** after fixes

## Expected Output
- Issues fixed
- Tests passing

## CLI Commands

### Read Context
```
score workflow read-artifact {{change_id}} '{"scope":"review"}'
score workflow read-artifact {{change_id}} '{"scope":"requirements"}'
```

### Generate Artifact
Use standard code editing tools (Read, Edit, Write) to fix the issues.

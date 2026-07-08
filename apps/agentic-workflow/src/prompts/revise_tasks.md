# Task: Revise Implementation Tasks

All cclab CLI commands operate on the current working directory.

## Change ID
{{change_id}}

## Iteration
{{iteration}}

## Instructions

1. **Read current tasks and review** to understand what needs revision

2. **Revise tasks** based on review feedback:
   - Address all HIGH severity issues
   - Adjust task ordering, dependencies, or scope as needed
   - Ensure tasks align with approved specs

3. **Write revised tasks**

## CLI Commands

### Read Context
```
score workflow read-artifact {{change_id}} '{"scope":"read_path:changes/{{change_id}}/tasks.md"}'
score workflow read-artifact {{change_id}} '{"scope":"read_path:changes/{{change_id}}/specs"}'
```

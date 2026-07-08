# Task: Explore Codebase for Change

All cclab CLI commands operate on the current working directory.

## Change ID
{{change_id}}

## Instructions

1. **Read context clarifications** to understand what the user wants

2. **Explore the codebase** to build reference context:
   - Identify relevant specs, knowledge docs, and code files
   - Analyze code structure and dependencies using Lens tools
   - Identify gaps between codebase, specs, and knowledge

3. **Create reference context** with findings

## CLI Commands

### Read Context
```
score workflow read-artifact {{change_id}} '{"scope":"read_path:changes/{{change_id}}/pre_clarifications.md"}'
```

### Write Context
```
score artifact write-artifact {{change_id}} <payload_path>
```

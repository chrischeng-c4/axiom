# Task: Merge Review

All cclab CLI commands operate on the current working directory.

## Change ID
{{change_id}}

## Today's Date
{{today}}

## Instructions

1. **Get Requirements** using MCP tool:
   - Use: `score workflow read-artifact {{change_id}}` with scope="requirements"
   - This retrieves proposal.md, tasks.md, and all specs/*.md

2. **Review Merge Quality** (focus on these ONLY):
   - Specs merged to .aw/tech-design/: Are requirements, scenarios, and diagrams preserved?
   - CHANGELOG.md: Does entry accurately describe the change?
   - Documentation completeness in .aw/changes/{{change_id}}/

3. **IMPORTANT - Do NOT Check**:
   - Archive directory existence (.aw/archive/) - that's created AFTER this review
   - Exact date matching - any date within +/-1 day of today is acceptable

4. **Create review_merge.md** using the CLI command (REQUIRED):
   - You MUST use `score artifact review-merge` to create the review
   - Do NOT write the file directly - the command ensures correct format

## Verdict Guidelines
- **APPROVED**: Merge quality is clean, documentation is complete
- **REVIEWED**: Minor issues that can be auto-fixed (formatting, missing sections)
- **REJECTED**: Fundamental problems (content loss, wrong spec merged)

## CLI Commands

### Read Context
```
score workflow read-artifact {{change_id}} '{"scope":"requirements"}'
```

### Create Merge Review (REQUIRED)
```
score artifact review-merge {{change_id}} <payload_path>
```

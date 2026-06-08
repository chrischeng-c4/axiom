---
change: sdd-workflow-cleanup
group: tools-cleanup
date: 2026-03-17
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should sdd_run_change prompts be updated to avoid mentioning the removed tools?
- **Answer**: Yes. All prompts that reference sdd_read_artifact or sdd_write_artifact should be updated to use direct file Read or the specific artifact CLI commands.

### Q2: General
- **Question**: Are there any other legacy aliases for these tools that should also be removed?
- **Answer**: Check for sdd_read_artifact and sdd_write_artifact in: mod.rs registrations, MCP tool definitions, agent prompts (create_reference_context.rs, create_change_spec.rs, etc.), and workflow/reference_context.rs. Remove all references.


---
change: consolidate-read-tools
date: 2026-02-09
---

# Clarifications

## Q1: Git Tools Scope
- **Question**: Should git-related tools (genesis_read_implementation_summary, genesis_list_changed_files) also be consolidated into read_file?
- **Answer**: Keep separate. Only consolidate file-reading tools.
- **Rationale**: Git tools run git diff/log commands, not file reads. Different parameters (base_branch, filter). Cramming them into read_file would overload the file parameter semantics.

## Q2: Write Tools Scope
- **Question**: Should write tools (genesis_write_knowledge, genesis_write_main_spec) be consolidated into a single genesis_write_file?
- **Answer**: Keep separate. Only consolidate read/list tools.
- **Rationale**: Only 2 write tools with different required parameters. Token savings minimal. Consolidation would require complex conditional required fields.

## Q3: Consolidation Scope
- **Question**: Which tools to consolidate?
- **Answer**: Merge 6 read/list tools into genesis_read_file: genesis_read_knowledge, genesis_list_knowledge, genesis_read_main_spec, genesis_list_main_specs, genesis_read_all_requirements, genesis_list_specs. Use scope prefixes in the file parameter (knowledge:path, main_spec:group/id, list:knowledge, list:main_specs, list:specs, requirements).
- **Rationale**: These are all file-reading operations with similar semantics. Consolidation saves ~1500 tokens per agent invocation from tool definition overhead.

## Q4: Git Workflow
- **Question**: Which git workflow to use?
- **Answer**: in_place — work directly on main branch
- **Rationale**: Simple refactoring task, no need for feature branch.


---
change: 1140
group: check-alignment-phase1
date: 2026-04-03
---

# Requirements

1. New CLI command `cclab sdd check-alignment [path]` that validates spec files for format compliance and logical consistency. 2. SpecDocument parser: parse spec .md files into structured representation (frontmatter, sections with type annotations, JSON blocks). No regex — use structured section format. 3. Format compliance rules: missing_section_annotation, duplicate_section, format_priority_violation. 4. Logical duplicate/conflict rules: duplicate_definition (same tool name in multiple JSON blocks), definition_conflict_required, definition_conflict_field_name, definition_conflict_schema, rpc_field_consistency. 5. Library function spec_alignment::check() callable from CLI, artifact tools (write-time), and merge workflow (Phase 3). 6. Output in text + JSON format, exit code 0 on clean / non-zero on violations.

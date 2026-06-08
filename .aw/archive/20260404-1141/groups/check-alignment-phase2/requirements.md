---
change: 1141
group: check-alignment-phase2
date: 2026-04-04
---

# Requirements

1. @spec annotation parser: language-agnostic token scan for '@spec {path}#{id}' in all comment syntaxes (// # -- <!-- /* ). 2. Coverage check module: scan codebase for @spec annotations, cross-reference with spec requirement IDs. Report: covered, uncovered_requirements, unspecced_functions, stale_annotations. 3. Lens integration: use Lens symbol extraction to find public functions without @spec annotations (unspecced detection). 4. Schema→Struct validation: compare JSON Schema properties with Rust struct fields. 5. Requirement→Scenario validation: check every R{N} is referenced by at least one S{N}. 6. Fix Phase 1 nested schema limitation: logical rules should traverse into result.schema.required and result.schema.properties for OpenRPC format. 7. Implementation agent prompt update: add @spec annotation instruction to create_change_implementation prompt template. 8. Extend check-alignment CLI output with coverage report section.

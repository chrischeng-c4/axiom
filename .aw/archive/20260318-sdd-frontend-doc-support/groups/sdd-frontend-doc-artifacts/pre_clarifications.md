---
change: sdd-frontend-doc-support
group: sdd-frontend-doc-artifacts
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: Neither. Wireframe is now its own section type (`wireframe`) in the new section type system (17 types). Each section has `<!-- type: wireframe lang: yaml -->` annotation. No ui_spec or api_spec with marker needed.

### Q2: General
- **Answer**: Derive primitives from tech-platform frontend. Scan the actual components used in the project to determine the MVP set.

### Q3: General
- **Answer**: DSL + JSON Schema + CLI generator only. Codegen from wireframe is delegated to cclab-lens (separate crate). SDD owns the spec, Lens owns the codegen.

### Q4: General
- **Answer**: Doc specs live alongside code specs in `groups/{group}/specs/`. Same `{id}.md` convention. No dedicated docs/ subfolder.

### Q5: General
- **Answer**: Doc is a section type (`overview` type with `doc_target` in frontmatter), not a separate file format. It participates in the standard per-spec lifecycle as sections within a spec file.

### Q6: General
- **Answer**: CLI command owns the write. Consistent with the CLI-driven architecture — agent calls `cclab sdd artifact create-change-spec` with structured flags, CLI handles merge-strategy application.

### Q7: General
- **Answer**: Rule-based. Same pattern as section_rules — keyword match on requirements text. Consistent, testable, auditable. Keywords: CLI command, public API, behavior change, config format.


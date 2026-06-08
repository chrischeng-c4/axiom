---
change: sdd-frontend-doc-support
group: sdd-frontend-doc-artifacts
date: 2026-03-18
---

# Requirements

Extend the SDD workflow with two new artifact types in `crate:sdd`:

**1. Wireframe YAML DSL (issue #897)**
- Define a JSON Schema for a wireframe-level YAML format with framework-agnostic primitives (`stack`, `form`, `nav-list`, `heading`, `action-group`, ...).
- Integrate into the SDD change-spec: wireframe YAML populates the `ui_spec` section (or `api_spec` with type marker); no new `spec_type` value to be added.
- Add validation of wireframe YAML against the schema in the SDD pipeline.
- Agent auto-fills wireframe when change touches frontend UI.
- Per-section-type CLI generator pattern (decided 2026-03-18): a `fill-section --type wireframe` subcommand handles agent-to-spec translation; agents never write raw wireframe YAML directly.

**2. User-facing doc as change artifact (issue #898)**
- Treat doc updates as a full spec/artifact sharing the per-spec lifecycle (create → fill → review → implement → merge).
- Doc spec frontmatter: `id`, `doc_target` (target path), `merge_strategy` (append | replace), `fill_sections`.
- Auto-trigger doc spec creation when change involves: CLI command changes, new/modified public API, behavior changes, or config format changes.
- Implement step: writes doc content to `doc_target` path on disk.
- Merge step: doc changes archived alongside code specs.

**Constraints:**
- Do not introduce new `spec_type` values for either artifact type.
- Both wireframe and doc artifacts must participate in the standard per-spec lifecycle, not be special-cased.
- Agents use CLI generators to produce spec content; never write spec lang directly.

---
change_id: pylibs-refactor
type: gap_spec_knowledge
created_at: 2026-02-24T10:14:28.401669+00:00
updated_at: 2026-02-24T10:14:28.401669+00:00
---

# Gap Analysis: Spec vs Knowledge

The following gaps were identified between the specification context and the knowledge context for the `pylibs-refactor` change.

| Severity | Type | Description | Action Needed | Repair Action |
| :--- | :--- | :--- | :--- | :--- |
| High | missing_spec_coverage | The refactor includes significant Titan tasks (issue_81, issue_135), and knowledge_context contains deep Titan expertise, but the spec_context completely lacks Titan specs. | Add Titan architecture and dialect specs to spec_context.md. | Scan and include cclab-titan specs in spec_context.md. |
| High | boundary_misalignment | issue_138 involves Quasar pyo3 exports and FastAPI integration, which relates to Orbit waker/GIL management in knowledge_context, but no corresponding specs are listed. | Add Orbit and Quasar architecture specs to spec_context.md. | Include cclab-orbit and cclab-quasar specs to ensure bridge implementation follows architectural constraints. |
| Medium | missing_spec_coverage | issue_189 focuses on Shield vs Pydantic gaps, but no Shield specs are listed to guide the refactor towards spec compliance. | Add Shield performance and validation specs to spec_context.md. | Include cclab-shield specs to align performance refactor with architectural requirements. |
| Medium | missing_spec_coverage | knowledge_context contains detailed contracts for code generation (spec-to-code/code-generator-contract.md), but these are not reflected in spec_context despite the refactor touching multiple generated libraries. | Add Spec-to-Code generator specs to spec_context.md. | Include relevant cclab-sdd/generator specs. |
| Low | missing_spec_coverage | issue_85 and issue_455 cover Nebula refactoring and HTTP-to-Fetch renaming, but their respective specs are missing from spec_context. | Add Nebula and Fetch specs to spec_context.md. | Include cclab-nebula and cclab-fetch specs. |

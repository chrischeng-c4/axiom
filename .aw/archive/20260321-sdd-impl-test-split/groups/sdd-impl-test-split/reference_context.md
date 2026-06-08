---
change: sdd-impl-test-split
group: sdd-impl-test-split
date: 2026-03-21
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| implement-task-logic | cclab-sdd | high | Split implementation dispatch into Phase 1 (code verification) and Phase 2 (test verification), Current single dispatch handles code + tests without verification, Need to verify build passes after code implementation, Need to verify test count matches spec test-plan count |
| change-spec-logic | cclab-sdd | high | Guard create_complete on failure - do not set create_complete=true when failed_sections is non-empty, Return error response for mainthread retry instead of unconditionally writing create_complete=true, Currently lines 519-524 unconditionally set create_complete=true even after section fills fail |
| state-machine | cclab-sdd | medium | Understand phase transitions and StatePhase enum values, Clarify how changes in create_change_impl.rs and create_change_spec.rs fit into overall SDD workflow, Reference for ChangeImplementationCreated, ChangeImplementationReviewed, ChangeImplementationRevised states |
| mcp-artifact-tools | cclab-sdd | medium | Understand artifact tool interface and payload conventions, Document how create_change_implementation and create_change_spec tools work, Reference for artifact_create and artifact_review tool boundaries |


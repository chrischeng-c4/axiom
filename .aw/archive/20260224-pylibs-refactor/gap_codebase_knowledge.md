---
change_id: pylibs-refactor
type: gap_codebase_knowledge
created_at: 2026-02-24T10:09:29.560639+00:00
updated_at: 2026-02-24T10:09:29.560639+00:00
---

# Gap Analysis: Codebase vs Knowledge

---
change_id: pylibs-refactor
type: gap_codebase_knowledge
---

| Severity | Type | Description | Ref | Action Needed | Repair Action |
| :--- | :--- | :--- | :--- | :--- | :--- |
| medium | convention_violation | `crates/cclab-queue/src/pyo3_bindings/mod.rs` (924 lines) exceeds the 500-line soft limit convention. | General SDD Convention | Split mod.rs into discrete submodules. | Create `task.rs`, `chain.rs`, `group.rs`, and `chord.rs`. |
| medium | convention_violation | `crates/cclab-mongo/src/pyo3_bindings/document.rs` (728 lines) exceeds the 500-line soft limit convention. | General SDD Convention | Split document.rs to reduce complexity. | Extract CRUD and helper methods into separate files. |
| high | pattern_mismatch | `cclab-titan` core components (`pool.rs`, `error.rs`) have 0 tests, violating maturity standards. | changelogs/improve-titan-maturity.md | Add comprehensive test suites for pooling and constraints. | Create `tests/pool.rs` and `tests/constraints.rs`. |
| medium | pattern_mismatch | Legacy `cclab-http` exists alongside new `cclab-fetch`, creating architectural ambiguity. | spec-to-code/index.md | Complete migration to `cclab-fetch` and remove legacy crate. | Delete `cclab-http` and update all workspace references. |
| medium | pattern_mismatch | `cclab-quasar` Python bindings lack parity with established spec-to-code mapping models. | spec-to-code/spec-model.md | Align handlers with spec requirements. | Update `PythonHandler` and `PyWebSocket` interfaces. |

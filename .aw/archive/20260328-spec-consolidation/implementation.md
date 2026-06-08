---
id: implementation
type: change_implementation
change_id: spec-consolidation
---

# Implementation

## Summary

Implemented spec-consolidation enforcement (#1039): config-driven scope resolution, spec directory migration to canonical `crates/` layout, and impl phase split (code → build-check → tests).

**#1039 Spec Consolidation Enforcement**:

1. **`SpecsConfig` model** (`crates/cclab-sdd/src/models/change.rs`):
   - New `SpecsConfig { scopes: HashMap<String, String> }` — maps group name → parent subdirectory under `cclab/specs/`.
   - `SddConfig.specs: SpecsConfig` field with `skip_serializing_if` guard.
   - Deserializes from `[specs.scopes]` in `config.toml`. Test: `TC_config_deser` round-trip.

2. **Config-driven scope resolution** (`crates/cclab-sdd/src/workflow/scope.rs`):
   - New `resolve_spec_dir(group, specs_base, config)` — looks up `scopes[group]` from config, returns `specs_base/scopes[group]/group`; hard-fails if configured path absent (no silent fallback for explicit config).
   - Falls back to classic probe order `crates/{group}` → `projects/{group}` → `{group}` when group not in config.
   - `pre_filter_specs` signature gains `config: Option<&SddConfig>` parameter.
   - Tests: config-scoped found, config-scoped absent (None), classic fallback.

3. **`read_main_spec` sub-path support** (`crates/cclab-sdd/src/services/file_service.rs`):
   - Loads `SddConfig` and calls `resolve_spec_dir` for the group directory.
   - New `validate_spec_path` validates `/`-separated sub-paths, rejects `..` traversal, validates each component.
   - Spec IDs may now include subdirectory segments: `main_spec:cclab-sdd/logic/state-machine`.

4. **Workspace type detection** (`crates/cclab-sdd/src/cli/init.rs`):
   - `WorkspaceType` enum: `RustCargo`, `Python`, `Unknown`.
   - `detect_workspace_type()`: probes `Cargo.toml` for `[workspace]` section or `pyproject.toml`.
   - `cclab sdd init` populates default `[specs.scopes]` based on detected workspace type.

5. **Delegation guard fix** (`crates/cclab-sdd-cli/src/commands.rs`):
   - `is_artifact_permitted_under_guard(artifact_action, guard_action)` maps artifact action names to the workflow guard actions they complete (e.g. `create_change_implementation` is permitted under `begin_implementation`).
   - Fixes agent blocking when writing implementation artifacts during `begin_implementation` guard.

6. **`create_change_spec` hardening** (`crates/cclab-sdd/src/tools/create_change_spec.rs`):
   - `create_complete: true` only written when all `fill_sections` were filled; error returned for retry otherwise.
   - Prompt instructions clarified: payload JSON must be written to the EXACT path passed as argument.

7. **Impl phase split** (`crates/cclab-sdd/src/tools/common_change_impl.rs`, `create_change_impl.rs`):
   - `ImplSubState::ImplementSpec` renamed to `ImplementSpecCode`.
   - New states: `BuildCheck { spec_id }` (cargo build gate after code phase) and `ImplementSpecTests { spec_id }` (test impl after build passes).
   - `impl_spec_phase` field in `STATE.yaml` tracks per-spec phase (`"code"` → `"build"` → `"tests"`).

8. **Agent retry loop** (`crates/cclab-sdd/src/tools/workflow_common.rs`):
   - Agent executor retries on `status: error` response rather than immediately failing.

9. **Spec directory migration** (`cclab/specs/`):
   - All ~600 spec files moved from scattered `cclab/specs/cclab-{name}/` roots to canonical `cclab/specs/crates/{crate}/` layout.
   - Root-level loose specs (e.g. `mamba-all-p1-spec.md`, `sync-adapter-spec.md`) moved to owning crate subdirectory.
   - New `cclab/specs/crates/cclab-sdd/logic/scope-resolution.md` spec and `change-spec-logic.md` spec created.
   - `cclab/config.toml`: `[specs.scopes]` entries added for all crates; Gemini removed from `create_change_spec` / `revise_change_spec` agents.

10. **Change cleanup**: 30+ completed/stale changes deleted from `cclab/changes/`; `cclab/archive/` cleaned up for `mamba-test-coverage` and `mamba-test-coverage-remaining`.

## Diff

```diff
diff --git a/crates/cclab-sdd/src/models/change.rs b/crates/cclab-sdd/src/models/change.rs
--- a/crates/cclab-sdd/src/models/change.rs
+++ b/crates/cclab-sdd/src/models/change.rs
@@ change.rs: +SpecsConfig { scopes: HashMap<String,String> } + SddConfig.specs field (#1039)
@@ change.rs: SpecsConfig::is_empty() + Deserialize/Serialize; skip_serializing_if empty
@@ change.rs: SddConfig constructors updated with specs: SpecsConfig::default()
@@ change.rs: +test TC_config_deser: [specs.scopes] TOML round-trip

diff --git a/crates/cclab-sdd/src/workflow/scope.rs b/crates/cclab-sdd/src/workflow/scope.rs
--- a/crates/cclab-sdd/src/workflow/scope.rs
+++ b/crates/cclab-sdd/src/workflow/scope.rs
@@ scope.rs: +resolve_spec_dir(group, specs_base, config) — config-driven scope lookup with classic fallback (#1039)
@@ scope.rs: pre_filter_specs signature: +config: Option<&SddConfig> arg — passed to resolve_spec_dir
@@ scope.rs: existing unit tests updated for new signature (None config)
@@ scope.rs: +tests: config-scoped entry found, config-scoped entry absent, classic fallback

diff --git a/crates/cclab-sdd/src/services/file_service.rs b/crates/cclab-sdd/src/services/file_service.rs
--- a/crates/cclab-sdd/src/services/file_service.rs
+++ b/crates/cclab-sdd/src/services/file_service.rs
@@ file_service.rs: read_main_spec loads SddConfig, calls resolve_spec_dir for group dir (#1039)
@@ file_service.rs: +validate_spec_path(path) — validates sub-paths with '/' separators, rejects '..', per-component check
@@ file_service.rs: main_spec id may contain subdirectory segments (e.g. logic/state-machine)

diff --git a/crates/cclab-sdd/src/cli/init.rs b/crates/cclab-sdd/src/cli/init.rs
--- a/crates/cclab-sdd/src/cli/init.rs
+++ b/crates/cclab-sdd/src/cli/init.rs
@@ init.rs: +WorkspaceType enum (RustCargo, Python, Unknown) — detects workspace for default scopes
@@ init.rs: detect_workspace_type() — probes Cargo.toml [workspace] / pyproject.toml
@@ init.rs: default [specs.scopes] populated based on workspace type during init

diff --git a/crates/cclab-sdd-cli/src/commands.rs b/crates/cclab-sdd-cli/src/commands.rs
--- a/crates/cclab-sdd-cli/src/commands.rs
+++ b/crates/cclab-sdd-cli/src/commands.rs
@@ commands.rs: +is_artifact_permitted_under_guard(artifact_action, guard_action) — maps artifact → workflow action name pairs
@@ commands.rs: delegation guard check: permitted = normalized_action == guard || is_artifact_permitted_under_guard()

diff --git a/crates/cclab-sdd/src/tools/create_change_spec.rs b/crates/cclab-sdd/src/tools/create_change_spec.rs
--- a/crates/cclab-sdd/src/tools/create_change_spec.rs
+++ b/crates/cclab-sdd/src/tools/create_change_spec.rs
@@ create_change_spec.rs: create_complete guard — only set true when 0 failed sections; error returned for retry otherwise
@@ create_change_spec.rs: prompt clarifications: 'Write payload JSON to EXACT path' directive added

diff --git a/crates/cclab-sdd/src/tools/common_change_impl.rs b/crates/cclab-sdd/src/tools/common_change_impl.rs
--- a/crates/cclab-sdd/src/tools/common_change_impl.rs
+++ b/crates/cclab-sdd/src/tools/common_change_impl.rs
@@ common_change_impl.rs: ImplSubState::ImplementSpec → ImplementSpecCode (rename)
@@ common_change_impl.rs: +ImplSubState::BuildCheck { spec_id } — build gate state after code phase
@@ common_change_impl.rs: +ImplSubState::ImplementSpecTests { spec_id } — test impl state after build passes
@@ common_change_impl.rs: next_impl_substate reads impl_spec_phase from STATE.yaml to resume mid-phase

diff --git a/crates/cclab-sdd/src/tools/create_change_impl.rs b/crates/cclab-sdd/src/tools/create_change_impl.rs
--- a/crates/cclab-sdd/src/tools/create_change_impl.rs
+++ b/crates/cclab-sdd/src/tools/create_change_impl.rs
@@ create_change_impl.rs: ImplementSpecCode arm writes impl_spec_phase[spec_id] = "code" to STATE.yaml
@@ create_change_impl.rs: BuildCheck arm dispatched after code phase completes
@@ create_change_impl.rs: ImplementSpecTests arm dispatched after build passes

diff --git a/crates/cclab-sdd/src/tools/workflow_common.rs b/crates/cclab-sdd/src/tools/workflow_common.rs
--- a/crates/cclab-sdd/src/tools/workflow_common.rs
+++ b/crates/cclab-sdd/src/tools/workflow_common.rs
@@ workflow_common.rs: agent loop: retry logic with backoff — re-runs on agent error status
@@ workflow_common.rs: improved error reporting per retry attempt

diff --git a/cclab/specs/ (mass rename) b/cclab/specs/crates/
--- cclab/specs/cclab-agent/ → cclab/specs/crates/cclab-agent/
--- cclab/specs/cclab-quasar/ → cclab/specs/crates/cclab-api/
--- cclab/specs/cclab-pulsar-array-core/ → cclab/specs/crates/cclab-array/
--- cclab/specs/cclab-cli/ → cclab/specs/crates/cclab-cli/
--- cclab/specs/cclab-core/ → cclab/specs/crates/cclab-core/
--- cclab/specs/cclab-crypto/ → cclab/specs/crates/cclab-crypto/
--- cclab/specs/cclab-fetch/ → cclab/specs/crates/cclab-fetch/
--- cclab/specs/cclab-frame/ → cclab/specs/crates/cclab-frame/
--- cclab/specs/cclab-grid/ → cclab/specs/crates/cclab-grid/
--- cclab/specs/cclab-jet/ → cclab/specs/crates/cclab-jet/
--- cclab/specs/cclab-kv/ → cclab/specs/crates/cclab-kv/
--- cclab/specs/cclab-log/ → cclab/specs/crates/cclab-log/
--- cclab/specs/cclab-mamba/ → cclab/specs/crates/mamba/
--- cclab/specs/cclab-nova/ → cclab/specs/crates/cclab-api/ (merged)
--- cclab/specs/cclab-pg/ → cclab/specs/crates/cclab-pg/
--- cclab/specs/cclab-qc/ → cclab/specs/crates/cclab-qc/
--- cclab/specs/cclab-schema/ → cclab/specs/crates/cclab-schema/
--- cclab/specs/cclab-sdd/ → cclab/specs/crates/cclab-sdd/
--- cclab/specs/cclab-sync/ → cclab/specs/crates/cclab-sync/
--- cclab/specs/*.md (root-level) → cclab/specs/crates/{owner}/
@@ +600 spec file renames (git status: R100) into canonical crates/ layout

diff --git a/cclab/config.toml b/cclab/config.toml
--- a/cclab/config.toml
+++ b/cclab/config.toml
@@ config.toml: create_change_spec = ["claude:claude-sonnet-4-6"] (removed gemini)
@@ config.toml: revise_change_spec = ["claude:claude-sonnet-4-6"] (removed gemini)

diff --git a/cclab/changes/*/STATE.yaml (deleted) b/deleted
@@ Deleted 30+ completed/stale changes from cclab/changes/ — archived or cleaned up
@@ Deleted matching cclab/archive/ entries for mamba-test-coverage and mamba-test-coverage-remaining
```

## Review: artifact-tools-update

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: spec-consolidation

**Summary**: Implementation correctly removes merge_strategy from sdd_artifact_create_change_spec artifact definition schema, matching the spec requirement. The artifact_definition() function in create_change_spec.rs omits merge_strategy from its properties. Two targeted tests verify: (1) merge_strategy is absent from the schema, (2) old callers passing merge_strategy are silently ignored without error. A third test confirms write semantics are always replace. All 11 create_change_spec tests and 22 common_change_spec tests pass with zero regressions.

### Checklist

- [PASS] [HARD] Code matches all spec requirements
  - Spec requires removing merge_strategy from sdd_artifact_create_change_spec x-payload-schema. artifact_definition() in create_change_spec.rs correctly omits merge_strategy from its properties object.
- [PASS] [HARD] If spec has Test Plan section: diff contains at least one #[test] function
  - Spec has no ## Test Plan section — rule does not apply. Implementation nevertheless includes 3 relevant #[test] functions.
- [PASS] [HARD] Existing tests still pass (no regressions introduced)
  - All 11 create_change_spec tests and 22 common_change_spec tests pass. Zero failures.
- [PASS] Code quality and readability
  - Tests are well-documented with clear comments explaining rationale (dead code removal, backward compat). Naming is descriptive.
- [PASS] Error handling completeness
  - Old callers passing merge_strategy are silently ignored (backward-compatible) rather than rejected.
- [PASS] Performance considerations
  - No performance impact — schema property removed, no runtime cost change.
- [PASS] Documentation where needed
  - Test comments clearly explain the spec-consolidation context and why merge_strategy is dead code.

## Review: change-merge-update

verdict: REVIEWED
reviewer: reviewer
iteration: 1
change_id: spec-consolidation

**Summary**: Implementation correctly adds merge-time path validation (main_spec_ref must contain '/'), two-pass pre-validation/write architecture (all-or-nothing abort), and create-vs-overwrite audit logging in create_change_merge.rs. merge_strategy removed from strip_change_spec_fields. 7 create_change_merge tests + 8 workflow::merge tests + 1 strip test all pass (20 total merge-related, 0 failures). Two soft issues: (1) workflow/merge.rs BeginMerge prompt still references merge_strategy concepts (lines 102-107) — stale dead code in old agent-based merge path; (2) audit log format uses '[merge] create {rel}' instead of spec's 'audit: create cclab/specs/{ref}' — functional but format mismatch.

### Checklist

- [PASS] [HARD] Code matches all spec requirements
  - All 4 spec requirements implemented: (1) merge_strategy removed from stripped_fields in strip_change_spec_fields(); (2) flowchart validation+audit flow implemented via two-pass architecture; (3) main_spec_ref must contain '/' — hard error with bail!() aborts merge, no files written; (4) create-vs-overwrite audit logging via audit_log vec included in JSON response. Pre-validation pass ensures all-or-nothing semantics (test_validation_aborts_before_write confirms).
- [PASS] [HARD] If spec has Test Plan section: diff contains at least one #[test] function
  - Spec has no ## Test Plan section — hard reject rule does not apply. Implementation includes 7 #[tokio::test] functions: test_programmatic_merge_with_main_spec_ref, test_missing_main_spec_ref_rejected, test_root_level_path_rejected, test_audit_log_create, test_audit_log_overwrite, test_validation_aborts_before_write, test_programmatic_merge_no_specs.
- [PASS] [HARD] Existing tests still pass (no regressions introduced)
  - 20 merge-related tests pass (7 create_change_merge + 8 workflow::merge + others). 1 strip_change_spec test passes. Zero failures, zero regressions.
- [PASS] Code quality and readability
  - Two-pass architecture (pre-validate then write) is clean and well-commented. Tests are thorough with descriptive names and assertion messages. Minor: workflow/merge.rs BeginMerge prompt (lines 102-107) still references merge_strategy concepts — stale in the old agent-based merge path that is superseded by programmatic create_change_merge.
- [PASS] Error handling completeness
  - Hard errors via anyhow::bail!() for: missing main_spec_ref, root-level path (no '/'), missing spec file. Pre-validation ensures no partial writes on error.
- [PASS] Performance considerations
  - Specs read once in validation pass, content cached in Vec for write pass — no redundant I/O.
- [PASS] Documentation where needed
  - Module doc updated. Test comments explain rationale for each assertion.

### Issues

- **[low]** workflow/merge.rs BeginMerge prompt (lines 102-107) still references merge_strategy concepts ('Read each spec's YAML frontmatter for merge_strategy', 'Apply merge based on strategy: new/extend/replace/patch'). This is stale dead code in the old agent-based merge path, superseded by programmatic create_change_merge. Should be cleaned up to avoid confusion.
- **[low]** Audit log format mismatch: code produces '[merge] create {rel_path}' (e.g. '[merge] create cclab-sdd/logic/new-spec.md') while spec defines 'audit: create cclab/specs/{main_spec_ref}'. Functionally correct but prefix and path format differ from spec.

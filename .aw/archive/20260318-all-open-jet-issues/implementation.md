---
id: implementation
type: change_implementation
change_id: all-open-jet-issues
---

# Implementation

## Summary

Updated jet specs to reflect current implementation state and created SDD workflow for 6 open jet issues.

**Spec updates — current implementation state:**

1. `cclab/specs/cclab-jet/scope-hoisting.md` (97L NEW): New spec documenting scope hoisting implementation status. Phase 1 (IIFE-based hoisting, `scope_hoist.rs` 209L→600L) is active — replaces `__jet__.define/require` registry with single IIFE + lightweight `_r` switch, react-bench 215KB→206.8KB. Phase 2 (true module flattening via `inline_module_body()`, 17 tests) is implemented but disabled — currently increases size to 252.8KB because `_m{n}.exports` references add bytes without Phase 3 mangling. Phase 3 (cross-module scope-aware mangling) not started. Includes ScopeHoistConfig JSON Schema.

2. `cclab/specs/cclab-jet/tree-shaking.md`: Current state updated to reflect 2026-03-18 reality — `tree_shake.rs` (619L) full ESM+CJS used-export analysis with namespace imports and re-exports; `dce.rs` (513L) dead branch elimination with UTF-8-safe byte_offsets lookup; `define.rs` (87L) NODE_ENV replacement + custom env vars. Pipeline updated: `build_graph → transform_modules → (define → DCE → tree_shake → scope_hoist) → generate_bundle`. react-bench now 206.8KB (down from 422KB).

3. `cclab/specs/cclab-jet/variable-mangling.md`: Current state updated — `minify.rs` (637L) whitespace/comment/console/debugger drop + bool literal replacement + UTF-8 safe byte_offsets; `mangle.rs` (665L) full AST-based scope analysis + function-local variable renaming (conservative: no globals/exports/property names); `fold.rs` (706L) constant folding (numeric, string, boolean, typeof, void) + dead-after-return elimination.

4. `cclab/specs/cclab-jet/pkg-manager.md`: Caching annotation updated from single-layer DashMap to two-layer `L1 DashMap (per session) + L2 persistent disk (~/.cache/jet/metadata/, XDG)`. `registry.rs` size updated 185L→370L reflecting disk cache, HTTP/2 adaptive window, XDG path support, and auto-migration implementation.

**SDD workflow — planning for open issues:**

5. `cclab/changes/all-open-jet-issues/` (new directory): SDD change capturing 6 open jet issues (#765 AOT build, #797 real-world validation, #881 cold install performance, #882 scope hoisting bundle size, #883 resolver bugs, #903 scope hoisting Phase 2). Grouped into 3 work streams: `jet-build-aot-production`, `jet-build-real-world-validation`, `jet-install-optimizations`. Reference contexts and pre/post clarifications completed for all groups.

6. `cclab/changes/all-open-jet-issues/specs/all-open-jet-issues-spec.md` (153L): Consolidated spec with R1-R10 requirements spanning AOT build pipeline, cross-module optimization, tree shaking, assets/source maps/CSS, real-world validation, metadata caching, HTTP/2 fetch overlapping, speculative prefetching, resolver robustness, and lock-only mode. Includes sequence diagram (optimized install pipeline) and flowchart (AOT production build pipeline).

## Diff

```diff
diff --git a/cclab/specs/cclab-jet/scope-hoisting.md b/cclab/specs/cclab-jet/scope-hoisting.md
new file mode 100644 (97 lines)
+++ b/cclab/specs/cclab-jet/scope-hoisting.md
@@ scope-hoisting.md: new spec — Phase 1 IIFE scope hoisting (done), Phase 2 true module flattening (implemented, disabled), Phase 3 cross-module mangling (not started); documents react-bench progress 215KB→206.8KB; ScopeHoistConfig JSON Schema

diff --git a/cclab/specs/cclab-jet/tree-shaking.md b/cclab/specs/cclab-jet/tree-shaking.md
--- a/cclab/specs/cclab-jet/tree-shaking.md
+++ b/cclab/specs/cclab-jet/tree-shaking.md
@@ tree-shaking.md: current state updated — react-bench 206.8KB (from 422KB), tree_shake.rs 619L, dce.rs 513L, define.rs 87L; pipeline updated to define→DCE→tree_shake→scope_hoist

diff --git a/cclab/specs/cclab-jet/variable-mangling.md b/cclab/specs/cclab-jet/variable-mangling.md
--- a/cclab/specs/cclab-jet/variable-mangling.md
+++ b/cclab/specs/cclab-jet/variable-mangling.md
@@ variable-mangling.md: current state updated — mangle.rs 665L AST-based scope analysis, fold.rs 706L constant folding, minify.rs 637L; react-bench 206.8KB (from 422KB); conservative mangling: no globals/exports/property names

diff --git a/cclab/specs/cclab-jet/pkg-manager.md b/cclab/specs/cclab-jet/pkg-manager.md
--- a/cclab/specs/cclab-jet/pkg-manager.md
+++ b/cclab/specs/cclab-jet/pkg-manager.md
@@ pkg-manager.md: x-caching updated to two-layer L1 DashMap + L2 disk; registry.rs size updated 185L→370L with disk cache, HTTP/2, XDG, auto-migration

diff --git a/cclab/changes/all-open-jet-issues/STATE.yaml b/cclab/changes/all-open-jet-issues/STATE.yaml
new file mode 100644 (58 lines)
+++ b/cclab/changes/all-open-jet-issues/STATE.yaml
@@ STATE.yaml: new SDD change for 6 open jet issues (#765, #797, #881, #882, #883, #903); 3 groups: jet-build-aot-production, jet-build-real-world-validation, jet-install-optimizations

diff --git a/cclab/changes/all-open-jet-issues/specs/all-open-jet-issues-spec.md b/cclab/changes/all-open-jet-issues/specs/all-open-jet-issues-spec.md
new file mode 100644 (153 lines)
+++ b/cclab/changes/all-open-jet-issues/specs/all-open-jet-issues-spec.md
@@ all-open-jet-issues-spec.md: consolidates R1-R10 across jet build AOT pipeline and install optimizations; 7 scenarios; sequence + flowchart diagrams; test plan; changes list
```

## Review: all-open-jet-issues-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: all-open-jet-issues

**Summary**: This change is a planning/documentation change — it produces SDD workflow artifacts and spec updates, not direct implementation of R1-R10. The change correctly achieves its scoped goal. Spec accuracy is verified against the codebase: scope-hoisting.md (new, 97L) documents Phase 1 (active, react-bench 206.8KB), Phase 2 (implemented but disabled, size regression to 252.8KB), and Phase 3 (not started) with correct ScopeHoistConfig schema. tree-shaking.md and variable-mangling.md current state sections accurately reflect tree_shake.rs (619L), dce.rs (513L), mangle.rs (665L), fold.rs (706L), minify.rs (637L), and the updated pipeline order. pkg-manager.md caching annotation correctly reflects the two-layer L1+L2 disk cache implementation in registry.rs (370L). The all-open-jet-issues-spec.md consolidates R1-R10 spanning AOT build pipeline, scope hoisting, tree shaking, assets/source maps/CSS, real-world validation, metadata caching, HTTP/2 overlapping, speculative prefetching, resolver robustness, and lock-only mode. Diagrams use correct Mermaid types (sequenceDiagram for install pipeline, flowchart for build pipeline). All SDD workflow artifacts (3 groups, reference contexts, clarifications) are complete.

### Checklist

- [PASS] Spec updates accurately reflect current codebase state
  - File sizes, pipeline order, and benchmark numbers in scope-hoisting.md, tree-shaking.md, variable-mangling.md, pkg-manager.md are consistent with the implementation from all-jet-issues change.
- [PASS] all-open-jet-issues-spec.md covers all required areas
  - Requirements R1-R10 present, 7 scenarios, sequence + flowchart diagrams with correct Mermaid types, test plan, changes list.
- [PASS] SDD workflow artifacts are complete
  - 3 groups (jet-build-aot-production, jet-build-real-world-validation, jet-install-optimizations) with reference contexts, pre/post clarifications all completed.


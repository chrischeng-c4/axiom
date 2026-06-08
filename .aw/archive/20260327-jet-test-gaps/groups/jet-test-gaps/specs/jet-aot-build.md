---
id: jet-aot-build
main_spec_ref: "crates/cclab-jet/logic/aot-build.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, test-plan, changes]
filled_sections: [overview, requirements, scenarios, test-plan, changes]
create_complete: true
---

# Jet Aot Build

## Overview

<!-- type: overview lang: markdown -->

Add integration tests for code splitting multi-entry scenarios in the jet AOT build pipeline (`bundler/splitting.rs`). Closes the third major test coverage gap identified in jet.

### Current Test Coverage

| Area | File | Existing Tests | Gap |
|------|------|----------------|-----|
| Single-entry splitting | `splitting.rs` | `test_no_dynamic_imports`, `test_dynamic_import_split` | None |
| Shared module extraction | `splitting.rs` | `test_shared_module_extraction` | None |
| Manual chunks | `splitting.rs` | `test_manual_chunks_routing`, `test_manual_chunks_empty_config` | None |
| Preload hints | `splitting.rs` | `test_preload_hints_for_shared_chunks` | Async preload metadata not validated |
| Multi-entry splitting | — | None | **No coverage** |
| Circular dynamic imports | — | None | **No coverage** |
| Chunk naming / dedup | `splitting.rs` | `test_chunk_naming` | Edge cases missing |

### Target Coverage

- Multi-entry: multiple `entry` roots share common modules → shared chunk extracted
- Async preload: `SplitResult.preload_hints` validated for multi-chunk graphs
- Circular dynamic imports: `import()` targets forming cycles → fallback behavior
- Diamond dependency: A→B, A→C, B→D, C→D with dynamic boundaries

### Scope

Rust `#[test]` functions in `crates/cclab-jet/src/bundler/splitting.rs` (unit) and `crates/cclab-jet/src/bundler/graph.rs` (module graph cycle with dynamic edges). No E2E tests.
## Requirements

<!-- type: requirements lang: markdown -->

### TR1: Multi-Entry Shared Chunk Extraction

```yaml
id: TR1
priority: high
tests_requirement: R2
```

Test that when two independent entry points both statically depend on a common module, `split_chunks` extracts it into a `Shared` chunk. Entry chunks must not contain the shared module. Validates the `module_count >= 2` logic in the shared detection path.

### TR2: Multi-Entry Separate Chunk Production

```yaml
id: TR2
priority: high
tests_requirement: R2
```

Test that multiple entry points each produce their own `Entry` chunk with non-overlapping module sets (excluding shared). Simulate by calling `split_chunks` once per entry and verifying disjoint `modules` vectors.

### TR3: Async Chunk Preload Metadata

```yaml
id: TR3
priority: high
tests_requirement: R8
```

Test that `split_chunks_with_config` produces correct `preload_hints` for multi-chunk graphs: shared chunks generate `is_static: true` hints, async chunks produce no hints. Validate `href` format is `assets/{name}.js`.

### TR4: Dynamic Import with Circular Dependencies

```yaml
id: TR4
priority: high
tests_requirement: R2
```

Test that when dynamic `import()` targets form a cycle (A dynamically imports B, B dynamically imports A), the splitter produces async chunks for both without infinite loops. Each async chunk should contain only its own module (the other is a separate async chunk, not inlined).

### TR5: Diamond Dependency with Dynamic Boundary

```yaml
id: TR5
priority: medium
tests_requirement: R2
```

Test diamond graph: entry→A (static), entry→B (dynamic), A→C (static), B→C (static). Module C is reachable from both entry chunk (via A) and async chunk (via B), so C must be extracted to a shared chunk. Validates shared extraction across static/dynamic boundaries.

### TR6: No-Module Async Chunk Edge Case

```yaml
id: TR6
priority: medium
tests_requirement: R2
```

Test that a dynamic import target with no further dependencies produces an async chunk containing only itself. Verifies the BFS termination and that the entry chunk does not absorb the target.
## Scenarios

<!-- type: scenarios lang: markdown -->

### S1: Two Entry Points Share a Utility Module (TR1)

1. Given `entry_a.js` → `shared_util.js` (static), `entry_b.js` → `shared_util.js` (static)
2. When `split_chunks` is called with `entry_a` as entry and all 3 modules
3. Then a `Shared` chunk exists containing `shared_util.js`
4. Then the entry chunk does NOT contain `shared_util.js`
5. Then `shared_util.js` appears in exactly one chunk

### S2: Multi-Entry Produces Disjoint Entry Chunks (TR2)

1. Given `entry_a.js` → `mod_a.js` (static), `entry_b.js` → `mod_b.js` (static), both → `common.js` (static)
2. When splitting from `entry_a`, the entry chunk contains `entry_a.js` and `mod_a.js` but NOT `mod_b.js`
3. When splitting from `entry_b`, the entry chunk contains `entry_b.js` and `mod_b.js` but NOT `mod_a.js`
4. Then `common.js` is in a shared chunk in both runs

### S3: Preload Hints Include Shared but Not Async (TR3)

1. Given entry → shared (static), entry → lazy (dynamic), lazy → shared (static)
2. When `split_chunks_with_config` is called
3. Then `preload_hints` contains an entry with `href: "assets/shared.js"` and `is_static: true`
4. Then `preload_hints` does NOT contain any entry referencing "lazy"

### S4: Circular Dynamic Imports Do Not Infinite Loop (TR4)

1. Given `main.js` → `a.js` (dynamic), `a.js` → `b.js` (dynamic), `b.js` → `a.js` (dynamic)
2. When `split_chunks` is called with `main.js` as entry
3. Then the function returns (does not hang)
4. Then async chunks exist for both `a.js` and `b.js`
5. Then the entry chunk contains only `main.js`

### S5: Diamond with Dynamic Boundary Extracts Shared (TR5)

1. Given `entry.js` → `a.js` (static), `entry.js` → `b.js` (dynamic), `a.js` → `c.js` (static), `b.js` → `c.js` (static)
2. When `split_chunks` is called
3. Then `c.js` is in a `Shared` chunk (reachable from entry via static A, and from async B)
4. Then `a.js` is in the entry chunk, `b.js` is in an async chunk
5. Then neither entry nor async chunk contains `c.js`

### S6: Leaf Dynamic Import Produces Single-Module Async Chunk (TR6)

1. Given `main.js` → `leaf.js` (dynamic), `leaf.js` has no further deps
2. When `split_chunks` is called
3. Then one async chunk exists containing exactly `[leaf.js]`
4. Then the entry chunk contains exactly `[main.js]`
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan

<!-- type: test-plan lang: markdown -->

### Execution

```bash
# All splitting tests
cargo test -p cclab-jet splitting::tests

# Specific new tests
cargo test -p cclab-jet test_multi_entry_shared_extraction
cargo test -p cclab-jet test_multi_entry_disjoint_chunks
cargo test -p cclab-jet test_preload_hints_multi_chunk
cargo test -p cclab-jet test_circular_dynamic_imports
cargo test -p cclab-jet test_diamond_dynamic_boundary_shared
cargo test -p cclab-jet test_leaf_dynamic_import_single_chunk
```

### Test Matrix

| Test Function | File | Req | Scenario | Assertions |
|---------------|------|-----|----------|------------|
| `test_multi_entry_shared_extraction` | `splitting.rs` | TR1 | S1 | Shared chunk exists, entry chunk excludes shared module |
| `test_multi_entry_disjoint_chunks` | `splitting.rs` | TR2 | S2 | Each entry produces disjoint module sets, common in shared |
| `test_preload_hints_multi_chunk` | `splitting.rs` | TR3 | S3 | Shared chunk has preload hint, async chunk has none |
| `test_circular_dynamic_imports` | `splitting.rs` | TR4 | S4 | Returns without hang, async chunks for both cycle members |
| `test_diamond_dynamic_boundary_shared` | `splitting.rs` | TR5 | S5 | C in shared, A in entry, B in async, no duplication |
| `test_leaf_dynamic_import_single_chunk` | `splitting.rs` | TR6 | S6 | Async chunk has exactly 1 module, entry has exactly 1 |

### Pass Criteria

- All 6 new tests pass: `cargo test -p cclab-jet splitting::tests` exits 0
- All 7 existing `splitting::tests` continue to pass (no regressions)
- No new `#[ignore]` annotations
- Total splitting test count: 13 (7 existing + 6 new)
## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  # TR1 + TR2: Multi-entry shared extraction and disjoint chunks
  - path: crates/cclab-jet/src/bundler/splitting.rs
    action: MODIFY
    desc: |
      Add `test_multi_entry_shared_extraction` in `mod tests`:
      Build graph: entry_a→shared_util (static), entry_b→shared_util (static).
      Call `split_chunks(&entry_a, &edges, &all)`. Assert Shared chunk
      contains shared_util, entry chunk excludes it.

      Add `test_multi_entry_disjoint_chunks` in `mod tests`:
      Build graph: entry_a→mod_a (static), entry_a→common (static),
      entry_b→mod_b (static), entry_b→common (static).
      Call `split_chunks` twice (once per entry). Assert each entry chunk
      contains only its own modules, common is in Shared chunk.

  # TR3: Async chunk preload metadata
  - path: crates/cclab-jet/src/bundler/splitting.rs
    action: MODIFY
    desc: |
      Add `test_preload_hints_multi_chunk` in `mod tests`:
      Build graph with entry→shared (static), entry→lazy (dynamic),
      lazy→shared (static). Call `split_chunks_with_config` with
      default ManualChunkConfig. Assert preload_hints contains
      "assets/shared.js" with is_static=true. Assert no hint for "lazy".

  # TR4: Circular dynamic imports
  - path: crates/cclab-jet/src/bundler/splitting.rs
    action: MODIFY
    desc: |
      Add `test_circular_dynamic_imports` in `mod tests`:
      Build graph: main→a (dynamic), a→b (dynamic), b→a (dynamic).
      Call `split_chunks`. Assert function returns (no hang).
      Assert entry chunk contains only main. Assert async chunks
      exist for both a and b.

  # TR5: Diamond with dynamic boundary
  - path: crates/cclab-jet/src/bundler/splitting.rs
    action: MODIFY
    desc: |
      Add `test_diamond_dynamic_boundary_shared` in `mod tests`:
      Build graph: entry→a (static), entry→b (dynamic), a→c (static),
      b→c (static). Call `split_chunks`. Assert c is in Shared chunk.
      Assert a is in entry chunk. Assert b is in async chunk.
      Assert c not duplicated in entry or async chunks.

  # TR6: Leaf dynamic import
  - path: crates/cclab-jet/src/bundler/splitting.rs
    action: MODIFY
    desc: |
      Add `test_leaf_dynamic_import_single_chunk` in `mod tests`:
      Build graph: main→leaf (dynamic), leaf has no deps.
      Call `split_chunks`. Assert async chunk modules == [leaf].
      Assert entry chunk modules == [main].
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews

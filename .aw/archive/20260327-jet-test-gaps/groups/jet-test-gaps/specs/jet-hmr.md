---
id: jet-hmr
main_spec_ref: "crates/cclab-jet/logic/hmr.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, test-plan, changes]
filled_sections: [overview, requirements, scenarios, test-plan, changes]
create_complete: true
---

# Jet Hmr

## Overview

<!-- type: overview lang: markdown -->

Add unit tests for the dev server HMR subsystem: module graph invalidation, circular dependency handling, and HMR boundary detection. Closes the second major test coverage gap identified in jet.

### Current Test Coverage

| Area | File | Existing Tests | Gap |
|------|------|----------------|-----|
| Graph build / edges | `module_graph.rs` | `t5_graph_built_from_imports` | None |
| Graph update / stale edges | `module_graph.rs` | `t6_graph_update_removes_stale_edges` | None |
| Self-accept boundary | `module_graph.rs` | `t7_invalidation_walk_finds_self_accept_boundary` | None |
| Parent-accept boundary | `module_graph.rs` | `t8_invalidation_walk_propagates_to_parent` | None |
| No boundary → FullReload | `module_graph.rs` | `t9_invalidation_walk_returns_full_reload` | None |
| React Refresh boundary | `module_graph.rs` | `invalidation_walk_finds_react_refresh_boundary` | None |
| Module removal / orphans | `module_graph.rs` | `remove_module_returns_orphans` | None |
| Unknown module | `module_graph.rs` | `find_boundary_unknown_module_returns_full_reload` | None |
| Self-accepting parent | `module_graph.rs` | `self_accepting_parent_acts_as_boundary` | None |
| Circular import chain | `module_graph.rs` | None | **No coverage** |
| Diamond graph multi-target | `module_graph.rs` | None | **No coverage** |
| Deep chain BFS traversal | `module_graph.rs` | None | **No coverage** |
| HmrUpdateResult parent-accept | `hmr.rs` | None | **No coverage** |
| HmrUpdateResult React Refresh | `hmr.rs` | None | **No coverage** |
| HmrMessage Connected serde | `hmr.rs` | None | **No coverage** |
| HmrMessage Update+acceptedBy | `hmr.rs` | None | **No coverage** |
| HmrMessage Error minimal | `hmr.rs` | None | **No coverage** |
| HmrManager broadcast/receive | `hmr.rs` | None | **No coverage** |

### Target Coverage

- Circular imports: A→B→C→A cycle — BFS terminates, finds boundary or returns FullReload
- Diamond graph: multiple importers each with boundaries — all targets collected
- Deep chain: 5+ modules — BFS traverses full depth without issue
- HmrUpdateResult: bridge from module graph to HMR result for parent-accept and React Refresh
- HmrMessage: serialization round-trip for all untested variants and field combinations
- HmrManager: async broadcast delivers messages to subscribers

### Scope

Rust `#[test]` functions in `crates/cclab-jet/src/dev_server/module_graph.rs` and `crates/cclab-jet/src/dev_server/hmr.rs`. No E2E tests.
## Requirements

<!-- type: requirements lang: markdown -->

### TR1: Circular Import Chain — BFS Termination

```yaml
id: TR1
priority: high
tests_requirement: R2
```

Test that circular imports (A→B→C→A) with no HMR boundary do not cause `find_hmr_boundary` to infinite loop. The BFS visited set must break the cycle. Returns `FullReload` since no boundary exists.

### TR2: Circular Import Chain with Boundary

```yaml
id: TR2
priority: high
tests_requirement: R2, R5
```

Test circular imports (A→B→C→A) where B is self-accepting. Changing C propagates C→A→B, finds B as boundary. Verifies BFS cycle handling and boundary detection work together.

### TR3: Diamond Import Graph — Multiple Boundary Targets

```yaml
id: TR3
priority: high
tests_requirement: R2, R5
```

Test diamond: entry→A (self-accepting), entry→B (self-accepting), A→util, B→util. Changing util propagates up both paths. Both A and B must be collected as hot-update targets.

### TR4: Deep Import Chain — BFS Handles Long Paths

```yaml
id: TR4
priority: medium
tests_requirement: R2, R5
```

Test chain of 5 modules: entry→m1→m2→m3→m4→leaf. Only m1 is self-accepting. Changing leaf must traverse 4 levels up to find m1 as boundary. Verifies BFS handles depth correctly.

### TR5: HmrUpdateResult Bridges Parent-Accept and React Refresh

```yaml
id: TR5
priority: high
tests_requirement: R5
```

Test `HmrUpdateResult::determine` for two untested paths: (1) parent module accepts the changed dep, (2) changed module has React Fast Refresh. Verifies the bridge from `ModuleGraph::find_hmr_boundary` to `HmrUpdateResult`.

### TR6: HmrMessage Serialization Coverage

```yaml
id: TR6
priority: medium
tests_requirement: R1, R3
```

Test serialization of untested `HmrMessage` variants: `Connected` (no fields), `Update` with `acceptedBy` populated, `Error` with only required `message` field (file/line/column absent). Ensures serde round-trip correctness.

### TR7: HmrManager Broadcast and Receive

```yaml
id: TR7
priority: medium
tests_requirement: R3
```

Test that `HmrManager::broadcast` delivers messages to active subscribers. After broadcast, subscriber receives the exact `HmrMessage`. Verifies the tokio broadcast channel integration.
## Scenarios

<!-- type: scenarios lang: markdown -->

### S1: Circular Import Chain Returns FullReload (TR1)

1. Given module graph: A→B (static), B→C (static), C→A (static) — none self-accepting, no React Refresh
2. When `find_hmr_boundary("/src/C.tsx")` is called
3. Then the function returns (does not hang)
4. Then result is `FullReload` with reason containing "no HMR boundary"

### S2: Circular Import Chain Finds Self-Accepting Boundary (TR2)

1. Given module graph: A→B (static), B→C (static), C→A (static) — B is self-accepting
2. When `find_hmr_boundary("/src/C.tsx")` is called
3. Then result is `HotUpdate` with targets containing `/src/B.tsx`
4. Then A is NOT in targets (B is the boundary, walk stops there)

### S3: Diamond Graph Collects Multiple Targets (TR3)

1. Given module graph: entry→A, entry→B, A→util, B→util — A and B both self-accepting
2. When `find_hmr_boundary("/src/util.ts")` is called
3. Then result is `HotUpdate` with targets containing both `/src/A.tsx` and `/src/B.tsx`
4. Then entry is NOT in targets

### S4: Deep Chain Traverses to Distant Boundary (TR4)

1. Given module graph: entry→m1→m2→m3→m4→leaf — only m1 is self-accepting
2. When `find_hmr_boundary("/src/leaf.ts")` is called
3. Then result is `HotUpdate` with targets containing `/src/m1.tsx`
4. Then m2, m3, m4 are NOT in targets

### S5: HmrUpdateResult Determine Parent-Accept (TR5)

1. Given module graph: parent→child, parent has `accepted_deps` containing `/src/child.tsx`
2. When `HmrUpdateResult::determine("/src/child.tsx", &graph)` is called
3. Then result is `HotUpdate` with targets containing `/src/parent.tsx`

### S6: HmrUpdateResult Determine React Refresh (TR5)

1. Given module graph: module `/src/App.tsx` with `has_react_refresh = true`, no importers
2. When `HmrUpdateResult::determine("/src/App.tsx", &graph)` is called
3. Then result is `HotUpdate` with targets containing `/src/App.tsx`

### S7: Connected Message Serializes Correctly (TR6)

1. Given `HmrMessage::Connected`
2. When serialized to JSON
3. Then output is `{"type":"connected"}`
4. Then no extra fields present

### S8: Update Message with acceptedBy Serializes (TR6)

1. Given `HmrMessage::Update { path: "/src/dep.tsx", timestamp: 1000, accepted_by: Some("/src/parent.tsx") }`
2. When serialized to JSON
3. Then `acceptedBy` field is present with value `/src/parent.tsx`
4. Then `type` is `update`

### S9: Error Message with Minimal Fields (TR6)

1. Given `HmrMessage::Error { message: "fail", file: None, line: None, column: None, frame: None }`
2. When serialized to JSON
3. Then only `type` and `message` fields are present
4. Then `file`, `line`, `column`, `frame` are absent (not null)

### S10: HmrManager Broadcast Delivers to Subscriber (TR7)

1. Given `HmrManager::new()` with one subscriber
2. When `broadcast(HmrMessage::Connected)` is called
3. Then subscriber receives `HmrMessage::Connected`
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
# All module_graph tests
cargo test -p cclab-jet module_graph::tests

# All hmr tests
cargo test -p cclab-jet hmr::tests

# Specific new tests
cargo test -p cclab-jet test_circular_import_chain_bfs_termination
cargo test -p cclab-jet test_circular_import_chain_with_boundary
cargo test -p cclab-jet test_diamond_graph_multiple_targets
cargo test -p cclab-jet test_deep_chain_bfs_traversal
cargo test -p cclab-jet determine_hot_update_for_parent_accept
cargo test -p cclab-jet determine_hot_update_for_react_refresh
cargo test -p cclab-jet hmr_message_connected_serialization
cargo test -p cclab-jet hmr_message_update_with_accepted_by
cargo test -p cclab-jet hmr_message_error_minimal_fields
cargo test -p cclab-jet hmr_manager_broadcast_receive
```

### Test Matrix

| Test Function | File | Req | Scenario | Assertions |
|---------------|------|-----|----------|------------|
| `test_circular_import_chain_bfs_termination` | `module_graph.rs` | TR1 | S1 | Function returns, result is FullReload |
| `test_circular_import_chain_with_boundary` | `module_graph.rs` | TR2 | S2 | B in targets, A not in targets, no hang |
| `test_diamond_graph_multiple_targets` | `module_graph.rs` | TR3 | S3 | Both A and B in targets, entry not |
| `test_deep_chain_bfs_traversal` | `module_graph.rs` | TR4 | S4 | m1 in targets, intermediates not |
| `determine_hot_update_for_parent_accept` | `hmr.rs` | TR5 | S5 | HotUpdate with parent as target |
| `determine_hot_update_for_react_refresh` | `hmr.rs` | TR5 | S6 | HotUpdate with module as target |
| `hmr_message_connected_serialization` | `hmr.rs` | TR6 | S7 | JSON has type=connected, no extra fields |
| `hmr_message_update_with_accepted_by` | `hmr.rs` | TR6 | S8 | acceptedBy field present in JSON |
| `hmr_message_error_minimal_fields` | `hmr.rs` | TR6 | S9 | Only type+message in JSON, optional fields absent |
| `hmr_manager_broadcast_receive` | `hmr.rs` | TR7 | S10 | Subscriber receives broadcast message |

### Pass Criteria

- All 10 new tests pass
- `cargo test -p cclab-jet module_graph::tests` exits 0 (9 existing + 4 new = 13 total)
- `cargo test -p cclab-jet hmr::tests` exits 0 (9 existing + 6 new = 15 total)
- No new `#[ignore]` annotations
- No regressions in existing tests
## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  # TR1 + TR2: Circular import chain tests
  - path: crates/cclab-jet/src/dev_server/module_graph.rs
    action: MODIFY
    desc: |
      Add `test_circular_import_chain_bfs_termination` in `mod tests`:
      Build graph: A→B (static), B→C (static), C→A (static). None self-accepting.
      Call `find_hmr_boundary("/src/C.tsx")`. Assert function returns (no hang).
      Assert result is FullReload with reason containing "no HMR boundary".

      Add `test_circular_import_chain_with_boundary` in `mod tests`:
      Build graph: A→B (static), B→C (static), C→A (static). B is self-accepting.
      Call `find_hmr_boundary("/src/C.tsx")`. Assert result is HotUpdate.
      Assert targets contains B. Assert A is NOT in targets.

  # TR3: Diamond graph multiple targets
  - path: crates/cclab-jet/src/dev_server/module_graph.rs
    action: MODIFY
    desc: |
      Add `test_diamond_graph_multiple_targets` in `mod tests`:
      Build graph: entry→A, entry→B, A→util, B→util. A and B both self-accepting.
      Call `find_hmr_boundary("/src/util.ts")`. Assert result is HotUpdate.
      Assert targets contains both A and B. Assert entry is NOT in targets.

  # TR4: Deep chain BFS traversal
  - path: crates/cclab-jet/src/dev_server/module_graph.rs
    action: MODIFY
    desc: |
      Add `test_deep_chain_bfs_traversal` in `mod tests`:
      Build graph: entry→m1→m2→m3→m4→leaf. Only m1 is self-accepting.
      Call `find_hmr_boundary("/src/leaf.ts")`. Assert result is HotUpdate.
      Assert targets contains m1. Assert m2, m3, m4, leaf NOT in targets.

  # TR5: HmrUpdateResult determine with parent-accept and React Refresh
  - path: crates/cclab-jet/src/dev_server/hmr.rs
    action: MODIFY
    desc: |
      Add `determine_hot_update_for_parent_accept` in `mod tests`:
      Build graph: parent→child. Set parent accepted_deps containing child.
      Call `HmrUpdateResult::determine("/src/child.tsx", &graph)`.
      Assert result is HotUpdate with targets containing parent.

      Add `determine_hot_update_for_react_refresh` in `mod tests`:
      Build graph: module with has_react_refresh=true, no importers.
      Call `HmrUpdateResult::determine("/src/App.tsx", &graph)`.
      Assert result is HotUpdate with targets containing App.tsx.

  # TR6: HmrMessage serialization coverage
  - path: crates/cclab-jet/src/dev_server/hmr.rs
    action: MODIFY
    desc: |
      Add `hmr_message_connected_serialization` in `mod tests`:
      Serialize HmrMessage::Connected. Assert JSON is {"type":"connected"}.
      Assert no extra fields present.

      Add `hmr_message_update_with_accepted_by` in `mod tests`:
      Serialize HmrMessage::Update with accepted_by = Some("/src/parent.tsx").
      Assert JSON contains "acceptedBy" field with correct value.

      Add `hmr_message_error_minimal_fields` in `mod tests`:
      Serialize HmrMessage::Error with only message, all Option fields None.
      Assert JSON contains only "type" and "message" keys.
      Assert file, line, column, frame are absent (skip_serializing_if).

  # TR7: HmrManager broadcast/receive
  - path: crates/cclab-jet/src/dev_server/hmr.rs
    action: MODIFY
    desc: |
      Add `hmr_manager_broadcast_receive` in `mod tests` (async with #[tokio::test]):
      Create HmrManager, subscribe once. Broadcast HmrMessage::Connected.
      Assert subscriber receives Connected message via rx.recv().await.
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

---
id: gc
title: Cycle-Detecting Garbage Collector
crate: mamba
files:
  - crates/mamba/src/runtime/gc.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 91213c22f
---

# Cycle-Detecting Garbage Collector

Mamba's GC is a CPython-style trial-deletion cycle collector running
on top of atomic reference counting. RC handles the common case (no
cycles); the GC sweeps periodically to break reference cycles in
container objects (list, dict, instance). Per-thread state — no
stop-the-world coordination, no cross-thread locks. The threshold
auto-triggers at 700 allocations since last collection.

Three load-bearing invariants:

1. **Trial deletion = refcount minus internal references** — Phase 2
   walks every tracked object's children; if a child is also tracked,
   decrement its `gc_refs`. Surviving `gc_refs > 0` means the object
   has *external* references (JIT locals, globals, non-tracked
   objects) and must be kept. This is the CPython algorithm; deviating
   from the visitor pattern (e.g., walking only one level) breaks
   cycle detection.
2. **`IMMORTAL_REFCOUNT` is unconditionally skipped** — JIT-embedded
   constants (rc = u32::MAX) are never enrolled in trial deletion,
   never marked, never freed.
3. **Re-entrant collection is forbidden** — the `collecting` flag
   gates `collect()`. If the visitor re-enters via a side effect
   (it shouldn't, but `Drop` impls might), the second call returns 0
   and the original sweep continues.

Safepoint protocol (`gc_safepoint`, `gc_register_thread`,
`gc_unregister_thread`) is now a no-op set kept only for call-site
compatibility with async_rt / class / iter — per-thread GC means no
cross-thread coordination is required.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: gc-types
types:
  GcState:        { kind: struct }
  Gc:             { kind: struct, label: "thread_local RefCell<GcState>" }
  TrackedSet:     { kind: struct, label: "HashSet<usize> — heap addresses of containers" }
  RootSet:        { kind: struct, label: "Vec<MbValue> — explicit roots from globals/stack" }
  MbObject:       { kind: struct, label: "from runtime::rc" }
  CollectorEntry: { kind: struct, label: "collect() — trial deletion 4-phase" }
edges:
  - { from: Gc,             to: GcState,    kind: owns }
  - { from: GcState,        to: TrackedSet, kind: owns }
  - { from: GcState,        to: RootSet,    kind: owns }
  - { from: TrackedSet,     to: MbObject,   kind: references, label: "addr → ptr" }
  - { from: RootSet,        to: MbObject,   kind: references, label: "value.as_ptr" }
  - { from: CollectorEntry, to: GcState,    kind: references, label: "borrow + reset alloc_count" }
---
classDiagram
    class GcState
    class Gc
    class TrackedSet
    class RootSet
    class MbObject
    class CollectorEntry
    Gc --> GcState : owns
    GcState --> TrackedSet : owns
    GcState --> RootSet : owns
    TrackedSet --> MbObject : addr
    RootSet --> MbObject : ptr
    CollectorEntry --> GcState : borrow
```

## GC state shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "gc-types"
$defs:
  GcState:
    type: object
    x-rust-type: GcState
    properties:
      tracked:
        type: array
        items: { type: integer, x-rust-type: usize, description: "heap address of container" }
        description: "HashSet<usize> in code"
      alloc_count:
        type: integer
        minimum: 0
        description: "incremented on every container alloc; reset at end of collect"
      threshold:
        type: integer
        minimum: 1
        default: 700
        description: "auto-trigger collection when alloc_count >= threshold"
      collections:
        type: integer
        x-rust-type: u64
        description: "lifetime collection count (counter only, never reset)"
      collecting:
        type: boolean
        description: "re-entrancy guard"
      enabled:
        type: boolean
        description: "gc_disable / gc_enable toggles automatic collection"
      roots:
        type: array
        items: { x-rust-type: MbValue }
        description: "explicit roots — globals + JIT-marked stack values"
    required: [tracked, alloc_count, threshold, collections, collecting, enabled, roots]
```

## Collection lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: gc-lifecycle
initial: Idle
nodes:
  Idle:        { kind: initial,  label: "alloc_count < threshold OR enabled = false" }
  Triggered:   { kind: transient, label: "alloc_count >= threshold AND enabled" }
  Snapshotting:{ kind: transient, label: "collecting = true; copy tracked + roots" }
  Phase1Init:  { kind: transient, label: "init gc_refs[addr] = rc (skip IMMORTAL)" }
  Phase2Sub:   { kind: transient, label: "subtract internal refs via visit_contained" }
  Phase3Mark:  { kind: transient, label: "mark survivors (gc_refs > 0 OR explicit roots)" }
  Phase4Sweep: { kind: transient, label: "free unmarked tracked objects" }
  PostSweep:   { kind: normal,    label: "alloc_count = 0; collections += 1; collecting = false" }
edges:
  - { from: Idle,         to: Triggered,    event: "container alloc → bump alloc_count" }
  - { from: Triggered,    to: Snapshotting, event: "collect() called" }
  - { from: Snapshotting, to: Phase1Init }
  - { from: Phase1Init,   to: Phase2Sub }
  - { from: Phase2Sub,    to: Phase3Mark }
  - { from: Phase3Mark,   to: Phase4Sweep }
  - { from: Phase4Sweep,  to: PostSweep }
  - { from: PostSweep,    to: Idle }
  - { from: Snapshotting, to: Idle, event: "re-entrant guard: collecting already true" }
---
stateDiagram-v2
    [*] --> Idle
    Idle --> Triggered: alloc threshold
    Triggered --> Snapshotting: collect
    Snapshotting --> Phase1Init: init gc_refs
    Phase1Init --> Phase2Sub: subtract internal
    Phase2Sub --> Phase3Mark: mark survivors
    Phase3Mark --> Phase4Sweep: free unmarked
    Phase4Sweep --> PostSweep: stats update
    PostSweep --> Idle: reset
    Snapshotting --> Idle: re-entrant skip
```

## Trial deletion logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: gc-trial-deletion
entry: enter
nodes:
  enter:        { kind: start,    label: "collect()" }
  reentrant:    { kind: decision, label: "collecting already true?" }
  short:        { kind: terminal, label: "return 0 (re-entrant skip)" }
  snapshot:     { kind: process,  label: "copy tracked + roots; set collecting = true" }
  init_refs:    { kind: process,  label: "Phase 1: gc_refs[addr] = rc (skip IMMORTAL)" }
  subtract:     { kind: process,  label: "Phase 2: visit_contained children; if tracked, gc_refs[child] -= 1" }
  identify:     { kind: process,  label: "Phase 3a: candidates = addr where gc_refs > 0" }
  mark_dfs:     { kind: process,  label: "Phase 3b: DFS-mark from candidates and explicit roots" }
  sweep:        { kind: process,  label: "Phase 4: for addr in tracked: if not marked, drop_inner + Box::from_raw" }
  reset:        { kind: process,  label: "alloc_count = 0; collections += 1; collecting = false" }
  done:         { kind: terminal, label: "return freed_count" }
edges:
  - { from: enter,      to: reentrant }
  - { from: reentrant,  to: short,     label: "yes" }
  - { from: reentrant,  to: snapshot,  label: "no" }
  - { from: snapshot,   to: init_refs }
  - { from: init_refs,  to: subtract }
  - { from: subtract,   to: identify }
  - { from: identify,   to: mark_dfs }
  - { from: mark_dfs,   to: sweep }
  - { from: sweep,      to: reset }
  - { from: reset,      to: done }
---
flowchart TD
    enter([collect]) --> reentrant{collecting already?}
    reentrant -->|yes| short([return 0])
    reentrant -->|no| snapshot[snapshot tracked + roots]
    snapshot --> init_refs[P1: gc_refs = rc]
    init_refs --> subtract[P2: subtract internal]
    subtract --> identify[P3a: candidates gc_refs > 0]
    identify --> mark_dfs[P3b: DFS-mark]
    mark_dfs --> sweep[P4: free unmarked]
    sweep --> reset[reset stats]
    reset --> done([freed_count])
```

## Auto-trigger interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: gc-auto-trigger
actors:
  - { id: JIT,       kind: system, label: "JIT-emitted code" }
  - { id: Allocator, kind: system, label: "mb_list_new / mb_dict_new / ..." }
  - { id: Gc,        kind: system, label: "gc.rs" }
  - { id: Visitor,   kind: system, label: "visit_contained / mark_object" }
messages:
  - { from: JIT,       to: Allocator, name: "create container" }
  - { from: Allocator, to: Gc,        name: "gc_track(ptr); gc_bump_alloc_count" }
  - { from: Gc,        to: Gc,        name: "alloc_count += 1; check vs threshold" }
  - { from: Gc,        to: Gc,        name: "if >= threshold AND enabled AND not collecting: collect()" }
  - { from: Gc,        to: Visitor,   name: "Phase 2: visit_contained for each tracked" }
  - { from: Visitor,   to: Gc,        name: "decrement gc_refs[child] for tracked children" }
  - { from: Gc,        to: Visitor,   name: "Phase 3: mark from gc_refs > 0 + explicit roots" }
  - { from: Visitor,   to: Gc,        name: "marked set populated" }
  - { from: Gc,        to: Gc,        name: "Phase 4: free unmarked" }
  - { from: Gc,        to: Allocator, name: "alloc_count = 0; return ptr to caller" }
  - { from: Allocator, to: JIT,       name: "container handle" }
---
sequenceDiagram
    participant JIT
    participant Allocator
    participant Gc
    participant Visitor
    JIT->>Allocator: create container
    Allocator->>Gc: gc_track ptr
    Gc->>Gc: alloc_count += 1
    Gc->>Gc: threshold? → collect
    Gc->>Visitor: P2 visit_contained
    Visitor->>Gc: dec gc_refs of tracked children
    Gc->>Visitor: P3 mark from survivors
    Visitor->>Gc: marked
    Gc->>Gc: P4 free unmarked
    Gc->>Allocator: alloc_count = 0
    Allocator-->>JIT: container handle
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: list-cycle
    given: data_structures/cycle.py creates a list that references itself
    when: the last external list reference is deleted and collection runs
    then: trial deletion identifies the cycle as unreachable and frees it
  - id: instance-cycle
    given: data_structures/instance_cycle.py creates two instances that reference each other
    when: both external instance references are deleted
    then: internal references subtract to zero and both instances are swept
  - id: manual-collect
    given: stdlib/gc_basic.py imports gc
    when: gc.collect executes
    then: explicit collection returns the freed count and preserves CPython-style API behavior
  - id: disable-enable
    given: stdlib/gc_basic.py disables automatic collection
    when: containers are allocated while gc.isenabled is false
    then: allocation count can rise without triggering auto-collection until gc.enable restores it
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: runtime-gc-test-plan
title: Cycle-Detecting Garbage Collector Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> ListCycle["data_structures/cycle.py"]
    Runner --> InstanceCycle["data_structures/instance_cycle.py"]
    Runner --> GcBasic["stdlib/gc_basic.py"]
    Runner --> ImmortalConst["language/jit_const_immortal.py"]
    Runner --> NestedDictCycle["data_structures/nested_dict_cycle.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/gc.rs
    action: modify
    impl_mode: hand-written
    description: "Per-thread cycle-detecting GC: GcState (tracked / roots / alloc_count / threshold=700 / collecting / enabled), trial-deletion 4-phase collect, gc_track / gc_untrack / gc_add_root / gc_remove_root, gc_safepoint no-op stubs. Hand-written; visitor algorithm mirrors CPython."
```

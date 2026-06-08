---
id: stdlib-time
title: stdlib time — Wall Clock and Sleep
crate: mamba
files:
  - crates/mamba/src/runtime/stdlib/time_mod.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 8323393b7
---

# stdlib `time`

Clock and sleep primitives. 4 entries today via `std::time` /
`std::thread::sleep`. CPython has many more (gmtime / localtime /
strftime / asctime / ctime / mktime / strptime / time_ns / etc.) —
all open gaps.

Three load-bearing invariants:

1. **`time.time()` returns float seconds since epoch** — UTC; uses
   `std::time::SystemTime::UNIX_EPOCH`. Not nanosecond-aligned today;
   `time.time_ns()` is gap.
2. **`time.sleep(secs)` blocks the calling OS thread** — not
   coroutine-aware. For async sleep use `asyncio.sleep` (per
   `runtime/async.md`).
3. **`time.monotonic()` and `time.perf_counter()` are distinct
   clocks** — monotonic guarantees no backward jumps;
   perf_counter offers higher precision. Mamba uses `Instant` for
   both (effectively the same source today).

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: time-types
types:
  TimeMod:    { kind: struct }
  StdTime:    { kind: struct, label: "std::time::SystemTime / Instant" }
  StdThread:  { kind: struct, label: "std::thread::sleep" }
edges:
  - { from: TimeMod, to: StdTime,   kind: references }
  - { from: TimeMod, to: StdThread, kind: references, label: "blocking sleep" }
---
classDiagram
    class TimeMod
    class StdTime
    class StdThread
    TimeMod --> StdTime : clocks
    TimeMod --> StdThread : sleep
```

## Function catalog
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "time-catalog"
$defs:
  StdlibFnEntry:
    type: object
    properties:
      python_name:    { type: string }
      mb_fn:          { type: string }
      arity:          { type: integer }
      cpython_parity: { type: string, enum: [full, partial, gap] }
      notes:          { type: string }
    required: [python_name, mb_fn, arity, cpython_parity]
  TimeCatalog:
    type: array
    items: { $ref: "#/$defs/StdlibFnEntry" }
    examples:
      - - { python_name: "time.time",         mb_fn: "mb_time_time",         arity: 0, cpython_parity: full }
        - { python_name: "time.monotonic",    mb_fn: "mb_time_monotonic",    arity: 0, cpython_parity: full }
        - { python_name: "time.perf_counter", mb_fn: "mb_time_perf_counter", arity: 0, cpython_parity: full }
        - { python_name: "time.sleep",        mb_fn: "mb_time_sleep",        arity: 1, cpython_parity: full }
        - { python_name: "time.time_ns",      mb_fn: "(gap)",                arity: 0, cpython_parity: gap }
        - { python_name: "time.gmtime / localtime / strftime / strptime", mb_fn: "(gap)", arity: -1, cpython_parity: gap }
```

## Acceptance scenarios
<!-- type: overview lang: markdown -->

```mermaid
---
id: time-acceptance
actors:
  - { id: User,    kind: actor }
  - { id: Mamba,   kind: system }
  - { id: Fixture, kind: system }
messages:
  - { from: User,    to: Mamba,   name: "run stdlib/time_basic.py" }
  - { from: Mamba,   to: Fixture, name: "t = time.time(); time.sleep(0.01); time.time() - t > 0.01" }
  - { from: Fixture, to: Mamba,   name: "True (elapsed reflects sleep)" }
---
sequenceDiagram
    actor User
    participant Mamba
    participant Fixture
    User->>Mamba: time
    Mamba->>Fixture: time + sleep
    Fixture-->>Mamba: elapsed
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
runner: "cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"
fixtures:
  - id: time_basic
    name: "stdlib/time_basic.py"
    paired: "stdlib/time_basic.expected"
  - id: time_monotonic
    name: "stdlib/time_monotonic.py"
    paired: "stdlib/time_monotonic.expected"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/stdlib/time_mod.rs
    action: modify
    impl_mode: hand-written
    description: "time / sleep / monotonic / perf_counter. Hand-written; gmtime / localtime / strftime / strptime / time_ns are gaps."
```

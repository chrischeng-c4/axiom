---
id: mamba-language-core-gate-closure-call-hot-with-perf-pin
summary: Bring release closure-call throughput to an enforceable CPython-relative perf gate without weakening closure cell semantics.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-closure-call-hot-contract
entry: dynamic closure call
nodes:
  handle: { kind: start, label: closure handle }
  cached: { kind: process, label: read cached closure module and qualname metadata }
  context: { kind: process, label: install semantic module and qualname context only when required }
  cells: { kind: process, label: install capture cells }
  dispatch: { kind: process, label: dispatch JIT frame }
  result: { kind: terminal, label: preserve result exception and ownership semantics }
edges:
  - { from: handle, to: cached }
  - { from: cached, to: context }
  - { from: context, to: cells }
  - { from: cells, to: dispatch }
  - { from: dispatch, to: result }
---
flowchart TD
    handle([closure handle]) --> cached[read cached closure module and qualname metadata]
    cached --> context[install semantic module and qualname context only when required]
    context --> cells[install capture cells]
    cells --> dispatch[dispatch JIT frame]
    dispatch --> result([preserve result exception and ownership semantics])
```

For a closure handle, `with_callable_module` must read `MbClosure.module`, `MbClosure.qualname`, and `MbClosure.name` directly through the closure registry rather than allocating temporary MbValue strings through `mb_func_get_module`, `mb_func_get_qualname`, and `mb_func_get_name`. It must retain the existing push/pop behavior whenever module or qualname context is needed, preserving nested definition qualification, traceback state, and module isolation. Non-closure callables retain the current path. This is the first measured dispatch optimization; the release perf pin remains the terminal contract and requires further specialization if the measured ratio is still below its floor.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/src/runtime/closure.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Closure metadata and invocation context require runtime ownership and reentrancy judgement not derivable by the current generator.
  - path: projects/mamba/src/runtime/class/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Dynamic closure dispatch must route through the cached context path without changing non-closure semantics.
  - path: projects/mamba/tests/cpython/_regression/core/language/closures/bench/closure_call_hot.py
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-perf-pin
    tracker: "#1478"
    reason: Perf pin measurement requires the canonical internal-time marker.
  - path: projects/mamba/tests/harness/cpython/config/perf/pins/closure_call_hot_1478.toml
    action: create
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-perf-pin
    tracker: "#1478"
    reason: The closure hot path needs an executable CPU and RSS perf gate.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: mamba-closure-call-hot-contract-verification
requirements:
  cached_context:
    id: R1
    text: "A closure callable-context lookup reads cached metadata without allocating MbValue metadata objects."
    kind: unit
    risk: high
    verify: runtime::closure::tests::closure_callable_context_uses_cached_metadata
  perf_gate:
    id: R3
    text: "The release closure workload has a real internal-time CPU/RSS pin result."
    kind: performance
    risk: high
    verify: perf_pin::closure_call_hot_1478
  semantics:
    id: R2
    text: "Nested closure capture and module context behavior remain CPython-conformant."
    kind: regression
    risk: high
    verify: conformance::_regression/core/language/closures/behavior.py
---
flowchart TD
    r1[R1 cached context] --> runtime_closure_tests_closure_callable_context_uses_cached_metadata[runtime::closure::tests::closure_callable_context_uses_cached_metadata]
    r2[R2 semantics] --> conformance_regression_core_language_closures_behavior_py[conformance::_regression/core/language/closures/behavior.py]
    r3[R3 perf gate] --> perf_pin_closure_call_hot_1478[perf_pin::closure_call_hot_1478]
```

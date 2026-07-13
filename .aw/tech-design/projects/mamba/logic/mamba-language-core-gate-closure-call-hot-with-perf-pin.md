---
id: mamba-language-core-gate-closure-call-hot-with-perf-pin
summary: Bring release closure-call throughput to an enforceable externally measured CPython-relative perf gate without weakening closure cell semantics.
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

For a closure handle, `with_callable_module` must read `MbClosure.module`, `MbClosure.qualname`, and `MbClosure.name` directly through the closure registry rather than allocating temporary MbValue strings through `mb_func_get_module`, `mb_func_get_qualname`, and `mb_func_get_name`. Closure creation establishes the defining module cache from the active module so this path never falls back to metadata-object getters in a hot call loop. It must retain the existing push/pop behavior whenever module or qualname context is needed, preserving nested definition qualification, traceback state, and module isolation. Non-closure callables retain the current path. Dynamic argument-owner frames keep their ownership semantics while using inline storage for ordinary closure arities. Dynamic in-place dispatch must box raw primitive VRegs before calling `mb_i*` helpers when an unannotated module binding has semantic type `Any`. For the narrow `acc ^= closure(arg) & immediate_int` HIR shape, lowering may use one fused gateway only when the closure fast path and all runtime values are immediate integers; all other cases execute the exact generic call, `&`, and `^=` sequence. The pure typed companion shape `acc ^= closure(items[index % count]) & immediate_int` may additionally fold modulo and plain-list getitem into that same boundary; the existing modulo/getitem helpers retain their behavior and fallback. A stricter non-escaping factory result of the exact `lambda x: x + literal_int` shape may instead use an inline checked 48-bit add and must route overflow or any non-int value to that generic fallback. The bootstrap module registry keeps `builtins`, `sys`, and vendored search precedence at startup; the native stdlib shells are materialized at the first ordinary import, so an import-free closure workload does not pay their resident footprint. The benchmark prints its final accumulator so the hot loop remains an observable workload. The release perf pin remains the terminal contract.

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
  - path: projects/mamba/src/runtime/module.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Import-free closure workloads must avoid materializing unrelated native stdlib module shells while preserving first-import behavior.
  - path: projects/mamba/src/runtime/stdlib/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: The thread-local module registry needs a matching bootstrap/full-registration lifecycle.
  - path: projects/mamba/src/runtime/class/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Dynamic closure dispatch must route through the cached context path without changing non-closure semantics.
  - path: projects/mamba/src/lower/hir_to_mir.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: The narrow dynamic closure-and-immediate-mask HIR shape needs a single fused runtime gateway while retaining the generic fallback.
  - path: projects/mamba/src/runtime/symbols.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Cranelift needs the fused closure-and-mask gateway's conservative dynamic return ABI.
  - path: projects/mamba/src/runtime/argument_owner.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Dynamic closure calls need inline argument-owner storage without weakening provenance transfer.
  - path: projects/mamba/src/runtime/builtins/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Exact immediate bitwise and modulo operands can bypass the generic operator ladder while preserving zero-divisor and fallback behavior.
  - path: projects/mamba/src/runtime/return_owner.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Inline returns carry no boxed companion and must avoid allocating a return-owner frame on the hot closure path.
  - path: projects/mamba/src/runtime/stdlib/traceback_mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Hot closure calls retain traceback frame identities as runtime values and materialize strings only for observability.
  - path: projects/mamba/src/runtime/stdlib/cprofile_mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Traceback frames can avoid eager text extraction only when no profiler observes call identities.
  - path: projects/mamba/src/codegen/cranelift/jit.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Dynamic in-place dispatch and the fused closure gateway must marshal a raw unannotated accumulator VReg before runtime helpers consume MbValues.
  - path: projects/mamba/src/driver/tests/jit.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-call-context-fastpath
    tracker: "#1478"
    reason: Module-level unannotated augmented assignment needs an end-to-end JIT regression assertion.
  - path: projects/mamba/tests/cpython/_regression/core/language/closures/bench/closure_call_hot.py
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-perf-pin
    tracker: "#1478"
    reason: The benchmark must make its accumulator observable without reducing the closure-call workload.
  - path: projects/mamba/tests/harness/cpython/config/perf/pins/closure_call_hot_1478.toml
    action: create
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-closure-perf-pin
    tracker: "#1478"
    reason: The closure hot path needs an executable external CPU and RSS perf gate while its fixture remains a pure workload.
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
  cached_module:
    id: R1b
    text: "Closure creation stores the active defining module for the cached callable context."
    kind: unit
    risk: high
    verify: runtime::closure::tests::closure_creation_caches_defining_module
  trace_frames:
    id: R1c
    text: "Live traceback frame inspection remains correct when frames retain runtime string values."
    kind: regression
    risk: high
    verify: runtime::stdlib::traceback_mod::tests::test_extract_stack_returns_live_stack_entries
  dynamic_augassign:
    id: R1d
    text: "An unannotated module integer is boxed before dynamic in-place XOR dispatch."
    kind: regression
    risk: high
    verify: driver::pipeline_tests::jit::test_jit_unannotated_module_augassign_boxes_raw_int_operands
  fused_hot_shape:
    id: R1e
    text: "A dynamic one-argument closure masked with an immediate integer, including the pure typed list-modulo argument shape, uses the fused gateway without removing the runtime fallback."
    kind: unit
    risk: high
    verify: lower::hir_to_mir::tests::test_dynamic_closure_bitand_ixor_lowers_to_fused_gateway
  static_leaf_hot_shape:
    id: R1f
    text: "An unrebound, non-escaping literal `x + capture` closure factory lowers the typed list-modulo hot shape to the inline static gateway."
    kind: unit
    risk: high
    verify: lower::hir_to_mir::tests::test_literal_factory_leaf_lowers_to_static_list_gateway
  perf_gate:
    id: R3
    text: "The observable release closure workload has a real externally measured CPU/RSS pin result."
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
    r1b[R1b defining module] --> runtime_closure_tests_closure_creation_caches_defining_module[runtime::closure::tests::closure_creation_caches_defining_module]
    r1c[R1c traceback frames] --> runtime_stdlib_traceback_mod_tests_test_extract_stack_returns_live_stack_entries[runtime::stdlib::traceback_mod::tests::test_extract_stack_returns_live_stack_entries]
    r1d[R1d dynamic augmented assignment] --> driver_pipeline_tests_jit_test_jit_unannotated_module_augassign_boxes_raw_int_operands[driver::pipeline_tests::jit::test_jit_unannotated_module_augassign_boxes_raw_int_operands]
    r1e[R1e fused hot shape] --> lower_hir_to_mir_tests_test_dynamic_closure_bitand_ixor_lowers_to_fused_gateway[lower::hir_to_mir::tests::test_dynamic_closure_bitand_ixor_lowers_to_fused_gateway]
    r1f[R1f static literal leaf] --> lower_hir_to_mir_tests_test_literal_factory_leaf_lowers_to_static_list_gateway[lower::hir_to_mir::tests::test_literal_factory_leaf_lowers_to_static_list_gateway]
    r2[R2 semantics] --> conformance_regression_core_language_closures_behavior_py[conformance::_regression/core/language/closures/behavior.py]
    r3[R3 perf gate] --> perf_pin_closure_call_hot_1478[perf_pin::closure_call_hot_1478]
```

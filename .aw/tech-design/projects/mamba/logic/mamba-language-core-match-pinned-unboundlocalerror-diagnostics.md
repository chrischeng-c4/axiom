---
id: mamba-language-core-match-pinned-unboundlocalerror-diagnostics
summary: Match the pinned CPython 3.12 UnboundLocalError diagnostic for ordinary and generated function-local pre-bind reads.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-unbound-local-diagnostic-applicability
entry: function-local read before assignment
nodes:
  read: { kind: start, label: "function-local read before assignment" }
  route: { kind: decision, label: "ordinary or generated-function path" }
  helper: { kind: process, label: "raise via mb_unbound_local_error_value" }
  generated: { kind: process, label: "emit generated-body UnboundLocalError" }
  message: { kind: process, label: "format pinned CPython 3.12 diagnostic" }
  propagate: { kind: terminal, label: "raise catchable UnboundLocalError" }
  nameerror: { kind: terminal, label: "deferred NameError route unchanged" }
edges:
  - { from: read, to: route }
  - { from: route, to: helper, label: "ordinary function" }
  - { from: route, to: generated, label: "generated function" }
  - { from: helper, to: message }
  - { from: generated, to: message }
  - { from: message, to: propagate }
  - { from: read, to: nameerror, label: "unresolved non-local name" }
---
flowchart TD
    read([function-local read before assignment]) --> route{ordinary or generated-function path?}
    route -- ordinary function --> helper[raise via mb_unbound_local_error_value]
    route -- generated function --> generated[emit generated-body UnboundLocalError]
    helper --> message[format pinned CPython 3.12 diagnostic]
    generated --> message
    message --> propagate([raise catchable UnboundLocalError])
    read -- unresolved non-local name --> nameerror([deferred NameError route unchanged])
```

The existing scope analysis continues to identify local reads before assignment and retains the `UnboundLocalError` subtype. This slice changes only the diagnostic string emitted by the normal runtime helper and the generated-function pre-bind path, making both emit `local variable '<name>' referenced before assignment`, the wording required by the pinned CPython 3.12 oracle. Deferred unresolved global names continue through `mb_deferred_name_read` and preserve their `NameError` behavior.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: mamba-unbound-local-diagnostic-verification
requirements:
  generated_prebind_message:
    id: R2
    text: "Generated function pre-bind checks emit the same pinned diagnostic instead of a divergent message literal."
    kind: regression
    risk: medium
    verify: lower::hir_to_mir::tests::generated_unbound_local_message_matches_runtime_helper
  live_oracle_scope_fixture:
    id: R3
    text: "The scope-resolution regression fixture matches the live pinned oracle with its cache disabled."
    kind: integration
    risk: high
    verify: conformance::_regression/core/scope_resolution/errors.py
  nameerror_boundary:
    id: R4
    text: "The same fixture continues to report a catchable NameError for an unresolved non-local name, proving that this diagnostic-only slice does not change NameError routing."
    kind: regression
    risk: medium
    verify: conformance::_regression/core/scope_resolution/errors.py
  runtime_helper_message:
    id: R1
    text: "The ordinary function helper raises the UnboundLocalError subtype with the pinned CPython 3.12 local-variable-before-assignment message."
    kind: functional
    risk: high
    verify: runtime::exception::tests::unbound_local_error_matches_pinned_cpython_message
---
flowchart TD
    r1[R1 runtime helper message] --> runtime_exception_tests_unbound_local_error_matches_pinned_cpython_message[runtime::exception::tests::unbound_local_error_matches_pinned_cpython_message]
    r2[R2 generated prebind message] --> lower_hir_to_mir_tests_generated_unbound_local_message_matches_runtime_helper[lower::hir_to_mir::tests::generated_unbound_local_message_matches_runtime_helper]
    r3[R3 live oracle scope fixture] --> conformance_regression_core_scope_resolution_errors_py[conformance::_regression/core/scope_resolution/errors.py]
    r4[R4 nameerror boundary] --> conformance_regression_core_scope_resolution_errors_py
```

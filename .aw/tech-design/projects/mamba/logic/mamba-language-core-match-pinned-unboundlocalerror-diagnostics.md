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

---
id: mamba-language-core-bind-instance-method-keywords
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-instance-method-keyword-contract
entry: mb_call_method_kwargs for a user instance method
nodes:
  resolve: { kind: start, label: resolve user method and parameter metadata }
  instance: { kind: decision, label: receiver is a user instance method }
  bindable: { kind: process, label: bind parameters after implicit self }
  fallback: { kind: process, label: retain native variadic and legacy fallback }
  dispatch: { kind: terminal, label: mb_call_method prepends receiver exactly once }
edges:
  - { from: resolve, to: instance }
  - { from: instance, to: bindable, label: yes }
  - { from: instance, to: fallback, label: no }
  - { from: bindable, to: dispatch }
  - { from: fallback, to: dispatch }
---
flowchart TD
    resolve([resolve user method and parameter metadata]) --> instance{receiver is a user instance method}
    instance -- yes --> bindable[bind parameters after implicit self]
    instance -- no --> fallback[retain native variadic and legacy fallback]
    bindable --> dispatch([mb_call_method prepends receiver exactly once])
    fallback --> dispatch
```

For a user method resolved from an `ObjData::Instance`, the keyword binder receives only explicit arguments. It must build its bindable parameter sequence after the leading instance receiver parameter, then pass those explicit bound values to `mb_call_method`, which owns adding the receiver once. Keyword lookup uses the already mangled parameter metadata unchanged, so `_Top__arg` binds successfully while `_O2__arg` is rejected as unexpected. Other receiver kinds retain the existing full-parameter and legacy fallback behavior.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/src/runtime/class/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-instance-method-keyword-binding
    tracker: "#1491"
    reason: The generic keyword binder must exclude implicit self for user instance methods before mb_call_method re-adds the receiver.
  - path: projects/mamba/tests/cpython/_regression/core/scope_resolution/name_mangling.py
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-instance-method-keyword-binding-oracle
    tracker: "#1491"
    reason: The CPython oracle must isolate valid and invalid mangled keyword binding on user instance methods.
```

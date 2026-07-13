---
id: mamba-language-core-resolve-bases-before-inheriting-slots
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-slot-inheritance-after-runtime-bases
entry: define a slotted child whose base expression is deferred
nodes:
  define: { kind: start, label: register child methods and provisional class object }
  bases: { kind: process, label: evaluate and install runtime base list }
  slots: { kind: process, label: merge own slots with the resolved parent MRO }
  init: { kind: process, label: parent initializer writes inherited slot on child instance }
  result: { kind: terminal, label: child exposes inherited and own slot fields }
  static: { kind: terminal, label: pre-resolved bases retain their existing class path }
edges:
  - { from: define, to: bases }
  - { from: bases, to: slots }
  - { from: slots, to: init }
  - { from: init, to: result }
  - { from: define, to: static, label: base list already resolved }
---
flowchart TD
    define([register child methods and provisional class object]) --> bases[evaluate and install runtime base list]
    bases --> slots[merge own slots with the resolved parent MRO]
    slots --> init[parent initializer writes inherited slot on child instance]
    init --> result([child exposes inherited and own slot fields])
    define -- base list already resolved --> static([pre-resolved bases retain their existing class path])
```

`mb_register_slots` merges names from the class MRO, so it is correct only after `mb_class_update_bases` has materialized deferred base expressions. The lowering path must queue each declared slot list and emit registration after runtime bases, before class finalization. A child that declares `y` and inherits a parent slot `x` then records both names; the parent initializer can assign `self.x` to the child instance. Classes with statically known bases retain the same effective order and observable behavior.

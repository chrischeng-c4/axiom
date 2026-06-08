---
change_id: mamba-p0-runtime
type: spec_context
created_at: 2026-02-15T17:01:26.391504+00:00
updated_at: 2026-02-15T17:01:26.391504+00:00
iteration: 2
complexity: critical
stage: spec
scanned_groups:
  - cclab-mamba
  - cclab-aurora
  - cclab-cli
  - cclab-core
  - cclab-genesis
  - cclab-grid
  - cclab-grid-db
  - cclab-ion
  - cclab-meteor
  - cclab-nebula
  - cclab-nova
  - cclab-nucleus
  - cclab-orbit
  - cclab-photon
  - cclab-prism
  - cclab-probe
  - cclab-pulsar
  - cclab-pulsar-array-core
  - cclab-quasar
  - cclab-sdd
  - cclab-server
  - cclab-shield
  - cclab-titan
  - cclab-vortex
  - nebula
---

# Spec Context

## Relevant Specs

- **mamba-stdlib-core** (group: cclab-mamba)
  - relevance: high
  - reason: Defines the minimal standard library (sys, os, math, json). This change extends it with core builtins (#378) and file I/O (#379).
  - key sections: R1 - Core sys Module, R2 - Core os Module, R3 - Core math Module, R4 - Core json Module
- **mamba-string-runtime** (group: cclab-mamba)
  - relevance: high
  - reason: Defines string operations and f-string interpolation. This change adds string methods (#375) building on existing string_ops.rs.
  - key sections: R2 - Runtime String Formatting, R3 - String Operations/Methods
- **mamba-oop-model** (group: cclab-mamba)
  - relevance: high
  - reason: Defines OOP model with inheritance, super(), dunder methods. Magic method dispatch (#380) and exception hierarchy (#381) build directly on this.
  - key sections: R1 - C3 Method Resolution Order, R3 - Magic Method Dispatch, R4 - Attribute Access Model
- **mamba-iteration-protocol** (group: cclab-mamba)
  - relevance: high
  - reason: Defines for-loop iteration protocol. Core builtins (#378) like enumerate, zip, reversed return iterators that use this protocol.
  - key sections: R1 - Obtain Iterator via __iter__, R2 - Advance Iterator via __next__, R3 - Built-in Iterators
- **mamba-gc-runtime** (group: cclab-mamba)
  - relevance: medium
  - reason: GC tracks container objects. New list/dict methods create objects that need GC tracking.
  - key sections: R1 - Track Container Objects
- **mamba-jit-backend** (group: cclab-mamba)
  - relevance: medium
  - reason: JIT symbol wiring. New runtime functions for methods/builtins need symbol registration.
  - key sections: Symbol Registration
- **mamba-codegen-logic** (group: cclab-mamba)
  - relevance: medium
  - reason: Comprehension/generator codegen creates lists and dicts. List/dict methods (#376, #377) are used by generated code.
  - key sections: R1 - Comprehension Lowering
- **mamba-async-runtime** (group: cclab-mamba)
  - relevance: low
  - reason: Async runtime uses exception propagation. Exception hierarchy (#381) affects error handling in coroutines.
  - key sections: R4 - Future Interoperability

## Dependencies

- mamba-oop-model → magic method dispatch (#380)
- mamba-oop-model → exception hierarchy (#381, exceptions are classes)
- mamba-string-runtime → string methods (#375)
- mamba-stdlib-core → core builtins (#378) and file I/O (#379)
- mamba-iteration-protocol → iterator-returning builtins (enumerate, zip, reversed)
- mamba-jit-backend → symbol registration for all new runtime functions

## Gaps

- No spec for method dispatch on built-in types (str.split, list.append, dict.get)
- No spec for exception class hierarchy as Python classes
- No spec for file I/O runtime objects
- No spec for core builtins beyond sys/os/math/json
- mamba-oop-model mentions dunder methods but lacks dispatch implementation detail

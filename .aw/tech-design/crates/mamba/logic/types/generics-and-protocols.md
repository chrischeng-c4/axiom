---
id: generics-and-protocols
title: Generics and Structural Protocols (PEP 544 / 695)
crate: mamba
files:
  - crates/mamba/src/types/generic.rs
  - crates/mamba/src/types/protocol.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: a47e76722
---

# Generics and Protocols

`types/generic.rs` (627 LOC) handles type parameters introduced by PEP
695 syntax (`def f[T]`, `class List[T]`) and their runtime
substitution. `types/protocol.rs` (537 LOC) handles structural typing
per PEP 544 — `Protocol` classes whose `isinstance` check reduces to
"does the value have these methods with these signatures?".

Three load-bearing invariants:

1. **PEP 695 type-params parse as `Vec<Name>` on FnDef / ClassDef** —
   no separate `TypeVar` declaration syntax needed; the bracket-list
   binds new TypeVars in the function / class scope. The lowerer
   creates `TypeVarId`s in the surrounding `TypeContext` automatically.
2. **Structural matching uses method-name + arity, NOT the `Protocol`
   class name** — `isinstance(x, Iterable)` succeeds if `x` has
   `__iter__`. Nominal subclass relation is unnecessary — that's the
   PEP 544 contract.
3. **Variance is invariant by default** — `List[Int] <: List[Object]`
   does NOT hold unless the user marks the TypeVar as covariant
   (`T_co`) or contravariant (`T_contra`). Mamba follows mypy's
   `_co` / `_contra` naming convention; future extension may add
   explicit `Covariant=True` kwargs.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: generics-types
types:
  GenericMod:    { kind: struct, label: "types/generic.rs — TypeVar binding + substitution" }
  ProtocolMod:   { kind: struct, label: "types/protocol.rs — structural matching" }
  TypeVarBind:   { kind: struct, label: "name + bound + variance" }
  ProtocolDef:   { kind: struct, label: "method signatures expected from any conforming type" }
  Substitution:  { kind: struct, label: "TypeVarId → TypeId map applied per call site" }
  TypeContext:   { kind: struct, label: "from types/type-representations" }
edges:
  - { from: GenericMod,   to: TypeVarBind,  kind: owns }
  - { from: GenericMod,   to: Substitution, kind: owns }
  - { from: ProtocolMod,  to: ProtocolDef,  kind: owns }
  - { from: GenericMod,   to: TypeContext,  kind: references, label: "TypeVarId allocation" }
  - { from: ProtocolMod,  to: TypeContext,  kind: references }
---
classDiagram
    class GenericMod
    class ProtocolMod
    class TypeVarBind
    class ProtocolDef
    class Substitution
    class TypeContext
    GenericMod --> TypeVarBind : owns
    GenericMod --> Substitution : owns
    ProtocolMod --> ProtocolDef : owns
    GenericMod --> TypeContext : refs
    ProtocolMod --> TypeContext : refs
```

## Generic + protocol shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "generics-types"
$defs:
  TypeVarBind:
    type: object
    properties:
      name:     { type: string }
      bound:
        oneOf:
          - { type: "null" }
          - { x-rust-type: TypeId }
        description: "T <: bound — used in subtype check"
      constraints:
        type: array
        items: { x-rust-type: TypeId }
        description: "T must be one of these (T = int | str pattern)"
      variance:
        type: string
        enum: [invariant, covariant, contravariant]
        default: invariant
    required: [name, bound, constraints, variance]
  ProtocolDef:
    type: object
    properties:
      name: { type: string }
      methods:
        type: array
        items:
          type: object
          properties:
            name: { type: string }
            sig:  { x-rust-type: TypeId, description: "Ty::Fn" }
          required: [name, sig]
    required: [name, methods]
  Substitution:
    type: object
    description: "TypeVarId → TypeId map applied at call site"
    additionalProperties:
      x-rust-type: TypeId
```

## Substitution lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: generic-substitution
initial: Unbound
nodes:
  Unbound:    { kind: initial,  label: "TypeVar declared via PEP 695 brackets; no concrete type" }
  Inferred:   { kind: normal,   label: "call site provides argument; checker infers TypeVar = ConcreteTy" }
  Substituted: { kind: normal,  label: "Substitution map applied; signature instantiated" }
  Erased:     { kind: terminal, label: "post-codegen: runtime sees only concrete types (no generic dispatch)" }
edges:
  - { from: Unbound,   to: Inferred,    event: "checker unifies arg types with TypeVar bounds" }
  - { from: Inferred,  to: Substituted, event: "build Substitution; rewrite Fn signature" }
  - { from: Substituted, to: Erased,    event: "lower to MIR; concrete TypeIds only" }
---
stateDiagram-v2
    [*] --> Unbound
    Unbound --> Inferred: unify args
    Inferred --> Substituted: build subst
    Substituted --> Erased: lower to MIR
    Erased --> [*]
```

## Protocol matching logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: protocol-match
entry: enter
nodes:
  enter:        { kind: start,    label: "isinstance(value, ProtocolClass) | structural unify" }
  is_proto:     { kind: decision, label: "ProtocolClass marked as Protocol?" }
  fall_nominal: { kind: process,  label: "fall back to nominal isinstance" }
  list_methods: { kind: process,  label: "ProtocolDef.methods list" }
  per_method:   { kind: process,  label: "for each: lookup_method on value's class" }
  has_method:   { kind: decision, label: "method exists?" }
  sig_match:    { kind: decision, label: "signature compatible (subtype on params + ret)?" }
  fail:         { kind: terminal, label: "false" }
  done:         { kind: terminal, label: "true" }
edges:
  - { from: enter,         to: is_proto }
  - { from: is_proto,      to: fall_nominal, label: "no" }
  - { from: is_proto,      to: list_methods, label: "yes" }
  - { from: list_methods,  to: per_method }
  - { from: per_method,    to: has_method }
  - { from: has_method,    to: sig_match,    label: "yes" }
  - { from: has_method,    to: fail,         label: "no" }
  - { from: sig_match,     to: per_method,   label: "yes — next method" }
  - { from: sig_match,     to: fail,         label: "no" }
  - { from: per_method,    to: done,         label: "exhausted" }
  - { from: fall_nominal,  to: done }
---
flowchart TD
    enter([protocol match]) --> is_proto{Protocol class?}
    is_proto -->|no| fall_nominal[nominal isinstance]
    is_proto -->|yes| list_methods[methods list]
    list_methods --> per_method[per method]
    per_method --> has_method{method on class?}
    has_method -->|yes| sig_match{sig compatible?}
    has_method -->|no| fail([false])
    sig_match -->|yes| per_method
    sig_match -->|no| fail
    per_method --> done([true])
    fall_nominal --> done
```

## Generic call interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: generic-call-flow
actors:
  - { id: Checker,  kind: system, label: "TypeChecker.check_call" }
  - { id: Generic,  kind: system, label: "types/generic.rs" }
  - { id: TypeCtx,  kind: system, label: "TypeContext" }
messages:
  - { from: Checker, to: Generic, name: "f has TypeVarBinds [T]; called with args [int_arg]" }
  - { from: Generic, to: Generic, name: "unify T with int_arg type" }
  - { from: Generic, to: TypeCtx, name: "build Substitution { T: int }" }
  - { from: Generic, to: Generic, name: "rewrite Fn signature: (int) → int" }
  - { from: Generic, to: Checker, name: "instantiated signature" }
  - { from: Checker, to: Checker, name: "check arg types against instantiated; ret = int" }
---
sequenceDiagram
    participant Checker
    participant Generic
    participant TypeCtx
    Checker->>Generic: call f[T](args)
    Generic->>Generic: unify T
    Generic->>TypeCtx: build subst
    Generic->>Generic: rewrite sig
    Generic-->>Checker: instantiated
    Checker->>Checker: check ret
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: pep695-generic-function
    given: language/pep695_generic_fn.py defines first[T](xs: list[T]) -> T
    when: Mamba type-checks calls with concrete argument types
    then: T is inferred per call site and the return type is concrete
  - id: pep695-generic-class
    given: language/pep695_generic_class.py defines class Box[T]
    when: methods are checked through the class TypeId
    then: type parameters are registered and substituted through method signatures
  - id: protocol-structural-match
    given: language/protocol_iter.py checks isinstance(my_iter, Iterable)
    when: protocol matching examines available methods
    then: __iter__ satisfies the structural protocol and the result is true
  - id: protocol-missing-method
    given: language/protocol_no_method.py checks isinstance(non_iter, Iterable)
    when: protocol matching cannot find __iter__
    then: the structural match fails and the result is false
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: generics-protocols-test-plan
title: Generics and Protocols Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test type_check_tests --release -- {name} --test-threads=1"]
    Runner --> Pep695Fn["test_pep695_generic_function"]
    Runner --> Pep695Class["test_pep695_generic_class"]
    Runner --> ProtocolMatch["test_protocol_structural_match"]
    Runner --> ProtocolNoMethod["test_protocol_missing_method_returns_false"]
    Runner --> Variance["test_typevar_invariant_default"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/types/generic.rs
    action: modify
    impl_mode: hand-written
    description: "TypeVarBind + Substitution + per-call substitution at TypeChecker::check_call. Hand-written; PEP 695 syntax handled in parser, type machinery here."
  - file: crates/mamba/src/types/protocol.rs
    action: modify
    impl_mode: hand-written
    description: "ProtocolDef + structural matching (method name + signature compatibility per PEP 544). Hand-written."
```

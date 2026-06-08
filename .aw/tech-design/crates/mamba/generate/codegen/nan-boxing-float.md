---
id: nan-boxing-float
title: NaN-Boxing Float Path — Codegen-Side Considerations
crate: mamba
files:
  - crates/mamba/src/codegen/cranelift/marshal.rs
  - crates/mamba/src/codegen/cranelift/mod.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 5236629f5
---

# NaN-Boxing Float Path

NaN-boxed `MbValue` (per `value-and-rc.md`) packs floats as plain
IEEE 754 doubles when they are NOT one of our tagged NaN patterns.
This sub-spec records the codegen-side considerations: how the JIT
emits float arithmetic without accidentally producing one of our tag
patterns, and how unpacking distinguishes float from tagged values.

Three load-bearing invariants:

1. **Canonical NaN passes through the float branch** — `f64::NAN` has
   bit pattern `0x7FF8000000000000`. Our tag prefix is
   `0xFFF8000000000000`. The MSB (sign bit) being 0 vs 1 distinguishes
   them. `is_float` checks `(bits & NAN_PREFIX) != NAN_PREFIX OR bits == canonical NaN.bits`.
2. **Float results never collide with tagged values** — IEEE 754
   arithmetic on finite doubles produces finite doubles or NaN; none
   match our tag prefix bit pattern except canonical NaN, which is
   handled. Adding a new tag would require auditing this.
3. **`from_float` canonicalizes any NaN that overlaps the prefix** —
   a stray `f64::from_bits(NAN_PREFIX | data)` could be mistaken for
   a tagged value; `from_float` checks and rewrites to canonical NaN.
   Skipping this lets a maliciously-constructed float spoof a tag.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: nan-float-types
types:
  Marshal:        { kind: struct, label: "from cranelift/marshal.rs" }
  Cranelift:      { kind: struct, label: "from cranelift/mod.rs" }
  ValueModule:    { kind: struct, label: "from runtime/value (NAN_PREFIX, TAG_*, PAYLOAD_MASK)" }
  FloatPath:      { kind: struct, label: "JIT-emitted IEEE 754 arith" }
  TagDispatch:    { kind: struct, label: "JIT-emitted tag-extract code" }
edges:
  - { from: Marshal,    to: ValueModule, kind: references }
  - { from: Cranelift,  to: Marshal,     kind: owns }
  - { from: Cranelift,  to: FloatPath,   kind: owns }
  - { from: Cranelift,  to: TagDispatch, kind: owns }
  - { from: TagDispatch, to: FloatPath,  kind: references, label: "fall through to float branch" }
---
classDiagram
    class Marshal
    class Cranelift
    class ValueModule
    class FloatPath
    class TagDispatch
    Marshal --> ValueModule : refs
    Cranelift --> Marshal : owns
    Cranelift --> FloatPath : owns
    Cranelift --> TagDispatch : owns
    TagDispatch --> FloatPath : fall through
```

## NaN-box layout shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "nan-float-types"
$defs:
  TagSlot:
    description: "Bits 48..50 within the NaN payload — 3-bit tag space"
    type: object
    properties:
      width: { type: integer, const: 3 }
      values:
        type: object
        additionalProperties: { type: string }
        examples:
          - { "0": PTR, "1": INT, "2": BOOL, "3": NONE, "4": FUNC, "5": NOTIMPLEMENTED }
  FloatVsTagged:
    description: "How the JIT distinguishes float from tagged"
    type: array
    items:
      type: object
      properties:
        bits_pattern: { type: string }
        kind:         { type: string, enum: [Float, TaggedValue] }
      required: [bits_pattern, kind]
    examples:
      - - { bits_pattern: "anything not matching NAN_PREFIX",       kind: Float }
        - { bits_pattern: "NAN_PREFIX | tag(48..50) | payload(48b)", kind: TaggedValue }
        - { bits_pattern: "f64::NAN.to_bits() (0x7FF800...)",       kind: Float, description: "canonical NaN — sign bit 0" }
```

## Float-path emit logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: float-emit
entry: enter
nodes:
  enter:        { kind: start,    label: "JIT emit: arith on potentially-float operand" }
  unbox_check:  { kind: process,  label: "marshal::is_float emits: (bits & NAN_PREFIX) != NAN_PREFIX OR bits == canonical NaN.bits" }
  is_float:     { kind: decision, label: "branch on the unbox check?" }
  emit_fadd:    { kind: process,  label: "Cranelift fadd / fsub / fmul / fdiv on raw f64 bits" }
  emit_int:     { kind: process,  label: "fall through to int / heap path" }
  pack_float:   { kind: process,  label: "marshal::from_float — canonicalize if NaN matches our prefix; emit f64::to_bits → MbValue" }
  done:         { kind: terminal, label: "result MbValue" }
edges:
  - { from: enter,       to: unbox_check }
  - { from: unbox_check, to: is_float }
  - { from: is_float,    to: emit_fadd, label: "float branch" }
  - { from: is_float,    to: emit_int,  label: "tagged branch" }
  - { from: emit_fadd,   to: pack_float }
  - { from: pack_float,  to: done }
  - { from: emit_int,    to: done }
---
flowchart TD
    enter([emit float-or-tagged arith]) --> unbox_check[is_float check]
    unbox_check --> is_float{float?}
    is_float -->|yes| emit_fadd[Cranelift fadd]
    is_float -->|no| emit_int[int / heap path]
    emit_fadd --> pack_float[from_float canonicalize]
    pack_float --> done([result MbValue])
    emit_int --> done
```

## Float arith interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: float-arith-flow
actors:
  - { id: JIT,      kind: system, label: "Cranelift backend" }
  - { id: Marshal,  kind: system, label: "marshal.rs helpers" }
  - { id: Cranelift, kind: system, label: "cranelift_codegen" }
messages:
  - { from: JIT,      to: Marshal,   name: "is_float(operand_value)" }
  - { from: Marshal,  to: Cranelift, name: "iconst NAN_PREFIX; band; icmp_eq" }
  - { from: Marshal,  to: JIT,       name: "is_float predicate value" }
  - { from: JIT,      to: Cranelift, name: "brif: float-block / int-block" }
  - { from: JIT,      to: Cranelift, name: "in float-block: bitcast i64→f64; fadd; bitcast back; canonicalize" }
  - { from: JIT,      to: Cranelift, name: "in int-block: int-fast-path or BigInt promote" }
  - { from: Cranelift, to: JIT,      name: "merged result MbValue" }
---
sequenceDiagram
    participant JIT
    participant Marshal
    participant Cranelift
    JIT->>Marshal: is_float check
    Marshal->>Cranelift: bit ops
    Marshal-->>JIT: predicate
    JIT->>Cranelift: brif float / int
    JIT->>Cranelift: float-block fadd + canonicalize
    JIT->>Cranelift: int-block path
    Cranelift-->>JIT: merged
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: nan-not-equal-self
    given: float_methods/special_values.py evaluates `x = float('nan'); x == x`
    when: codegen routes canonical NaN through the float branch
    then: IEEE 754 comparison returns false
  - id: mixed-int-float
    given: arithmetic/mixed_int_float.py evaluates `1 + 2.0`
    when: tag dispatch selects the float block
    then: the int operand promotes to f64 and the result is 3.0
  - id: infinity-pass-through
    given: language/nan_inf.py evaluates positive and negative infinity values
    when: from_float packs the results
    then: infinity bit patterns pass through without NaN-prefix canonicalization
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
runner: "cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"
fixtures:
  - id: nan_neq_self
    name: "float_methods/special_values.py"
    paired: "float_methods/special_values.expected"
    verifies: ["NaN != NaN; canonical NaN through float branch"]
  - id: int_plus_float
    name: "arithmetic/mixed_int_float.py"
    paired: "arithmetic/mixed_int_float.expected"
    verifies: ["int + float promote to f64; tag dispatch correct"]
  - id: inf_arith
    name: "language/nan_inf.py"
    paired: "language/nan_inf.expected"
    verifies: ["Inf passes; arithmetic with Inf yields IEEE results"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/codegen/cranelift/marshal.rs
    action: modify
    impl_mode: hand-written
    description: "is_float / from_float / NaN-canonicalization helpers; emit Cranelift bit ops for tag dispatch. Hand-written; bit-pattern invariants are platform-independent ABI."
```

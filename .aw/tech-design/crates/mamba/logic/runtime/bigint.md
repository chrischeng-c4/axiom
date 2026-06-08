---
id: bigint
title: BigInt Fallback for 48-bit Integer Overflow
crate: mamba
files:
  - crates/mamba/src/runtime/bigint_ops.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: fc12e3434
---

# BigInt Fallback for 48-bit Integer Overflow

`MbValue` integer fast path uses 48-bit signed inline encoding (see
`value-and-rc.md`); anything outside `[-(2^47), 2^47-1]` promotes to a
heap-allocated `num_bigint::BigInt` wrapped in `ObjData::BigInt`.
`bigint_ops.rs` is the bridge: per-op overflow detection (`checked_add`
/ `checked_mul` etc.), i128-widened intermediate results when both
operands fit i64, and full `BigInt` arithmetic when either side is
already heap-promoted.

Three load-bearing invariants:

1. **Promotion is strictly one-way per operation** — `mb_int_add(a, b)`
   never demotes a heap BigInt back to inline even if the result fits;
   demotion would change the `ObjKind` mid-flight and break any caller
   that already type-checked the result. Demotion happens only on
   construction-time fast-path checks (e.g., `int(string)` reading a
   small value).
2. **Mixed inline + heap operands always go through `to_bigint`** —
   the inline operand is widened to a fresh `BigInt` clone; both
   sides converted, then full BigInt arith. Skipping the conversion
   would route through inline-only paths that overflow undetected.
3. **`mb_int_eq` / `mb_int_cmp` convert both sides** — equality and
   comparison ARE allowed across inline/heap because the answer is a
   bool, not an MbValue. CPython makes `1 == 10**100` False; Mamba
   does too via `to_bigint` on both sides.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: bigint-types
types:
  ObjDataBigInt:  { kind: struct, label: "ObjData::BigInt(num_bigint::BigInt)" }
  MbValue:        { kind: struct, label: "from runtime::value (INT or PTR tag)" }
  ObjKindBigInt:  { kind: struct, label: "ObjKind::BigInt = 11" }
  CheckedArith:   { kind: struct, label: "Rust core: checked_add / checked_sub / checked_mul" }
  BigIntCrate:    { kind: struct, label: "num_bigint::BigInt + num_traits::ToPrimitive" }
edges:
  - { from: ObjDataBigInt, to: BigIntCrate,   kind: owns }
  - { from: ObjDataBigInt, to: ObjKindBigInt, kind: references, label: "header.kind discriminator" }
  - { from: MbValue,       to: ObjDataBigInt, kind: references, label: "PTR tag → BigInt" }
  - { from: MbValue,       to: CheckedArith,  kind: references, label: "INT tag fast path" }
---
classDiagram
    class ObjDataBigInt
    class MbValue
    class ObjKindBigInt
    class CheckedArith
    class BigIntCrate
    ObjDataBigInt --> BigIntCrate : owns
    ObjDataBigInt --> ObjKindBigInt : kind tag
    MbValue --> ObjDataBigInt : PTR
    MbValue --> CheckedArith : INT
```

## Overflow / promotion shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "bigint-types"
$defs:
  Int48Bounds:
    type: object
    description: "Inline NaN-box integer range"
    properties:
      INT48_MAX: { type: integer, const: 140737488355327, description: "(1 << 47) - 1" }
      INT48_MIN: { type: integer, const: -140737488355328, description: "-(1 << 47)" }
    required: [INT48_MAX, INT48_MIN]
  IntegerMbValue:
    description: "Either inline INT or heap PTR with ObjKind::BigInt"
    oneOf:
      - { title: Inline,  description: "tag=INT, payload sign-extends to i64", x-rust-type: "MbValue (INT)" }
      - { title: Heap,    description: "tag=PTR, ObjData::BigInt(BigInt)",     x-rust-type: "MbValue (PTR-BigInt)" }
  BinopFastPath:
    description: "Decision tree per op (add/sub/mul)"
    type: object
    properties:
      both_inline:           { type: boolean, description: "as_int both operands" }
      checked_op_succeeds:   { type: boolean, description: "i64 checked_*; overflow?" }
      result_fits_inline:    { type: boolean, description: "fits_inline(result)?" }
      action:
        type: string
        enum:
          [from_int_inline, from_i128_promote, to_bigint_both_full_arith]
```

## Promotion / arithmetic logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: bigint-arith-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "mb_int_add | sub | mul (a, b)" }
  both_inline:  { kind: decision, label: "both as_int Some?" }
  checked_op:   { kind: decision, label: "checked_op succeeds?" }
  fits_back:    { kind: decision, label: "fits_inline(result)?" }
  inline_ret:   { kind: process,  label: "MbValue::from_int(result)" }
  i128_promote: { kind: process,  label: "bigint_from_i128(ia as i128 op ib as i128)" }
  to_big_both:  { kind: process,  label: "to_bigint(a) op to_bigint(b)" }
  big_arith:    { kind: process,  label: "BigInt op (full precision)" }
  alloc_big:    { kind: process,  label: "bigint_from_big(result)" }
  done:         { kind: terminal, label: "return MbValue (inline or BigInt)" }
edges:
  - { from: enter,        to: both_inline }
  - { from: both_inline,  to: checked_op,    label: "yes" }
  - { from: both_inline,  to: to_big_both,   label: "no (mixed or both heap)" }
  - { from: checked_op,   to: fits_back,     label: "ok" }
  - { from: checked_op,   to: i128_promote,  label: "overflow" }
  - { from: fits_back,    to: inline_ret,    label: "yes" }
  - { from: fits_back,    to: i128_promote,  label: "no (post-checked)" }
  - { from: to_big_both,  to: big_arith }
  - { from: big_arith,    to: alloc_big }
  - { from: i128_promote, to: done }
  - { from: alloc_big,    to: done }
  - { from: inline_ret,   to: done }
---
flowchart TD
    enter([mb_int_add / sub / mul]) --> both_inline{both as_int?}
    both_inline -->|yes| checked_op{checked op?}
    both_inline -->|no| to_big_both[to_bigint both]
    checked_op -->|ok| fits_back{fits 48-bit?}
    checked_op -->|overflow| i128_promote[bigint_from_i128]
    fits_back -->|yes| inline_ret[from_int inline]
    fits_back -->|no| i128_promote
    to_big_both --> big_arith[BigInt full]
    big_arith --> alloc_big[bigint_from_big]
    inline_ret --> done([MbValue])
    i128_promote --> done
    alloc_big --> done
```

## Equality / hash interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: bigint-eq-hash
actors:
  - { id: Caller,    kind: system, label: "mb_eq / mb_hash entry" }
  - { id: BigInt,    kind: system, label: "bigint_ops.rs" }
  - { id: NumBigInt, kind: system, label: "num_bigint::BigInt" }
messages:
  - { from: Caller,    to: BigInt,    name: mb_int_eq(a, b) }
  - { from: BigInt,    to: BigInt,    name: "to_bigint(a); to_bigint(b)" }
  - { from: BigInt,    to: NumBigInt, name: "ba == bb" }
  - { from: NumBigInt, to: BigInt,    name: bool, returns: bool }
  - { from: BigInt,    to: Caller,    name: bool }
  - { from: Caller,    to: BigInt,    name: mb_int_hash(val) }
  - { from: BigInt,    to: BigInt,    name: "to_bigint(val)" }
  - { from: BigInt,    to: NumBigInt, name: "rem-by-prime hash" }
  - { from: NumBigInt, to: BigInt,    name: i64, returns: i64 }
  - { from: BigInt,    to: Caller,    name: hash_value }
---
sequenceDiagram
    participant Caller
    participant BigInt
    participant NumBigInt
    Caller->>BigInt: mb_int_eq
    BigInt->>BigInt: to_bigint both
    BigInt->>NumBigInt: ==
    NumBigInt-->>BigInt: bool
    BigInt-->>Caller: bool
    Caller->>BigInt: mb_int_hash
    BigInt->>BigInt: to_bigint
    BigInt->>NumBigInt: rem hash
    NumBigInt-->>BigInt: i64
    BigInt-->>Caller: hash
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: int-overflow-promotes
    given: arithmetic/int_overflow.py computes a value outside the 48-bit inline range
    when: checked inline arithmetic overflows or no longer fits
    then: bigint_from_i128 promotes the result without precision loss
  - id: factorial-big
    given: arithmetic/factorial_big.py computes factorial values beyond inline range
    when: operands are heap BigInts or mixed inline/heap
    then: to_bigint converts both sides and full BigInt arithmetic stays exact
  - id: mixed-int-equality
    given: arithmetic/mixed_int_eq.py compares inline and heap integers
    when: mb_int_eq or mb_int_cmp runs
    then: both sides convert to BigInt and produce CPython-compatible booleans
  - id: hash-consistency
    given: arithmetic/hash_consistency.py hashes inline and heap integer representations
    when: mb_int_hash runs
    then: to_bigint normalizes the value and hash remains stable across representations
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: runtime-bigint-test-plan
title: BigInt Fallback Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> Overflow["arithmetic/int_overflow.py"]
    Runner --> Factorial["arithmetic/factorial_big.py"]
    Runner --> MixedEq["arithmetic/mixed_int_eq.py"]
    Runner --> Hash["arithmetic/hash_consistency.py"]
    Runner --> FromBytes["int_methods/from_bytes_signed.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/bigint_ops.rs
    action: modify
    impl_mode: hand-written
    description: "BigInt promotion bridge: fits_inline / bigint_from_i128 / bigint_from_big / extract_bigint / to_bigint, mb_int_add / sub / mul / cmp / eq / hash with checked-then-promote fast path. Hand-written; depends on num_bigint."
```

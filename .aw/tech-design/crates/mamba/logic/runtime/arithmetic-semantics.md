---
id: arithmetic-semantics
title: Floor-Division Modulo and divmod Semantics
crate: mamba
files:
  - crates/mamba/src/runtime/builtins.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: fc12e3434
---

# Floor-Division Modulo and divmod Semantics

Python 3.12's modulo (`%`), floor-division (`//`), and `divmod`
follow **floor-division semantics**: the result of `a // b` is the
largest integer not greater than the true quotient, and `a % b` has
the same sign as `b`. This differs from C-style truncation (Rust's
default `%`) and from Rust's `rem_euclid` (always-positive Euclidean).
This spec records the exact arithmetic so future hand-edits or codegen
do not drift.

This is a focused sub-spec of `runtime/builtins.rs` covering only
`mb_mod`, `mb_floordiv`, `mb_divmod`. General arithmetic
(add/sub/mul/etc.) lives in `builtins.md` (the broader runtime spec)
and `bigint.md` (overflow promotion).

Three load-bearing invariants:

1. **Same-sign-as-divisor for `%`** — `(-7) % 3 == 2` (not `-1`);
   `7 % (-3) == -2` (not `1`). Implemented by adjusting C-style
   remainder: if `r != 0 && sign(r) != sign(b)`, return `r + b`.
2. **Floor for `//`** — `(-7) // 3 == -3` (not `-2`); `7 // (-3) == -3`
   (not `-2`). C-style truncation gives `-2` in both cases; the fix is
   to subtract 1 when `r != 0 && sign(r) != sign(b)`.
3. **`str % X` short-circuits to printf-format** — `mb_mod` checks
   if `a` is a Str ptr first and routes to
   `string_ops::mb_str_percent_format`. This must precede the
   numeric path; otherwise `'%s' % name` would coerce to numeric and
   fail.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: arith-types
types:
  ArithApi:        { kind: struct, label: "mb_mod / mb_floordiv / mb_divmod" }
  StringFormat:    { kind: struct, label: "string_ops::mb_str_percent_format" }
  BigIntPromote:   { kind: struct, label: "bigint_ops (for promoted operands)" }
  ExceptionMod:    { kind: struct, label: "exception.rs (ZeroDivisionError)" }
  Builtins:        { kind: struct, label: "from runtime::builtins" }
edges:
  - { from: ArithApi,     to: Builtins,      kind: owns,       label: "lives in builtins.rs" }
  - { from: ArithApi,     to: StringFormat,  kind: references, label: "str % X path" }
  - { from: ArithApi,     to: BigIntPromote, kind: references, label: "promoted operands" }
  - { from: ArithApi,     to: ExceptionMod,  kind: references, label: "ZeroDivisionError" }
---
classDiagram
    class ArithApi
    class StringFormat
    class BigIntPromote
    class ExceptionMod
    class Builtins
    ArithApi --> Builtins : owns
    ArithApi --> StringFormat : str %
    ArithApi --> BigIntPromote : promote
    ArithApi --> ExceptionMod : ZeroDivision
```

## Operator shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "arith-types"
$defs:
  ModResult:
    description: "a % b — same sign as b"
    type: object
    properties:
      a: { x-rust-type: MbValue }
      b: { x-rust-type: MbValue }
      r: { description: "C-style remainder pre-adjustment" }
      result:
        description: "if r != 0 AND sign(r) != sign(b): r + b; else r"
    required: [a, b, r, result]
  FloorDivResult:
    description: "a // b — floor of true quotient"
    type: object
    properties:
      a: { x-rust-type: MbValue }
      b: { x-rust-type: MbValue }
      q: { description: "C-style truncated quotient" }
      r: { description: "C-style remainder" }
      result:
        description: "if r != 0 AND sign(r) != sign(b): q - 1; else q"
    required: [a, b, q, r, result]
  DivmodResult:
    description: "(q, r) tuple consistent with // and %"
    type: object
    properties:
      a: { x-rust-type: MbValue }
      b: { x-rust-type: MbValue }
      q: { description: "floor-div quotient" }
      r: { description: "matching modulo remainder" }
    required: [a, b, q, r]
```

## Mod / floordiv / divmod logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: floor-mod-flow
entry: enter
nodes:
  enter:        { kind: start,    label: "mb_mod | mb_floordiv | mb_divmod (a, b)" }
  is_str_a:     { kind: decision, label: "(mb_mod only) a is Str ptr?" }
  str_format:   { kind: terminal, label: "mb_str_percent_format(tmpl, b)" }
  both_int:     { kind: decision, label: "both as_int Some?" }
  int_zero:     { kind: decision, label: "b == 0?" }
  int_zerodiv:  { kind: terminal, label: "ZeroDivisionError" }
  int_rem:      { kind: process,  label: "r = a % b (C-style)" }
  int_adj_mod:  { kind: process,  label: "if r!=0 AND (r XOR b) < 0: r += b" }
  int_q:        { kind: process,  label: "q = a / b (C-style trunc)" }
  int_adj_q:    { kind: process,  label: "if r!=0 AND (r XOR b) < 0: q -= 1" }
  int_done_mod: { kind: terminal, label: "MbValue::from_int(r)" }
  int_done_q:   { kind: terminal, label: "MbValue::from_int(q)" }
  int_done_dm:  { kind: terminal, label: "(q, r) tuple" }
  num_path:     { kind: process,  label: "as_float on each side; mixed → f64" }
  flt_zero:     { kind: decision, label: "bf == 0.0?" }
  flt_zerodiv:  { kind: terminal, label: "ZeroDivisionError" }
  flt_rem:      { kind: process,  label: "r = af % bf (IEEE 754)" }
  flt_adj_mod:  { kind: process,  label: "if r!=0 AND signum mismatch: r += bf" }
  flt_done:     { kind: terminal, label: "from_float(result)" }
edges:
  - { from: enter,       to: is_str_a }
  - { from: is_str_a,    to: str_format,   label: "yes (mb_mod)" }
  - { from: is_str_a,    to: both_int,     label: "no" }
  - { from: both_int,    to: int_zero,     label: "yes" }
  - { from: int_zero,    to: int_zerodiv,  label: "yes" }
  - { from: int_zero,    to: int_rem,      label: "no — mb_mod path" }
  - { from: int_rem,     to: int_adj_mod }
  - { from: int_adj_mod, to: int_done_mod }
  - { from: int_zero,    to: int_q,        label: "no — mb_floordiv path" }
  - { from: int_q,       to: int_adj_q }
  - { from: int_adj_q,   to: int_done_q }
  - { from: int_zero,    to: int_done_dm,  label: "no — mb_divmod path: combine adj_mod+adj_q" }
  - { from: both_int,    to: num_path,     label: "no" }
  - { from: num_path,    to: flt_zero }
  - { from: flt_zero,    to: flt_zerodiv,  label: "yes" }
  - { from: flt_zero,    to: flt_rem,      label: "no" }
  - { from: flt_rem,     to: flt_adj_mod }
  - { from: flt_adj_mod, to: flt_done }
---
flowchart TD
    enter([a op b]) --> is_str_a{a is Str ptr?}
    is_str_a -->|yes| str_format([str percent format])
    is_str_a -->|no| both_int{both INT?}
    both_int -->|yes| int_zero{b == 0?}
    int_zero -->|yes| int_zerodiv([ZeroDivisionError])
    int_zero -->|no — mod| int_rem[C-style r]
    int_rem --> int_adj_mod[r XOR b less than 0: r += b]
    int_adj_mod --> int_done_mod([from_int r])
    int_zero -->|no — floordiv| int_q[C-style q]
    int_q --> int_adj_q[r XOR b less than 0: q -= 1]
    int_adj_q --> int_done_q([from_int q])
    int_zero -->|no — divmod| int_done_dm([combined q,r tuple])
    both_int -->|no| num_path[as_float]
    num_path --> flt_zero{bf == 0?}
    flt_zero -->|yes| flt_zerodiv([ZeroDivisionError])
    flt_zero -->|no| flt_rem[IEEE 754 r]
    flt_rem --> flt_adj_mod[signum mismatch: r += bf]
    flt_adj_mod --> flt_done([from_float])
```

## Sign-fixup interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: sign-fixup
actors:
  - { id: User,    kind: actor }
  - { id: Mamba,   kind: system }
  - { id: Mod,     kind: system, label: "mb_mod" }
messages:
  - { from: User,    to: Mamba, name: "(-7) % 3" }
  - { from: Mamba,   to: Mod,   name: "mb_mod(-7, 3)" }
  - { from: Mod,     to: Mod,   name: "C-style: -7 % 3 = -1; sign(-1) != sign(3)" }
  - { from: Mod,     to: Mod,   name: "adjust: -1 + 3 = 2" }
  - { from: Mod,     to: Mamba, name: "from_int(2)", returns: MbValue }
  - { from: Mamba,   to: User,  name: 2 }
  - { from: User,    to: Mamba, name: "7 % (-3)" }
  - { from: Mamba,   to: Mod,   name: "mb_mod(7, -3)" }
  - { from: Mod,     to: Mod,   name: "C-style: 7 % -3 = 1; sign(1) != sign(-3)" }
  - { from: Mod,     to: Mod,   name: "adjust: 1 + (-3) = -2" }
  - { from: Mod,     to: Mamba, name: "from_int(-2)" }
  - { from: Mamba,   to: User,  name: -2 }
---
sequenceDiagram
    actor User
    participant Mamba
    participant Mod
    User->>Mamba: (-7) % 3
    Mamba->>Mod: mb_mod
    Mod->>Mod: C-style -1; mismatch; +b → 2
    Mod-->>Mamba: 2
    Mamba-->>User: 2
    User->>Mamba: 7 % (-3)
    Mamba->>Mod: mb_mod
    Mod->>Mod: C-style 1; mismatch; +b → -2
    Mod-->>Mamba: -2
    Mamba-->>User: -2
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: int-floor-mod-div
    given: arithmetic/divmod_and_pow.py evaluates negative operands
    when: (-7) % 3, (-7) // 3, and divmod(-7, 3) run
    then: results match CPython floor-modulo semantics: 2, -3, and (-3, 2)
  - id: float-floor-mod-div
    given: arithmetic/float_basic.py evaluates floating operands
    when: modulo and floor-division run on f64 values
    then: IEEE remainder is adjusted to Python sign semantics
  - id: zero-division
    given: arithmetic/zero_div.py divides or mods by zero
    when: integer or float zero divisors are passed
    then: ZeroDivisionError is raised with CPython-compatible behavior
  - id: str-percent-format
    given: string_methods/percent_format.py uses string percent formatting
    when: mb_mod receives a Str left operand
    then: it routes to mb_str_percent_format before numeric arithmetic dispatch
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: runtime-arithmetic-semantics-test-plan
title: Floor-Division Modulo and Divmod Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> Divmod["arithmetic/divmod_and_pow.py"]
    Runner --> Float["arithmetic/float_basic.py"]
    Runner --> Zero["arithmetic/zero_div.py"]
    Runner --> Percent["string_methods/percent_format.py"]
    Runner --> Overflow["arithmetic/floor_div_big.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/builtins.rs
    action: modify
    impl_mode: hand-written
    description: "mb_mod / mb_floordiv / mb_divmod with floor-division sign fixup; ZeroDivisionError on zero divisor; mb_mod short-circuits to mb_str_percent_format on Str ptr. Hand-written; the sign-fixup invariant is the contract."
```

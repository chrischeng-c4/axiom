---
id: builtins
title: Built-in Functions and Operator Intrinsics
crate: mamba
files:
  - crates/mamba/src/runtime/builtins.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: fc12e3434
---

# Built-in Functions and Operator Intrinsics

Mamba's `runtime/builtins.rs` (~4900 LOC) is the runtime side of every
JIT-emitted operator and Python built-in callable. Three groups live
here:

1. **Arithmetic + bitwise operators** — `mb_add` / `mb_sub` / `mb_mul`
   / `mb_truediv` / `mb_floordiv` / `mb_mod` / `mb_pow` / `mb_neg` /
   `mb_pos` / `mb_invert` / `mb_lshift` / `mb_rshift` / `mb_bitand` /
   `mb_bitor` / `mb_bitxor` / `mb_matmul`. These dispatch by tag; INT
   fast path → BigInt fallback (see `bigint.md`); float path; Instance
   path for `__add__` / `__radd__` etc. Modulo/floordiv/divmod
   semantics live in `arithmetic-semantics.md`.
2. **Comparison + identity** — `mb_eq` / `mb_ne` / `mb_lt` / `mb_le` /
   `mb_gt` / `mb_ge` / `mb_not` / `mb_is_truthy`. CPython value-equality
   (numeric coercion across int/float/bool/BigInt; container element-
   wise; Instance dunder dispatch with `NotImplemented` fallback to
   reflected operator).
3. **Built-in callables** — `mb_print` / `mb_len` / `mb_str` /
   `mb_repr` / `mb_int` / `mb_float` / `mb_bool` / `mb_abs` /
   `mb_min` / `mb_max` / `mb_sum` / `mb_sorted` / `mb_reversed` /
   `mb_range` / `mb_enumerate` / `mb_zip` / `mb_map` / `mb_filter` /
   `mb_all` / `mb_any` / `mb_round` / `mb_format` / `mb_input` /
   `mb_chr` / `mb_ord` / `mb_hex` / `mb_oct` / `mb_bin` / `mb_pow` /
   `mb_callable` / `mb_isinstance` / `mb_issubclass` / `mb_hash` /
   `mb_id` / `mb_type`. These are exposed under their bare Python name
   in the global namespace via `runtime::symbols`.

Three load-bearing invariants:

1. **`mb_eq` walks NotImplemented → reflected → identity** —
   Instance `__eq__` returning `NotImplemented` triggers a `__eq__`
   lookup on the right operand; if that also returns `NotImplemented`,
   the answer falls through to `is` identity. Skipping the reflected
   step makes `1 == MyInt(1)` False even when `MyInt.__eq__` would
   have agreed.
2. **`mb_print` honors `output::is_capturing`** — when capture is
   active (test harness, `redirect_stdout`), `mb_print` writes to the
   capture buffer instead of stdout. The check happens once per call;
   doing it at module init only would miss test reconfiguration.
3. **`mb_value_cmp_pub` is the global comparator** — used by
   `list.sort`, `min`, `max`, `sorted`. Does NOT dispatch
   user-defined `__lt__` for primitives (fast path); only switches to
   dunder dispatch on Instance values. Inverting this would slow
   primitive sort by 10x.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: builtins-types
types:
  Builtins:        { kind: struct, label: "runtime::builtins module — ~80 pub fn" }
  ArithGroup:      { kind: struct, label: "mb_add / sub / mul / truediv / floordiv / mod / pow / bit*" }
  CompareGroup:    { kind: struct, label: "mb_eq / ne / lt / le / gt / ge / not / is_truthy" }
  CallableGroup:   { kind: struct, label: "mb_print / len / str / repr / abs / sum / min / max / sorted / range / etc." }
  ValueModule:     { kind: struct, label: "from runtime::value (NaN-box dispatch)" }
  BigIntOps:       { kind: struct, label: "from runtime::bigint_ops" }
  ClassModule:     { kind: struct, label: "runtime::class (dunder dispatch)" }
  StringOps:       { kind: struct, label: "runtime::string_ops (mb_str → value_to_string)" }
  IterModule:      { kind: struct, label: "runtime::iter (sorted / reversed / range)" }
  ExceptionMod:    { kind: struct, label: "runtime::exception (TypeError / ZeroDivisionError)" }
  Output:          { kind: struct, label: "runtime::output (capture buffer)" }
edges:
  - { from: Builtins,      to: ArithGroup,    kind: owns }
  - { from: Builtins,      to: CompareGroup,  kind: owns }
  - { from: Builtins,      to: CallableGroup, kind: owns }
  - { from: ArithGroup,    to: ValueModule,   kind: references }
  - { from: ArithGroup,    to: BigIntOps,     kind: references }
  - { from: ArithGroup,    to: ClassModule,   kind: references, label: "Instance __op__" }
  - { from: CompareGroup,  to: ClassModule,   kind: references, label: "__eq__ / __lt__" }
  - { from: CompareGroup,  to: ValueModule,   kind: references }
  - { from: CallableGroup, to: StringOps,     kind: references, label: "mb_str / mb_repr" }
  - { from: CallableGroup, to: IterModule,    kind: references, label: "iter combinators" }
  - { from: CallableGroup, to: Output,        kind: references, label: "mb_print capture" }
  - { from: ArithGroup,    to: ExceptionMod,  kind: references, label: "ZeroDivisionError" }
---
classDiagram
    class Builtins
    class ArithGroup
    class CompareGroup
    class CallableGroup
    class ValueModule
    class BigIntOps
    class ClassModule
    class StringOps
    class IterModule
    class ExceptionMod
    class Output
    Builtins --> ArithGroup : owns
    Builtins --> CompareGroup : owns
    Builtins --> CallableGroup : owns
    ArithGroup --> ValueModule : tag dispatch
    ArithGroup --> BigIntOps : promote
    ArithGroup --> ClassModule : Instance __op__
    CompareGroup --> ClassModule : __eq__/__lt__
    CompareGroup --> ValueModule : tag
    CallableGroup --> StringOps : str/repr
    CallableGroup --> IterModule : iter wrap
    CallableGroup --> Output : capture
    ArithGroup --> ExceptionMod : ZeroDivision
```

## Operator dispatch shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "builtins-types"
$defs:
  BinopDispatch:
    description: "Two-operand operator dispatch precedence"
    type: object
    properties:
      step1: { description: "both INT → fast path (with BigInt promotion if overflow)" }
      step2: { description: "either is float (or int+float) → f64 path" }
      step3: { description: "either is BigInt heap → to_bigint both; full BigInt op" }
      step4: { description: "either is Instance → dunder dispatch (__add__ / __radd__)" }
      step5: { description: "left is container (List/Tuple/Str/Bytes) and op is + or * → concat/repeat" }
      fallback: { description: "TypeError unsupported operand types" }
    required: [step1, step2, step3, step4, step5, fallback]
  ComparisonDispatch:
    description: "mb_eq / mb_lt walk"
    type: object
    properties:
      step1: { description: "both numeric → coerce + compare" }
      step2: { description: "both same container type → element-wise" }
      step3: { description: "Instance with __eq__/__lt__ → dispatch" }
      step4: { description: "Instance returns NotImplemented → try reflected on RHS" }
      step5: { description: "reflected also NotImplemented → identity (mb_eq) or TypeError (mb_lt)" }
    required: [step1, step2, step3, step4, step5]
  BuiltinCallableSurface:
    description: "Roster of public callables exposed under bare Python names"
    type: array
    items: { type: string }
    examples:
      - [print, len, str, repr, int, float, bool, abs, min, max, sum,
         sorted, reversed, range, enumerate, zip, map, filter, all, any,
         round, format, input, chr, ord, hex, oct, bin, pow,
         callable, isinstance, issubclass, hasattr, getattr, setattr,
         delattr, vars, dir, id, type, hash]
```

## Binary operator dispatch logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: binop-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "mb_add | sub | mul | truediv | floordiv | mod | pow | bit* (a, b)" }
  str_pct:      { kind: decision, label: "(mb_mod) a is Str ptr?" }
  str_format:   { kind: terminal, label: "mb_str_percent_format" }
  both_int:     { kind: decision, label: "both as_int Some?" }
  int_fast:     { kind: process,  label: "checked op; fits → from_int; else BigInt promote" }
  is_numeric:   { kind: decision, label: "either is_int OR is_float?" }
  float_path:   { kind: process,  label: "as_float each side; f64 op" }
  is_big:       { kind: decision, label: "either is BigInt heap?" }
  big_path:     { kind: process,  label: "to_bigint both; BigInt op; alloc heap or demote" }
  is_inst:      { kind: decision, label: "either is Instance?" }
  dunder_disp:  { kind: process,  label: "__op__(self, other) → if NotImplemented: __rop__(other, self)" }
  is_container: { kind: decision, label: "(+ or *) and a is List/Tuple/Str/Bytes?" }
  container_op: { kind: process,  label: "concat / repeat per type" }
  type_err:     { kind: terminal, label: "TypeError: unsupported operand types" }
  done:         { kind: terminal, label: "return MbValue" }
edges:
  - { from: enter,        to: str_pct }
  - { from: str_pct,      to: str_format,   label: "yes (mb_mod only)" }
  - { from: str_pct,      to: both_int,     label: "no" }
  - { from: both_int,     to: int_fast,     label: "yes" }
  - { from: both_int,     to: is_numeric,   label: "no" }
  - { from: is_numeric,   to: float_path,   label: "yes" }
  - { from: is_numeric,   to: is_big,       label: "no" }
  - { from: is_big,       to: big_path,     label: "yes" }
  - { from: is_big,       to: is_inst,      label: "no" }
  - { from: is_inst,      to: dunder_disp,  label: "yes" }
  - { from: is_inst,      to: is_container, label: "no" }
  - { from: is_container, to: container_op, label: "yes" }
  - { from: is_container, to: type_err,     label: "no" }
  - { from: int_fast,     to: done }
  - { from: float_path,   to: done }
  - { from: big_path,     to: done }
  - { from: dunder_disp,  to: done }
  - { from: container_op, to: done }
---
flowchart TD
    enter([mb_op a,b]) --> str_pct{a is Str ptr? mb_mod}
    str_pct -->|yes| str_format([percent format])
    str_pct -->|no| both_int{both INT?}
    both_int -->|yes| int_fast[checked + BigInt promote]
    both_int -->|no| is_numeric{numeric?}
    is_numeric -->|yes| float_path[f64]
    is_numeric -->|no| is_big{BigInt heap?}
    is_big -->|yes| big_path[to_bigint full]
    is_big -->|no| is_inst{Instance?}
    is_inst -->|yes| dunder_disp[__op__ then __rop__]
    is_inst -->|no| is_container{container concat?}
    is_container -->|yes| container_op[+/* per type]
    is_container -->|no| type_err([TypeError])
    int_fast --> done([result])
    float_path --> done
    big_path --> done
    dunder_disp --> done
    container_op --> done
```

## print and capture interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: print-capture
actors:
  - { id: User,    kind: actor }
  - { id: JIT,     kind: system }
  - { id: Print,   kind: system, label: "mb_print" }
  - { id: V2S,     kind: system, label: "value_to_string (string_ops)" }
  - { id: Output,  kind: system, label: "output::is_capturing / write_captured" }
  - { id: Stdout,  kind: system, label: "println! to native stdout" }
messages:
  - { from: User,   to: JIT,     name: "print(x)" }
  - { from: JIT,    to: Print,   name: mb_print(x) }
  - { from: Print,  to: V2S,     name: value_to_string(x) }
  - { from: V2S,    to: Print,   name: rendered_string, returns: String }
  - { from: Print,  to: Output,  name: "is_capturing()?" }
  - { from: Output, to: Print,   name: "true | false", returns: bool }
  - { from: Print,  to: Output,  name: "write_captured (capture branch)" }
  - { from: Print,  to: Stdout,  name: "println! (no-capture branch)" }
  - { from: Print,  to: JIT,     name: "MbValue::none (return value)" }
---
sequenceDiagram
    actor User
    participant JIT
    participant Print
    participant V2S
    participant Output
    participant Stdout
    User->>JIT: print(x)
    JIT->>Print: mb_print(x)
    Print->>V2S: value_to_string
    V2S-->>Print: rendered
    Print->>Output: is_capturing?
    Output-->>Print: bool
    alt capture
      Print->>Output: write_captured
    else no capture
      Print->>Stdout: println
    end
    Print-->>JIT: none
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: mixed-operator-dispatch
    given: arithmetic/mixed_ops_broad.py combines numeric, string, and tuple operands
    when: mb_add, mb_mul, and related binary operators run
    then: dispatch chooses numeric, container concat/repeat, or instance dunder paths in priority order
  - id: reflected-comparison
    given: comparison/mixed_types_broad.py compares primitives and user instances
    when: mb_eq receives NotImplemented from the left operand
    then: reflected comparison is attempted before identity fallback
  - id: builtin-iteration
    given: builtins/iteration.py calls range, list, sum, and sorted
    when: callable builtins consume iterables
    then: iter wrappers, numeric accumulation, and mb_value_cmp_pub produce CPython-compatible results
  - id: repr-str
    given: builtins/repr_str.py renders nested containers
    when: mb_repr or mb_str delegates to value_to_string
    then: container rendering and repr-in-container behavior match the runtime string contract
  - id: print-capture
    given: print_options/basic.py calls print with sep and end kwargs
    when: output capture is active
    then: mb_print writes the rendered text to output::write_captured instead of stdout
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: runtime-builtins-test-plan
title: Built-in Functions and Operator Intrinsics Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> Arith["arithmetic/mixed_ops_broad.py"]
    Runner --> Compare["comparison/mixed_types_broad.py"]
    Runner --> Iteration["builtins/iteration.py"]
    Runner --> ReprStr["builtins/repr_str.py"]
    Runner --> Print["print_options/basic.py"]
    Runner --> HashIdType["builtins/hash_id_type.py"]
    Runner --> Conversions["builtins/conversions.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/builtins.rs
    action: modify
    impl_mode: hand-written
    description: "Arithmetic + comparison + ~80 built-in callables; tag-priority dispatch; output-capture aware print; reflected dunder fallback. Hand-written; binop and comparison ladders are the contract; sub-specs in arithmetic-semantics.md (% / // / divmod) and bigint.md (overflow promotion)."
```

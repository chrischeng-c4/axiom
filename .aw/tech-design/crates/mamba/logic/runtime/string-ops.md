---
id: string-ops
title: String Operations and Stringification
crate: mamba
files:
  - crates/mamba/src/runtime/string_ops.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: dbbaf7396
---

# String Operations and Stringification

Mamba's string operations module. Two concerns live here: the
`dispatch_str_method` table that routes per-method calls (`s.upper()`,
`s.split(...)`, `s.format(...)`) to individual handler functions, and
`value_to_string`, the public stringification that powers `print`,
`str(x)`, f-strings, and exception traceback rendering.

Three load-bearing invariants:

1. **`splitlines` unwraps a trailing kwargs dict** — JIT lowers
   `s.splitlines(keepends=True)` as a positional `Dict{"keepends": True}`;
   the dispatcher peels it off so the underlying `mb_str_splitlines`
   sees the bool (commit `dbbaf7396`).
2. **`value_to_string` for `KeyError` Instance applies repr-quoting**
   — `'x'` not `x` — matching CPython's `KeyError.__str__` override.
   The exception's `message` field is stored raw; the quoting lives in
   the printer (commit `dbbaf7396`).
3. **`__str__` precedes `__repr__` precedes raw `message`** — when a
   value is an Instance, `value_to_string` dispatches the dunder chain
   before falling through to the exception-message fast path.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: string-ops-types
types:
  StringOps:        { kind: struct, label: "string_ops module" }
  DispatchStrMethod: { kind: struct, label: "dispatch_str_method(name, receiver, args)" }
  ValueToString:    { kind: struct, label: "value_to_string(val) -> String" }
  HandlerFns:       { kind: struct, label: "70+ pub fn mb_str_*" }
  MbValue:          { kind: struct }
  ObjDataStr:       { kind: struct, label: "ObjData::Str" }
  ObjDataInstance:  { kind: struct, label: "ObjData::Instance (for __str__/__repr__/KeyError)" }
  DictKey:          { kind: enum,   label: "dict_ops::DictKey (for kwargs unwrap)" }
  ClassLookup:      { kind: struct, label: "class::lookup_method (dunder dispatch)" }
edges:
  - { from: StringOps,         to: DispatchStrMethod, kind: owns }
  - { from: StringOps,         to: ValueToString,     kind: owns }
  - { from: StringOps,         to: HandlerFns,        kind: owns }
  - { from: DispatchStrMethod, to: HandlerFns,        kind: references, label: "match name → handler" }
  - { from: DispatchStrMethod, to: DictKey,           kind: references, label: "splitlines kwargs unwrap" }
  - { from: HandlerFns,        to: ObjDataStr,        kind: references, label: "as_str / new_str" }
  - { from: HandlerFns,        to: MbValue,           kind: references }
  - { from: ValueToString,     to: ObjDataInstance,   kind: references, label: "Instance dispatch" }
  - { from: ValueToString,     to: ClassLookup,       kind: references, label: "__str__ / __repr__" }
---
classDiagram
    class StringOps
    class DispatchStrMethod
    class ValueToString
    class HandlerFns
    class MbValue
    class ObjDataStr
    class ObjDataInstance
    class DictKey
    class ClassLookup
    StringOps --> DispatchStrMethod : owns
    StringOps --> ValueToString : owns
    StringOps --> HandlerFns : owns
    DispatchStrMethod --> HandlerFns : routes
    DispatchStrMethod --> DictKey : kwargs unwrap
    HandlerFns --> ObjDataStr : as_str / new_str
    HandlerFns --> MbValue : MbValue
    ValueToString --> ObjDataInstance : Instance branch
    ValueToString --> ClassLookup : __str__ / __repr__
```

## Method dispatch shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "string-ops-dispatch"
$defs:
  DispatchEntry:
    type: object
    properties:
      name:        { type: string, description: "Python str method name" }
      handler:     { type: string, description: "fn name in string_ops.rs" }
      arity:       { type: integer, minimum: 0, description: "positional arg count" }
      kwargs:
        type: array
        items: { type: string }
        description: "supported kwargs unwrapped from trailing dict (e.g. keepends)"
      cpython_ref:  { type: string, description: "https://docs.python.org section anchor" }
    required: [name, handler, arity]
  StringMethodTable:
    type: object
    properties:
      methods:
        type: array
        items: { $ref: "#/$defs/DispatchEntry" }
    examples:
      - methods:
          - { name: upper,       handler: mb_str_upper,       arity: 0 }
          - { name: lower,       handler: mb_str_lower,       arity: 0 }
          - { name: strip,       handler: mb_str_strip,       arity: 1 }
          - { name: find,        handler: mb_str_find,        arity: 3 }
          - { name: replace,     handler: mb_str_replace,     arity: 3 }
          - { name: split,       handler: mb_str_split,       arity: 2 }
          - { name: splitlines,  handler: mb_str_splitlines,  arity: 1, kwargs: [keepends] }
          - { name: format,      handler: mb_str_format,      arity: -1, description: "varargs" }
          - { name: format_map,  handler: mb_str_format_map,  arity: 1 }
          - { name: maketrans,   handler: mb_str_maketrans,   arity: 3 }
          - { name: translate,   handler: mb_str_translate,   arity: 1 }
```

## Method dispatch logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: dispatch-str-method
entry: enter
nodes:
  enter:        { kind: start,    label: "dispatch_str_method(name, receiver, args)" }
  match_name:   { kind: decision, label: "match name" }
  case_simple:  { kind: process,  label: "0-arg: upper / lower / casefold / strip / isdigit / ..." }
  case_search:  { kind: process,  label: "search/index/count: arg(0), optional start/end" }
  case_modify:  { kind: process,  label: "replace / split / join / partition / ..." }
  case_format:  { kind: process,  label: "format (varargs) / format_map (single dict)" }
  case_trans:   { kind: process,  label: "maketrans / translate" }
  case_split:   { kind: decision, label: "splitlines: arg(0) is Dict?" }
  unwrap_kw:    { kind: process,  label: "extract DictKey::Str(keepends) → bool" }
  positional:   { kind: process,  label: "use raw arg(0)" }
  call_split:   { kind: process,  label: "mb_str_splitlines(receiver, keepends)" }
  unknown:      { kind: terminal, label: "return MbValue::none() — unknown method" }
  done:         { kind: terminal, label: "return handler result" }
edges:
  - { from: enter,        to: match_name }
  - { from: match_name,   to: case_simple, label: "no-arg / strip" }
  - { from: match_name,   to: case_search, label: "find / index / count / startswith / endswith" }
  - { from: match_name,   to: case_modify, label: "replace / split / join / partition / removeprefix / removesuffix" }
  - { from: match_name,   to: case_format, label: "format / format_map" }
  - { from: match_name,   to: case_trans,  label: "maketrans / translate" }
  - { from: match_name,   to: case_split,  label: "splitlines" }
  - { from: match_name,   to: unknown,     label: "default" }
  - { from: case_split,   to: unwrap_kw,   label: "yes (kwargs dict)" }
  - { from: case_split,   to: positional,  label: "no" }
  - { from: unwrap_kw,    to: call_split }
  - { from: positional,   to: call_split }
  - { from: call_split,   to: done }
  - { from: case_simple,  to: done }
  - { from: case_search,  to: done }
  - { from: case_modify,  to: done }
  - { from: case_format,  to: done }
  - { from: case_trans,   to: done }
---
flowchart TD
    enter([dispatch_str_method]) --> match_name{name?}
    match_name -->|0-arg / strip| case_simple[handler]
    match_name -->|search| case_search[handler]
    match_name -->|modify| case_modify[handler]
    match_name -->|format / format_map| case_format[handler]
    match_name -->|maketrans / translate| case_trans[handler]
    match_name -->|splitlines| case_split{arg 0 is Dict?}
    match_name -->|unknown| unknown([none])
    case_split -->|yes| unwrap_kw[unwrap kwargs DictKey::Str keepends]
    case_split -->|no| positional[use raw arg 0]
    unwrap_kw --> call_split[mb_str_splitlines]
    positional --> call_split
    call_split --> done([return result])
    case_simple --> done
    case_search --> done
    case_modify --> done
    case_format --> done
    case_trans --> done
```

## Stringification interaction (value_to_string)
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: value-to-string-flow
actors:
  - { id: Caller,    kind: system, label: "print / str() / f-string emit" }
  - { id: V2S,       kind: system, label: "value_to_string" }
  - { id: ClassLook, kind: system, label: "class::lookup_method" }
  - { id: Instance,  kind: system, label: "Instance fields" }
messages:
  - { from: Caller,    to: V2S,       name: value_to_string(val) }
  - { from: V2S,       to: V2S,       name: "primitive? (int/float/bool/None)" }
  - { from: V2S,       to: Caller,    name: "format directly", returns: String }
  - { from: V2S,       to: V2S,       name: "ptr kind? (Str/List/Dict/Tuple/Set/...)" }
  - { from: V2S,       to: Caller,    name: "container repr_in_container join", returns: String }
  - { from: V2S,       to: ClassLook, name: "Instance: lookup_method __str__" }
  - { from: ClassLook, to: V2S,       name: method_or_none, returns: MbValue }
  - { from: V2S,       to: Instance,  name: "call __str__(self) if found" }
  - { from: Instance,  to: V2S,       name: result, returns: MbValue }
  - { from: V2S,       to: ClassLook, name: "fallback: lookup_method __repr__" }
  - { from: ClassLook, to: V2S,       name: method_or_none, returns: MbValue }
  - { from: V2S,       to: Instance,  name: "call __repr__(self) if found" }
  - { from: V2S,       to: Instance,  name: "no dunder: read 'message' field" }
  - { from: V2S,       to: V2S,       name: "if class_name == KeyError: format!('\\'{}\\''.replace) — repr-quote" }
  - { from: V2S,       to: Caller,    name: "raw or quoted message", returns: String }
---
sequenceDiagram
    participant Caller
    participant V2S
    participant ClassLook
    participant Instance
    Caller->>V2S: value_to_string(val)
    Note over V2S: primitive? (int/float/bool/None) → format directly
    V2S-->>Caller: String
    Note over V2S: container? → repr_in_container join
    V2S-->>Caller: String
    V2S->>ClassLook: Instance? lookup __str__
    ClassLook-->>V2S: method or none
    V2S->>Instance: call __str__ if found
    Instance-->>V2S: result
    V2S->>ClassLook: fallback lookup __repr__
    ClassLook-->>V2S: method or none
    V2S->>Instance: call __repr__ if found
    V2S->>Instance: no dunder — read message field
    Note over V2S: KeyError → repr-quote message
    V2S-->>Caller: String
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->
```yaml
scenarios:
  - id: splitlines-keepends
    given: string_methods/splitlines_expandtabs.py calls splitlines with keepends keyword
    when: dispatch_str_method receives a trailing kwargs dict
    then: it unwraps DictKey::Str keepends and preserves line terminators
  - id: keyerror-repr
    given: exceptions/keyerror_repr.py raises KeyError for a missing key
    when: print and repr render the exception
    then: value_to_string repr-quotes the message and mb_repr preserves KeyError formatting
  - id: translate-format-map
    given: string_methods/translate_and_format_map.py uses maketrans, translate, and format_map
    when: string ops dispatch those methods
    then: dictionary-backed mapping produces CPython-compatible strings
  - id: percent-format
    given: string_methods/percent_format.py uses percent formatting
    when: mb_str_format handles the binary operation
    then: formatted output matches CPython
```

## Tests
<!-- type: test-plan lang: mermaid -->
```mermaid
---
id: runtime-string-ops-test-plan
title: String Operations Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> Splitlines["string_methods/splitlines_expandtabs.py"]
    Runner --> KeyError["exceptions/keyerror_repr.py"]
    Runner --> Translate["string_methods/translate_and_format_map.py"]
    Runner --> Percent["string_methods/percent_format.py"]
    Runner --> Layout["string_methods/layout_search_broad.py"]
    Runner --> SplitJoin["string_methods/split_join_broad.py"]
    Runner --> Predicates["string_methods/predicates_broad.py"]
    Runner --> Padding["string_methods/padding_and_count.py"]
    Runner --> IndexIter["string_methods/index_iter.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/string_ops.rs
    action: modify
    impl_mode: hand-written
    description: "String method dispatch table, ~70 per-method handlers, value_to_string with __str__/__repr__/KeyError dispatch. Hand-written; spec is the design contract."
```

---
id: stdlib-json
title: stdlib json — JSON Encode and Decode
crate: mamba
files:
  - crates/mamba/src/runtime/stdlib/json_mod.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: c964bdaf9
---

# stdlib `json`

Round-trip JSON between Python values and string. Three entry points
exist on the Python side (`json.dumps`, `json.loads`, `json.dumps(obj, indent=N)`);
each maps to one `mb_json_*` function over `serde_json` under the hood.

Three load-bearing invariants:

1. **`mb_json_dumps` walks ObjData; raises TypeError on Instance**
   without `__json__` dunder — set / frozenset are converted to list,
   tuple to list (CPython parity), dict keys must be str/int/bool/None
   (non-hashable raises).
2. **`mb_json_loads` produces nested ObjData** — JSON object → Dict
   with str keys; JSON array → List; numbers are Int when whole, else
   Float (matches CPython int-vs-float promotion rules).
3. **JSONDecodeError inherits ValueError** — matches CPython
   hierarchy. Raised on parse failure with line + column info from
   serde_json's error position.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: json-types
types:
  JsonMod:        { kind: struct, label: "json_mod.rs" }
  SerdeJson:      { kind: struct, label: "serde_json crate" }
  RcModule:       { kind: struct, label: "from runtime::rc (ObjData walk)" }
  ExceptionMod:   { kind: struct, label: "from runtime::exception (TypeError / JSONDecodeError / ValueError)" }
edges:
  - { from: JsonMod, to: SerdeJson,    kind: references, label: "to_string / from_str" }
  - { from: JsonMod, to: RcModule,     kind: references, label: "encode walk; decode build" }
  - { from: JsonMod, to: ExceptionMod, kind: references }
---
classDiagram
    class JsonMod
    class SerdeJson
    class RcModule
    class ExceptionMod
    JsonMod --> SerdeJson : to/from string
    JsonMod --> RcModule : walk
    JsonMod --> ExceptionMod : raise
```

## Function catalog
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "json-catalog"
$defs:
  StdlibFnEntry:
    type: object
    properties:
      python_name:    { type: string }
      mb_fn:          { type: string }
      arity:          { type: integer }
      kwargs:         { type: array, items: { type: string } }
      cpython_parity: { type: string, enum: [full, partial, gap] }
      raises:         { type: array, items: { type: string } }
      notes:          { type: string }
    required: [python_name, mb_fn, arity, cpython_parity]
  JsonCatalog:
    type: array
    items: { $ref: "#/$defs/StdlibFnEntry" }
    examples:
      - - { python_name: "json.dumps",        mb_fn: "mb_json_dumps",         arity: 1, cpython_parity: partial, raises: [TypeError], notes: "no kwargs (default, sort_keys, separators) yet" }
        - { python_name: "json.dumps",        mb_fn: "mb_json_dumps_pretty",  arity: 2, kwargs: [indent], cpython_parity: partial, notes: "(obj, indent=N) form only" }
        - { python_name: "json.loads",        mb_fn: "mb_json_loads",         arity: 1, cpython_parity: partial, raises: [JSONDecodeError], notes: "no kwargs (parse_int, object_hook, etc.)" }
        - { python_name: "json.JSONDecodeError", mb_fn: "(class)",            arity: -1, cpython_parity: full, notes: "subclass of ValueError; carries .msg, .doc, .pos, .lineno, .colno" }
```

## Encode + decode logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: json-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "json.dumps(obj) | json.loads(s)" }
  is_encode:    { kind: decision, label: "encode (dumps) or decode (loads)?" }
  walk_value:   { kind: process,  label: "encode: classify ObjData kind" }
  encode_prim:  { kind: process,  label: "Int / Float / Bool / None / Str → serde_json::Value primitive" }
  encode_list:  { kind: process,  label: "List / Tuple → array; recurse each elem" }
  encode_dict:  { kind: process,  label: "Dict → object; key must be str/int/bool/None" }
  encode_set:   { kind: process,  label: "Set / FrozenSet → array (CPython parity)" }
  encode_inst:  { kind: decision, label: "Instance has __json__ dunder?" }
  call_dunder:  { kind: process,  label: "dispatch __json__(self) → recurse" }
  raise_type:   { kind: terminal, label: "TypeError: not JSON serializable" }
  to_string:    { kind: process,  label: "serde_json::to_string(value); pretty form uses to_string_pretty" }
  parse_string: { kind: process,  label: "serde_json::from_str → Value tree" }
  build_value:  { kind: process,  label: "decode: walk Value tree; alloc Mamba ObjData per node" }
  done:         { kind: terminal, label: "return Str (encode) or MbValue (decode)" }
edges:
  - { from: enter,        to: is_encode }
  - { from: is_encode,    to: walk_value,    label: "encode" }
  - { from: is_encode,    to: parse_string,  label: "decode" }
  - { from: walk_value,   to: encode_prim,   label: "primitive" }
  - { from: walk_value,   to: encode_list,   label: "List/Tuple" }
  - { from: walk_value,   to: encode_dict,   label: "Dict" }
  - { from: walk_value,   to: encode_set,    label: "Set/FrozenSet" }
  - { from: walk_value,   to: encode_inst,   label: "Instance" }
  - { from: encode_inst,  to: call_dunder,   label: "yes" }
  - { from: encode_inst,  to: raise_type,    label: "no" }
  - { from: encode_prim,  to: to_string }
  - { from: encode_list,  to: to_string }
  - { from: encode_dict,  to: to_string }
  - { from: encode_set,   to: to_string }
  - { from: call_dunder,  to: to_string }
  - { from: to_string,    to: done }
  - { from: parse_string, to: build_value }
  - { from: build_value,  to: done }
---
flowchart TD
    enter([json op]) --> is_encode{encode?}
    is_encode -->|encode| walk_value{ObjData kind?}
    is_encode -->|decode| parse_string[serde from_str]
    walk_value -->|primitive| encode_prim[Value]
    walk_value -->|List/Tuple| encode_list[array]
    walk_value -->|Dict| encode_dict[object]
    walk_value -->|Set/FrozenSet| encode_set[array]
    walk_value -->|Instance| encode_inst{has __json__?}
    encode_inst -->|yes| call_dunder[recurse]
    encode_inst -->|no| raise_type([TypeError])
    encode_prim --> to_string[serde to_string]
    encode_list --> to_string
    encode_dict --> to_string
    encode_set --> to_string
    call_dunder --> to_string
    parse_string --> build_value[walk Value tree]
    to_string --> done([result])
    build_value --> done
```

## Acceptance scenarios
<!-- type: overview lang: markdown -->

```mermaid
---
id: json-acceptance
actors:
  - { id: User,    kind: actor }
  - { id: Mamba,   kind: system }
  - { id: Fixture, kind: system }
messages:
  - { from: User,    to: Mamba,   name: "run stdlib/json_round_trip.py" }
  - { from: Mamba,   to: Fixture, name: "json.loads(json.dumps(obj))" }
  - { from: Fixture, to: Mamba,   name: "deep equal to original (str/int/float/bool/None/list/dict)" }
  - { from: User,    to: Mamba,   name: "run stdlib/json_pretty.py" }
  - { from: Mamba,   to: Fixture, name: "json.dumps(d, indent=2)" }
  - { from: Fixture, to: Mamba,   name: "indented + newline-separated output matches CPython" }
  - { from: User,    to: Mamba,   name: "run stdlib/json_decode_error.py" }
  - { from: Mamba,   to: Fixture, name: "json.loads('{bad}')" }
  - { from: Fixture, to: Mamba,   name: "JSONDecodeError with line+col; isinstance(e, ValueError) is True" }
  - { from: User,    to: Mamba,   name: "run stdlib/json_set_to_array.py" }
  - { from: Mamba,   to: Fixture, name: "json.dumps({1, 2, 3})" }
  - { from: Fixture, to: Mamba,   name: "[1, 2, 3] (set encoded as array per CPython parity hack)" }
---
sequenceDiagram
    actor User
    participant Mamba
    participant Fixture
    User->>Mamba: round_trip
    Mamba->>Fixture: dumps then loads
    Fixture-->>Mamba: deep equal
    User->>Mamba: pretty
    Mamba->>Fixture: indent=2
    Fixture-->>Mamba: indented
    User->>Mamba: decode_error
    Mamba->>Fixture: bad json
    Fixture-->>Mamba: JSONDecodeError
    User->>Mamba: set encode
    Mamba->>Fixture: set
    Fixture-->>Mamba: array
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
runner: "cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"
fixtures:
  - id: json_round_trip
    name: "stdlib/json_round_trip.py"
    paired: "stdlib/json_round_trip.expected"
  - id: json_pretty
    name: "stdlib/json_pretty.py"
    paired: "stdlib/json_pretty.expected"
  - id: json_decode_error
    name: "stdlib/json_decode_error.py"
    paired: "stdlib/json_decode_error.expected"
  - id: json_unicode
    name: "stdlib/json_unicode.py"
    paired: "stdlib/json_unicode.expected"
    verifies: ["non-ASCII strings round-trip"]
  - id: json_nested
    name: "stdlib/json_nested.py"
    paired: "stdlib/json_nested.expected"
    verifies: ["deeply nested dicts and lists round-trip"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/stdlib/json_mod.rs
    action: modify
    impl_mode: hand-written
    description: "json.dumps / loads / dumps_pretty over serde_json. Hand-written; encode walk + decode build are mechanical translations between ObjData ↔ serde_json::Value. Phase-1 codegen target with stdlib-fn schema + a recursive walker."
```

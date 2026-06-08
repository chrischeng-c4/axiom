---
id: stdlib-collections
title: stdlib collections — Counter, deque, OrderedDict, defaultdict, namedtuple, ChainMap
crate: mamba
files:
  - crates/mamba/src/runtime/stdlib/collections_mod.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 1e0a3b96d
---

# stdlib `collections`

Container subclasses + factories. Six families:

- `Counter` — dict subclass for counting hashables; `most_common(n)`
  returns sorted (key, count) tuples
- `deque` — double-ended queue with O(1) appendleft / popleft / rotate
- `OrderedDict` — dict that remembers insertion order (vacuous in
  Mamba since base `dict` is already insertion-ordered via IndexMap;
  exists for API parity)
- `defaultdict(factory)` — dict that calls factory on missing key
- `namedtuple(name, fields)` — class factory producing tuple subclass
  with named field access
- `ChainMap` — view over multiple dicts; lookup walks the chain

Three load-bearing invariants:

1. **Counter / deque / OrderedDict / defaultdict / ChainMap are
   Instance-class wrappers, NOT new ObjData variants** — each carries
   `class_name = "collections.<X>"` with a `_data` field holding the
   backing storage (Dict / List / etc.). `mb_iter` (per `iter.md`)
   special-cases these class names to walk `_data`.
2. **`Counter.most_common` keeps DictKey identity** — commit
   `80b05cb81` fixed the regression where `most_common` rebuilt
   entries as `(String, i64)` pairs instead of preserving the
   `DictKey` variant (Int / Str / Bool / Instance / Other). Required
   for user-class instances as Counter keys.
3. **`namedtuple` produces a real class registered in
   `runtime::class::CLASS_REGISTRY`** — instances of the namedtuple
   are normal `ObjData::Instance` with the registered class name; the
   class has methods `_make`, `_replace`, `_asdict`, plus per-field
   accessors generated at registration time.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: collections-types
types:
  CollectionsMod:    { kind: struct, label: "collections_mod.rs" }
  CounterInstance:   { kind: struct, label: "Instance class_name=collections.Counter" }
  DequeInstance:     { kind: struct, label: "Instance class_name=collections.deque" }
  OrderedDictInst:   { kind: struct, label: "Instance class_name=collections.OrderedDict" }
  DefaultdictInst:   { kind: struct, label: "Instance class_name=collections.defaultdict" }
  ChainMapInstance:  { kind: struct, label: "Instance class_name=collections.ChainMap" }
  NamedtupleClass:   { kind: struct, label: "registered class via runtime::class CLASS_REGISTRY" }
  DictOps:           { kind: struct, label: "from runtime::dict_ops (DictKey / IndexMap)" }
  ListOps:           { kind: struct, label: "from runtime::list_ops (deque storage)" }
  IterModule:        { kind: struct, label: "from runtime::iter (special-case class names)" }
edges:
  - { from: CollectionsMod,   to: CounterInstance,  kind: owns }
  - { from: CollectionsMod,   to: DequeInstance,    kind: owns }
  - { from: CollectionsMod,   to: OrderedDictInst,  kind: owns }
  - { from: CollectionsMod,   to: DefaultdictInst,  kind: owns }
  - { from: CollectionsMod,   to: ChainMapInstance, kind: owns }
  - { from: CollectionsMod,   to: NamedtupleClass,  kind: owns }
  - { from: CounterInstance,  to: DictOps,          kind: references, label: "_data is IndexMap<DictKey, MbValue>" }
  - { from: DequeInstance,    to: ListOps,          kind: references, label: "_data is List" }
  - { from: IterModule,       to: CollectionsMod,   kind: references, label: "iter() special-case for class names" }
---
classDiagram
    class CollectionsMod
    class CounterInstance
    class DequeInstance
    class OrderedDictInst
    class DefaultdictInst
    class ChainMapInstance
    class NamedtupleClass
    class DictOps
    class ListOps
    class IterModule
    CollectionsMod --> CounterInstance : owns
    CollectionsMod --> DequeInstance : owns
    CollectionsMod --> OrderedDictInst : owns
    CollectionsMod --> DefaultdictInst : owns
    CollectionsMod --> ChainMapInstance : owns
    CollectionsMod --> NamedtupleClass : owns
    CounterInstance --> DictOps : _data
    DequeInstance --> ListOps : _data
    IterModule --> CollectionsMod : iter special-case
```

## Function catalog
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "collections-catalog"
$defs:
  StdlibFnEntry:
    type: object
    properties:
      python_name:    { type: string }
      mb_fn:          { type: string }
      arity:          { type: integer }
      kwargs:         { type: array, items: { type: string } }
      cpython_parity: { type: string, enum: [full, partial, gap] }
      notes:          { type: string }
    required: [python_name, mb_fn, arity, cpython_parity]
  CollectionsCatalog:
    type: object
    properties:
      counter:
        type: array
        items: { $ref: "#/$defs/StdlibFnEntry" }
        examples:
          - - { python_name: "collections.Counter",            mb_fn: "mb_counter_new",         arity: 1, cpython_parity: full,    notes: "iterable input; tally each element" }
            - { python_name: "collections.Counter.most_common", mb_fn: "mb_counter_most_common", arity: 2, cpython_parity: full,    notes: "preserves DictKey (commit 80b05cb81)" }
      deque:
        type: array
        items: { $ref: "#/$defs/StdlibFnEntry" }
        examples:
          - - { python_name: "collections.deque",          mb_fn: "mb_deque_new",        arity: 0, cpython_parity: partial, notes: "no maxlen kwarg yet" }
            - { python_name: "deque.append",               mb_fn: "mb_deque_append",     arity: 2, cpython_parity: full }
            - { python_name: "deque.appendleft",           mb_fn: "mb_deque_appendleft", arity: 2, cpython_parity: full }
            - { python_name: "deque.pop",                  mb_fn: "mb_deque_pop",        arity: 1, cpython_parity: full }
            - { python_name: "deque.popleft",              mb_fn: "mb_deque_popleft",    arity: 1, cpython_parity: full }
            - { python_name: "deque.rotate",               mb_fn: "mb_deque_rotate",     arity: 2, cpython_parity: full,    notes: "(deque, n); negative n rotates right" }
      ordereddict:
        type: array
        items: { $ref: "#/$defs/StdlibFnEntry" }
        examples:
          - - { python_name: "collections.OrderedDict", mb_fn: "mb_ordereddict_new", arity: 0, cpython_parity: full, notes: "Mamba dict already insertion-ordered; this is API parity" }
      defaultdict:
        type: array
        items: { $ref: "#/$defs/StdlibFnEntry" }
        examples:
          - - { python_name: "collections.defaultdict", mb_fn: "mb_defaultdict_new", arity: 1, cpython_parity: full, notes: "factory called on missing key during __getitem__" }
      namedtuple:
        type: array
        items: { $ref: "#/$defs/StdlibFnEntry" }
        examples:
          - - { python_name: "collections.namedtuple", mb_fn: "mb_namedtuple", arity: 2, cpython_parity: partial, notes: "(name, fields); __match_args__ default; rename / defaults kwargs gap" }
      chainmap:
        type: array
        items: { $ref: "#/$defs/StdlibFnEntry" }
        examples:
          - - { python_name: "collections.ChainMap", mb_fn: "mb_chainmap_new", arity: -1, cpython_parity: partial, notes: "*maps; lookup walks chain in order" }
```

## Class instance lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: collections-instance-lifecycle
initial: Constructed
nodes:
  Constructed: { kind: initial, label: "mb_counter_new / mb_deque_new / etc. — Instance with _data field" }
  Mutated:     { kind: normal,  label: "append / appendleft / setitem / delitem updates _data" }
  Iterated:    { kind: normal,  label: "for x in obj — iter.rs special-case class_name" }
  Released:    { kind: terminal, label: "rc → 0; release_contained_values_pub walks _data" }
edges:
  - { from: Constructed, to: Mutated,  event: "any mutating method" }
  - { from: Constructed, to: Iterated, event: "iter() special-case dispatch" }
  - { from: Mutated,     to: Mutated,  event: "more mutations" }
  - { from: Mutated,     to: Iterated }
  - { from: Iterated,    to: Mutated }
  - { from: Constructed, to: Released, event: "rc drop" }
  - { from: Mutated,     to: Released }
---
stateDiagram-v2
    [*] --> Constructed
    Constructed --> Mutated: mutation
    Constructed --> Iterated: iter()
    Mutated --> Mutated: more
    Mutated --> Iterated: iter()
    Iterated --> Mutated: mutation
    Constructed --> Released: rc drop
    Mutated --> Released: rc drop
    Released --> [*]
```

## Counter / deque / namedtuple dispatch
<!-- type: logic lang: mermaid -->

```mermaid
---
id: collections-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "mb_<X>_new / method call" }
  classify:     { kind: decision, label: "which collections type?" }
  build_counter: { kind: process, label: "Counter: walk iterable; tally into IndexMap<DictKey, MbValue>; wrap as Instance class_name=collections.Counter" }
  build_deque:  { kind: process,  label: "deque: empty Vec; wrap as Instance class_name=collections.deque" }
  build_oddict: { kind: process,  label: "OrderedDict: empty IndexMap; same shape as dict" }
  build_ddict:  { kind: process,  label: "defaultdict: factory + IndexMap; __getitem__ delegates to factory on miss" }
  build_chainm: { kind: process,  label: "ChainMap: store maps Vec; lookup walks chain" }
  build_nt:     { kind: process,  label: "namedtuple: register class via runtime::class with field accessors + _make / _replace / _asdict + __match_args__" }
  call_method:  { kind: process,  label: "dispatch via class_name special-case; mutate _data or read view" }
  done:         { kind: terminal, label: "return MbValue (Instance / class)" }
edges:
  - { from: enter,         to: classify }
  - { from: classify,      to: build_counter, label: "Counter" }
  - { from: classify,      to: build_deque,   label: "deque" }
  - { from: classify,      to: build_oddict,  label: "OrderedDict" }
  - { from: classify,      to: build_ddict,   label: "defaultdict" }
  - { from: classify,      to: build_chainm,  label: "ChainMap" }
  - { from: classify,      to: build_nt,      label: "namedtuple" }
  - { from: classify,      to: call_method,   label: "method on existing instance" }
  - { from: build_counter, to: done }
  - { from: build_deque,   to: done }
  - { from: build_oddict,  to: done }
  - { from: build_ddict,   to: done }
  - { from: build_chainm,  to: done }
  - { from: build_nt,      to: done }
  - { from: call_method,   to: done }
---
flowchart TD
    enter([collections op]) --> classify{which?}
    classify -->|Counter ctor| build_counter[tally + Instance]
    classify -->|deque ctor| build_deque[Vec + Instance]
    classify -->|OrderedDict| build_oddict[IndexMap + Instance]
    classify -->|defaultdict| build_ddict[factory + IndexMap]
    classify -->|ChainMap| build_chainm[Vec maps]
    classify -->|namedtuple factory| build_nt[register class]
    classify -->|method call| call_method[dispatch + mutate]
    build_counter --> done([handle])
    build_deque --> done
    build_oddict --> done
    build_ddict --> done
    build_chainm --> done
    build_nt --> done
    call_method --> done
```

## Counter creation interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: counter-flow
actors:
  - { id: User,    kind: actor }
  - { id: JIT,     kind: system }
  - { id: Cmod,    kind: system, label: "collections_mod" }
  - { id: DictOps, kind: system, label: "runtime::dict_ops (DictKey)" }
messages:
  - { from: User,   to: JIT,    name: "Counter('aabbc')" }
  - { from: JIT,    to: Cmod,   name: "mb_counter_new('aabbc')" }
  - { from: Cmod,   to: Cmod,   name: "iterate input; per char: tally += 1" }
  - { from: Cmod,   to: DictOps, name: "to_dict_key(char)" }
  - { from: DictOps, to: Cmod,  name: "DictKey::Str" }
  - { from: Cmod,   to: Cmod,   name: "_data IndexMap entry" }
  - { from: Cmod,   to: JIT,    name: "Instance class_name=collections.Counter, _data={a:2,b:2,c:1}" }
  - { from: User,   to: JIT,    name: "c.most_common(2)" }
  - { from: JIT,    to: Cmod,   name: "mb_counter_most_common(c, 2)" }
  - { from: Cmod,   to: Cmod,   name: "clone DictKey entries; sort_by count desc; take 2; tuple-ify" }
  - { from: Cmod,   to: JIT,    name: "[(a,2), (b,2)]" }
---
sequenceDiagram
    actor User
    participant JIT
    participant Cmod
    participant DictOps
    User->>JIT: Counter input
    JIT->>Cmod: mb_counter_new
    Cmod->>DictOps: to_dict_key per char
    DictOps-->>Cmod: DictKey
    Cmod-->>JIT: Counter Instance
    User->>JIT: most_common(2)
    JIT->>Cmod: mb_counter_most_common
    Cmod->>Cmod: clone + sort + take
    Cmod-->>JIT: pairs
```

## Acceptance scenarios
<!-- type: overview lang: markdown -->

```mermaid
---
id: collections-acceptance
actors:
  - { id: User,    kind: actor }
  - { id: Mamba,   kind: system }
  - { id: Fixture, kind: system }
messages:
  - { from: User,    to: Mamba,   name: "run stdlib/collections_counter.py" }
  - { from: Mamba,   to: Fixture, name: "Counter('aabbc'); .most_common(2)" }
  - { from: Fixture, to: Mamba,   name: "preserves DictKey identity (commit 80b05cb81)" }
  - { from: User,    to: Mamba,   name: "run stdlib/collections_deque.py" }
  - { from: Mamba,   to: Fixture, name: "deque(); appendleft(1); append(2); rotate(1); list" }
  - { from: Fixture, to: Mamba,   name: "[2, 1] after rotate" }
  - { from: User,    to: Mamba,   name: "run stdlib/collections_defaultdict.py" }
  - { from: Mamba,   to: Fixture, name: "d = defaultdict(list); d['x'].append(1)" }
  - { from: Fixture, to: Mamba,   name: "factory creates list on miss" }
  - { from: User,    to: Mamba,   name: "run stdlib/collections_namedtuple.py" }
  - { from: Mamba,   to: Fixture, name: "Pt = namedtuple('Pt', ['x','y']); p = Pt(1, 2); p.x; match p..." }
  - { from: Fixture, to: Mamba,   name: "field access + match-args" }
---
sequenceDiagram
    actor User
    participant Mamba
    participant Fixture
    User->>Mamba: counter
    Mamba->>Fixture: Counter + most_common
    Fixture-->>Mamba: DictKey preserved
    User->>Mamba: deque
    Mamba->>Fixture: append + rotate
    Fixture-->>Mamba: rotated
    User->>Mamba: defaultdict
    Mamba->>Fixture: factory on miss
    Fixture-->>Mamba: list created
    User->>Mamba: namedtuple
    Mamba->>Fixture: Pt(1,2); .x
    Fixture-->>Mamba: field access
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
runner: "cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"
fixtures:
  - id: counter_basic
    name: "stdlib/collections_counter.py"
    paired: "stdlib/collections_counter.expected"
  - id: deque_basic
    name: "stdlib/collections_deque.py"
    paired: "stdlib/collections_deque.expected"
  - id: defaultdict_basic
    name: "stdlib/collections_defaultdict.py"
    paired: "stdlib/collections_defaultdict.expected"
  - id: namedtuple_basic
    name: "stdlib/collections_namedtuple.py"
    paired: "stdlib/collections_namedtuple.expected"
  - id: ordereddict_parity
    name: "stdlib/collections_ordereddict.py"
    paired: "stdlib/collections_ordereddict.expected"
  - id: chainmap_basic
    name: "stdlib/collections_chainmap.py"
    paired: "stdlib/collections_chainmap.expected"
  - id: counter_dictkey_identity
    name: "class_system/instance_counter_key.py"
    paired: "class_system/instance_counter_key.expected"
    verifies: ["Counter with user-class instance keys preserves DictKey (commit 80b05cb81)"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/stdlib/collections_mod.rs
    action: modify
    impl_mode: hand-written
    description: "Counter / deque / OrderedDict / defaultdict / namedtuple / ChainMap as Instance-class wrappers around runtime primitives. Hand-written; namedtuple registers a real class — the most algorithmic of the family. Counter / deque / OrderedDict are Phase-1 codegen targets; namedtuple is Phase 4 (compiler-compiler-shaped class registration)."
```

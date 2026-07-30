# Class conformance domain topology

Issue: #3026
Parent inventory: #2968
Source revision: `a93f8cb553ebb17b89765e5e424cff70775fb5b8`

This Stage 1 DDD slice classifies three TLS collections that participate in
protocol and abstract-base-class conformance:

- `RUNTIME_CHECKABLE_PROTOCOLS`;
- `ABC_VIRTUAL_SUBCLASSES`;
- `USER_ABC_OWN_ABSTRACT`.

The current collections use textual class names and are read beside
`CLASS_REGISTRY` without sharing its identity, definition version, publication
boundary, operation lease, cleanup, or execution-context ownership.

The target has two authoritative owners inside one context-owned ClassDomain:

- immutable protocol and abstract-class policy is a direct value of
  `ClassDefinition`;
- dynamic `ABC.register()` relations belong to a typed
  `VirtualSubclassRegistry`.

The target does not create another protocol or abstract side table. The
existing `Arc<ClassDefinition>` is the sole lifetime lease for its immutable
conformance value. No `src/**` change occurs in this inventory slice.

## Bounded context

```text
ExecutionContext
└── ClassDomain
    ├── ClassDefinitionRegistry
    │   └── definitions[ClassRuntimeKey] -> Arc<ClassDefinition>
    │       ├── version: ClassDefinitionVersion
    │       ├── mro: StableMro
    │       ├── members: OwnedMemberSurface
    │       └── conformance: ClassConformancePolicy
    │           ├── protocol: ProtocolPolicy
    │           └── abstract_class: AbstractClassPolicy
    ├── VirtualSubclassRegistry
    │   ├── relations[(ClassSubjectKey, ClassSubjectKey)]
    │   └── visibility: ConformanceVisibilityGeneration
    └── ClassPublicationCoordinator
        └── definition visibility generation
```

The definition registry remains the sole class-definition owner. The virtual
registry owns dynamic relations, not class definitions or Python values.

## Scope

This slice covers:

- runtime-checkable protocol marking and checking;
- derivation of protocol required-member surface;
- collections ABC and user ABC virtual registration;
- nominal, virtual, structural, and custom `__subclasshook__` ordering;
- own and effective abstract-method policy;
- abstract instantiation, type-surface, and inspect consumers;
- returned Python aliases from decorators and register calls;
- snapshot, replace, cleanup, TLS exit, context isolation, and retirement;
- the target definition/relation version and lease protocols.

It does not cover:

- the separate legacy `ABSTRACT_METHODS` TLS map;
- general method lookup cache migration;
- all `isinstance` and `issubclass` special cases;
- callable ABI migration;
- class-definition implementation itself;
- every typing or collections ABC behavior;
- source implementation or test execution.

`ABSTRACT_METHODS` is a required sibling inventory. Source implementation must
not leave it as a second abstract-policy owner or delete it without its own
exact-set review.

## Aggregate and domain values

| Value | Role | Lifetime |
|---|---|---|
| `ContextHandle` | operation authority | validated at each public entry |
| `ContextId` | isolation identity | one per execution context |
| `ClassRuntimeKey` | exact user-class identity | unique within one context |
| `NativeTypeKey` | exact native-type identity | stable process description interpreted in context |
| `ClassSubjectKey` | user/native conformance subject | value identity for one context |
| `ClassDefinitionVersion` | immutable definition identity | one per successful definition publication |
| `ClassConformancePolicy` | immutable definition behavior | direct field of one definition |
| `ProtocolPolicy` | protocol kind/checkability/surface | direct immutable definition value |
| `AbstractClassPolicy` | own/effective abstract names | direct immutable definition value |
| `VirtualSubclassRegistry` | dynamic relation aggregate | one per ClassDomain |
| `ConformanceVisibilityGeneration` | virtual-relation read view | advances on new relation commit |
| `Arc<ClassDefinition>` | definition operation lease | keeps detached definition alive |
| `OwnedMemberAlias` | callable/member use authority | explicit Python ownership claim |
| `OwnedClassAlias` | decorator/register return | explicit returned Python claim |

Candidate identities:

```rust
enum ClassSubjectKey {
    User(ClassRuntimeKey),
    Native(NativeTypeKey),
}

struct ClassConformancePolicy {
    protocol: ProtocolPolicy,
    abstract_class: AbstractClassPolicy,
}

struct ProtocolPolicy {
    kind: ProtocolKind,
    runtime_checkable: bool,
    required_members: Box<[MemberName]>,
}

enum ProtocolKind {
    NotProtocol,
    MethodOnly,
    HasDataMembers,
}

struct AbstractClassPolicy {
    own_abstract_methods: Box<[MemberName]>,
    effective_abstract_methods: Box<[MemberName]>,
}
```

The arrays are direct owned values. They are not nested `Arc` leases. The
enclosing `Arc<ClassDefinition>` keeps every field alive.

## Frozen inventory

The exact selector is:

```bash
rg -n \
  'static RUNTIME_CHECKABLE_PROTOCOLS|static ABC_VIRTUAL_SUBCLASSES|static USER_ABC_OWN_ABSTRACT|RUNTIME_CHECKABLE_PROTOCOLS\.with|ABC_VIRTUAL_SUBCLASSES\.with|USER_ABC_OWN_ABSTRACT\.with' \
  projects/mamba/src/runtime/class/mod.rs
```

Its exact stdout has SHA-256:

`762e2cf0b1d4ed178aac555c8ee97f08a306c72ed9e54b8fc3a8d7fbdcde594a`

The selector emits 17 production rows and zero test rows. The frozen test
module begins at line `22077`.

### `RUNTIME_CHECKABLE_PROTOCOLS`

| Line | Operation | Owner |
|---:|---|---|
| `197` | declaration | module `thread_local!` |
| `578` | insert | `mark_runtime_checkable` |
| `586` | membership read | `is_runtime_checkable_protocol` |
| `22009` | snapshot clone | `snapshot_thread_class_state` |
| `22029` | replace | `replace_thread_class_state` |

### `ABC_VIRTUAL_SUBCLASSES`

| Line | Operation | Owner |
|---:|---|---|
| `201` | declaration | module `thread_local!` |
| `1155` | relation read | `collections_abc_virtual_match` |
| `1210` | relation insert | `mb_collections_abc_register` |
| `5708` | relation insert | `mb_user_abc_register` |
| `22010` | snapshot clone | `snapshot_thread_class_state` |
| `22030` | replace | `replace_thread_class_state` |
| `22073` | best-effort cleanup | `cleanup_all_classes` |

### `USER_ABC_OWN_ABSTRACT`

| Line | Operation | Owner |
|---:|---|---|
| `207` | declaration | module `thread_local!` |
| `238` | insert/replace | `mb_class_set_abstractmethods` |
| `458` | read | `compute_user_abstractmethods` |
| `22011` | snapshot clone | `snapshot_thread_class_state` |
| `22031` | replace | `replace_thread_class_state` |

The identity partition is `5 + 7 + 5 = 17`.

### Frozen design inputs

This topology refines:

- `projects/mamba/tech-design/concurrency/execution-context.md`;
- `projects/mamba/tech-design/concurrency/state-topology/class-definition-registry.md`;
- `projects/mamba/tech-design/concurrency/state-topology/class-identity-origin-catalog.md`.

It preserves the accepted class identity, definition version, publication, and
operation-lease contracts.

## Current protocol marker

```rust
RUNTIME_CHECKABLE_PROTOCOLS: RefCell<HashSet<String>>
```

The set stores a textual class name only. It owns no class, member, protocol
surface, context, definition version, or Python value.

`typing_mod::runtime_checkable`:

1. resolves the textual class name;
2. verifies that current `CLASS_REGISTRY` bases describe a Protocol;
3. inserts that text through `mark_runtime_checkable`;
4. retains `cls`;
5. returns `cls`.

The retain establishes the returned `OwnedClassAlias`. It is not ownership held
by the marker set.

`is_runtime_checkable_protocol` is consumed at:

| Reference | Semantic use |
|---|---|
| `class/mod.rs:13132` | reject `isinstance` on undecorated Protocol |
| `class/mod.rs:13236` | enable structural instance matching |
| `class/mod.rs:13808` | validate Protocol `issubclass` |

The required protocol surface is not stored in the marker. Each query derives
it live from the current class record:

- non-dunder method names;
- non-dunder class attributes;
- non-dunder annotation keys;
- whether any required member is data rather than method-only.

An `isinstance` structural match compares that derived surface against class
MRO members plus a snapshot of the instance's field keys. An `issubclass`
structural match rejects data protocols and checks class MRO members for
method-only protocols.

The marker and the derived surface can describe different class definitions.

## Current abstract-class declarations

```rust
USER_ABC_OWN_ABSTRACT:
    RefCell<HashMap<String, HashSet<String>>>
```

`mb_class_set_abstractmethods` extracts a list of names and inserts/replaces the
set for one textual class name. Its producers include:

- class lowering at `hir_to_mir.rs:6313`;
- dynamic type-object construction at
  `runtime/builtins/type_objects.rs:565`.

`compute_user_abstractmethods` reads the class record and the own-declaration
map together. It:

1. builds the most-derived-first class/MRO chain;
2. unions all own abstract declarations in the chain;
3. finds the first definition of each candidate in that chain;
4. keeps it abstract when the first definition declares it abstract;
5. removes it when the first definition is a concrete method or class
   attribute;
6. sorts the remaining names.

Consumers are:

| Reference | Semantic use |
|---|---|
| `class/mod.rs:508` | reject abstract-class instantiation |
| `class/mod.rs:566` | construct type `__abstractmethods__` frozenset |
| `stdlib/inspect_mod.rs:1824` | inspect abstract classification |

The result is recomputed rather than stored. The MRO/member view and own
declaration map have no shared visibility version.

`ABSTRACT_METHODS`, declared at `class/mod.rs:14048` as
`RefCell<HashMap<String, Vec<String>>>`, is a distinct legacy abstract path. It
is cleared at line `22071`. It is deliberately not added to this frozen
denominator.

## Current virtual subclass relations

```rust
ABC_VIRTUAL_SUBCLASSES:
    RefCell<HashSet<(String, String)>>
```

Each entry is `(child_name, parent_name)`. The set owns only Rust strings.

Two producers share it:

- `mb_collections_abc_register`, dispatched at `class/mod.rs:19071`;
- `mb_user_abc_register`, dispatched at `class/mod.rs:19076`.

Both validate a resolvable child name, insert the string pair, retain `child`,
and return `child`. The retain establishes a returned alias. The relation does
not own the Python class object.

Insertion of the same pair is idempotent in the Rust set. There is no
authoritative version distinguishing a new edge from a duplicate.

`collections_abc_virtual_match`:

- normalizes collections ABC aliases;
- scans registered pairs;
- accepts an exact parent;
- also accepts when the registered parent is nominally below the query parent.

It is used by `collections_abc_type_or_virtual_match`,
`class_matches_collections_abc`, and user ABC resolution at
`class/mod.rs:14003`.

User ABC resolution orders:

1. nominal MRO relation;
2. custom `__subclasshook__`;
3. explicit virtual relation.

## Current callback and lifetime boundary

`user_abc_subclasshook` at `class/mod.rs:5721`:

1. obtains `__subclasshook__` from `lookup_method`;
2. unwraps the descriptor;
3. extracts a registered callable address when available;
4. ends the `CALLABLE_REGISTRY` borrow;
5. invokes either the direct extern function or `mb_call_spread`;
6. converts exception/`None`/`NotImplemented` into fallback behavior.

No live `CLASS_REGISTRY`, `CALLABLE_REGISTRY`, or
`ABC_VIRTUAL_SUBCLASSES` guard spans the call. Guard freedom is a useful
current property.

The selected hook is still a raw value with no owned member, definition, or
code lease. This is an unauthenticated lifetime boundary at call entry. It is
not evidence of a currently observed concurrent mutation, UAF, or exception.

## Current same-name behavior

| Path | Current result |
|---|---|
| repeat runtime-checkable decoration | set insertion is a no-op |
| same-name new class without decorator | old marker remains |
| abstract writer on replacement path | textual key is replaced |
| same-name path without abstract writer | old declaration remains |
| duplicate virtual registration | pair insertion is a no-op |
| same-name child replacement | old child-name relation applies |
| same-name parent replacement | old parent-name relation applies |

A new current class record can inherit state created for an older runtime
identity because every admitted key is text.

## Current lifecycle

Snapshot clones all three Rust-only collections. Replace installs them
sequentially and then resets method lookup caches. There is no atomic
conformance visibility commit.

Central cleanup:

- best-effort clears `ABC_VIRTUAL_SUBCLASSES` through `try_borrow_mut`;
- does not clear `RUNTIME_CHECKABLE_PROTOCOLS`;
- does not clear `USER_ABC_OWN_ABSTRACT`.

A failed virtual-relation cleanup borrow can skip the clear. That state is
representable; no live conflict event is proved by the source.

TLS exit drops all three Rust containers. They own no Python values.

Same-context workers on different OS threads have unrelated collections.
Independent contexts executed sequentially on one OS thread can observe
ambient carryover until replace, cleanup, or TLS exit.

## Target definition policy

`ProtocolPolicy` and `AbstractClassPolicy` are derived while constructing one
proposed immutable definition.

Initial class publication computes:

- protocol kind from stable bases/MRO;
- required protocol members from the proposed member/annotation surface;
- own abstract declarations from class construction input;
- effective abstract methods from stable leased base definitions plus proposed
  members.

The definition, bases, MRO, members, protocol surface, and abstract policy
publish atomically. Incomplete or conflicting dependencies fail the
publication; they do not expose a partial policy.

Later `@runtime_checkable` decoration:

1. validates `ContextHandle`;
2. resolves exact `ClassRuntimeKey`;
3. leases the current definition;
4. verifies it is a Protocol;
5. constructs a new definition version for the same runtime key;
6. commits only if the expected definition version is current;
7. drops the publication guard;
8. returns an owned alias of the same class object.

Decoration changes `ClassDefinitionVersion`, not `ClassRuntimeKey`. A
same-display-name new class construction allocates a different
`ClassRuntimeKey`.

Member, bases, or MRO changes also create a new definition version and
recompute both protocol and abstract policy.

## Target virtual registration

Registration:

1. validates `ContextHandle`;
2. resolves parent and child to exact `ClassSubjectKey` values;
3. leases user definitions needed for validation;
4. validates that the parent supports registration;
5. prepares the returned `OwnedClassAlias`;
6. acquires the narrow virtual-relation publication guard;
7. checks context phase and inserts the typed pair;
8. advances `ConformanceVisibilityGeneration` only when insertion is new;
9. drops the guard;
10. returns the owned child alias.

Failed validation publishes nothing. Duplicate registration returns the child
alias but does not advance generation.

Relations do not keep a class definition alive. Definition retirement prunes
relations containing its `ClassRuntimeKey`; an already admitted definition
operation remains alive through its `Arc` lease.

## Target query and callback protocol

Protocol checks lease the target definition. Required members and
runtime-checkability come from that same immutable definition version.
Instance-field keys are copied under the object field guard; the guard drops
before any class/member callback or Python error construction.

User ABC subclass resolution:

1. validates context and leases exact parent/child definitions;
2. performs nominal resolution from the leased MRO;
3. selects and retains an `OwnedMemberAlias` for `__subclasshook__`;
4. drops all class/member guards;
5. invokes the hook;
6. returns a definite hook answer when provided;
7. on fallback, revalidates the context and reads the current typed virtual
   relation view;
8. returns the relation result.

The virtual-relation lookup after the callback is the relation linearization
point. A reentrant `register()` performed by the hook is therefore visible to
the fallback without holding a guard across Python code.

## Context lifecycle

Same-context workers share the ClassDomain. Independent contexts have distinct
class key spaces, definition registries, virtual relations, and generations.

Thread snapshot/replace contains no conformance definition or relation state.
TLS carries only the scoped context attachment required by the execution
context design.

Quiescence:

1. rejects new operations and registrations;
2. waits for aggregate-owned children;
3. detaches definition and relation lookup;
4. prunes/drains relations;
5. releases owned definition/member state outside internal guards;
6. reaches `Retired`.

Existing `Arc<ClassDefinition>` leases remain valid until their final holder
finishes. Retirement does not make an admitted operation dangle.

## Failure semantics

| Failure | Required result |
|---|---|
| missing/retired context | reject without consulting TLS |
| unknown class subject | explicit type/class error; no relation |
| non-protocol decorator target | Python `TypeError`; no new definition |
| expected definition conflict | publish nothing; retry or return error |
| incomplete MRO/member policy | fail closed; no partial definition |
| invalid register parent/child | Python error; no relation/generation change |
| duplicate register edge | return child alias; no generation change |
| hook selection ownership failure | do not invoke raw member |
| hook raises/returns `NotImplemented` | apply defined fallback after guards drop |
| context retires after admission | admitted leases finish; new relation read is rejected |

## Invariants

1. `ClassDefinitionRegistry` is the sole class-definition owner.
2. `ClassConformancePolicy` is a direct immutable definition value.
3. The outer `Arc<ClassDefinition>` is the sole policy lifetime lease.
4. Protocol required members publish with the member surface they describe.
5. Runtime-checkability belongs to one definition version.
6. Decoration retains the same `ClassRuntimeKey`.
7. Decoration produces a new `ClassDefinitionVersion`.
8. A new same-display-name class receives a distinct `ClassRuntimeKey`.
9. Own abstract declarations publish with their class definition.
10. Effective abstract methods publish with members, bases, and MRO.
11. Incomplete abstract/protocol derivation exposes no partial definition.
12. No protocol or admitted abstract TLS side table remains.
13. Dynamic virtual edges belong only to `VirtualSubclassRegistry`.
14. Virtual edges use exact typed subjects, never display names.
15. A new relation advances conformance visibility exactly once.
16. Failed and duplicate relation insertion does not advance visibility.
17. Duplicate registration still returns the required owned child alias.
18. Protocol queries use one leased target definition version.
19. Abstract instantiation/type/inspect consumers use the same leased policy.
20. `__subclasshook__` selection creates an owned member alias.
21. No domain, definition, relation, member, or object guard spans Python code.
22. Hook fallback reads virtual relations after callback reentry completes.
23. Returned decorator/register aliases are distinct from registry ownership.
24. Same-context threads share conformance state.
25. Independent contexts isolate same-display-name classes and relations.
26. Thread snapshot/replace contains no conformance state.
27. Retirement rejects new operations and preserves admitted leases.
28. Definition retirement removes virtual relations containing its runtime key.

## Forbidden changes

1. Do not recreate protocol or abstract TLS/global side tables.
2. Do not key virtual relations by textual class name.
3. Do not mutate conformance values inside a published definition.
4. Do not add a nested policy lease inside `Arc<ClassDefinition>`.
5. Do not advance relation generation for failed or duplicate registration.
6. Do not treat decorator update as a new class identity.
7. Do not let a new same-name class inherit old conformance state.
8. Do not store raw `MbValue` inside the virtual relation registry.
9. Do not invoke `__subclasshook__` under internal guards.
10. Do not copy conformance payload through thread snapshots.
11. Do not retain `ABSTRACT_METHODS` as a second target owner.
12. Do not merge the sibling `ABSTRACT_METHODS` source change before its
    inventory decision is accepted.

## Planned implementation paths

- `projects/mamba/src/runtime/execution_context.rs`
- `projects/mamba/src/runtime/class/mod.rs`
- `projects/mamba/src/runtime/stdlib/typing_mod.rs`
- `projects/mamba/src/runtime/stdlib/inspect_mod.rs`
- `projects/mamba/src/runtime/builtins/type_objects.rs`
- `projects/mamba/src/runtime/symbols.rs`
- `projects/mamba/src/lower/hir_to_mir.rs`
- `projects/mamba/src/runtime/mod.rs`
- focused Rust tests in the existing modules above

`runtime/mod.rs` changes centralized per-TLS cleanup routing to context-local
ClassDomain retirement. `type_objects.rs` and `hir_to_mir.rs` are producers of
abstract declarations; instantiation rejection remains in `class/mod.rs`.

## Verification map

| Test | Contract |
|---|---|
| method-only protocol instance | runtime-checkable `isinstance` structural match |
| method-only protocol subclass | runtime-checkable `issubclass` structural match |
| data protocol instance | instance data members participate |
| data protocol subclass | correct `TypeError` |
| undecorated protocol checks | both checks reject before structural match |
| decorator rejects non-protocol | no definition publication |
| decorator publication | same key, new version, old lease stable |
| own abstract method | blocks instantiation |
| inherited abstract method | blocks instantiation |
| concrete override | removes effective abstract member |
| type/inspect agreement | one leased effective policy |
| member/MRO mutation | recomputes policy in new definition |
| collections ABC register | typed relation becomes visible |
| user ABC register | typed relation becomes visible |
| duplicate registration | result alias returned; generation unchanged |
| subclasshook precedence | nominal/hook/virtual order |
| subclasshook guard probe | no internal guard during callback |
| subclasshook reentrant register | fallback sees post-callback relation |
| same display-name replacement | old policy/relation does not transfer |
| same-context workers | share definition/relation state |
| independent contexts | isolate identical display names |
| snapshot omission | no conformance payload |
| context retirement | reject new operations, drain leases |
| decorator/register alias | explicit retain/release balance |
| structural legacy search | zero admitted TLS identities remain |

The later implementation ticket must map each row to an exact runnable test or
structural command. The numeric total is only a floor.

## Acceptance gates for the later source slice

Minimum structural gate:

```bash
rg -n \
  'RUNTIME_CHECKABLE_PROTOCOLS|ABC_VIRTUAL_SUBCLASSES|USER_ABC_OWN_ABSTRACT' \
  projects/mamba/src
```

It passes only when it returns no matches.

Focused Rust gates must cover class, typing, inspect, execution-context, and
type-object producer modules. Exact Cargo selectors belong to the later source
ticket after live test-module names are confirmed.

The sibling `ABSTRACT_METHODS` inventory must be accepted before a source
implementation can claim a single abstract-policy owner. Passing this slice
alone does not prove complete Tier 1 or free-threaded execution.

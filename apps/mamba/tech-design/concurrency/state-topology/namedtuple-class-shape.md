# Namedtuple class shape topology

Literal `namedtuple(...)` bases and `typing.NamedTuple` class declarations
currently cross four representations:

1. AST recognition;
2. `NamedTupleBaseSpec` in HIR;
3. a post-definition runtime call;
4. `NamedTupleBaseShape` in a class-name-keyed TLS map.

The Python-visible `_fields` and `__match_args__` members are then published
through a different owner, `CLASS_REGISTRY`. Constructor defaults are read
through that class registry after the TLS shape has already been cloned.

This split is incompatible with free-threaded class publication. A class shape,
its derived members, its defaults, and its runtime identity must describe one
immutable class-definition version.

The target has no namedtuple shape registry. A validated
`NamedTupleShape` is optional immutable metadata directly owned by
`ClassDefinition`. The existing `Arc<ClassDefinition>` is the operation lease.
There is no separately leased shape and no display-name-keyed compatibility
owner.

```text
ExecutionContext
└── ClassDomain
    ├── ClassPublicationCoordinator
    │   └── active[ClassTransactionId] -> ClassDefinitionTransaction
    │       └── namedtuple_shape: Option<NamedTupleShapeDraft>
    └── ClassDefinitionRegistry
        └── definitions[ClassRuntimeKey] -> Arc<ClassDefinition>
            ├── version: ClassDefinitionVersion
            ├── namedtuple_shape: Option<NamedTupleShape>
            └── class_attrs
                ├── "_fields" -> OwnedMemberValue
                └── "__match_args__" -> OwnedMemberValue
```

## Bounded context

This design covers the state topology for class-statement shapes produced by:

- literal `class Child(namedtuple("Base", ["x", "y"]))`;
- equivalent literal string and tuple field forms;
- `class Point(typing.NamedTuple)` and bare `NamedTuple`;
- construction and class introspection that consume the resulting shape.

It covers:

- AST-to-HIR recognition;
- HIR-to-MIR transport;
- runtime staging and publication;
- class-definition identity and version coherence;
- constructor/default lookup;
- `_fields`, `__match_args__`, and `_field_defaults`;
- same-name rebinding;
- same-context children and independent contexts;
- cleanup and context retirement.

It does not redesign:

- the general `collections.namedtuple` factory implementation;
- tuple storage or all tuple methods;
- instance marker compatibility behavior;
- the class registry as a whole;
- class layout, slots, method cache, ABC, or protocol policy;
- all class-definition transactions.

The general factory and its instance-marker helpers remain a separate behavior
surface. They do not become an owner or direct reader of class-definition
`NamedTupleShape`.

## Aggregate and domain values

| Value | Role | Cardinality / lifetime |
|---|---|---|
| `ContextHandle` | context authority | one validated handle per operation |
| `ClassRuntimeKey` | nominal class identity | unique within one context |
| `ClassDefinitionVersion` | publication identity | monotonic per runtime key |
| `ClassDefinitionTransaction` | provisional owner | one per in-flight definition |
| `NamedTupleShapeDraft` | validated publication input | zero or one per transaction |
| `NamedTupleShape` | immutable definition value | zero or one per definition version |
| `NamedTupleTypeName` | validated source tuple display name | one per shape |
| `NamedTupleFieldName` | validated ordered field name | zero or more per shape |
| `OwnedMemberValue` | installed Python ownership claim | one per derived class member |
| `OwnedMemberAlias` | returned Python alias | one per live member result |
| `Arc<ClassDefinition>` | definition/operation lease | one per active lookup or operation |

`NamedTupleShape` is a value inside the immutable `ClassDefinition`; it is not
an aggregate root. `ClassDefinitionVersion` identifies that complete
definition publication; it is not a container that owns the shape:

```rust
struct NamedTupleShape {
    tuple_name: NamedTupleTypeName,
    fields: Vec<NamedTupleFieldName>,
}

struct ClassDefinition {
    key: ClassRuntimeKey,
    version: ClassDefinitionVersion,
    display_name: ClassDisplayName,
    // bases, MRO, members, layout, callables, and lifecycle metadata
    namedtuple_shape: Option<NamedTupleShape>,
    class_attrs: MemberMap<OwnedMemberValue>,
}
```

The shape contains Rust-owned validated values. It owns no Python object.
Python-visible `_fields` and `__match_args__` are separately owned member
claims, but they are derived and installed within the same definition version.

The `Arc<ClassDefinition>` already keeps the full immutable definition alive.
Wrapping `NamedTupleShape` in another `Arc`, lease, registry entry, or
generation would create an unnecessary second lifetime and is forbidden.

## Frozen inventory

The frozen source revision is:

`d9ea443158c54cdd99bd434cc86df2c2f9d0e117`

The exact code-reference selector is:

```bash
rg -n 'static NAMEDTUPLE_BASE_SHAPES|NAMEDTUPLE_BASE_SHAPES\.with' \
  apps/mamba/src/runtime/class/mod.rs
```

Its sorted-newline SHA-256 digest is:

`bfb9e06280553b22701ba160a7ad8444480b6d6d071656848dcbd5f9d67e5b56`

The selector excludes comments by construction. It emits five production rows
and zero test rows. The test module begins at line 22077.

### Production ledger

| Line | Category | Actual owner | Current role |
|---:|---|---|---|
| 179 | declaration | module `thread_local!` block | `RefCell<HashMap<String, NamedTupleBaseShape>>` |
| 2729 | mutation | `mb_class_set_namedtuple_base` | insert or replace by class-name text |
| 3505 | direct read | `namedtuple_subclass_shape` | clone shape by class-name text |
| 22008 | snapshot | `snapshot_thread_class_state` | clone whole map into `ThreadClassState` |
| 22028 | replace | `replace_thread_class_state` | overwrite current thread's whole map |

The partition is:

- declaration: 1;
- mutation: 1;
- direct read: 1;
- snapshot: 1;
- replace: 1;
- cleanup: 0;
- test: 0.

There is no per-class removal, same-name invalidation, class-version check,
direct cleanup, or retirement row.

### Frozen design inputs

This topology refines, but does not replace:

- `apps/mamba/tech-design/concurrency/execution-context.md`;
- `apps/mamba/tech-design/concurrency/state-topology/class-definition-registry.md`;
- `apps/mamba/tech-design/concurrency/state-topology/class-layout-policy.md`.

`ClassDefinitionRegistry` remains the sole class-definition owner.
`ClassLayoutPolicy` may consume shape-derived constraints but does not own a
copy.

## Current producer chain

### AST recognition

`literal_namedtuple_base_spec` recognizes a call-shaped base whose callable
leaf is `namedtuple`. It requires:

- a positional literal tuple name;
- a positional fields argument;
- fields represented as a literal string, list, or tuple;
- list/tuple items that are string literals.

The literal string form replaces commas with spaces and splits on whitespace.
The list and tuple forms preserve item order.

`is_typing_namedtuple_base` recognizes a bare `NamedTuple` identifier or an
attribute whose leaf is `NamedTuple`.

`class_body_namedtuple_fields` derives the typing form from annotated class
body declarations, discarding dunder names and preserving source order.

If a list/tuple literal contains a non-string item,
`literal_namedtuple_fields` returns `None`. Dynamic/unclassified bases remain
on the ordinary runtime-base path. That behavior is distinct from the runtime
shape ABI's later filtering of list values and must not be described as an
observed publication error.

### HIR transport

`hir::NamedTupleBaseSpec` stores:

```rust
pub struct NamedTupleBaseSpec {
    pub tuple_name: String,
    pub fields: Vec<String>,
}
```

`HirClass::namedtuple_base` carries `Option<NamedTupleBaseSpec>`.
`PendingClassRegistration` copies the same optional value during MIR lowering.

This representation is compile-time metadata. It has no runtime identity,
definition version, Python ownership claim, or execution-context authority.

### MIR emission

The current lowering emits:

1. `mb_class_define_multi_named`;
2. documentation/classcell staging as applicable;
3. `mb_class_set_namedtuple_base` when the spec exists;
4. other class metadata;
5. class finalization.

The definition and shape cross separate external calls. Source proves that
there is no single atomic rollback protocol tying the two current writes
together. It does not, by itself, prove that a particular exception currently
occurs between those instructions.

### Runtime publication

`mb_class_set_namedtuple_base`:

1. extracts class-name and tuple-name strings;
2. returns early when either is empty;
3. accepts only an `ObjData::List` field carrier;
4. filters that list through `extract_str`;
5. silently omits non-string runtime items;
6. inserts a Rust-owned `NamedTupleBaseShape` into the TLS map;
7. allocates separate Python string and tuple objects for `_fields`;
8. allocates another set for `__match_args__`;
9. looks up the class by the same text key;
10. inserts the two derived members only if that class record exists.

The TLS insertion happens before the class-record lookup. A missing class
record can therefore leave a shape entry with no matching visible derived
members.

The two derived tuples are independent allocations. Replacing the class
attributes ignores displaced values, so the broader current class-member
ownership and leak findings apply.

## Current ownership and replacement

`NamedTupleBaseShape` owns only:

- `tuple_name: String`;
- `fields: Vec<String>`.

It does not own:

- the class definition;
- a `ClassRuntimeKey`;
- the Python class object;
- `_fields` or `__match_args__`;
- field defaults;
- instance marker values;
- a definition or member lease.

The TLS key is textual class name. It conflates:

- display name;
- binding name;
- declaration identity;
- active runtime identity;
- definition version.

If another namedtuple definition calls the setter with the same text, it
overwrites the prior Rust shape entry.

If an ordinary class later reuses the same text, it does not call the setter.
The prior shape remains and can contaminate constructor/default behavior for
the ordinary class.

Two distinct runtime keys with the same display name cannot coexist safely in
this map. A copied text key cannot prove which definition produced the shape.

## Current lookup and consumers

### Direct TLS reader

`namedtuple_subclass_shape` is the only direct registry reader. It clones the
Rust shape under a short `RefCell` borrow.

The short borrow means later work does not hold that `RefCell` guard. The clone
is still not a class-definition lease and does not make subsequent member
reads version-coherent.

### Indirect shape consumers

| Owner | Source row | Role |
|---|---:|---|
| `namedtuple_subclass_field_defaults` | 3516 | clones shape, then reads matching default attrs from `CLASS_REGISTRY` |
| `mb_class_new_with_args` | 4234 | clones shape, then calls field seeding |
| `seed_namedtuple_subclass_fields` | 3542 | binds arguments/defaults and writes instance fields |
| `mb_getattr_impl` | 7906 | type-object `_field_defaults` path |
| `mb_getattr_impl` | 9073 | string-token `_field_defaults` path |

`namedtuple_subclass_default_for_field` performs a separate class-registry
read for each missing constructor field. Shape, defaults, and class members can
therefore come from different textual replacements or reentrant observations.

### Instance compatibility helpers

The following paths inspect instance marker fields, not the TLS shape:

| Owner | Source row | Marker behavior |
|---|---:|---|
| `namedtuple_hidden_dict_fields` | 6794 | hides marker/field keys from instance `__dict__` |
| `mb_getattr_impl` | 8362 | reconstructs `_fields` / `__match_args__` |
| `mb_getattr_impl` | 8384 | binds `_asdict` / `_replace` |
| `mb_call_method` | 20477 | dispatches `_asdict` / `_replace` |

`seed_namedtuple_subclass_fields` writes:

- `_namedtuple_name`: the constructed class display name;
- `_namedtuple_fields`: the ordered field list;
- `_namedtuple_base`: the source tuple name;
- each field value, with a retain before insertion.

These are per-instance compatibility payloads. They do not identify or own the
class definition.

General `collections.namedtuple_factory` paths are likewise not direct readers
of `NAMEDTUPLE_BASE_SHAPES`.

## Current snapshot, cleanup, and exit

`snapshot_thread_class_state` clones the entire Rust shape map. It does not:

- acquire a class-definition lease;
- retain Python derived members;
- validate that the class record exists;
- bind the shape to a definition version.

`replace_thread_class_state` overwrites the current TLS map from the snapshot.
It can transport text-keyed shape visibility independently from the class
definition and its Python-owned members.

`cleanup_all_classes` clears `CLASS_REGISTRY` but omits
`NAMEDTUPLE_BASE_SHAPES`. The same OS thread can therefore retain stale shape
metadata after centralized cleanup and expose it to a later execution.

TLS destruction eventually drops the Rust strings and vectors for that thread.
It does not repair earlier class-member ownership or establish ordered
context retirement.

## Existing behavior boundary

Current collections fixtures include:

- `namedtuple_subclass_repr_and_dict.py`;
- `namedtuple_class_introspection.py`;
- `namedtuple_defaults_fill_rightmost.py`;
- `test_named_tuple__test_namedtuple_subclass_issue_24931.py`;
- `test_named_tuple__test_defaults.py`;
- `test_named_tuple__test_match_args.py`.

Their bodies cover portions of:

- `_fields`;
- `__match_args__`;
- `_field_defaults`;
- constructor/default behavior;
- representation and dictionary behavior.

They do not prove:

- `ClassRuntimeKey` identity;
- atomic class/shape/member publication;
- rollback after validation/publication failure;
- same-version shape/default lookup;
- leased old-definition behavior;
- same-context worker sharing;
- independent-context isolation;
- snapshot omission;
- cleanup or context retirement.

Planned tests remain `NOT EXECUTED` in this Stage 1 design slice.

## Target shape validation

When AST/HIR lowering classifies a base as a static namedtuple shape, it
produces `NamedTupleShapeDraft` before runtime publication. Dynamic bases that
are not classified remain ordinary runtime base expressions and do not
silently acquire partial shape metadata. Validation of a classified or staged
shape establishes:

- a nonempty tuple display name;
- an ordered list of field names;
- string-only field inputs;
- the field-name rules required by the accepted Mamba/CPython behavior;
- no silent runtime omission of invalid fields;
- deterministic `_fields` and `__match_args__` derivation.

Invalid input fails before an accepted definition becomes visible. The
transaction records the failure; it does not install a shape, derived members,
or a partial class-definition version.

The draft is a transport value, not a published owner. It becomes immutable
`NamedTupleShape` only during definition finalization.

## Target transaction and publication

`ClassDefinitionTransaction` carries:

```rust
struct ClassDefinitionTransaction {
    id: ClassTransactionId,
    context_id: ContextId,
    runtime_key: ClassRuntimeKey,
    base_version: Option<ClassDefinitionVersion>,
    // bases, namespace, members, classcell, kwargs, metaclass state
    namedtuple_shape: Option<NamedTupleShapeDraft>,
}
```

The lowering/runtime boundary stages the shape on the transaction before the
definition commit. It does not publish into a sibling map.

Finalization:

1. validates the current `ContextHandle`;
2. resolves the active class transaction by typed id/runtime key;
3. validates and freezes `NamedTupleShapeDraft`;
4. constructs `_fields` and `__match_args__` owned member values outside the
   registry guard;
5. builds one complete immutable next `ClassDefinition`;
6. installs the shape and derived members in that same version;
7. swaps the registry-visible `Arc<ClassDefinition>` atomically;
8. drops the publication guard;
9. releases displaced/detached owned values outside all guards;
10. marks the transaction committed.

Any failure before step 7 leaves no visible next version. Any cleanup after a
failed build happens outside the registry guard.

The old `mb_class_set_namedtuple_base` post-publication owner disappears. A
replacement ABI may stage the shape on the transaction, but it must not write
published state by class-name text.

## Target lookup and version coherence

A constructor or introspection operation:

1. validates `ContextHandle`;
2. resolves `ClassRuntimeKey`;
3. clones one `Arc<ClassDefinition>` under a narrow registry guard;
4. drops the registry guard;
5. reads `namedtuple_shape`, defaults, `_fields`, and `__match_args__` from
   that one immutable definition version;
6. clones any required `OwnedMemberAlias`;
7. performs allocation, retain/release, callback, exception, and instance
   mutation work with no registry/member guard held;
8. drops aliases and the definition lease after the operation completes.

Defaults and shape cannot be fetched through separate current-version
lookups. Missing required fields and too-many-positional errors use the field
count from the leased version.

Python-visible `_field_defaults` is derived from the leased shape and the
leased version's default members. It cannot mix a prior field list with current
attributes.

## Target rebinding and leases

Same-display-name publication distinguishes two cases:

1. a new declaration/identity receives its own `ClassRuntimeKey`;
2. a replacement of the same logical runtime key receives a new
   `ClassDefinitionVersion`;
3. changes registry visibility at the publication point;
4. makes new lookups resolve only the new binding/version;
5. leaves already leased old definitions unchanged.

An old `Arc<ClassDefinition>` continues to expose:

- its old `ClassRuntimeKey`;
- its old version;
- its old shape;
- its old `_fields`, `__match_args__`, and defaults;
- its owned member claims.

Later rebinding does not mutate the old definition. Deregistration visibility
does not wait for the old lease count to reach zero; the lease only controls
detached-object lifetime.

Distinct runtime keys with the same display name remain distinct. Display text
is presentation, never lookup authority.

## Target worker and context behavior

Same-context children share `ClassDefinitionRegistry` through the accepted
context handle. They do not receive a copied shape map.

Independent contexts own independent class domains. The same display name may
resolve to different runtime keys and shapes without collision.

Compatibility thread snapshots omit namedtuple shape state. If transitional
thread state remains for unrelated class concerns, it cannot become a second
shape owner or recreate name-keyed visibility.

## Target retirement

Context retirement:

1. rejects new context/class-definition operations;
2. detaches registry-visible definitions;
3. drains in-flight constructor and member aliases;
4. waits for operation/class-definition leases;
5. releases owned Python member claims outside registry guards;
6. drops detached Rust definitions and their shape values;
7. retires the class domain.

There is no namedtuple-specific cleanup map. Shape retirement follows its
owning class definition.

Process or OS-thread exit is not the ordinary reclamation protocol.

## Target invariants

1. `ClassDefinitionRegistry` is the sole published owner of class shape.
2. `NamedTupleShape` is a direct immutable value inside one
   `ClassDefinition`, identified by its `ClassDefinitionVersion`.
3. `Arc<ClassDefinition>` is the only shape-bearing operation lease.
4. No global, TLS, context-side, or sibling namedtuple shape registry exists.
5. `ClassRuntimeKey`, not display text, identifies the owning class.
6. Definition, shape, `_fields`, and `__match_args__` publish atomically.
7. Invalid shape input fails before any next definition version is visible.
8. A failed publication exposes neither shape nor derived members.
9. Field order is preserved from the accepted source representation.
10. Literal string/list/tuple and typing forms have explicit validation.
11. Runtime field transport never silently drops an invalid field.
12. `_fields` and `__match_args__` are derived from the same shape value.
13. Derived Python members hold explicit owned claims in the same definition.
14. Shape and defaults are read from one leased definition version.
15. `_field_defaults` cannot mix fields and attrs from different versions.
16. Constructor arity uses the leased version's field count.
17. Missing-field behavior uses the leased version's defaults.
18. Same-name ordinary rebinding cannot inherit a prior shape.
19. Same-name namedtuple publication uses a distinct runtime key for a new
    declaration, or a distinct version when replacing the same runtime key.
20. Distinct runtime keys may share display text without sharing shape.
21. Old definition leases retain their old shape and owned members.
22. Later rebinding never mutates an already published definition.
23. New visibility changes immediately; it does not wait for old leases.
24. Same-context children share the context-owned definition registry.
25. Independent contexts isolate same-name definitions and shapes.
26. Thread snapshots do not copy shape state.
27. Cleanup cannot leave shape residue after definition retirement.
28. No registry/member guard spans allocation or Python work.
29. No registry/member guard spans retain, release, or deallocation.
30. No registry/member guard spans callbacks, exception work, or instance
    mutation.
31. Instance marker fields are payload, not class identity or ownership.
32. `ClassLayoutPolicy` may consume shape-derived constraints but owns no copy.
33. General namedtuple factory/marker machinery remains a separate behavior
    surface.
34. Context retirement drains definition/member leases before dropping shape
    and owned members.

## Forbidden changes

1. Do not retain or rename `NAMEDTUPLE_BASE_SHAPES`.
2. Do not create another shape map in `ExecutionContext`,
   `ClassDefinitionRegistry`, thread state, or a compatibility singleton.
3. Do not use class-name or display-name text as shape lookup authority.
4. Do not publish the class and shape through independent visible mutations.
5. Do not derive `_fields` or `__match_args__` after the definition swap.
6. Do not treat `_fields`, `__match_args__`, `_namedtuple_fields`, or
   `_namedtuple_base` as the class-shape owner.
7. Do not wrap `NamedTupleShape` in an independent `Arc` or lease.
8. Do not clone shape state through worker/thread snapshots.
9. Do not mutate an old definition version after same-name rebinding.
10. Do not delay visibility removal until old leases drain.
11. Do not read shape and defaults through separate current-definition
    lookups.
12. Do not silently filter invalid runtime field values.
13. Do not hold a definition/member guard across allocation.
14. Do not hold a definition/member guard across retain/release or callbacks.
15. Do not report general `collections.namedtuple` marker helpers as direct
    class-shape readers.
16. Do not rely on TLS or process exit as the normal retirement mechanism.

## Planned implementation paths

Required source paths:

- `apps/mamba/src/hir/mod.rs`
- `apps/mamba/src/lower/ast_to_hir.rs`
- `apps/mamba/src/lower/hir_to_mir.rs`
- `apps/mamba/src/runtime/execution_context.rs`
- `apps/mamba/src/runtime/class/mod.rs`
- `apps/mamba/src/runtime/symbols.rs`
- `apps/mamba/src/runtime/mod.rs`

Required test owners:

- focused Rust tests in the owning HIR/lowering/runtime modules;
- focused CPython behavior fixtures under
  `apps/mamba/tests/cpython/behavior/std-libs/collections/`.

Implementation must stay one cohesive class-definition owner slice. It must
not opportunistically migrate the general namedtuple factory, slots, method
cache, ABC/protocol state, or unrelated thread state.

## Planned focused tests

1. `ast_to_hir.rs::test_literal_namedtuple_field_ordering_and_types` — literal
   string, list, and tuple forms preserve the accepted order.
2. `ast_to_hir.rs::test_typing_namedtuple_class_body_filtering` — typing form
   preserves non-dunder annotated fields in source order.
3. `ast_to_hir.rs::test_invalid_namedtuple_field_spec_rejection` — an invalid
   classified/staged shape cannot publish; an unclassified dynamic base stays
   on the runtime-base path without partial shape metadata.
4. `class/mod.rs::test_atomic_namedtuple_class_publication` — definition,
   shape, `_fields`, and `__match_args__` appear in one version.
5. `class/mod.rs::test_namedtuple_constructor_arity_enforcement` — too many
   arguments report the leased version's field count.
6. `class/mod.rs::test_field_defaults_version_coherence` — defaults and shape
   come from one version.
7. `class/mod.rs::test_instance_marker_field_seeding` — instance markers
   preserve class display name, tuple base name, and field order.
8. `class/mod.rs::test_same_name_ordinary_class_rebinding_isolation` — a plain
   rebind cannot inherit the prior namedtuple shape.
9. `class/mod.rs::test_same_name_different_shape_rebinding` — a distinct
   declaration gets a distinct key, same-key replacement gets a distinct
   version, and neither mutates the old definition.
10. `class/mod.rs::test_leased_old_definition_shape_immutability` — an old
    definition lease keeps its original shape after rebinding.
11. `execution_context.rs::test_same_context_worker_shared_registry` —
    same-context children resolve the shared definition without shape copying.
12. `execution_context.rs::test_independent_context_shape_isolation` —
    same-name definitions in independent contexts remain isolated.
13. `class/mod.rs::test_thread_state_snapshot_shape_omission` — snapshot and
    replace contain no namedtuple shape state.
14. `class/mod.rs::test_context_cleanup_zero_shape_residue` — retired
    definitions cannot contaminate a later execution.
15. `class/mod.rs::test_fields_and_match_args_atomic_shape_match` — both
    visible tuples match the one published shape/version.
16. `class/mod.rs::test_missing_required_constructor_field_arity_failure` —
    missing required field uses the leased version's count/defaults.
17. `class/mod.rs::test_duplicate_display_names_distinct_runtime_keys` —
    distinct runtime keys with identical display text keep distinct shapes.
18. `class/mod.rs::test_publication_failure_zero_shape_or_derived_members` —
    forced draft/finalization failure exposes no shape or derived member.
19. `class/mod.rs::test_guard_free_execution_across_allocation` — allocation,
    retain/release, callback, exception, and instance mutation execute after
    owner guards drop.

Every planned test is `NOT EXECUTED` in this design-only slice.

## Acceptance boundary

This Stage 1 slice is complete when:

- the five-row frozen inventory remains reproducible;
- the target has exactly one shape owner;
- class identity and definition version are explicit;
- shape and derived members publish atomically;
- constructor/default/introspection reads use one definition lease;
- same-name rebinding preserves old leased versions without stale new
  visibility;
- worker sharing and independent-context isolation are explicit;
- snapshots omit shape state;
- retirement follows the class definition;
- no guard spans Python work;
- planned paths, invariants, forbidden changes, and tests remain exact.

It does not claim the source migration or planned tests have executed.

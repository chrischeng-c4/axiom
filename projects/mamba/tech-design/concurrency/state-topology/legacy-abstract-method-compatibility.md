# Legacy abstract-method compatibility topology

Issue: #3033
Parent inventory: #2968
Sibling design: #3026
Source revision: `a53bb05f080a328c0b2330437e8e54f441195032`

This Stage 1 DDD slice classifies `ABSTRACT_METHODS`, a second textual-name TLS
owner for abstract-method metadata. Current class lowering and stdlib `abc`
behavior use newer, separate paths, but three legacy `mb_*` helpers remain in
the runtime symbol table.

The target deletes `ABSTRACT_METHODS` without replacement. Abstract policy
converges on the direct immutable
`ClassDefinition::ClassConformancePolicy` accepted in #3026.

The admitted source proves no in-repo production emitter for the three legacy
helpers. It does not prove that their registered `mb_*` names are absent from
the public runtime ABI. The execution-context contract preserves that ABI.
Symbol removal is therefore blocked pending an explicit compatibility audit.
Until then, the names remain as stateless adapters into the sole versioned
class-definition owner.

No `src/**` change occurs in this inventory slice.

## Bounded context

```text
ExecutionContext
└── ClassDomain
    └── ClassDefinitionRegistry
        └── definitions[ClassRuntimeKey] -> Arc<ClassDefinition>
            └── conformance: ClassConformancePolicy
                └── abstract_class: AbstractClassPolicy

Runtime ABI compatibility surface
├── mb_abstractmethod
├── mb_register_abstract
└── mb_check_abstract
    └── stateless adapters into ClassDomain
```

The compatibility surface is not an owner. It keeps no registry, cache, TLS
payload, generation, or Python value between calls.

## Scope

This slice covers:

- `ABSTRACT_METHODS`;
- `mb_abstractmethod`;
- `mb_register_abstract`;
- `mb_check_abstract`;
- their runtime symbol registrations;
- comparison with active class lowering and stdlib `abc` paths;
- current ownership, borrow, cleanup, test, and TLS lifecycle;
- target TLS deletion and compatibility adapters.

It does not cover:

- the ABI audit required to remove a registered `mb_*` name;
- implementation of `ExecutionContext`;
- implementation of `ClassDefinitionRegistry`;
- the already accepted `USER_ABC_OWN_ABSTRACT` migration;
- all stdlib `abc` behavior;
- source changes or test execution.

## Frozen inventory

Exact selector:

```bash
rg -n \
  'static ABSTRACT_METHODS|ABSTRACT_METHODS\.with' \
  projects/mamba/src/runtime/class/mod.rs
```

Exact stdout SHA-256:

`4e1cc994f8303d3d51b66457ae4f3873d55fbb2f6c983821ace665df1cb46602`

The selector emits four production rows and one test row. The test module
begins at line `22077`.

| Line | Partition | Operation |
|---:|---|---|
| `14048` | production | TLS declaration |
| `14077` | production | insert/replace in `mb_register_abstract` |
| `14088` | production | read in `mb_check_abstract` |
| `22071` | production | best-effort cleanup |
| `25061` | test | post-cleanup assertion |

The partition is `4 + 1 = 5`.

## Frozen design inputs

This topology refines:

- `projects/mamba/tech-design/concurrency/execution-context.md`;
- `projects/mamba/tech-design/concurrency/state-topology/class-definition-registry.md`;
- `projects/mamba/tech-design/concurrency/state-topology/class-conformance-domain.md`.

It preserves:

- public `mb_*` ABI compatibility until explicitly reviewed;
- typed class identity;
- same-key/new-version publication;
- immutable definition policy;
- `Arc<ClassDefinition>` as the sole policy lease;
- no second abstract-policy owner.

## Current storage

```rust
thread_local! {
    static ABSTRACT_METHODS:
        RefCell<HashMap<String, Vec<String>>>;
}
```

The map owns Rust strings and vectors only. It owns no `MbValue`, Python RC
claim, class definition, context identity, definition version, or callable.

The textual key conflates display name and runtime identity.

## Current writer

`mb_register_abstract`:

1. converts `class_name` to a string, defaulting failure to `""`;
2. accepts only list-shaped `method_names`;
3. clones each string element;
4. ignores non-string elements;
5. preserves input order;
6. preserves duplicate names;
7. inserts/replaces the vector for the textual class key.

Same-name repeated registration replaces the full vector. A same-name class
creation that does not call the helper inherits the old vector.

No Python callback or release occurs while the TLS mutable borrow is held.

## Current reader

`mb_check_abstract` has two separate class-registry reads.

First read:

1. borrows `ABSTRACT_METHODS`;
2. borrows `CLASS_REGISTRY` inside it;
3. when the class record exists, iterates its stored `cls.mro` as-is;
4. appends declaration vectors for every textual MRO member;
5. when the class record is absent, falls back to the map's own textual key;
6. clones the resulting Rust strings;
7. drops both borrows.

The current class's own declarations participate only when its stored MRO
contains its name.

Second read:

1. borrows `CLASS_REGISTRY` again;
2. checks only the current class record's `methods` keys;
3. returns false when any collected abstract name is absent;
4. otherwise returns true.

It does not:

- inspect class attributes;
- compute first-definition-wins behavior across the MRO;
- distinguish an abstract override from a concrete override;
- call `compute_user_abstractmethods`;
- use a definition or MRO version;
- invoke Python code under the borrow.

The algorithm is not equivalent to the accepted effective-abstract policy.

## Legacy helper reachability

| Symbol | Registered | In-repo emitter/caller | Current behavior |
|---|---|---|---|
| `mb_abstractmethod` | `symbols.rs:2941` | none found | raw identity passthrough |
| `mb_register_abstract` | `symbols.rs:2947` | none found | writes legacy TLS |
| `mb_check_abstract` | `symbols.rs:2953` | none found | reads legacy TLS/current class |

Runtime symbol registration proves that a name/address can be resolved. It
does not prove that current lowering emits the name.

Source search finds no use in current lowering, codegen, or stdlib modules
outside registration. That proves absence from the admitted in-repo call
graph. It does not prove external/runtime ABI compatibility.

Therefore:

- the TLS owner is removable;
- the symbol names are not yet removable;
- an ABI inventory is the explicit removal gate.

## Active sibling paths

Current class lowering uses `mb_class_set_abstractmethods` at
`hir_to_mir.rs:6313`. Dynamic type creation calls the same sibling path from
`runtime/builtins/type_objects.rs:565`.

The stdlib decorator uses `abc_mod::mb_abc_abstractmethod`, which wraps the
function with `__isabstractmethod__` metadata. It does not use
`ABSTRACT_METHODS`.

`abc_mod::mb_abc_update_abstractmethods` is currently a no-op identity shim. It
does not recompute or publish policy.

These are separate current paths. Their existence does not make the legacy
map correct or the legacy symbol names removable.

## Current returned-value ownership

`class::mb_abstractmethod` returns its `func` input without `retain_if_ptr`.
`abc_mod::mb_abc_update_abstractmethods` returns its `cls` input without
`retain_if_ptr`.

The admitted implementations do not prove an owned return claim. The exact
caller ABI may establish a borrowed-to-return convention elsewhere, but that
contract was not proved in this slice.

Target adapters return explicit `OwnedClassAlias` or owned decorated-function
aliases. They do not rely on an ambiguous raw passthrough.

## Current lifecycle

`ThreadClassState` omits `ABSTRACT_METHODS`. Snapshot and replace neither copy
nor reset it.

`cleanup_all_classes` uses `try_borrow_mut` and clears the map only when the
borrow succeeds. A skipped clear is representable. No live conflict event is
proved.

TLS exit drops all Rust strings and vectors.

Same-context workers on different OS threads have unrelated maps. Independent
contexts run sequentially on one OS thread can observe residual entries until
cleanup or thread exit.

## Current tests

| Test | Coverage |
|---|---|
| `test_abstractmethod_passthrough` | raw identity behavior |
| `test_check_abstract_concrete` | one current-class method case |
| `test_cleanup_all_classes_clears_abstract_methods` | successful cleanup |

Only the cleanup assertion at line `25061` is part of the exact selector.

The tests do not cover:

- missing own name in stored MRO;
- inherited abstract declarations;
- concrete override semantics;
- duplicates;
- same-name replacement;
- skipped cleanup;
- cross-thread/context behavior;
- returned alias ownership;
- whether any production emitter uses the legacy symbols.

## Target TLS deletion

`ABSTRACT_METHODS` is deleted. There is no successor map.

Abstract declarations and effective abstract methods live only in
`ClassDefinition::ClassConformancePolicy`. Active class construction publishes
them atomically with members, bases, and MRO.

The legacy cleanup row and registry-only cleanup assertion disappear with the
owner.

## Target compatibility adapters

Until an ABI audit authorizes symbol removal:

### `mb_abstractmethod`

- routes to the active abstract-decorator behavior;
- creates the required `__isabstractmethod__` metadata;
- returns an explicit owned function/decorator result;
- stores no ambient state.

### `mb_register_abstract`

- validates the active context;
- resolves exact `ClassRuntimeKey`;
- leases the current definition;
- builds proposed own/effective abstract policy;
- publishes a new `ClassDefinitionVersion` for the same key;
- returns only after the atomic publication result;
- stores no side-table state.

### `mb_check_abstract`

- validates the active context;
- resolves and leases the definition;
- reads immutable `effective_abstract_methods`;
- returns whether the set is empty;
- stores no state.

The adapters are compatibility projections onto the authoritative aggregate.
They do not own a generation or lifetime.

## Target `abc.update_abstractmethods`

`abc.update_abstractmethods(cls)`:

1. validates context and exact class identity;
2. leases the current definition and base definitions;
3. recomputes effective abstract methods from immutable MRO/member policy;
4. constructs a new definition version off-lock;
5. commits for the same `ClassRuntimeKey` if the expected version is current;
6. drops the publication guard;
7. returns an explicit owned alias of `cls`.

A version conflict retries or returns an explicit error. It never mutates a
published definition in place and never falls back to legacy TLS.

## ABI removal gate

The three symbol registrations may be removed only when a separate audit
proves all of:

- they are not declared public compatibility surface;
- no supported compiled artifact resolves them;
- no plugin/extension contract references them;
- no current or retained compiler emitter can generate them;
- release compatibility policy permits removal;
- structural tests fail if the names return unexpectedly.

Until then, stateless adapters are the fail-closed choice.

## Context lifecycle

Same-context workers share leased class definitions. Independent contexts have
distinct class key spaces and policies.

Thread snapshots carry no abstract-policy payload. Context retirement rejects
new publication, detaches definitions, and allows admitted
`Arc<ClassDefinition>` leases to drain.

No class/member/publication guard spans Python callback or Python release.

## Invariants

1. `ABSTRACT_METHODS` has zero source references after migration.
2. No replacement TLS, global map, or side table is created.
3. `ClassConformancePolicy` is the sole abstract-policy owner.
4. Policy is a direct immutable `ClassDefinition` field.
5. The outer definition lease is the sole policy lifetime authority.
6. Symbol removal remains blocked until the ABI audit passes.
7. Retained symbols are stateless compatibility adapters.
8. `mb_abstractmethod` adapter produces real decorator metadata.
9. `mb_abstractmethod` adapter returns an explicit owned alias.
10. `mb_register_abstract` publishes for the same runtime key.
11. Registration creates a new immutable definition version.
12. Registration publishes policy with members/MRO dependencies atomically.
13. `mb_check_abstract` reads one leased effective policy.
14. `mb_check_abstract` owns no registry or cache.
15. Active lowering uses the same class publication protocol.
16. `abc.update_abstractmethods` is not a no-op.
17. Update keeps `ClassRuntimeKey` and changes definition version.
18. Update returns an explicit owned class alias.
19. Same-name new class identity inherits no old policy.
20. Same-context workers share policy.
21. Independent contexts isolate policy.
22. Thread snapshot/replace contains no abstract side state.
23. Retirement rejects new operations and preserves admitted leases.
24. No Python callback/release occurs under internal guards.

## Forbidden changes

1. Do not migrate the map to another ambient owner.
2. Do not mutate published definition policy in place.
3. Do not delete registered symbols without the ABI audit.
4. Do not infer active emission from registration.
5. Do not preserve textual class names as runtime identity.
6. Do not make adapters owners of policy or generation.
7. Do not keep `abc.update_abstractmethods` as a no-op.
8. Do not return raw aliases when an owned return is required.
9. Do not copy policy through thread snapshots.
10. Do not equate the legacy reader with effective abstract computation.
11. Do not hold class/publication guards across Python work.

## Planned source paths

Immediate TLS removal and adapters:

- `projects/mamba/src/runtime/class/mod.rs`
- `projects/mamba/src/runtime/symbols.rs`
- `projects/mamba/src/runtime/stdlib/abc_mod.rs`

Versioned class/context dependencies:

- planned new `projects/mamba/src/runtime/execution_context.rs`
- `projects/mamba/src/runtime/mod.rs`
- `projects/mamba/src/lower/hir_to_mir.rs`
- `projects/mamba/src/runtime/builtins/type_objects.rs`

The first three paths contain the legacy owner/symbol/stdlib surfaces. The
remaining paths provide the accepted context and active class publication
seams; they are not invented as current implementations.

## Verification map

| Planned test | Location |
|---|---|
| zero legacy state references | `class/mod.rs::tests::test_zero_legacy_abstract_state` |
| compatibility symbol or audited removal | `symbols.rs::tests::test_legacy_abstract_symbol_contract` |
| decorator adapter metadata/ownership | `class/mod.rs::tests::test_abstractmethod_adapter_owned_result` |
| register same-key/new-version | `class/mod.rs::tests::test_register_abstract_adapter_publication` |
| check leased effective policy | `class/mod.rs::tests::test_check_abstract_adapter_lease` |
| own/inherited/concrete override | `class/mod.rs::tests::test_effective_abstract_override_chain` |
| update after member mutation | `abc_mod.rs::tests::test_update_abstractmethods_after_mutation` |
| same-display-name replacement | `class/mod.rs::tests::test_abstract_policy_identity_isolation` |
| same-context and independent contexts | `execution_context.rs::tests::test_abstract_policy_context_boundaries` |
| snapshot omission and retirement | `execution_context.rs::tests::test_abstract_policy_retirement` |
| update returned alias | `abc_mod.rs::tests::test_update_abstractmethods_owned_alias` |

The later source ticket must bind every row to a runnable test or structural
gate. This inventory does not authorize implementation before the class
context/publication prerequisite exists.

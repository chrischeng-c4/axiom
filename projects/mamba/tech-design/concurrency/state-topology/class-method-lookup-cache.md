# Class method lookup cache topology

`lookup_method` currently caches a raw `MbValue` under two hashes of textual
class and member names. The cache is thread-local, has no class-definition
identity, and establishes no Python ownership claim. Its sibling generation
counter is written but never read.

This is not an authoritative class state owner, but it can return authoritative
answers. A stale positive can expose a displaced raw value; a stale negative
can hide a newly published method. Free-threaded correctness therefore requires
the projection to be explicitly subordinate to the versioned class aggregate.

The target is a context-owned, non-authoritative `MethodLookupCache` inside the
class domain. It uses exact typed identities and the aggregate's published
`ClassVisibilityGeneration`. Positive entries contain only a typed member
location and `Weak<ClassDefinition>`. A cache hit must upgrade and validate the
definition before it can return an `OwnedMemberAlias`.

```text
ExecutionContext
└── ClassDomain
    └── ClassDefinitionRegistry
        ├── visibility: ClassVisibilityGeneration
        ├── definitions[ClassRuntimeKey] -> Arc<ClassDefinition>
        └── method_lookup_cache
            └── MethodLookupKey
                ├── receiver: ClassRuntimeKey
                ├── member: MemberName
                └── visibility: ClassVisibilityGeneration
                    -> Hit(Weak<ClassDefinition>, MemberLocation)
                     | Miss
```

The cache owns no class definition and no Python member. Failure to update the
projection is a performance event. Failure to publish the authoritative
definition/generation is a correctness event and fails closed.

## Bounded context

This design covers:

- `METHOD_CACHE`;
- `METHOD_CACHE_GEN`;
- key construction;
- positive and negative method lookup caching;
- MRO lookup order;
- invalidation caller reachability;
- raw `MbValue` lifetime;
- thread snapshot/reset/cleanup/TLS exit;
- target cache key/value/version protocol;
- same-context worker sharing;
- independent-context isolation;
- context retirement.

It does not cover:

- `SIMPLE_CLASS_CACHE`;
- the class layout fast path;
- the full `ClassDefinitionRegistry` migration;
- all class mutation implementation;
- callable ABI resolution or invocation;
- slots, ABC, protocol, classcell, kwargs, or namedtuple state;
- every downstream consumer of `lookup_method`.

`SIMPLE_CLASS_CACHE` shares current reset/invalidation helpers but has a
different semantic destination: immutable `ClassLayoutPolicy`. It remains a
separate owner slice.

## Aggregate and domain values

| Value | Role | Lifetime |
|---|---|---|
| `ContextHandle` | operation authority | validated at lookup entry |
| `ContextId` | isolation identity | one per execution context |
| `ClassRuntimeKey` | nominal receiver/owner identity | unique within context |
| `ClassDefinitionVersion` | immutable definition identity | one per published definition |
| `ClassVisibilityGeneration` | aggregate read-view identity | one per successful visibility commit |
| `MemberName` | exact typed member identity | one per lookup |
| `MemberLocation` | typed owner/member/source position | valid for one owner version |
| `MethodLookupKey` | exact projection key | valid for one visibility generation |
| `CachedMethodLookup::Hit` | non-owning positive projection | weak until validated |
| `CachedMethodLookup::Miss` | negative projection | exact generation only |
| `Arc<ClassDefinition>` | definition operation lease | keeps detached definition alive |
| `OwnedMemberAlias` | returned Python ownership alias | keeps selected member usable |

Candidate target types:

```rust
struct MethodLookupKey {
    receiver: ClassRuntimeKey,
    member: MemberName,
    visibility: ClassVisibilityGeneration,
}

enum CachedMethodLookup {
    Hit {
        owner_key: ClassRuntimeKey,
        owner_version: ClassDefinitionVersion,
        owner: Weak<ClassDefinition>,
        location: MemberLocation,
    },
    Miss,
}

enum MemberLocation {
    Method(MemberName),
    ClassAttribute(MemberName),
}
```

The standard map implementation may hash these typed values internally.
Correctness comes from exact equality over the full key, not from exposing or
storing a lossy hash pair.

`MemberLocation` does not contain a raw `MbValue`. The upgraded immutable owner
definition remains the source of the member and of the explicit returned alias.

## Frozen inventory

The frozen source revision is:

`51e491bb0c59572a475b7faa5256efd6c2faec64`

The exact code-reference selector is:

```bash
rg -n \
  'static METHOD_CACHE|static METHOD_CACHE_GEN|METHOD_CACHE\.with|METHOD_CACHE_GEN\.with' \
  projects/mamba/src/runtime/class/mod.rs
```

Its sorted-newline SHA-256 digest is:

`9b01916e88375e4d03d923c49fee042db2581c0eca32556c7ec755b957615cce`

The selector excludes comments by construction. It emits eight production
rows and zero test rows. The test module begins at line 22077.

### Production ledger

| Line | Category | Actual owner | Current operation |
|---:|---|---|---|
| 183 | declaration | module `thread_local!` | raw method cache |
| 187 | declaration | module `thread_local!` | write-only generation |
| 1228 | invalidation write | `invalidate_method_cache` | wrapping generation bump |
| 1229 | invalidation write | `invalidate_method_cache` | conditional broad clear |
| 12251 | lookup read | `lookup_method` | copied raw cache hit |
| 12276 | lookup insert | `lookup_method` | conditional raw result insert |
| 21976 | reset write | `reset_class_lookup_caches` | conditional broad clear |
| 21978 | reset write | `reset_class_lookup_caches` | reset generation to zero |

The partition is:

- declarations: 2;
- invalidation writes: 2;
- lookup read: 1;
- lookup insert: 1;
- reset writes: 2;
- snapshot fields: 0;
- direct test rows: 0.

### Frozen design inputs

This topology refines:

- `projects/mamba/tech-design/concurrency/execution-context.md`;
- `projects/mamba/tech-design/concurrency/state-topology/class-definition-registry.md`;
- `projects/mamba/tech-design/concurrency/state-topology/class-callable-resolution.md`.

`ClassDefinitionRegistry` remains the sole definition/member owner.
`ClassVisibilityGeneration` is the aggregate publication boundary already
required by the class registry. This design does not create a cache-specific
source-of-truth generation.

## Current key and lookup

`hash_str` creates a new `DefaultHasher`, hashes one `&str`, and returns a
`u64`.

`lookup_method` computes:

```rust
let cache_key = (hash_str(class_name), hash_str(method_name));
```

The key omits:

- `ContextId`;
- exact class name;
- `ClassRuntimeKey`;
- class definition version;
- exact member name;
- MRO owner identity;
- MRO/publication generation.

Two different inputs with equal hash pairs are the same current cache key.
This can alias:

- two classes;
- two member names;
- one positive and one negative result;
- two class-definition versions.

The cache read returns a copied `MbValue` immediately when the hash pair is
present.

On a miss, `lookup_method`:

1. reads the textual receiver from `CLASS_REGISTRY`;
2. walks its textual `mro`;
3. checks `methods` before `class_attrs` for each MRO record;
4. copies the first raw value;
5. returns `MbValue::none()` if no record matches;
6. attempts to insert that result.

Both positive values and `MbValue::none()` are cached. A collision can
therefore produce either a wrong positive or a false negative.

## Current ownership and lifetime

Cache insertion copies the 64-bit `MbValue` representation. It does not call
`retain_if_ptr` and does not acquire:

- an `Arc<ClassDefinition>`;
- a member alias;
- a callable lease;
- a JIT module lease;
- any other Python ownership claim.

Clearing or dropping the map does not call `release_if_ptr`. This is internally
consistent only because insertion established no claim.

The cached bits rely on an external class/member owner remaining valid.
Invalidation must make the copied bits unreachable before a displaced member
claim is released. If a clear is skipped, the cache can outlive that current
owner relationship.

A hit returns raw bits with no uniform caller contract. Downstream callers may:

- compare with `None`;
- inspect tags;
- retain a result;
- wrap a method;
- resolve callable metadata;
- invoke later.

Cache presence proves none of those lifetime requirements.

## Current generation

`METHOD_CACHE_GEN`:

- starts at zero in each OS thread;
- increments with `wrapping_add(1)` before invalidation clear;
- resets to zero during reset;
- has no production read.

It does not:

- appear in the cache key;
- authenticate an entry;
- validate a hit;
- recover from a skipped clear;
- prevent ABA after wrap/reset;
- coordinate workers.

The current counter is write-only state, not a version protocol.

## Current conditional cache mutation

Both broad clear and cache insertion use ignored `try_borrow_mut` results.
A conflicting borrow would make the operation a silent no-op.

No current conflicting-borrow event is proved by the frozen source. The
failure branch is nevertheless representable and must not be reported as an
observed race or panic.

The consequences differ:

- skipped insertion loses only an optimization;
- skipped clear can preserve a stale raw positive or negative because the
  generation is unused.

This distinction is required in the target. A missed derived projection update
may remain correctness-neutral; a missed authoritative publication/version
update may not.

## Current invalidation caller reachability

| Call row | Actual owner | Reachability |
|---:|---|---|
| 1591 | `mb_class_register_named_impl` | after new class record insertion |
| 1996 | `mb_class_update_bases` | invalid/empty name returns first; otherwise reached even if class record is absent; nested attr setters may invalidate earlier |
| 2507 | `sync_class_namespace_from_dict` | empty extracted entries return first; nonempty path reaches call even if class record is absent |
| 2850 | `mb_class_set_class_attr` | empty names or failed `__parameters__` validation return first; otherwise reached even if class record is absent |
| 11586 | `mb_setattr_default` | only recognized class object + `__class__` + valid metaclass branch |
| 11758 | `mb_delattr_default` | only recognized class object branch; reached after found or not-found removal |
| 12242 | `class_replace_method` | reached after attempted replacement/removal even if class record is absent |
| 22032 | `replace_thread_class_state` | unconditional `reset_class_lookup_caches` call |
| 22074 | `cleanup_all_classes` | unconditional `reset_class_lookup_caches` call |

Current broad invalidation often runs after a no-op. That does not establish a
target requirement to advance authoritative visibility after a no-op.

### Registration

`mb_class_register_named_impl` inserts a class record, invalidates the broad
cache, then immediately calls `lookup_method("__init__")`. The lookup can
populate the new cache again.

### Bases and MRO

`mb_class_update_bases` can invoke `mb_class_set_class_attr` for
`__orig_bases__` and `__parameters__`, causing nested invalidations. It then
attempts to update bases/MRO and invalidates again even when the textual class
record was absent.

The target class publication transaction advances one visibility generation
for one accepted aggregate change rather than counting helper calls.

### Namespace and class attributes

`sync_class_namespace_from_dict` returns on an empty extracted entry list. A
nonempty list reaches invalidation even if the class record lookup did not
apply any entry.

`mb_class_set_class_attr` returns on invalid names or a failed
`__parameters__` validation. Otherwise it invalidates after the class lookup,
including the no-class branch that released the incoming retain.

The target advances visibility only when a new definition version actually
publishes.

### Metaclass reassignment

`mb_setattr_default` invalidates only when:

- the receiver resolves as a class;
- the assigned attr is `__class__`;
- the new value resolves to a registered metaclass;
- the assignment is applied.

Invalid metaclass assignment raises and returns without the cache call.

### Removal and replacement

`mb_delattr_default` invalidates inside the recognized class-object branch
whether or not a member was removed. A missing member then raises
`AttributeError`.

`class_replace_method` invalidates after an attempted mutation even if no class
record matched.

Target publication generation advances only for a successful visible member
change. Not-found or no-class paths preserve the prior generation.

## Current snapshot, cleanup, and exit

`snapshot_thread_class_state` omits both `METHOD_CACHE` and
`METHOD_CACHE_GEN`.

`replace_thread_class_state` replaces class registries, then calls
`reset_class_lookup_caches` at line 22032.

`cleanup_all_classes`, declared at line 22042, calls the same reset helper at
line 22074.

The reset helper conditionally clears the cache and unconditionally sets the
write-only generation to zero.

OS-thread TLS destruction eventually drops the raw map container. It performs
no Python releases because the map established no claims. TLS exit is not a
context retirement protocol.

## Existing test boundary

Direct source tests include:

- `test_dunder_binop_dispatch`;
- `test_dunder_unaryop_dispatch`;
- `test_dispatch_binop_reverse`;
- `test_class_attr_assignment_replaces_method_and_preserves_hash_none`;
- `test_dunder_method_dispatch_type_created_class`.

They prove selected method/MRO lookup results and one replacement surface.
Their bodies do not prove:

- a cache hit versus full lookup;
- negative caching;
- hash collision isolation;
- definition/member leases;
- visibility-generation validation;
- same-context sharing;
- independent-context isolation;
- contention fallback;
- retirement.

Planned tests remain `NOT EXECUTED` in this Stage 1 design slice.

## Target authoritative read view

`ClassDefinitionRegistry` exposes an immutable read view:

```rust
struct ClassReadView {
    context_id: ContextId,
    visibility: ClassVisibilityGeneration,
    receiver: Arc<ClassDefinition>,
    mro: Vec<Arc<ClassDefinition>>,
}
```

The registry captures the visibility generation and the receiver/MRO
definition leases under one narrow publication/read boundary. The registry
guard then drops.

If publication commits after the view is acquired, the view remains a valid
old-generation operation snapshot because its `Arc` leases keep the immutable
definitions alive. New operations acquire the new generation.

## Target cache lookup

### Positive hit

1. validate the `ContextHandle`;
2. acquire the current `ClassVisibilityGeneration`;
3. build the exact typed `MethodLookupKey`;
4. read the projection entry under a narrow cache guard;
5. drop the cache guard;
6. upgrade `Weak<ClassDefinition>`;
7. validate context, owner key, owner version, member location, and visibility;
8. create `OwnedMemberAlias` from the immutable owner;
9. return the alias with no registry/cache/member guard held.

If upgrade or validation fails, remove/prune opportunistically and perform a
full authoritative lookup.

### Negative hit

A `Miss` is usable only when:

- receiver runtime key is exact;
- member name is exact;
- visibility generation equals the current read view.

No additional raw sentinel is returned from the projection.

### Cache miss

1. capture one authoritative `ClassReadView`;
2. walk leased definitions in MRO order;
3. check `methods` before `class_attrs`;
4. build an `OwnedMemberAlias` for a hit;
5. prepare `Hit` or `Miss` under that view's generation;
6. attempt a projection insert;
7. return the resolved result even when insert is skipped.

No cache or registry guard spans:

- Python allocation;
- retain/release;
- descriptors;
- callbacks;
- exceptions;
- callable resolution;
- unsafe invocation;
- downstream result use.

## Target publication generation

The aggregate publication coordinator owns
`ClassVisibilityGeneration`.

A successful visibility-changing commit:

1. builds complete immutable definition/sibling state;
2. installs provisional records;
3. publishes the new aggregate visibility;
4. advances generation exactly once at that commit point;
5. makes old-generation cache entries unreachable.

Generation does not advance for:

- invalid input;
- no matching class/member;
- failed validation;
- rolled-back transaction;
- semantically unchanged/idempotent publication.

The generation:

- is context-scoped;
- is monotonic and non-reused;
- never resets during context lifetime;
- uses checked advancement;
- fails closed on exhaustion/overflow.

Correctness does not require synchronously clearing every cache entry. Old
entries may be pruned later.

## Target projection maintenance

Projection insert, prune, and old-generation compaction are opportunistic.

If the cache lock is unavailable:

- an insert is skipped;
- a stale entry remains unreachable by generation;
- a lookup falls back to the authoritative registry;
- publication is unaffected.

The cache lock is never required while advancing visibility generation.

Memory bounds require periodic generation-aware pruning, but pruning latency
does not change lookup correctness.

## Target worker and context behavior

Same-context children use the shared context handle, class registry,
visibility generation, and method projection. No thread snapshot copies the
cache.

Independent contexts have distinct:

- `ContextId`;
- class registries;
- runtime keys;
- visibility generations;
- method projections.

Equal display names or member names across contexts do not share entries.

## Target retirement

Context retirement:

1. rejects new context/class operations;
2. freezes publication generation;
3. detaches method projection visibility;
4. detaches registry-visible definitions;
5. drains active read views and `OwnedMemberAlias` values;
6. drops weak projection entries;
7. drops detached definitions after their final `Arc`;
8. retires the class domain.

Weak cache entries do not delay definition or Python-member retirement.

## Target invariants

1. `ClassDefinitionRegistry` remains the sole class/member source of truth.
2. `MethodLookupCache` is a non-authoritative projection.
3. The projection owns no Python value.
4. The projection owns no strong class-definition lifetime.
5. Every key contains exact `ClassRuntimeKey`.
6. Every key contains exact `MemberName`.
7. Every key contains current `ClassVisibilityGeneration`.
8. Internal map hashing never replaces exact typed equality.
9. Hash collisions cannot alias class/member identities.
10. Positive entries contain a typed location and weak owner.
11. A hit upgrades and validates the owner before member access.
12. A failed weak upgrade falls back to authoritative lookup.
13. A returned member uses `OwnedMemberAlias`.
14. A negative entry is valid for one exact generation only.
15. Methods precede class attributes during the MRO walk.
16. One read view supplies receiver, MRO, and visibility generation.
17. A successful visibility-changing publication advances generation once.
18. Invalid, not-found, unchanged, and rolled-back operations do not advance.
19. Generation is context-scoped, monotonic, checked, and non-reused.
20. Old-generation positives and negatives are unreachable after commit.
21. Correctness never depends on a synchronous broad cache clear.
22. Projection insert/prune failure is correctness-neutral.
23. Lookup falls back when projection access is unavailable or invalid.
24. No cache/registry guard spans Python work.
25. Same-context children share registry and projection without snapshots.
26. Independent contexts isolate keys, generations, and entries.
27. Weak entries do not extend detached definition/member lifetime.
28. Active aliases remain valid during detachment until their own release.
29. Context retirement rejects new lookup before detaching projection.
30. Snapshot/replace state contains no method projection.
31. `METHOD_CACHE_GEN` has no write-only target equivalent.
32. `SIMPLE_CLASS_CACHE` remains outside this owner slice.

## Forbidden changes

1. Do not retain `METHOD_CACHE` as TLS or a renamed raw-value map.
2. Do not key by lossy `(u64, u64)` hash pairs.
3. Do not store raw `MbValue` in projection entries.
4. Do not treat cache presence as member/callable lifetime.
5. Do not create a second authoritative member registry.
6. Do not store strong `Arc<ClassDefinition>` in long-lived cache entries.
7. Do not create a cache-specific write-only generation.
8. Do not reset or wrap visibility generation during context lifetime.
9. Do not advance visibility on no-op/not-found/failed publication.
10. Do not make publication depend on cache clear/insert/prune success.
11. Do not hold owner/cache guards across Python work or invocation.
12. Do not copy projection entries through `ThreadClassState`.
13. Do not share projection/generation across independent contexts.
14. Do not delay deregistration visibility until old aliases drain.
15. Do not fold `SIMPLE_CLASS_CACHE` into `MethodLookupCache`.

## Planned implementation paths

Required source paths:

- `projects/mamba/src/runtime/execution_context.rs`;
- `projects/mamba/src/runtime/class/mod.rs`;
- `projects/mamba/src/runtime/mod.rs`.

Required tests are focused Rust tests in the owning runtime modules.

The implementation remains one derived method-lookup projection slice. It
does not migrate layout fast paths or every method dispatch caller.

## Planned focused tests

1. `test_method_lookup_positive_hit_returns_owned_alias` — hit returns the
   same semantic member through an owned alias.
2. `test_method_lookup_negative_hit_exact_generation` — miss is generation
   bound.
3. `test_method_lookup_precedence_methods_over_class_attrs` — method wins.
4. `test_method_lookup_collision_resistant_keys` — forced hash collisions do
   not alias exact keys.
5. `test_method_lookup_distinct_runtime_keys_same_display_name` — runtime
   identities remain distinct.
6. `test_class_registration_advances_visibility_generation` — successful
   registration advances exactly once.
7. `test_base_mro_update_invalidates_child_results` — positive and negative
   inherited results move to the next view.
8. `test_namespace_sync_invalidation` — applied entries advance visibility.
9. `test_class_attr_replacement_invalidation` — replacement hides the prior
   entry.
10. `test_metaclass_reassignment_invalidation` — successful reassignment
    advances visibility.
11. `test_class_member_deletion_successful_advances_visibility` — removal
    advances and hides the member.
12. `test_class_member_deletion_not_found_preserves_visibility` — not-found
    preserves generation and valid entries.
13. `test_method_replacement_and_removal_invalidation` — both successful
    operations publish a new view.
14. `test_stale_raw_member_lifetime_prevention` — displaced raw values are
    never returned.
15. `test_detached_definition_weak_upgrade_fails_closed` — failed weak upgrade
    resolves authoritatively.
16. `test_projection_write_contention_skips_and_falls_back_to_authoritative_resolution`
    — insert/prune contention does not affect publication or result.
17. `test_same_context_worker_sharing_without_snapshot_copy` — children share
    one projection.
18. `test_independent_context_isolation` — equal names do not cross contexts.
19. `test_context_retirement_with_active_alias` — detached projection does not
    invalidate an existing alias.
20. `test_generation_overflow_fails_closed` — generation cannot wrap/reuse.
21. `test_no_cache_guard_across_alias_invocation` — downstream use occurs
    after all owner/cache guards drop.

Every planned test is `NOT EXECUTED` in this design-only slice.

## Acceptance boundary

This Stage 1 slice is complete when:

- the eight-row inventory remains reproducible;
- all nine invalidation/reset caller families retain their exact reachability;
- raw-value and write-only-generation hazards are explicit;
- the projection has one authoritative owner dependency;
- typed key/value and read-view semantics are complete;
- positive and negative entries are version bound;
- publication correctness is independent of cache maintenance;
- same-context and independent-context behavior is explicit;
- retirement drains aliases without strong cache retention;
- planned paths, invariants, forbidden changes, and tests remain exact.

It does not claim source implementation or planned-test execution.

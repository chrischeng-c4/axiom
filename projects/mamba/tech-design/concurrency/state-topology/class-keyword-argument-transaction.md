# Class keyword argument transaction topology

Issue: #3018
Parent inventory: #2968
Source revision: `e07b6d1443e974b17ab0d5e06d2c0297c1803e22`

This Stage 1 DDD slice classifies the TLS map used to carry non-metaclass
class-header keyword arguments from lowering or `eval_exec` into PEP 487 class
creation hooks.

The current protocol retains raw values in an unordered string-keyed side map,
later removes the map by another string, copies it through worker snapshots,
and releases it through several unrelated paths. It has no typed transaction
authority, ordered argument contract, one-shot lease, unwind-safe owner, or
context retirement.

The target creates no second aggregate. Immutable ordered
`ClassKeywordArguments` belong to the accepted
`ClassDefinitionTransaction`, owned by `ClassPublicationCoordinator`.
Published definitions remain owned by `ClassDefinitionRegistry`; published
type objects remain owned by `TypeObjectRegistry`.

No `projects/mamba/src/**` change occurs in this slice.

## Bounded context

```text
ExecutionContext
├── ClassDomain
│   ├── ClassPublicationCoordinator
│   │   └── transactions[ClassTransactionId]
│   │       └── keyword_arguments: ClassKeywordArgumentState
│   ├── ClassDefinitionRegistry
│   └── TypeObjectRegistry
└── ExecutionThreadState[*]
    └── scoped class-definition bindings
        └── ClassTransactionLease

OS-thread compatibility binding
└── ContextHandle
```

The coordinator owns provisional protocol state. An execution child owns only
a scoped lease/capability to one transaction. TLS may locate the active
`ContextHandle`; it owns no keyword-argument payload.

## Domain values

| Type | Kind | Meaning |
|---|---|---|
| `ClassTransactionId` | typed identity | context, definition, generation |
| `ClassTransactionLease` | operation lease | keeps one transaction alive |
| `ClassKeywordName` | validated value | one Python keyword name |
| `ClassKeywordArgument` | ordered value | name plus one owned Python claim |
| `ClassKeywordArguments` | immutable ordered value | normalized class-header kwargs |
| `ClassKeywordDigest` | compatibility value | digest over ordered names and value identities |
| `OwnedArgumentValue` | RAII claim | exactly one transaction-owned Python claim |
| `HookArgumentLease` | one-shot lease | detached view used by creation hooks |
| `ClassKeywordArgumentState` | state machine | absent, staged, leased, applied, committed, aborted |
| `ClassDisplayName` | metadata | Python-visible name, never authority |

Borrowed lowering inputs, transaction-owned claims, a hook-call alias, a
kwargs-dict-owned claim, a TypedDict attribute claim, and a transaction lease
are different ownership domains.

## Frozen inventory

The admitted identity is:

`projects/mamba/src/runtime/class/mod.rs::KWARGS_REGISTRY`

The exact code selector is:

`rg -n 'static KWARGS_REGISTRY|KWARGS_REGISTRY\.with' projects/mamba/src/runtime/class/mod.rs`

The SHA-256 of its sorted newline-terminated output is:

`001aa9aac88952c035a3d9fcffadc7e811028313b0c623d93237fbb966dba75e`

The selector emits eight physical code rows and eight occurrences. The frozen
test boundary is `class/mod.rs:22077`.

| Frozen row | Partition | Operation | Enclosing owner |
|---:|---|---|---|
| `159` | production | TLS declaration | `thread_local!` |
| `1753` | production | remove/take | `take_class_kwargs` |
| `2064` | production | insert/replace | `mb_class_set_kwargs` |
| `21982` | production | clone/retain snapshot | `snapshot_thread_class_state` |
| `22024` | production | raw replace | `replace_thread_class_state` |
| `22050` | production | conditional cleanup take | `cleanup_all_classes` |
| `26060` | test | direct read/assert | `test_r10_class_set_kwargs_stores_correctly` |
| `26069` | test | direct removal | `test_r10_class_set_kwargs_stores_correctly` |

Category reconciliation is six production rows plus two test rows. Comments
mentioning the symbol are outside the code-reference selector and
denominator.

## Current aggregate

```rust
thread_local! {
    static KWARGS_REGISTRY:
        RefCell<HashMap<String, HashMap<String, MbValue>>> =
            RefCell::new(HashMap::new());
}
```

The outer string has no declared identity domain:

- compiled producers pass an execution runtime-key value;
- `eval_exec` passes its interpreter class-name string;
- readers accept any `&str`;
- snapshots copy the map to another OS-thread TLS instance.

The inner `HashMap` destroys source insertion order. Python code can observe
keyword order through `kwargs.keys()`, so order is behavior, not an
implementation detail.

Neither level carries `ContextId`, `ClassTransactionId`, definition
generation, phase, digest, or ownership type.

## Current producers

### Compiled static registration

The ordinary lowering path evaluates class keyword expressions, builds
parallel key and value lists, and emits:

```text
mb_class_set_kwargs(runtime_key, keys, values)
mb_class_define_multi_named(runtime_key, display_name, ...)
```

`class_runtime_key_value` supplies the execution runtime key. Staging occurs
before registration so later creation hooks can remove the side-map entry.

### Compiled deferred runtime bases

Two deferred queues cover:

- individually runtime-evaluated base expressions;
- one runtime-evaluated base-list expression.

Both call `emit_runtime_class_kwargs` with the class runtime key before
`mb_class_update_bases`.

### `eval_exec`

`runtime/builtins/eval_exec.rs` separately:

1. excludes the `metaclass` selection keyword;
2. evaluates each remaining value;
3. builds key/value lists;
4. calls `mb_class_set_kwargs`;
5. calls `mb_class_define_multi`.

This producer uses `name.clone()` as its outer string. It does not receive
typed transaction authority from the compiled runtime-key path.

All three producer families describe the same domain event and must converge
on one normalization API.

## Current normalization

`mb_class_set_kwargs` copies the two input lists to Rust vectors and pairs them
with `zip`.

Current behavior is:

1. unequal key/value lengths truncate silently to the shorter list;
2. a direct entry is admitted only when `extract_str` succeeds;
3. the special key `"**"` accepts only an `ObjData::Dict`;
4. expansion admits only `DictKey::Str` entries;
5. unsupported mappings and non-string keys are ignored;
6. every admitted value is retained;
7. a duplicate name silently replaces the prior value and releases its claim;
8. the ordered input is finally collapsed into an unordered `HashMap`;
9. a second staging under the same outer string replaces the whole prior map.

Direct and expanded duplicates are therefore current silent last-write-wins
behavior. That fact does not decide the target policy. Target duplicate
handling is a typed normalization decision and must match the declared Python
contract fail-closed.

## Current creation-hook flow

```mermaid
sequenceDiagram
    participant Definition as class definition
    participant TLS as KWARGS_REGISTRY
    participant Hooks as creation-hook dispatcher
    participant TypedDict as TypedDict policy
    participant Descriptor as __set_name__
    participant Base as __init_subclass__

    Definition->>TLS: stage outer string -> retained values
    Hooks->>Hooks: clear creation_hooks_pending
    Hooks->>TLS: remove map; transfer claims
    Hooks->>TypedDict: read total; install independent claim
    Hooks->>Descriptor: invoke in sorted attribute order
    Hooks->>Base: build kwargs dict; invoke hook
    Hooks->>Hooks: release taken-map claims
```

`dispatch_type_new_creation_hooks` clears the class's pending bit before
taking arguments. It applies TypedDict policy, then runs descriptor
`__set_name__`, then base `__init_subclass__`.

A Python exception from either hook breaks the labeled block and reaches the
final `release_class_kwargs`. A missing base handler with non-empty arguments
raises `TypeError` and also reaches that release.

`discard_pending_type_new_creation_hooks` clears the pending bit, removes the
side-map entry, and releases the taken claims without invoking hooks.

Dispatch callers occur in ordinary registration, class finalization,
multi-class definition, direct registration, and the `type` construction
paths. Discard callers cover non-type metaclass results, metaclass errors,
classcell validation failures, fallback errors, and invalid metaclass objects.

## Current ownership ledger

1. `class_name`, key-list, and value-list parameters are borrowed inputs.
2. Each admitted direct value obtains one registry claim with
   `retain_if_ptr`.
3. Each admitted `"**"` value obtains its own registry claim.
4. Duplicate displacement releases the displaced local-map claim.
5. Whole-map displacement releases every claim in the prior registry map.
6. `take_class_kwargs` removes the map and transfers its claims without a new
   retain.
7. TypedDict `total` is borrowed from the taken map, retained independently,
   and installed as the class attribute `__total__`.
8. `build_kwargs_dict` calls `mb_dict_setitem`; the Python kwargs dict creates
   one independent `store_owned` claim for each inserted value.
9. The hook-call alias and dict-owned claim do not consume the taken-map claim.
10. Normal completion, Python hook error, and explicit discard release every
    taken-map claim.
11. Rust panic/unwind during hook work drops raw `MbValue` bits in the Rust map
    without `release_if_ptr`.
12. Snapshot clones every nested map and explicitly retains every copied
    value.
13. `ThreadClassState` has no `Drop`; abandoning an uninstalled snapshot leaks
    its added claims.
14. Replace raw-overwrites current TLS state without releasing the displaced
    map's claims. The returned separately retained snapshot does not balance
    that loss.
15. Cleanup releases values only after a successful conditional take. Borrow
    failure leaves the live map and claims installed.
16. The direct unit-test removal bypasses `release_class_kwargs`. Its current
    integer values hide the pointer-valued leak.
17. OS-thread TLS exit drops Rust containers without explicit Python-claim
    retirement.

## Current release-under-guard hazards

Four inspected paths can retain or release while an unrelated state guard is
live:

1. The whole-map replacement uses an `if let` whose Rust 2021 scrutinee
   temporary keeps the `KWARGS_REGISTRY` `RefMut` alive through
   `release_class_kwargs(previous)`.
2. `"**"` expansion holds the source dict read guard while duplicate
   displacement calls `release_if_ptr`.
3. TypedDict `__total__` insertion holds `CLASS_REGISTRY`'s mutable borrow
   while retaining the incoming value and releasing a displaced attribute.
4. `mb_dict_setitem` holds the target dict write guard while `store_owned` and
   `release_owned(old_value)` execute.

The first lifetime was independently reproduced with a same-shaped `RefCell`
drop/reborrow probe, which panicked `RefCell already borrowed`.

The target rule is uniform: detach or displace under a narrow guard, end that
guard, then retain, release, deallocate, raise, or call Python. The final
taken-map release, snapshot retain loop, and successful cleanup drain already
occur outside `KWARGS_REGISTRY` borrows and preserve that boundary.

## Current worker and context behavior

`ThreadClassState` contains `kwargs_registry`. Snapshot clones provisional
arguments and adds claims; replace installs the copy in a child's OS-thread
TLS.

This is transport, not shared execution-context ownership. It permits a
worker to inherit provisional state without a transaction lease and gives
neither parent nor child a context-bound one-shot authority.

Independent OS threads have separate TLS maps by accident. A logical context
moving between threads does not preserve typed ownership, while unrelated
contexts using the same TLS thread can observe state unless cleanup is exact.

## Existing behavior-test owners

| Owner | Proven seam |
|---|---|
| `driver/mod.rs::runtime_class_base_runs_init_subclass_once_with_kwargs` | deferred runtime base invokes once with `flag=7` |
| `runtime/class/mod.rs::test_s1_init_subclass_receives_kwargs` | staged kwargs reach `__init_subclass__` |
| `runtime/class/mod.rs::test_s2_init_subclass_no_kwargs_no_handler` | empty kwargs need no handler |
| `runtime/class/mod.rs::test_s3_init_subclass_kwargs_without_handler_raises_type_error` | non-empty kwargs without handler raise |
| `runtime/class/mod.rs::test_r10_init_subclass_without_kwargs_calls_hook` | no-kwargs hook invocation |
| `runtime/class/mod.rs::test_r10_class_set_kwargs_stores_correctly` | direct TLS staging contents |
| `tests/cpython/behavior/pep/484/typeddict_class_form.py` | TypedDict exposes default `__total__` |
| `tests/harness/cpython/config/seeds/spec/lang_typeddict.py` | `total=False` reaches `__total__` |

No current focused test proves observable mixed direct/expanded order,
duplicate conflict behavior, unequal-list rejection, pointer-valued
displacement, independent hook-dict claims, unwind cleanup, reentrant drop
safety, snapshot/replace balance, one-shot consumption, child lease authority,
independent-context isolation, or context retirement.

## Target ordered normalization

All producers call one normalization boundary before coordinator staging:

```text
borrowed evaluated entries
  -> validate outer transaction authority
  -> expand in source order
  -> validate every name and mapping
  -> detect duplicate/conflict
  -> acquire RAII OwnedArgumentValue claims
  -> freeze ordered ClassKeywordArguments + digest
```

The builder owns every acquired claim. Any validation error drops the builder
outside runtime/container guards and restores the pre-call state.

The normalized representation is an immutable ordered sequence or ordered map,
not a plain `HashMap`. It preserves:

- direct source order;
- each `"**"` mapping's iteration order;
- relative order between direct and expanded groups.

Unequal parallel-list lengths, unsupported expansion objects, non-string
expanded keys, and duplicate names fail closed with the declared Python error.
Partial normalization never stages a prefix.

Metaclass selection is not part of this value. The producer removes and
resolves it before constructing `ClassKeywordArguments`.

## Target transaction state

```rust
enum ClassKeywordArgumentState {
    Absent,
    Staged {
        arguments: ClassKeywordArguments,
        digest: ClassKeywordDigest,
    },
    HookLeaseIssued {
        lease_id: HookLeaseId,
        digest: ClassKeywordDigest,
    },
    Applied {
        digest: ClassKeywordDigest,
    },
    Committed,
    Aborted,
}
```

This state is a field of `ClassDefinitionTransaction`; it is not a sibling
registry.

Transitions are:

| From | Event | To | Result |
|---|---|---|---|
| `Absent` | valid stage | `Staged` | install ordered owned claims |
| `Staged` | same id and digest replay | `Staged` | compatible no-op |
| `Staged` | different digest replay | unchanged | typed conflict |
| `Staged` | issue hook lease | `HookLeaseIssued` | one-shot detach/lease |
| `HookLeaseIssued` | second issue | unchanged | reject |
| `HookLeaseIssued` | hooks succeed | `Applied` | preserve publication effects |
| `HookLeaseIssued` | Python error/unwind | `Aborted` | rollback and drain |
| `Staged` | nondelegating/non-type result | `Aborted` | discard and drain |
| `Applied` | aggregate commit | `Committed` | no provisional payload remains |
| nonterminal | context retirement | `Aborted` | reject new work and drain |

Compatible replay requires the same typed transaction and ordered digest.
Equal display name, equal raw string, or equal unordered key/value set is not
compatibility.

## Target hook lease

The coordinator validates and detaches a one-shot `HookArgumentLease` under a
narrow guard. Hook preparation and execution happen after that guard ends.

The lease:

- keeps the transaction alive;
- owns or leases the immutable ordered arguments;
- is bound to `ContextHandle` and `ClassTransactionId`;
- cannot be copied into `ThreadClassState`;
- releases its claims through RAII on success, Python error, early return, or
  Rust unwind.

TypedDict policy obtains a separate owned claim for published `__total__`.
Building the Python kwargs dict obtains independent dict-owned claims. Those
claims outlive or retire according to their own owners, not the hook lease.

Descriptor `__set_name__` remains before base `__init_subclass__`. Neither
callback runs with coordinator, definition-registry, source-dict, target-dict,
or compatibility-TLS guards held.

## Target commit and rollback

Commit publishes one aggregate visibility generation containing:

- the already accepted class-definition record;
- the accepted type-object record;
- classcell results when required;
- applied creation-hook state;
- no provisional keyword payload.

Rollback order is:

1. mark the transaction aborting and reject new leases;
2. detach provisional visibility and owned payloads under the coordinator
   guard;
3. end all coordinator/registry/container guards;
4. restore scoped child bindings;
5. release hook dicts, TypedDict provisional claims, and argument claims;
6. remove provisional definition/type-object records through their sole
   owners;
7. publish the terminal aborted state or retire it.

No destructor, callback, exception construction, or Python release runs while
an owning map or coordinator guard is live.

## Target worker and retirement behavior

`ThreadClassState` and worker payloads omit provisional class keyword
arguments. A same-context child may participate only when the class-definition
protocol issues an explicit scoped transaction lease. A wrong thread or
independent context cannot infer authority from display name or runtime-key
text.

Context retirement:

1. marks the context closing and rejects new stages/hook leases;
2. quiesces active hooks and child transaction leases;
3. restores scoped execution-child bindings;
4. aborts every uncommitted transaction;
5. detaches pending argument values and publication records under guards;
6. drains claims and leases outside guards;
7. retires committed definition and type-object records in dependency order;
8. marks the context retired.

Retirement is the owner boundary. OS-thread TLS destruction and best-effort
cleanup are not substitutes.

## Target invariants

1. One `ExecutionContext` owns one `ClassPublicationCoordinator`.
2. One `ClassTransactionId` names one context, definition, and generation.
3. `ClassKeywordArguments` is a field of `ClassDefinitionTransaction`.
4. No second kwargs registry or aggregate exists.
5. TLS contains no class keyword payload.
6. Display name is metadata, never authority.
7. Runtime-key text is not transaction authority.
8. Every stage presents `ContextHandle` and `ClassTransactionId`.
9. Metaclass selection is excluded before keyword normalization.
10. All producer families use one normalization boundary.
11. Normalization preserves direct source order.
12. Normalization preserves `"**"` mapping order.
13. Mixed direct/expanded relative order is observable and preserved.
14. Unequal key/value input fails closed.
15. Unsupported expansion objects fail closed.
16. Non-string expanded keys fail closed.
17. Duplicate names follow one explicit fail-closed policy.
18. Partial normalization publishes nothing.
19. Every admitted value has exactly one transaction-owned claim.
20. An immutable ordered digest defines compatible replay.
21. Same-id same-digest replay is a no-op.
22. Different-digest replay is a conflict.
23. Hook-argument lease issuance is one-shot.
24. Taking a lease does not duplicate ownership silently.
25. Hook dict values obtain independent dict-owned claims.
26. TypedDict `__total__` obtains an independent published-attribute claim.
27. `__set_name__` precedes `__init_subclass__`.
28. Python hook errors abort and drain the transaction.
29. Rust unwind drains argument claims through RAII.
30. Nondelegating and non-type metaclass results discard pending arguments.
31. No coordinator guard spans Python work.
32. No registry or container guard spans retain/release/deallocation.
33. Detach/displace occurs before release.
34. Worker snapshots omit provisional kwargs.
35. Same-context child work requires an explicit scoped lease.
36. Independent contexts isolate identical display names.
37. Commit leaves no provisional keyword payload.
38. Retirement rejects new work before quiescence.
39. Retirement drains claims outside guards.
40. Published definitions and type objects retain their accepted sole owners.

## Forbidden changes

1. Do not replace `KWARGS_REGISTRY` with another TLS/global kwargs map.
2. Do not add a second keyword-argument transaction owner.
3. Do not key authority by display name or raw runtime-key string.
4. Do not use unordered storage for Python-visible kwargs.
5. Do not preserve silent `zip` truncation.
6. Do not ignore invalid `"**"` mappings or non-string keys.
7. Do not publish partially normalized arguments.
8. Do not treat a copied `MbValue` as an owned claim.
9. Do not merge transaction, hook-dict, or TypedDict attribute claims.
10. Do not release displaced values under `RefCell`, registry, or dict guards.
11. Do not hold guards across Python callbacks or exception construction.
12. Do not copy provisional arguments into worker snapshots.
13. Do not use cleanup or TLS exit as context retirement.
14. Do not report planned tests as executed evidence.

## Planned implementation paths

| Path | Planned responsibility |
|---|---|
| `projects/mamba/src/runtime/execution_context.rs` | context-owned coordinator, ids, leases, retirement |
| `projects/mamba/src/lower/hir_to_mir.rs` | compiled producers use typed ordered staging |
| `projects/mamba/src/runtime/builtins/eval_exec.rs` | interpreter producer uses the same normalization |
| `projects/mamba/src/runtime/class/mod.rs` | retire side map; hook lease, TypedDict, dispatch/discard transitions |
| `projects/mamba/src/runtime/dict_ops.rs` | detach-before-release mutation boundary |
| `projects/mamba/src/runtime/builtins/type_objects.rs` | type/metaclass transaction propagation |
| `projects/mamba/src/runtime/mod.rs` | context binding and compatibility surface |

## Planned focused tests

1. compiled static direct kwargs success;
2. deferred runtime base-expression kwargs;
3. deferred runtime base-list kwargs;
4. `eval_exec` parity;
5. direct argument insertion order;
6. mixed direct plus `"**"` observable order;
7. multiple expansion-group order;
8. invalid expansion object failure;
9. non-string expanded key failure;
10. unequal key/value input failure;
11. duplicate normalization conflict;
12. compatible same-digest replay;
13. conflicting restage rejection;
14. whole-map displacement RC balance outside guards;
15. independent hook-dict claim;
16. independent TypedDict `__total__` claim;
17. descriptor Python-error cleanup;
18. `__init_subclass__` Python-error cleanup;
19. Rust-unwind RAII cleanup;
20. reentrant destructor at each displaced-release seam;
21. one-shot consume and discard;
22. nondelegating metaclass rollback;
23. non-type metaclass-result rollback;
24. same-context child lease;
25. wrong-thread denial;
26. independent contexts with identical display names;
27. no provisional payload in snapshot/replace;
28. pointer-valued abandoned snapshot balance;
29. context retirement with an active hook lease;
30. no guard across Python callback, dict mutation, or release.

These tests are planned and were not executed by this measure-only slice.

## Acceptance boundary

This slice is complete when:

- the eight-row identity ledger remains exact;
- every producer and dispatch/discard edge is mapped;
- ordered normalization and every ownership claim are explicit;
- the four release-under-guard hazards are preserved as implementation
  constraints;
- `ClassDefinitionTransaction` is the sole provisional owner;
- worker transport omits provisional kwargs;
- rollback and context retirement are fail closed;
- implementation remains a later AGY-owned `projects/mamba/src/**` slice.

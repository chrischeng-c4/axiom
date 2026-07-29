# Iterator domain state topology

Issue: #2992  
Parent inventory: #2968  
Source revision: `ccf2609f25`

This Stage 1 slice classifies iterator entities, handle allocation, range
subtype markers, and StopIteration signaling in `runtime/iter.rs`. It defines
their DDD ownership and free-threaded reentrancy contract without changing
`src/**`.

## Bounded contexts

```text
Process
└── ProcessHandleAllocator

ExecutionContext
├── IteratorDomain
│   └── iterators[IteratorId]
│       ├── lifecycle + exclusive advancement claim
│       ├── IteratorKind payload
│       ├── owned Python values
│       └── inner-iterator leases
├── GeneratorDomain
└── shared Python heap

ExecutionThreadState
└── exceptions
    └── StopIterationState
        ├── signaled
        └── value
```

`IteratorDomain` is owned by one `ExecutionContext`. The OS thread that creates
an iterator is current storage locality, not semantic ownership. A process
allocator supplies collision-free typed handles but cannot inspect, advance,
or retire a context's iterator.

StopIteration is child execution state. Its flag from `iter.rs` and payload
from `generator.rs` are one exception-domain value.

## Aggregate and entities

`IteratorDomain` is a sub-aggregate of `ExecutionContext`.

| Type | Kind | Identity / value |
|---|---|---|
| `IteratorDomain` | aggregate root | `ContextId` |
| `Iterator` | entity | `ContextId + IteratorId` |
| `IteratorAdvanceClaim` | lease | `ContextId + IteratorId + ClaimGeneration` |
| `IteratorId` | value | process-unique typed handle |
| `IteratorPhase` | value | `Live`, `Advancing`, `Exhausted`, `Retiring`, `Retired`, `Failed` |
| `IteratorKind` | value/payload | explicit variant, including `RangeIterator` |
| `InnerIteratorLease` | ownership edge | parent iterator + child iterator id |
| `OwnedMbValue` | value owner | retained Python value with exactly-once release |
| `StopIterationState` | child value | signaled flag + optional owned payload |

Only one `IteratorAdvanceClaim` may exist for an iterator. The claim pins the
entity across any user/JIT callback while the domain registry itself remains
unlocked.

## Frozen inventory

The four production identities have sorted newline SHA-256
`13146c4bc6ac0bd3cf37536b8d01322aa48c231d4c8bb788c4ac52b38d369731`.
There are no test-only static declarations in this file.

| Current symbol | Current storage | Current role | Target owner / disposition |
|---|---|---|---|
| `ITERATORS` | TLS `RefCell<HashMap<u64, MbIterator>>` | iterator entity graph | `ExecutionContext.IteratorDomain` |
| `NEXT_ITER_ID` | TLS `Cell<u64>` | iterator integer allocation | process `ProcessHandleAllocator` |
| `RANGE_ITERATOR_IDS` | TLS `RefCell<HashSet<u64>>` | range-iterator side discriminator | remove; subtype belongs to `IteratorKind` |
| `STOP_ITERATION` | TLS `Cell<bool>` | exhaustion signal | `ExecutionThreadState.exceptions.stop_iteration` |

The current exact reference totals are 159 `ITERATORS.with`, 4
`RANGE_ITERATOR_IDS.with`, 2 `NEXT_ITER_ID.with`, and 4
`STOP_ITERATION.with`.

The accepted AGY appendix contained every real `ITERATORS.with` match exactly
once and no duplicate, but also included adjacent non-selector lines 868, 900,
and 968. Controller normalization removes those three context rows. The
normalized family counts are:

| Family | Matches |
|---|---:|
| identity/type probes | 5 |
| generator registration | 2 |
| construction/insertion | 24 |
| advancement/state mutation | 77 |
| peek/exhaustion/length | 5 |
| user-code reentrancy seams | 23 |
| composite inner ownership/lookup | 12 |
| normal recursive release | 10 |
| forced cleanup | 1 |
| **Total** | **159** |

## Current behavior and defects

### Plain integer identity is TLS-relative

`NEXT_ITER_ID` starts at `1 << 32` independently on every OS thread. Two
threads can allocate the same integer and insert unrelated entities into their
separate `ITERATORS` maps. A handle's meaning therefore changes with caller
TLS.

The current numeric bases are:

- iterator: `1 << 32`;
- generator: `1 << 39`;
- coroutine: `1 << 40`.

Disjoint kind ranges avoid a same-thread cross-kind collision but do not prevent
same-kind duplication across worker TLS or reuse after cleanup reset.

### One ambient map owns a heterogeneous graph

`ITERATORS` stores range, collection, mapping, generator, user-defined,
callable, map/filter, zip/enumerate, chain, cycle, groupby, and other iterator
variants. Composite variants reference inner entries by raw integer ID.

Generator registration inserts an adapter entry keyed by the generator's
integer handle. Registry identity, generator lifetime, and iterator lifetime
are coordinated through matching numbers and manual unregister calls rather
than typed leases.

### RefCell avoidance encodes the reentrancy protocol

The current implementation recognizes that user code can re-enter the iterator
registry. It uses two distinct techniques:

1. remove the iterator entity before invoking user code, then reinsert it;
2. copy/snapshot needed data, end the `RefCell` borrow, invoke user code, and
   borrow again.

Examples include callable-sentinel, generator, user-defined, map/filter, and
map-N advancement. Replacing `RefCell` with one mutex held over the same
functions would deadlock or serialize callbacks and would violate the current
reentrancy intent.

Remove/reinsert also creates a temporary "missing handle" projection and makes
failure/panic recovery responsible for restoring the entity exactly once.

### Shared groupby state is not free-threaded

`GroupByOuter` and `GroupByGroup` share
`Rc<RefCell<GroupByState>>`; construction occurs through
`Rc::new(RefCell::new(...))`. Both types are non-`Send` and non-`Sync`.
Moving only the outer registry does not make the groupby entity graph safe
across workers.

### Retained values require variant-specific release

Several iterator variants retain Python objects: backing list/tuple/dict/set
objects, user-defined iterator objects, callables, sentinels, predicates, and
other callback values.

`mb_iter_release` removes one entity and `release_iter` releases retained
values. Composite variants recursively remove/release inner iterator entries.
Correctness depends on variant-specific knowledge and on each inner ID having
one ownership edge.

### Forced cleanup intentionally leaks

`cleanup_all_iterators` clears `ITERATORS` without calling `release_iter`
because current entries may contain borrowed or imprecisely retained
`MbValue`s. Process teardown is expected to reclaim them. It also clears the
range marker set, resets the TLS ID counter, and resets the StopIteration flag.

That behavior is unsuitable for independent context retirement: it leaks
context-owned heap values, can reuse live handles, and cannot prove that a
different TLS registry is untouched.

### StopIteration flag and value are split

`STOP_ITERATION` distinguishes a yielded Python `None` from exhaustion.
Generator completion separately writes `LAST_STOP_VALUE`. Both are TLS raw
state, so a signal and payload can be read from different worker-local
lifetimes or reset by different module cleanup paths.

## Iterator entity contract

The target entity has typed, explicit ownership:

```text
Iterator {
    id: IteratorId,
    phase: IteratorPhase,
    claim_generation: ClaimGeneration,
    kind: IteratorKind,
    peeked: Option<OwnedMbValue>,
    inner_leases: Vec<InnerIteratorLease>,
}
```

Every `IteratorKind` variant declares:

- values it owns strongly;
- values it borrows only for one call;
- inner iterators it owns or merely observes;
- callback/JIT dependencies;
- advancement state;
- release behavior.

The type system distinguishes `RangeIterable` from `RangeIterator`. The
`RANGE_ITERATOR_IDS` side set and its mark/unmark/cleanup coordination are
deleted.

## Handle allocation

`ProcessHandleAllocator` issues a typed handle:

```text
RuntimeHandle {
    kind: Iterator | Generator | Coroutine | ...,
    id: NonZeroU64,
}
```

IDs are never reset while the process lives. Context retirement removes the
context mapping but does not make an old integer valid for a new entity.

The unchanged `MbValue` ABI may temporarily encode the handle as an integer,
but every lookup also validates the current `ContextId` and handle kind. A raw
integer alone is not authority to access another context.

## Advancement and reentrancy

```mermaid
sequenceDiagram
    participant Child as ExecutionThreadState
    participant Domain as IteratorDomain
    participant Iter as Iterator entity
    participant User as User or JIT callback

    Child->>Domain: claim(ContextId, IteratorId)
    Domain->>Iter: acquire exclusive advance claim
    alt already advancing
        Iter-->>Child: fail with re-entry/concurrency error
    else claim acquired
        Iter-->>Child: scoped snapshot + owned callback handles
        Note over Domain: registry lock released
        Child->>User: invoke callback / nested next / generator resume
        User-->>Child: value, exhaustion, or exception
        Child->>Domain: commit by id + claim generation
        Domain->>Iter: update phase/peek/state; release claim
    end
```

No domain registry lock is held during arbitrary user/JIT code. The entity need
not disappear from lookup while advancing; its phase and claim make re-entry
explicit. Panic/error cleanup releases the claim through RAII and commits one
defined failure state.

Two workers racing to advance one iterator cannot both mutate its cursor.
Different iterators in the same context may advance concurrently unless they
share an explicit inner/group state owner.

## Composite ownership

Composite iterators use typed `InnerIteratorLease`s rather than unqualified
integer IDs. Each edge declares whether parent retirement retires the child or
only releases a reference.

Recursive release traverses lease ownership, detects cycles or repeated edges,
and releases each entity once. A parent cannot remove an inner iterator that
another live owner still leases.

Generator adapters hold a `GeneratorId` lease into `GeneratorDomain`. Advancing
the adapter acquires the generator execution claim after releasing any
iterator-domain registry lock. Generator retirement releases the adapter lease
through the aggregate boundary rather than a best-effort matching-number
unregister.

## Groupby ownership

Groupby shared state becomes a context-owned `GroupBySession` entity referenced
by the outer and active group iterators. Its mutable cursor/group ownership is
guarded by a session claim or synchronization primitive that is never held
across user key-function execution.

`Rc<RefCell<_>>` is removed. A mechanical `Arc<Mutex<_>>` replacement is not
enough unless lock scope and callback boundaries satisfy the claim contract.

## StopIteration contract

Flag and payload become one child-owned state:

```text
StopIterationState =
    Clear
  | Signaled { value: Option<OwnedMbValue> }
```

Producing exhaustion stores the payload and signal atomically within the
current `ExecutionThreadState`. Consuming it moves out the payload and returns
to `Clear`. Child exception-state retirement releases any unconsumed value.

Iterator and generator cleanup do not independently reset halves of this state.
Compatibility `mb_*` entries resolve the installed child state through scoped
handles.

## Normal retirement

`mb_iter_release` becomes `IteratorDomain::retire`:

1. reject or wait while an advancement claim exists;
2. mark the entity `Retiring`;
3. release callback/backing-object `OwnedMbValue`s;
4. release inner iterator and generator leases;
5. retire groupby/session ownership if this was the final lease;
6. mark `Retired` and remove the context mapping.

Retirement is idempotent and cannot touch another context.

## Context quiescence and forced reset

`ExecutionContext::quiesce` closes iterator admission and waits for every
advance claim. It then retires all iterator entities through the same typed
release contract.

A failed-context forced reset may skip user-visible callbacks, but it still
releases all values and leases. Failure to prove quiescence fails closed.
Forced reset never clears another context, resets the process handle allocator,
or treats process exit as ordinary context cleanup.

## Invariants

1. Every iterator belongs to exactly one `ExecutionContext.IteratorDomain`.
2. Every live runtime handle is process-unique and kind-qualified.
3. Raw integer bits alone cannot access an iterator.
4. One iterator has at most one advancement claim.
5. The domain registry is unlocked during user/JIT/generator execution.
6. Panic/error paths restore or retire a claimed entity exactly once.
7. Every retained Python value has one typed release path.
8. Composite ownership uses leases, not unqualified integer recursion.
9. Generator adapter lifetime is coordinated with `GeneratorDomain`.
10. Range subtype is stored on the entity; no parallel marker set exists.
11. Groupby shared state is free-thread safe without holding a lock over user
    code.
12. StopIteration flag and payload are one child exception value.
13. Normal and forced context retirement both require quiescence and release
    owned values.
14. Context cleanup cannot reset process identity state or another context's
    iterator graph.

## Migration order

1. Finish the remaining #2968 owner slices and close the exact-set parent.
2. Land #2839's scoped context and child-state handles.
3. Add `ProcessHandleAllocator`, typed handles, `IteratorDomain`, lifecycle,
   and exclusive advancement claims.
4. Move `ITERATORS` into the context domain while preserving the public ABI.
5. Convert callback/reentrancy paths to claim/snapshot/invoke/commit without a
   registry lock across user code.
6. Add typed owned values and inner leases; integrate generator adapters.
7. Replace groupby `Rc<RefCell<_>>` with the reviewed session owner.
8. Remove `RANGE_ITERATOR_IDS` and TLS ID reset.
9. Merge StopIteration flag/payload into child exception state.
10. Replace forced leak cleanup with quiescent typed retirement.

Each source step is a separate bounded AGY ticket. Handle allocation, registry
migration, reentrancy, ownership release, groupby, and exception integration
must not be one implementation packet.

## Forbidden fixes

- Moving `ITERATORS` into one process-global mutex map.
- Holding a registry or iterator lock across user/JIT/generator execution.
- Preserving remove/reinsert without panic-safe ownership evidence.
- Treating TLS as permanent iterator affinity.
- Reusing IDs after worker or context cleanup.
- Keeping `RANGE_ITERATOR_IDS` as a second subtype owner.
- Replacing `Rc<RefCell<_>>` with `Arc<Mutex<_>>` without reviewing lock scope.
- Clearing retained iterator values instead of releasing typed ownership.
- Keeping StopIteration flag and payload in separate cleanup domains.

## Verification surface

- Inventory: exactly 4 production declarations and 0 test-only statics.
- Digest:
  `13146c4bc6ac0bd3cf37536b8d01322aa48c231d4c8bb788c4ac52b38d369731`.
- Selector totals remain 159 / 4 / 2 / 4 until migration removes the old state.
- Two contexts allocate and use iterators without integer alias or lookup
  crossover.
- Two workers racing one iterator yield one claim and deterministic loser
  behavior; two independent iterators overlap.
- User callbacks re-enter the iterator domain without deadlock or missing-entity
  corruption.
- Panic during callback releases the claim and preserves one valid entity state.
- Composite and generator-adapter retirement release each lease once.
- Groupby outer/group workers cannot race shared state or hold a lock across key
  callbacks.
- Normal and failed-context retirement have balanced retained-value ownership.
- StopIteration signal and payload move atomically within one child state.
- Context cleanup never resets the process allocator or another context's
  iterators.
- Snapshot rule: #2992 permits no AGY repository writes and no controller
  `projects/mamba/src/**` changes.

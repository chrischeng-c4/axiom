# Weak-reference registry lifecycle — DDD contract

Scope: ownership and teardown of the runtime's weak-reference registries. This
contract separates the first-invalid-access diagnosis from the eventual source
repair. It is consumed by #2560, #2539, and the ExecutionContext migration
#2530.

## Bounded context

`WeakReferenceLifecycle` owns the relationship between:

- one referent;
- the ref, proxy, and finalizer wrappers registered for it;
- the registry-owned retain on each wrapper;
- referent-death notification;
- runtime-context teardown.

It does not own `copy` semantics. The copy corpus is a composition that exposes
this boundary; a crash observed in `test_deepcopy_atomic` is a victim event
until the detector names the first invalid ownership operation.

It also does not own general garbage collection. GC publishes
`ReferentCollected`; this context detaches weak entries without re-entering an
unsafe partial teardown.

## Aggregate root

`WeakRegistry` is the aggregate root for exactly one runtime execution context.
Process-global or bare thread-local registry identity is migration input, not
the target model.

```text
Empty
  -> Active
  -> Draining
  -> Drained
```

`Active` accepts registration and lookup. `Draining` rejects new registration,
detaches every entry, marks wrappers dead, and reconciles registry-owned
retains. `Drained` contains no wrapper capable of being returned by a later
execution context.

### Entities and value objects

| Type | Kind | Identity / value |
|---|---|---|
| `WeakRegistry` | aggregate root | execution-context id |
| `WeakEntry` | entity | registry id + entry sequence |
| `ReferentIdentity` | value | context id + allocation generation + address |
| `WrapperIdentity` | value | wrapper object identity |
| `RegistryRetain` | value | exactly one owned retain per registered wrapper |
| `EntryKind` | value | `Ref`, `Proxy`, or `Finalizer` |
| `EntryState` | value | `Live`, `ReferentDead`, `Detached`, or `Released` |
| `TerminalClass` | value | `Clean`, `DetectorAbort`, `LeakImbalance`, or `Incomplete` |

An address alone is presentation data. Allocator reuse must not make an entry
from an older referent generation visible for a newer object at the same
address.

## Entry lifecycle

```text
Created
  -> Registered
  -> ReferentDead
  -> Detached
  -> Released
```

- `Created -> Registered` transfers one retain to the registry.
- A reused no-callback wrapper returned to a caller gains a separate caller
  retain; it does not transfer the registry retain.
- `ReferentDead` clears the target-observable state before callbacks can
  inspect the wrapper.
- `Detached` removes the entry from lookup before any registry retain is
  released.
- `Released` reconciles exactly the retain acquired at registration.

Context teardown may transition `Registered -> Detached -> Released` without a
referent-death callback. It may not leave a registered wrapper reachable by the
next context.

## Invariants

1. **One registry retain.** Every registered wrapper owns exactly one
   registry retain, independent of caller-visible references.
2. **Detach before release.** No lookup may observe an entry after its registry
   retain begins release.
3. **No address resurrection.** Reusing a raw address cannot associate a new
   referent with an old entry.
4. **Reuse returns ownership.** `weakref.ref(obj)` reuse returns a caller-owned
   reference while preserving the registry-owned reference.
5. **Alias fields own independently.** Multiple wrapper fields containing one
   pointer each own the retain that their teardown releases.
6. **Teardown is complete or red.** Missing terminal evidence, a detector
   abort without attribution, or unreconciled retains is `Incomplete`, never
   `Clean`.
7. **Victim is not culprit.** The test active at an allocator trap is not
   called the writer unless a first-invalid-access event or paired provenance
   proves it.
8. **One factor per comparison.** Source revision, binary SHA, test order,
   profile, detector mode, and trial count are fixed except for the named arm.
9. **Release behavior is unchanged by diagnosis.** Attribution and counters
   used to localize this defect remain debug/test-only.
10. **Context ownership is explicit.** The final repair must compose with
    `ExecutionContext`; it may not introduce another process-global cleanup
    list.

## Domain services

### `WeakEntryRegistrar`

Owned seam: registration and no-callback lookup in
`src/runtime/stdlib/weakref_mod.rs`.

- acquires the registry retain;
- records referent and wrapper identity;
- returns reused wrappers with a distinct caller retain;
- never treats an unresolved or stale entry as live.

### `ReferentDeathCoordinator`

Owned seam: `notify_referent_collected` and its RC/GC caller.

- detaches the referent's entries;
- marks wrappers dead before callbacks;
- schedules safe retain reconciliation without re-entering a partially
  deallocated object graph.

### `WeakRegistryTeardown`

Owned seam: the weakref portion of context teardown.

- drains both weakref and finalizer registries;
- prevents registration while draining;
- reconciles every registry retain or reports an incomplete terminal;
- is idempotent for an already-drained context.

### `FirstInvalidOwnershipAttributor`

Owned seam: #2585's detector and the pinned test composition.

- holds source revision, binary SHA, optimization profile, test order, and
  detector mode fixed;
- emits the first guarded retain/release operation, pointer class, and active
  test;
- distinguishes detector abort, native death, ordinary test failure, and
  clean terminal;
- writes measurement artifacts outside the repository.

## #2560 diagnostic slice

#2560 is diagnosis only. It must:

1. build or identify one opt-level-1 integration-test binary and pin its
   SHA-256;
2. derive the ordered `copy` composition from that binary;
3. prove a non-zero fixed-composition native-death or detector-abort rate;
4. add the weakref witness to the same composition and compare one-factor arms;
5. name the first invalid operation and active test when the detector fires;
6. produce a source-provenance shortlist without editing `src/**`.

A green arm alone is not a fix claim. Heap-layout changes can suppress this
family without repairing it.

## Work-item decomposition

| Order | Work item | Output | Source writes |
|---:|---|---|---|
| 1 | #2560 residual copy/weakref attribution | pinned composition, rate table, detector event, source shortlist | none |
| 2 | repair leaf filed from #2560 evidence | one bounded ownership/lifecycle change plus regression test | AGY-owned `src/**` only |
| 3 | ExecutionContext integration | registry field + context-local drain | AGY-owned `src/**` only |
| 4 | family verification | paired opt profiles and fixed-composition soak | none unless a new defect is proven |

The controller owns this contract, ticket boundaries, oracle, verification,
commit, and closure. AGY owns every eventual `apps/mamba/src/**` edit.

## Forbidden fixes

- adding a retain without showing which owned slot it balances;
- clearing a registry by dropping entries whose retained wrappers are still
  discoverable;
- keying durable identity by address alone;
- calling GC collection during teardown to make a count reach zero;
- treating a missing result line, signal, or truncated trial as clean;
- changing test order, profile, or binary between comparison arms;
- editing generated copy/weakref fixtures to avoid the runtime path.

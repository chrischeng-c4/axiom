# Weak-reference domain state topology

Issue: #2998
Parent inventory: #2968
Source revision: `f7e678d437`

This Stage 1 slice classifies weak-reference and finalizer registration state
declared by `runtime/stdlib/weakref_mod.rs`. It defines referent identity,
notification, callback, ownership, and retirement boundaries for free-threaded
execution without changing `src/**` or claiming completion of #1466.

## Bounded contexts

```text
ExecutionContext
├── WeakReferenceDomain
│   ├── referents[ObjectIdentity]
│   │   ├── weak_entries[WeakEntryId]
│   │   └── finalizers[FinalizerId]
│   ├── callback_claims[CallbackClaimId]
│   └── phase
├── GcDomain
│   └── object-lifecycle notification source
├── GlobalValueDomain
│   └── roots, not a liveness oracle
└── ThreadDomain
    └── registered mutators

ExecutionThreadState
└── scoped ContextHandle

Test harness
└── callback counters and captured argument bits
```

`WeakReferenceDomain` belongs to one `ExecutionContext`. It is the authority
for weak entry and finalizer lookup for objects owned by that context.

`GcDomain` and reference-count retirement publish object-lifecycle events into
the domain. They do not own the weakref registry or run arbitrary Python
callbacks inside collection/deallocation phases. `ExecutionThreadState`
supplies only the scoped context handle needed to find the domain.

## Aggregate and values

| Type | Kind | Identity / value |
|---|---|---|
| `WeakReferenceDomain` | aggregate sub-root | `ContextId` |
| `ObjectIdentity` | typed value | `ContextId + ObjectId + ObjectGeneration` |
| `WeakEntry` | entity | `ContextId + WeakEntryId` |
| `FinalizerEntry` | entity | `ContextId + FinalizerId` |
| `WeakReferentHandle` | non-owning generation-safe value | `ObjectIdentity` |
| `OwnedEntryHandle` | owned value | one retained weakref/proxy/finalizer claim |
| `CallbackClaim` | owned one-shot entity | callback + arguments + source entry generation |
| `DomainPhase` | value | open / sealing / draining / retired |
| `EntryState` | value | live / claimed / dead / detached / completed |

A weak referent handle never increments the referent's strong count. Lookup
validates both context and object generation. A registry entry owns the
weakref/proxy/finalizer object that it publishes, but registry ownership is
separate from ownership of fields stored inside that object.

## Frozen inventory

The two production and three test-only identities have byte-sorted,
newline-terminated SHA-256
`1b14a75e0c8f45373e59a78e98997b061b4d09285689c1696c0f6d66bab5158b`.

| Current symbol | Kind | Current storage / role | Target owner or disposition |
|---|---|---|---|
| `FINALIZE_REGISTRY` | production | TLS map from raw referent key to retained finalizer objects | `ExecutionContext.WeakReferenceDomain` |
| `WEAKREF_REGISTRY` | production | TLS map from raw referent key to retained weakref/proxy objects | `ExecutionContext.WeakReferenceDomain` |
| `FINALIZE_CALLBACK_COUNT` | test-only | process atomic callback count | scoped test harness |
| `REF_CALLBACK_ARG_BITS` | test-only | process atomic captured callback argument | scoped test harness |
| `REF_CALLBACK_COUNT` | test-only | process atomic callback count | scoped test harness |

The accepted selector evidence contains 30 physical path/line rows and 30
identity occurrences:

| Identity | Occurrences |
|---|---:|
| `WEAKREF_REGISTRY` | 10 |
| `FINALIZE_REGISTRY` | 4 |
| `FINALIZE_CALLBACK_COUNT` | 6 |
| `REF_CALLBACK_COUNT` | 6 |
| `REF_CALLBACK_ARG_BITS` | 4 |
| **Total** | **30** |

The tenth `WEAKREF_REGISTRY` row is the ownership comment beside finalizer
removal. It is intentionally part of the frozen textual denominator.

## Current construction variants

### `weakref.ref`

`mb_weakref_ref` reuses a no-callback `ReferenceType` entry when possible.
Otherwise it constructs an Instance with `_target`, integer `_target_id`,
`_dead`, `_global_tracked`, callback aliases, and class metadata, then retains
the Instance once for `WEAKREF_REGISTRY`.

The `_target`, primary `__callback__`, and other raw parameters are copied into
the field map without a local retain. This slice cannot prove whether the
caller transfers an owned argument slot or supplies a borrowed value. The
current ABI edge is therefore unresolved:

- if borrowed, ordinary Instance field destruction can over-release it;
- if transferred, the caller or argument-container cleanup must not release
  the same slot, which this source does not prove.

The target replaces this ambiguity with typed owned and borrowed field APIs.
Weak references store `WeakReferentHandle`, never a strong/raw `_target`
pointer whose ownership depends on call-site convention.

### Proxy wrapper

The wrapper branch creates `ProxyType` or `CallableProxyType`, owns a fresh
string `_target_id`, stores callback aliases, and registers the proxy.

Only the integer-function-handle branch also stores `_target`. Other wrappers
recover a raw address from `_target_id`; this is not a lifetime pin and can
authorize an allocation that later reuses the address.

The target stores one typed `WeakReferentHandle` for every proxy wrapper.
Function-handle and heap-object referents use typed handle variants rather than
encoding authority in a string.

### Legacy no-wrapper proxy

When no wrapper is required, the implementation returns the referent itself,
performs two explicit retains, and creates no registry entry. This is a strong
alias compatibility carve-out, not weak-reference state.

It remains outside `WeakReferenceDomain` until a product decision removes the
carve-out or replaces it with a real proxy. Its two retains require their own
caller/return ownership gate and cannot be inferred from wrapper behavior.

### Finalizer

`mb_weakref_finalize` explicitly retains both `_obj` and `_func`, then the
finalizer registry explicitly retains the finalizer Instance.

The `_obj` retain makes the current finalizer strong: while the finalizer
remains alive, ordinary reference counting cannot reach zero solely by
releasing external references. `expire_unbound_finalizers` therefore uses the
current TLS global namespace as a heuristic trigger.

The target `FinalizerEntry` owns callback/function/argument data but stores only
a `WeakReferentHandle` for the referent. Object-lifecycle notification, not
absence from one worker's globals, authorizes the finalizer claim.

## Current ownership ledger

| Storage | Current ownership evidence | Current retirement |
|---|---|---|
| weakref/proxy `_target` | unresolved raw argument-field edge | Instance destruction releases it; `mark_weakref_dead` replaces it without releasing the old value |
| ref integer `_target_id` | immediate | no release |
| proxy string `_target_id` | fresh rc=1 allocation | Instance field destruction |
| primary `__callback__` | unresolved raw argument-field edge | Instance field destruction |
| duplicate `_callback` | one explicit extra retain | Instance field destruction |
| finalizer `_obj` | explicit retain | `finalize_call` replaces and releases; otherwise Instance destruction |
| finalizer `_func` | explicit retain | Instance destruction |
| finalizer `_args` list | fresh rc=1 container; element transfer is unresolved | Instance destruction |
| registry entry object | explicit retain in registry push | direct notification removes the map entry but intentionally skips release |
| reused plain entry return | explicit retain | caller retirement |
| no-wrapper proxy return | two explicit retains | caller/return retirement |
| liveness-sweep snapshot | copied `MbValue` bits, no retain | no independent lifetime claim |

An implementation ticket must not resolve an `unresolved` row by adding or
removing a retain locally. It first defines and enforces the callable argument
ownership convention or uses a typed API whose signature makes the transfer
explicit.

## Current behavior and defects

### TLS splits registration from notification

Registration occurs in the creator worker's TLS map. `mb_release` and
`GcDomain` notification run on whichever worker retires the referent.

If worker A registers a weakref and worker B performs the final release,
worker B removes from its own empty registry. Worker A keeps a stale entry,
does not mark the weakref dead, and does not run the finalizer.

Target notification carries `ContextId + ObjectIdentity` and reaches the
context-owned domain independently of worker affinity.

### Raw referent keys are not identity

Heap referents use their pointer address. Immediate values fold bits into a
47-bit key. Neither key includes context, handle kind, or generation.

A same-worker direct notification removes the key before address reuse. A stale
collision still occurs when notification reaches another worker's TLS, a
deallocation path omits notification, or a liveness sweep marks an entry dead
without removing it.

Leaked entry ownership after a successful removal is a different defect:
skipping `release_if_ptr` leaks the retained weakref/finalizer object but no
longer leaves that map key available for collision.

### TLS globals are not a liveness oracle

The `expire_unbound_*` helpers snapshot the active worker's
`GLOBAL_ID_NAMESPACE`. Absence from that one map does not prove that the
referent is unreachable from another worker, task, stack, context-local
registry, or heap edge.

The copied registry snapshot also does not retain its entries for the duration
of the sweep. Current TLS limits simultaneous access on one worker, but it does
not supply the target free-threaded lifetime claim.

Target expiry is driven by typed object-lifecycle notification. Global-value
roots participate in GC reachability; they do not independently declare weak
referents dead.

### Direct notifications have asymmetric callback behavior

`notify_referent_collected` removes both registry vectors after its short TLS
borrow ends.

- Weakref/proxy entries are marked dead. Ref callbacks are not invoked.
- Finalizer entries call `run_finalize_once`, so user code may run.

Ref callbacks run only from `expire_unbound_ref_callbacks`. These paths must
not be collapsed into one generic callback claim.

### Finalizers run inside unsafe lifecycle phases

On the RC path, notification occurs after the referent count reaches zero and
before the immortal reentrancy marker, GC untrack, contained-value release,
and Box drop.

On the GC path, notification occurs after selected objects are marked
immortal and the TLS GC borrow is released, but before contained-value release
and object deallocation finish. The collector phase remains active.

Registry borrows are not held during the callback. The defect is executing
arbitrary Python inside object deallocation or graph-sweep phases.

The target lifecycle event first creates owned one-shot callback claims without
running Python. Claims execute only after the collector/deallocator publishes a
safe callback phase.

### Field mutation loses ownership

`mark_weakref_dead` replaces `_target` with `None` but discards the returned old
value. If the field owned a strong reference, that reference leaks. If it was
borrowed, ordinary field destruction was already unsafe. The raw map cannot
express the difference.

`finalize_call` is better defined: it sets `alive=false`, replaces `_obj`, and
releases the returned old object before invoking `_func`. That release can
re-enter referent notification. The reentrant finalizer sees `alive=false` and
does not invoke `_func` again; the registry entry removal and leaked registry
retain remain separate effects.

`finalize_detach` sets `alive=false` but leaves `_obj` installed and retained
until final Instance destruction.

### Runtime reset omits both registries

`cleanup_all_runtime_state` contains no weakref/finalizer cleanup. Repeated
executions on one worker can therefore observe old entries. OS-thread TLS
destruction drops Rust maps of raw `MbValue` bits without balancing the
explicit registry retains.

Context retirement requires an explicit drain; thread exit is not aggregate
retirement.

## Target notification and callback contract

```mermaid
sequenceDiagram
    participant Life as RC or GcDomain
    participant Weak as WeakReferenceDomain
    participant Queue as Context callback queue
    participant Py as Python callback

    Life->>Weak: retire(ObjectIdentity, cause)
    Weak->>Weak: seal entry generation and extract entries
    Weak->>Weak: mark dead / create owned one-shot callback claims
    Weak-->>Life: notification recorded; no Python executed
    Life->>Life: finish untrack, edge release, and object deallocation
    Life->>Queue: publish safe callback phase
    Queue->>Weak: claim CallbackClaimId once
    Queue->>Py: invoke retained callback and arguments
    Py-->>Queue: result or side-channel exception
    Queue->>Weak: publish completed/failed and retire claim
```

Required invariants:

1. An object-lifecycle event reaches exactly one context-owned domain regardless
   of the executing worker.
2. `ObjectGeneration` rejects delayed notification and address reuse.
3. Weak registration does not add a strong referent owner.
4. Each registry entry, field, returned alias, and callback claim has an
   independently balanced ownership ledger.
5. Registry/entry synchronization is released before arbitrary Python runs.
6. RC and GC phases finish unsafe object mutation/deallocation before callback
   dispatch.
7. A callback/finalizer claim is one-shot under reentrancy, panic, exception,
   and duplicate lifecycle notification.
8. No context may inspect or expire another context's weak entries.

## Context retirement

Retirement is ordered:

1. Move the domain from `open` to `sealing`; reject new registrations and
   acquire/finish all in-flight notification claims.
2. Quiesce the context's mutators and object-lifecycle publishers.
3. Drain registry maps into owned entry handles.
4. For each entry, atomically claim and retain any callback/function/arguments
   required by the context's normal-retirement policy. This happens before the
   registry claim can be the last owner of the entry object.
5. Mark weak entries dead and finalizer entries claimed/detached using typed
   replace operations that return old owned fields.
6. Release exactly one registry retain per drained entry. Do not manually
   destroy remaining Instance fields; externally held entries retain their own
   fields until final object destruction.
7. Run or cancel the already-owned callback claims according to normal versus
   failed/abrupt retirement policy, outside registry and GC/deallocation
   guards.
8. Retire callback claims and publish `retired`. Repeated retirement is a
   no-op with the same terminal result.

## Migration seams

1. Stage 2 introduces the context-owned domain, typed object identity, and
   scoped lookup without changing weakref surface behavior.
2. RC and GC notification routing move together; mixed TLS/context
   notification is forbidden.
3. Registry entry ownership moves before liveness sweeps or cleanup are
   redirected.
4. Raw `_target_id` address recovery is replaced by generation-checked handles.
5. Callback queuing lands before finalizers are allowed to execute outside the
   current direct path.
6. Typed field APIs resolve every unresolved current ABI ownership edge before
   retain counts are changed.
7. Legacy TLS maps are removed only after two-worker notification, callback,
   and context-retirement gates pass.

## Verification gates

- Exact inventory gate: five declarations, frozen digest, and all 30 selector
  rows stay reconciled until migration begins.
- Cross-worker notification gate: register on worker A, final-release on worker
  B, then observe one dead weakref/finalizer outcome in the same context.
- Two-context isolation gate: equal raw addresses or immediate bit patterns in
  different contexts never share entries or callbacks.
- Generation gate: delayed notification for generation N cannot retire an
  entry for generation N+1 at the reused address.
- Weakness gate: registration does not increase referent strong ownership; the
  referent can retire while the weak entry remains externally live.
- Ownership gate: entry, field, alias, snapshot, and callback-claim ledgers
  balance under create/reuse/dead/detach/callback/context-retire paths.
- Callback-phase gate: injected ref callbacks and finalizers never observe
  half-released object fields and never run under registry or GC graph guards.
- Reentrancy gate: a finalizer whose `_obj` release re-enters notification runs
  its function exactly once.
- Reset gate: retiring context A drains only A; repeated reset cannot expose A's
  weak entries in context B.
- Address-reuse gate: forced reuse cannot resolve an old weak handle to the new
  object.

## Dependency and retirement rules

- #2998 is a Stage 1 design slice under #2968; it changes no source.
- #2968 must close before Stage 2 work item #2839 can be dispatched.
- `GcDomain` supplies coordinated object-lifecycle events; it is not the owner
  of weakref registries.
- `GlobalValueDomain` publishes roots but cannot serve as the weakref liveness
  oracle.
- `ThreadDomain` quiescence precedes domain retirement.
- Completing the ownership migration does not by itself close the broader
  Python weakref semantics gap tracked by #1466.

# Representation recursion guard state topology

Issue: #2999
Parent inventory: #2968
Source revision: `2ea9059a3f`

This Stage 1 slice classifies the recursion guard shared by container
`repr`/printing paths. It defines logical-frame ownership, typed identity, and
unwind-safe cleanup without changing `src/**`.

## Bounded context

```text
ExecutionContext
└── ExecutionChild
    └── ExecutionThreadState
        └── ReprFrame
            └── entries[ReprDepth]
                └── ReprEntry

OS worker TLS
└── scoped ExecutionThreadState handle only
```

`ExecutionThreadState.ReprFrame` is the semantic owner. Representation descent
is synchronous today, but its recursion state belongs to the logical execution
frame, not to whichever OS worker happens to run it.

A TLS bridge may resolve the installed `ExecutionThreadState` while existing
ABI calls are migrated. It cannot store the recursion stack itself.

## Aggregate and values

| Type | Kind | Identity / value |
|---|---|---|
| `ReprFrame` | child aggregate | `ContextId + ExecutionThreadStateId` |
| `ReprEntry` | stack entry | typed object identity + guard token |
| `ReprObjectIdentity` | value | `ContextId + ObjectId + ObjectGeneration` |
| `ReprGuard` | RAII lexical claim | frame handle + guard token + entry depth |
| `ReprEnterResult` | value | `Entered(ReprGuard)` or `Cycle` |
| `ReprFrameState` | value | healthy or corrupted |

`ReprGuard` does not retain a mutable borrow of the stack while formatting.
It records a token and reacquires the frame briefly during `Drop`, so nested
formatting can enter the same frame.

## Frozen inventory

The one production identity has sorted newline-terminated SHA-256
`fc115bd1bd1225885e65865ee83aa3e58fca48c4731f48c2b30058499598ad04`.
There are no test-only identities.

| Current symbol | Current storage | Current role | Target owner |
|---|---|---|---|
| `IN_PROGRESS` | TLS `RefCell<Vec<usize>>` | raw container pointers in active repr descent | `ExecutionThreadState.ReprFrame` |

The accepted selector evidence contains 33 physical rows and 33 occurrences:

| Family | Occurrences |
|---|---:|
| `IN_PROGRESS` | 3 |
| `enter` | 15 |
| `leave` | 15 |
| **Total** | **33** |

The 15 caller pairs cover List, Dict, Tuple, Set, and FrozenSet in each of:

- `string_ops::value_to_string`;
- `builtins::mb_print`;
- `builtins::print_repr`.

## Current behavior

`enter(ptr)` checks whether the raw address is already anywhere in the TLS
vector. A repeat returns `false` without pushing. A new address is pushed and
returns `true`.

`leave(ptr)` pops the top entry. Equality with `ptr` is checked only by
`debug_assert_eq!`; optimized builds silently discard a mismatched top.

All 15 call sites manually follow this shape:

```text
if enter(object) == Cycle:
    emit cycle marker
else:
    recursively format children
    leave(object)
```

The cycle branch did not push, so it must not leave. In
`value_to_string` that branch returns its marker immediately. In `mb_print` and
`print_repr` it selects the marker branch and continues after the enclosing
match; it is not a Rust function early return.

## Caller matrix

| Surface | Variant | Pair count | Cycle marker | Successful cleanup |
|---|---|---:|---|---|
| `value_to_string` | List | 1 | `[...]` | manual leave after result construction |
| `value_to_string` | Dict | 1 | `{...}` | manual leave after result construction |
| `value_to_string` | Tuple | 1 | `(...)` | manual leave after result construction |
| `value_to_string` | Set | 1 | `set(...)` | manual leave after result construction |
| `value_to_string` | FrozenSet | 1 | `frozenset(...)` | manual leave after result construction |
| `mb_print` | List/Dict/Tuple/Set/FrozenSet | 5 | printed marker | manual leave at end of entered branch |
| `print_repr` | List/Tuple/Dict/Set/FrozenSet | 5 | printed marker | manual leave at end of entered branch |
| **Total** | | **15** | | |

Every entered ordinary-return path reaches its matching leave. Mamba's
language-level pending-exception side channel does not itself perform a Rust
early return or unwind in these branches, so it still reaches leave unless a
concrete helper panics.

## Current defects

### Panic/unwind leaves a stale entry

After a successful enter, a panic from a poisoned/read lock, output path, or
recursive formatting helper bypasses manual leave. The raw address remains in
TLS for all later representation calls on that worker.

The stale entry can:

- falsely report a cycle for a later call on the same live object;
- falsely report a cycle after the allocator reuses the same address;
- contaminate a nested or later execution context installed on the same OS
  worker.

Process abort has no later observable state. A caught unwind does.

### Release builds do not enforce LIFO

If a caller leaves the wrong pointer, debug builds assert. Release builds pop
the current top unconditionally. This can remove another active entry while
leaving the mismatched entry below it, corrupting future cycle decisions.

The target validates guard tokens in every profile. A mismatch marks the frame
corrupted and removes only the exact token when safe; it does not silently pop
another entry. During an already-active unwind it records failure without
panicking again.

### Raw addresses lack generation and context authority

The vector stores `usize`, so it cannot distinguish object generations at a
reused address or the same numeric address observed under another context.
Address reuse is harmless while every pair is balanced; it becomes observable
as soon as a stale entry survives.

The target identity includes context and allocation generation. A raw pointer
may be used transiently to reach the object, but it is not the stack key.

### Worker TLS is the wrong lifetime

Synchronous formatting does not currently suspend or migrate workers between
enter and leave. That limits today's migration exposure but does not make TLS
the owner.

Nested execution contexts on one worker share the current TLS stack, and
runtime reset does not clear it. `cleanup_all_runtime_state` contains no
repr-guard reset; only balanced leave or OS-thread destruction empties state.

The target frame is created and retired with its `ExecutionThreadState`.
Context/child retirement checks that no live guard remains.

## Target enter/leave contract

```mermaid
sequenceDiagram
    participant Caller as repr caller
    participant Frame as ExecutionThreadState.ReprFrame
    participant Child as recursive formatting

    Caller->>Frame: enter(ReprObjectIdentity)
    alt identity already active
        Frame-->>Caller: Cycle
        Caller->>Caller: emit marker; no guard to drop
    else first active entry
        Frame-->>Caller: ReprGuard(token, depth)
        Caller->>Child: format descendants
        Child-->>Caller: value or unwind
        Caller->>Frame: ReprGuard.drop(token)
        Frame->>Frame: validate top and retire exact entry
    end
```

Required invariants:

1. `Cycle` creates no stack entry and therefore no cleanup obligation.
2. `Entered` returns exactly one guard, and exactly that guard retires the
   entry once.
3. Normal return, Rust early return, `?`, and caught panic/unwind all run
   `Drop`.
4. Pending Mamba exceptions remain ordinary host returns and also run `Drop`.
5. LIFO mismatch is detected in debug and release builds.
6. A mismatch never silently removes another live guard.
7. Object generation and context authority are validated before cycle match.
8. No process-global lock or cross-context stack participates.

## Failure and retirement matrix

| Event | Current result | Target result |
|---|---|---|
| first enter | push raw address | push typed identity + token, return guard |
| recursive re-entry | return false, no push | return `Cycle`, no guard |
| ordinary completion | caller manually pops | guard drops |
| Mamba side-channel exception | ordinary path normally reaches manual pop | guard drops by lexical scope |
| Rust early return after enter | any new branch can omit manual pop | guard drops |
| caught panic/unwind | stale TLS address | guard drops during unwind |
| mismatched leave | debug assert; release pops wrong top | frame corruption recorded; exact token policy |
| nested context | shares worker TLS | separate `ReprFrame` |
| worker migration | unsupported while active; state stays behind | logical frame carries state if migration is introduced |
| runtime reset | no reset | child quiescence proves stack empty |
| child/context retirement | TLS outlives aggregate | rejects/records live guards, then retires frame |

## Migration seams

1. Introduce `ReprFrame`, typed identity, and tokenized `ReprGuard`.
2. Add scoped current-thread-state lookup for unchanged repr entry points.
3. Convert one complete surface at a time:
   `value_to_string`, then `print_repr`, then `mb_print`.
4. A surface may not mix manual leave and RAII for the same entry.
5. Remove `IN_PROGRESS` only after all 15 callers use guards and exact-set
   scanning finds no direct enter/leave API.

## Verification gates

- Exact inventory gate: one identity, frozen digest, and 33-row denominator.
- Caller gate: all 15 current pairs remain enumerated until migration; no new
  caller can bypass the guard API.
- Marker gate: each of the five container variants preserves its CPython-shaped
  cycle marker across all three surfaces.
- Unwind gate: inject a caught panic after every successful enter family and
  prove the next repr on the same worker is clean.
- Side-channel gate: a nested repr that sets a pending Mamba exception returns
  with an empty frame.
- Release-profile LIFO gate: intentional mismatch is detected and cannot pop a
  sibling token.
- Address-reuse gate: a stale generation token cannot match a new object at the
  same address.
- Nested-context gate: an active outer context entry is invisible to an inner
  context on the same worker.
- Isolation gate: two contexts format cyclic objects concurrently without a
  global lock or cross-context marker.
- Retirement gate: quiescent child retirement requires an empty frame and is
  idempotent.

## Dependency and retirement rules

- #2999 is a Stage 1 design slice under #2968; it changes no source.
- #2968 must close before Stage 2 work item #2839 can be dispatched.
- The context shell and scoped ABI binding precede source migration.
- `ThreadDomain` owns the child record; `ReprFrame` remains child state, not a
  process service.
- The first-pass AGY report met the exact-set and caller-matrix contract. The
  controller normalized cycle-branch wording: only `value_to_string` returns
  immediately; builtins marker branches simply never acquired a guard.

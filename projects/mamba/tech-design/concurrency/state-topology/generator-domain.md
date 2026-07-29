# Generator domain state topology

Issue: #2991  
Parent inventory: #2968  
Source revision: `534f744756`

This Stage 1 slice classifies the generator registry, stack-switch frames,
value-transfer cells, StopIteration payload, generator ID allocator, and legacy
shared capture in `runtime/generator.rs`. It defines their DDD ownership and
retirement contract without changing `src/**`.

## Bounded contexts

```text
Process
└── GeneratorIdAllocator

ExecutionContext
├── GeneratorDomain
│   └── generators[GeneratorId]
│       ├── lifecycle + exclusive execution claim
│       ├── coroutine context + mapped stack
│       ├── captured closure context
│       ├── owned Python values
│       ├── IteratorRegistration
│       └── JitModuleLease
├── IteratorDomain
├── OutputCapture
└── JitSession

ExecutionThreadState
├── generator_frames
│   ├── caller context stack
│   ├── active generator handle
│   └── resume/yield transfer frame
└── exceptions
    └── stop_iteration_value
```

`GeneratorDomain` is context-owned. The OS thread that creates a generator is
not its semantic owner. A process allocator supplies collision-free IDs but
cannot look up, resume, close, or retire a context's generator.

The currently executing resume/yield frame is child-owned. The generator
entity, its coroutine stack, and its suspension state remain context-owned
across callers.

## Aggregate and entities

`GeneratorDomain` is a sub-aggregate of `ExecutionContext`.

| Type | Kind | Identity / value |
|---|---|---|
| `GeneratorDomain` | aggregate root | `ContextId` |
| `Generator` | entity | `ContextId + GeneratorId` |
| `GeneratorExecutionClaim` | lease | `ContextId + GeneratorId + ClaimGeneration` |
| `GeneratorId` | value | opaque process-unique integer |
| `GeneratorPhase` | value | `Created`, `Suspended`, `Running`, `Completed`, `Closing`, `Retired`, `Failed` |
| `GeneratorFrame` | child state | `ExecutionThreadId + frame depth` |
| `GeneratorBodyRef` | value | symbol identity + live `JitModuleLease` |
| `OwnedMbValue` | value owner | retained Python value with exactly-once release |
| `IteratorRegistration` | lease | generator identity registered with the context iterator domain |

```text
Created -> Running -> Suspended -> Running -> Completed -> Retired
              \                         /
               -> Closing ------------/
               -> Failed ------------/
```

Only one `GeneratorExecutionClaim` may exist for a generator. The claim changes
the phase to `Running` before exposing its coroutine context. It returns the
entity to `Suspended`, `Completed`, or `Failed` before raw stack/context access
ends.

## Frozen inventory

The eight production identities have sorted newline SHA-256
`fe78b4b4bc7bd594dfc3f12f23117284917f5d97c6f874c39861739a49f71536`.
There are no test-only static declarations in this file.

| Current symbol | Current storage | Current role | Target owner / disposition |
|---|---|---|---|
| `GENERATORS` | TLS `RefCell<HashMap<u64, GenEntry>>` | generator entity registry | `ExecutionContext.GeneratorDomain` |
| `GEN_ACTIVE` | TLS `GenActive` | active frame and raw resume cache | `ExecutionThreadState.generator_frames` |
| `GEN_XFER` | TLS `GenXfer` | yield/send/completion/throw transfer | `ExecutionThreadState.generator_frames` |
| `CALLER_CTX_STACK` | TLS `CallerCtxStack` | nested caller CPU contexts | `ExecutionThreadState.generator_frames` |
| `RUNNING_GEN_STACK` | TLS `RefCell<Vec<u64>>` | nested re-entry detection | `ExecutionThreadState.generator_frames` |
| `LAST_STOP_VALUE` | TLS `Cell<u64>` | StopIteration payload bits | `ExecutionThreadState.exceptions.stop_iteration_value` |
| `NEXT_GEN_ID` | process `AtomicU64` | generator handle allocation | process `GeneratorIdAllocator` |
| `SHARED_CAPTURE` | TLS `RefCell<Option<Arc<Mutex<Vec<u8>>>>>` | legacy output buffer | remove; depend on `ExecutionContext.OutputCapture` |

The two `thread_local!` invocation rows, structs, constants, methods, and
cleanup functions are evidence, not additional state identities.

## Current behavior and defects

### TLS is mistaken for generator ownership

`mb_generator_create` inserts a `GenEntry` only into the creator OS thread's
`GENERATORS` map. An integer handle used on another OS thread queries an
unrelated map and appears unknown. Generator identity and availability
therefore change with the caller's TLS.

The target contract is not permanent creator-thread affinity. A generator is a
context entity with one exclusive execution claim. Any permitted context child
may attempt the claim; concurrent resume fails with the Python re-entry error.

### Raw context pointers outlive the borrow that produced them

`GEN_ACTIVE` caches `*mut CoroContext` and `*const CapturedCellContext` into
allocations owned by a `GenEntry`. Completion, close, release, and forced reset
have separate invalidation paths.

`mb_generator_release` invalidates matching `last_resumed_*` fields but does
not reject release of `active_id`, clear `active_ctx`, or inspect
`RUNNING_GEN_STACK`. Removing an active entry can unmap the coroutine stack
while it is executing or before a swap returns.

Neither cleanup entry point locally proves that no generator is running before
it clears entries. The current safety precondition is ambient and unenforced.

### Generator value ownership is incomplete

`GenEntry` owns Rust allocations: its boxed CPU context, mapped coroutine stack,
boxed captured-cell context, strings, vectors, and maps. It has no `Drop`
implementation that releases stored Python values.

`origin_func`, arguments, locals, yielded/sent/returned values, and transfer
cells store copied `MbValue` handles. Container insertion alone does not retain
the referenced heap object. Forced reset explicitly acknowledges that values
captured on discarded stacks leak at the `MbValue` layer.

### Transfer state encodes ownership in integers

`GEN_XFER.throw` uses:

```text
0        -> no signal
1        -> close
2 or more -> raw Box<(String, String)> pointer
```

Normal yield consumption reconstructs and drops the Box. `clear_throw_xfer`
also reconstructs and drops a stale pointer. Other transfer cells carry raw
`MbValue::to_bits()` without a type-level retain/transfer contract.

### Cleanup boundaries are inconsistent

`cleanup_all_generators` closes entries whose current phase is `Suspended`,
allowing close/finally behavior, unregisters iterator handles, and clears the
registry. It does not directly reset the seven sibling identities.

`cleanup_generator_state_for_runtime_reset` discards entries without resuming
finally blocks, clears active/transfer/caller/running/StopIteration state, and
globally resets `NEXT_GEN_ID`. It does not clear `SHARED_CAPTURE`.

Resetting the process atomic from one OS thread can reuse an ID while another
thread or context still has a live generator with that ID.

### Output capture has a second owner

`SHARED_CAPTURE` creates a separate `Arc<Mutex<Vec<u8>>>` when output capture is
active. Flush drains bytes but leaves the TLS slot installed. Neither generator
cleanup path resets it, so it may bridge unrelated later executions.

## Generator entity contract

The target entity owns:

```text
Generator {
    id: GeneratorId,
    phase: GeneratorPhase,
    claim_generation: ClaimGeneration,
    coroutine_context: CoroContext,
    coroutine_stack: CoroStack,
    body: GeneratorBodyRef,
    capture_context: CapturedCellContext,
    arguments: Vec<OwnedMbValue>,
    locals: Map<String, OwnedMbValue>,
    return_value: Option<OwnedMbValue>,
    pending_signal: Option<GeneratorSignal>,
    iterator_registration: IteratorRegistration,
}
```

`GeneratorBodyRef` holds semantic function/module identity and a live
`JitModuleLease`; a raw finalized address is a cached projection, not the
lifetime proof. The module cannot retire while any generator may execute its
body.

Every Python value stored beyond the current ABI call has an explicit strong
owner. Replacement retains the new value before publishing and releases the
old value after it becomes unreachable. Generator retirement releases each
owned value exactly once.

## Claim and resume contract

```mermaid
sequenceDiagram
    participant Child as ExecutionThreadState
    participant Domain as GeneratorDomain
    participant Generator as Generator entity
    participant Frame as child GeneratorFrame

    Child->>Domain: claim(ContextId, GeneratorId)
    Domain->>Generator: compare phase and acquire exclusive claim
    alt another child owns claim
        Generator-->>Child: ValueError(generator already executing)
    else claim acquired
        Generator-->>Frame: scoped context/stack/capture handles
        Child->>Frame: push caller context and transfer signal
        Frame->>Generator: swap and execute
        Generator->>Frame: yield, complete, or fail
        Frame->>Frame: restore parent frame through RAII
        Frame->>Domain: return phase + values; release claim
    end
```

The frame may borrow coroutine pointers only while the claim keeps the
generator entity pinned. Release, close, context cleanup, and JIT retirement
cannot remove or move its allocations while a claim exists.

Nested generator execution is represented by child frame objects rather than a
fixed ambient array. A configured depth limit may remain, but exceeding it
raises a Python/runtime error before writing outside the frame stack.

## Cross-worker transfer contract

Python semantic identity is not OS-thread-local. Serial resume from another
context child is allowed only after all of these are proven:

1. no prior child holds the execution claim;
2. the suspended CPU context and mapped stack are portable to the new worker on
   the supported platform;
3. the new child installs the same `ExecutionContext` and its own
   `ExecutionThreadState`;
4. closure, exception, output, tracing, and iterator dependencies resolve
   through explicit context/child handles rather than old TLS payloads;
5. the prior worker holds no borrowed pointer into the generator.

If the current stackful-coroutine implementation cannot satisfy that proof, the
implementation ticket must replace or adapt the representation. Silently
making generator objects permanently creator-thread-bound is not a compatible
free-threaded solution.

## Transfer values and exceptions

The integer-coded `GEN_XFER` is replaced by a typed, child-owned frame:

```text
GeneratorSignal =
    Send(OwnedMbValue)
  | Throw { type_name: String, message: String }
  | Close

GeneratorOutcome =
    Yielded(OwnedMbValue)
  | Completed(OwnedMbValue)
  | Failed(Exception)
```

Producing a signal/outcome transfers one explicit owner. Consuming it clears
the slot. Dropping an abandoned frame releases the payload automatically; no
`Box::into_raw` discriminator remains.

`LAST_STOP_VALUE` moves to the child exception state because it is the payload
of the child's StopIteration signal. Generator completion produces it, but
exception-state retirement owns its lifetime. This is the controller
normalization over the AGY report's broader `generator_frames` label.

## Iterator and output integration

Generator creation acquires one context-owned `IteratorRegistration`. Generator
retirement releases it before the generator ID becomes unreachable. Cleanup
does not separately reconstruct integer handles to search an ambient iterator
registry.

`SHARED_CAPTURE` is removed. Generator output resolves the installed
`ExecutionContext.OutputCapture`, the same owner used by non-generator output.
A child or generator frame may hold a scoped handle but never a second buffer
owner.

## Normal retirement

`ExecutionContext::quiesce` closes admission to generator creation/resume and
waits until no execution claim exists. Then `GeneratorDomain`:

1. requests close for every suspended generator;
2. runs required `finally` behavior while its JIT and closure leases are live;
3. records completion or failure;
4. releases iterator registrations;
5. releases all owned Python values;
6. drops captured contexts, coroutine contexts, and mapped stacks;
7. releases JIT module leases;
8. removes generator entities.

Retirement is idempotent. It never resets the process allocator and never
touches another context.

## Forced reset

Forced reset is permitted only after the same no-claim quiescence proof. It may
skip Python close/finally behavior only for an explicitly failed context whose
semantic outcome has already been recorded as abandoned.

Even then it releases all typed owned values and leases before dropping mapped
stacks. Failure to prove quiescence fails closed; it cannot clear the map and
hope no raw pointer remains.

## Invariants

1. Every generator belongs to exactly one `ExecutionContext.GeneratorDomain`.
2. Generator IDs are process-unique and are not reset while the process lives.
3. One generator has at most one execution claim.
4. A generator cannot retire while a claim or borrowed coroutine pointer exists.
5. A raw address or integer handle is never the identity or lifetime owner.
6. A body address is usable only under its live `JitModuleLease`.
7. Stored Python values have explicit retain/transfer/release ownership.
8. Yield, send, return, throw, and close use typed transfer states.
9. StopIteration payload lifetime is child exception-state lifetime.
10. Iterator registration lifetime is exactly generator entity lifetime.
11. Generator output has the same context-owned capture as ordinary output.
12. Normal retirement runs required close/finally behavior before dropping code
    and stacks.
13. Forced reset requires quiescence and cannot reset process identity state.
14. Cross-worker resume is either proven safe or fails explicitly; TLS location
    cannot silently become a semantic affinity rule.
15. Context cleanup cannot change another context's generators, frames, IDs,
    output, or exceptions.

## Migration order

1. Finish all #2968 owner slices and close the exact-set inventory.
2. Land #2839's scoped `ExecutionContext` and `ExecutionThreadState` handles.
3. Add `GeneratorDomain`, `GeneratorId`, lifecycle phases, and the exclusive
   execution claim without changing the public generator ABI.
4. Move `GENERATORS` and iterator registration to the context domain; keep JIT
   and closure leases explicit.
5. Replace `GEN_ACTIVE`, `CALLER_CTX_STACK`, `RUNNING_GEN_STACK`, and
   `GEN_XFER` with scoped child frame objects and typed transfer values.
6. Move StopIteration payload into child exception state and remove
   `SHARED_CAPTURE` in favor of context output capture.
7. Add explicit stored-`MbValue` ownership and generator Drop/retirement.
8. Remove global ID reset and require quiescence for both cleanup modes.
9. Prove serial cross-worker resume or replace the stackful representation
   before claiming free-threaded compatibility.

Each source migration is a separate bounded AGY ticket. The generator registry,
child frames, stored-value ownership, cleanup, and cross-worker proof are not
one implementation packet.

## Forbidden fixes

- Moving the TLS registry into one process-global generator map.
- Retaining creator-thread affinity solely because the current implementation
  uses TLS.
- Wrapping raw context pointers in a broad mutex without pin/claim lifetime.
- Clearing a running generator or unmapping its active stack.
- Keeping integer-coded owned pointers in transfer cells.
- Resetting `NEXT_GEN_ID` during context or worker cleanup.
- Treating `Vec<MbValue>` or `HashMap<String, MbValue>` as owning heap values
  without retain/release behavior.
- Recreating a generator-private output buffer beside `OutputCapture`.
- Releasing JIT code while a generator body may resume.

## Verification surface

- Inventory: exactly 8 production declarations and 0 test-only statics.
- Digest:
  `fe78b4b4bc7bd594dfc3f12f23117284917f5d97c6f874c39861739a49f71536`.
- Two contexts can use equal-shaped generator workloads without handle,
  iterator, output, exception, or cleanup crossover.
- Two children racing to resume one generator yield one execution claim and
  one `ValueError`, with no context corruption.
- Serial cross-worker resume is either proven on every supported architecture
  or explicitly remains a blocking unsupported case.
- Release and both cleanup modes refuse to remove a currently claimed
  generator; mapped stack lifetime is witnessed.
- Normal cleanup runs `finally`; forced reset requires quiescence and records
  abandonment without leaking stored Python values.
- Repeated context creation/retirement cannot reuse a live generator ID.
- Throw/close and yielded/sent/returned heap values have balanced ownership.
- Iterator registration disappears exactly when its generator retires.
- Generator and ordinary output enter the same context capture.
- JIT module retirement blocks until every dependent generator retires.
- Snapshot rule: #2991 permits no AGY repository writes and no controller
  `projects/mamba/src/**` changes.

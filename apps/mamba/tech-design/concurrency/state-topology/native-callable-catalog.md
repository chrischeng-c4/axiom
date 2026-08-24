# Native callable catalog state topology

Issue: #2982
Parent inventory: #2968
Source revision: `b04f11f2c1`

This Stage 1 slice classifies the native callable-address set, constructor-type
name map, and collision audit log. These are process-native dispatch facts
stored in OS-thread TLS, so their current storage lifetime is shorter than
their semantic lifetime.

## Bounded context

```text
Process
└── NativeCallableCatalog
    ├── callables[NativeCallableId]
    │   ├── code_address
    │   └── AbiFlags
    ├── type_dispatch[CodeAddress]
    │   └── approved NativeTypeId alias group
    └── construction evidence
        ├── accepted aliases
        └── rejected collisions

ExecutionContext
└── JitSession[ModuleId]
    └── CallableAbiRegistry
        └── live JIT addresses + module leases
```

The process catalog refines #2981's `NativeCallableAbiCatalog`: native ABI
flags and native constructor identities are sealed together. It remains
separate from the context-owned JIT overlay, whose addresses are valid only
under a live module lease.

## Frozen inventory

The three admitted identities have sorted newline SHA-256
`cd0f3827216374f2f5bbc9f57a4af2f7402b0d639105961302cbd145331ec207`.

| Current symbol | Current type | Current role | Target |
|---|---|---|---|
| `NATIVE_FUNC_ADDRS` | `RefCell<HashSet<u64>>` | selects native call ABI by raw address | immutable callable records |
| `NATIVE_TYPE_NAMES` | `RefCell<HashMap<u64, String>>` | resolves a constructor address to one class name | approved semantic type identities |
| `NATIVE_TYPE_NAME_COLLISIONS` | `RefCell<Vec<(u64, String, String)>>` | records overwritten names after collision | builder rejection/alias evidence |

The raw selector surfaces at the frozen revision are:

- 328 `NATIVE_FUNC_ADDRS.with` rows;
- 12 `NATIVE_TYPE_NAMES.with` rows;
- 133 `register_native_type_name(` rows;
- 2 `NATIVE_TYPE_NAME_COLLISIONS.with` rows.

For `NATIVE_FUNC_ADDRS.with`, seven rows are outside `runtime/stdlib`: central
registration, read, and cleanup at `runtime/module.rs:1256,1271,2106`, two
module tests at `3992,3996`, and two integration-test rows at
`runtime/tests/runtime_integration.rs:452,458`. The other 321 rows are located
under `runtime/stdlib`; that location is not a mutation proof. They include
production registrations and test reads such as registration-count snapshots.

The 12 type-name rows separate exactly into one comment/example, one central
mutation, eight production reads, and two module tests. The two collision rows
are one production mutation and one test read. The 133 helper rows are its
central definition plus 132 stdlib call sites, including
`threading_mod.rs:289`.

## Current behavior

All three declarations live in one `thread_local!` block. Stdlib registration
populates them on the registering OS thread. A worker starts with fresh empty
TLS and does not inherit the native callable/type facts even though those facts
describe process-linked Rust code.

`mb_register_native_modules` inserts native function addresses. Dynamic call
queries `NATIVE_FUNC_ADDRS` to select the native ABI. Class, builtins, and
ctypes paths query `NATIVE_TYPE_NAMES` to interpret a function address as a
constructor/type.

`register_native_type_name` detects that an address already has a different
name, appends `(address, previous_name, new_name)` to the collision vector, and
then unconditionally inserts the new name. The map is therefore last-write
wins. The vector is diagnostic evidence, not prevention.

Central cleanup clears `NATIVE_FUNC_ADDRS` but does not clear
`NATIVE_TYPE_NAMES` or `NATIVE_TYPE_NAME_COLLISIONS`. Cleanup can erase native
ABI facts on the current thread while leaving type facts and collision history
behind, and it cannot repair another thread's empty or stale TLS.

## Builder and publication

Native modules register semantic declarations into a validating builder, not
directly into an address-keyed ambient map:

```mermaid
sequenceDiagram
    participant Modules as Native modules
    participant Builder as NativeCallableCatalogBuilder
    participant Link as Linked code addresses
    participant Catalog as NativeCallableCatalog
    participant Worker as Worker context

    Modules->>Builder: declare NativeCallableId, NativeTypeId, ABI, alias policy
    Link->>Builder: bind final code address
    Builder->>Builder: validate duplicates, aliases, and collisions
    alt unapproved address reuse
        Builder--xCatalog: fail closed
    else valid catalog
        Builder->>Catalog: seal immutable records
        Catalog-->>Worker: shared read-only reference
    end
```

Construction completes before any worker executes Python. Publication is
all-or-nothing: a worker never observes a partially populated catalog. After
sealing, runtime code can query it but cannot register, replace, or clear
entries.

## Identity and collision policy

Raw addresses are lookup accelerators, not domain identities.
`NativeCallableId` and `NativeTypeId` are stable semantic identifiers derived
from reviewed native declarations.

One address may represent multiple type names only when the builder receives
an explicit reviewed alias group for a deliberately shared dispatcher. The
current test allowlist is migration evidence, not the final identity model.
Two distinct declarations that collapse to one address through identical code
folding are rejected unless that exact semantic alias relation was declared.

The builder also rejects:

- one semantic callable bound to conflicting ABI flags;
- one type identity rebound to an unrelated callable;
- a native catalog record pointing into a JIT module;
- an address-only registration without a stable semantic identity;
- production construction that includes test-only dispatchers.

Rejected collisions are returned as structured construction errors. They are
not written into a mutable runtime log followed by last-write-wins insertion.

## Query contract

Native lookup is process-wide:

```text
native_catalog.lookup(code_address)
    -> immutable NativeCallableRecord
```

Callable dispatch overlays it with #2981's current-context registry:

```text
lookup_callable(address, current_context) =
    current_context.jit_sessions.find_live(address)
    OR process.native_callable_catalog.lookup(address)
```

JIT lookup must prove `ModuleId` and a live `JitModuleLease`. Native lookup
must prove a sealed catalog record. Neither domain may copy the other's raw
addresses into its own lifetime.

## Retirement

The process catalog is not part of per-context or per-module cleanup. It lives
for the process/runtime image and is released only with that owner.

Context retirement removes JIT address records before releasing their module
leases. It never clears or rebuilds the native catalog. Worker shutdown drops
only its reference to the shared catalog.

## Invariants

1. Every native callable and type has a stable semantic identity.
2. The native catalog is fully validated before publication.
3. The published catalog is immutable and shared by all workers/contexts.
4. Worker creation requires no per-thread native re-registration.
5. A raw address is never the sole identity or lifetime proof.
6. Unapproved address reuse and ICF collisions fail catalog construction.
7. Approved aliases are explicit semantic groups, not last-write-wins names.
8. One native callable has one reviewed ABI record.
9. Test-only native records cannot enter the production catalog.
10. No process-native record points into JIT executable memory.
11. Context cleanup cannot mutate the process catalog.
12. JIT addresses remain context/module-owned and lease-bound.

## Current risks

- Workers can see empty native dispatch/type registries.
- Per-thread repeated registration can produce different collision histories.
- Last-write-wins type names hide the displaced identity.
- Cleanup clears native ABI facts but preserves related type/collision facts.
- An ICF collision is diagnosed only after registration and still overwrites.
- Raw address reuse has no stable semantic identity or lifetime boundary.
- Test-only and production static addresses share the same storage shape.

## Dependency and source order

1. Finish the remaining #2968 owner slices.
2. Close #2968 before starting #2839.
3. Introduce the #2839 execution-context shell without moving native facts
   into it.
4. Add semantic native declaration IDs and builder validation.
5. Fold #2981 native ABI flags into the sealed process catalog.
6. Publish one immutable catalog before worker creation.
7. Move dynamic-call and constructor/type queries to the catalog.
8. Remove per-thread native registration and native catalog cleanup only after
   all producers/consumers migrate.

Forbidden fixes include cloning TLS registries into each worker, rebuilding
the catalog per context, retaining last-write-wins type names, treating the
collision log as prevention, putting JIT addresses into the process catalog,
or clearing process-native facts during context retirement.

## Verification surface

- Inventory: exactly 3 admitted native-catalog rows.
- Digest:
  `cd0f3827216374f2f5bbc9f57a4af2f7402b0d639105961302cbd145331ec207`.
- Builder rejects an undeclared shared address.
- Builder accepts one explicitly declared shared-dispatcher alias group.
- Two workers observe identical immutable native callable/type records.
- Worker creation performs no registration mutation.
- Native catalog remains stable while one context and one JIT module retire.
- JIT/native overlay returns only a live context record or a sealed native
  record.
- Address-reuse negative control cannot inherit a retired JIT record or replace
  a native semantic identity.
- Cleanup has no operation that mutates the sealed native catalog.
- Snapshot rule: #2982 permits no AGY repository writes and no controller
  `apps/mamba/src/**` changes.

# Class callable resolution topology

Issue: #3019
Parent inventory: #2968
Source revision: `c0ab4037188e4fb6ce84cfe8b03ad170a86b232f`

This Stage 1 DDD slice classifies the TLS address set used by class dispatch as
historical permission to cast an integer into a function pointer and call it.

The current set records neither semantic callable identity, ABI, code owner,
JIT module generation, class-definition version, nor executable-memory
lifetime. It grows when class members are installed, is copied through worker
snapshots, and is not pruned when members or definitions are replaced.

The target creates no class-owned callable catalog. It reuses:

- the accepted immutable process `NativeCallableCatalog`;
- the accepted current-context `JitSession::CallableAbiRegistry`.

A versioned `ClassDefinition` stores typed `CallableHandle` member metadata.
A non-owning resolver returns a `CallableInvocationLease` before any unsafe
call. That lease carries the ABI and lifetime evidence required for
guard-free invocation.

No `projects/mamba/src/**` change occurs in this slice.

## Bounded context

```text
Process
└── NativeCallableCatalog
    └── NativeCallableId -> NativeCallableRecord
        ├── CodeAddress
        ├── AbiFlags
        └── ProcessImage

ExecutionContext
├── ClassDomain
│   └── ClassDefinitionRegistry
│       └── ClassDefinitionVersion
│           └── members[name] -> CallableHandle
├── JitSession[ModuleId]
│   └── CallableAbiRegistry
│       └── CallableHandle -> JitCallableRecord
│           ├── CodeAddress
│           ├── AbiFlags
│           ├── CodeGeneration
│           └── JitModuleLease
└── CallableResolver
    └── non-owning native + current-context overlay

Execution child
└── CallableInvocationLease
```

`NativeCallableCatalog` and `CallableAbiRegistry` remain the sole record
owners. `ClassDefinitionRegistry` owns member-to-handle bindings.
`CallableResolver` is a query facade and owns no duplicate record.

## Domain values

| Type | Kind | Meaning |
|---|---|---|
| `CallableHandle` | typed semantic identity | callable id plus lifetime domain |
| `NativeCallableId` | process identity | reviewed native declaration |
| `ModuleId` | JIT owner identity | one compiled executable module |
| `CodeGeneration` | version value | prevents address-reuse inheritance |
| `CodeAddress` | lookup value | entrypoint, never authority alone |
| `AbiFlags` | immutable contract | arity, kwargs, variadic, boxed return |
| `CallShape` | request value | actual receiver/positional/keyword shape |
| `ProcessImage` | lifetime authority | native linked-code lifetime |
| `JitModuleLease` | lifetime authority | keeps executable JIT memory alive |
| `ClassDefinitionVersion` | owner version | immutable member-binding generation |
| `CallableInvocationLease` | operation lease | resolved callable, ABI, and live code |

Raw `u64`, `usize`, heap pointer, tagged int, copied `MbValue`, membership
boolean, semantic handle, and invocation lease are distinct domains.

## Frozen inventory

The admitted identity is:

`projects/mamba/src/runtime/class/mod.rs::CALLABLE_REGISTRY`

The exact code-reference selector is:

`rg -n 'static CALLABLE_REGISTRY|CALLABLE_REGISTRY\.with' projects/mamba/src/runtime/class/mod.rs`

The SHA-256 of its sorted newline-terminated output is:

`a3ebdcf21f6ebc367822414cad18d6bcdb7e0e538767e11bb65dd60077ee76fe`

The selector excludes comments by construction and emits 45 physical code
rows. The frozen test boundary is `class/mod.rs:22077`.

### Production ledger

| Row | Operation | Enclosing owner |
|---:|---|---|
| `139` | TLS declaration | `thread_local!` |
| `1474` | membership read | `call_set_name_if_present` |
| `1555` | publication mutation | `mb_class_register_named_impl` |
| `1608` | membership read | `mb_class_register_named_impl` |
| `1712` | membership read | `dispatch_type_new_creation_hooks` |
| `2004` | membership read | `mb_class_update_bases` |
| `2518` | membership read | `refresh_cached_init` |
| `2797` | publication mutation | `mb_class_set_class_attr` |
| `3696` | membership read | `call_init_for_instance` |
| `3744` | membership read | `call_init_for_instance_kwargs` |
| `4104` | membership read | `instance_new_with_init_impl` |
| `5737` | membership read | `user_abc_subclasshook` |
| `9779` | membership read | `call_method_value2` |
| `9809` | membership read | `call_method_value_with_args` |
| `10016` | membership read | `invoke_descriptor_set` |
| `10078` | membership read | `invoke_descriptor_delete` |
| `10093` | membership read | `invoke_descriptor_delete` |
| `12220` | publication mutation | `class_replace_method` |
| `12978` | membership read | `invoke_binop_method` |
| `14960` | membership read | `mb_obj_getitem` |
| `15145` | membership read | `mb_obj_getitem` |
| `16271` | membership read | `mb_context_exit` |
| `16508` | membership read | `mb_call_method1` |
| `21164` | membership read | `mb_call_method` |
| `21395` | membership read | `mb_call_method` |
| `21563` | membership read | `mb_call_method` |
| `22000` | snapshot | `snapshot_thread_class_state` |
| `22020` | replace | `replace_thread_class_state` |
| `22046` | cleanup | `cleanup_all_classes` |

The partition is:

| Category | Count |
|---|---:|
| declaration | 1 |
| publication mutation | 3 |
| membership read | 22 |
| snapshot | 1 |
| replace | 1 |
| cleanup | 1 |
| **production total** | **29** |

### Test ledger

| Row | Test owner | Operation |
|---:|---|---|
| `23271` | `test_init_subclass_basic` | direct insert |
| `25241` | `test_s1_init_subclass_receives_kwargs` | direct insert |
| `25332` | `test_s4_class_getitem_subscript` | direct insert |
| `25404` | `test_s6_set_name_called_on_descriptors` | direct insert |
| `25460` | `test_s6_set_name_called_on_descriptors` | membership read |
| `26022` | `test_r10_init_subclass_without_kwargs_calls_hook` | direct insert |
| `26191` | `test_r11_class_getitem_inherited` | direct insert |
| `26771` | `metaclass_non_type_result_is_canonical_and_skips_init` | direct insert |
| `26833` | `metaclass_type_new_reuses_staged_identity_and_initializes_result` | direct insert |
| `26840` | `metaclass_type_new_reuses_staged_identity_and_initializes_result` | direct insert |
| `26921` | `test_1821_closure_handle_descriptor_set_delete_dispatches_correctly` | direct insert |
| `26987` | `test_1821_closure_handle_set_name_dispatches_correctly` | direct insert |
| `27039` | `test_1821_class_set_class_attr_registers_resolved_addr_for_closure_handle` | membership read |
| `27063` | `test_1843_closure_handle_missing_dispatches_without_wild_call` | direct insert |
| `27122` | `test_1843_closure_handle_binop_dispatches_not_silent_noop` | direct insert |
| `27166` | `test_1843_closure_handle_class_getitem_dispatches_not_silent_noop` | direct insert |

The test partition is 14 direct insertions plus two reads. Reconciliation is:

`29 production + 16 test = 45 physical rows`.

## Current aggregate

```rust
thread_local! {
    static CALLABLE_REGISTRY:
        RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
}
```

Membership means only that the raw address was inserted on this OS thread or
copied into it. The record contains no:

- callable identity;
- native/JIT discriminator;
- ABI or arity;
- module id or code generation;
- executable-memory lease;
- contributing class/member/version;
- context id;
- publication or retirement state.

The set owns Rust integers. It owns no Python reference claim and no executable
code.

## Current publication

### Initial class registration

`mb_class_register_named_impl` iterates every method before publishing the
class definition.

For each method it:

1. unwraps `classmethod`, `staticmethod`, or other descriptor form;
2. resolves the unwrapped value through `extract_registered_func_addr`;
3. inserts the nonzero resolved address;
4. independently resolves the raw method value;
5. inserts that nonzero address too.

The set records neither which representation contributed an address nor the
method/class that made it visible.

### Dynamic class attribute

`mb_class_set_class_attr` asks the dynamic callable resolver whether the value
is method-like. It inserts both descriptor-unwrapped and raw resolved forms
before retaining and publishing the class attribute.

### Method replacement

`class_replace_method` also inserts both resolved forms. This happens for
later replacement/removal flows, but the set has no per-member removal.

Replacing a method, removing it, replacing a class definition, dropping a
callable value, or unloading a future module does not revoke the historical
address. The set is additive until whole-set replace, cleanup, or TLS exit.

## Current extraction

`extract_registered_func_addr` special-cases int-tagged closure handles:

1. query `mb_closure_get_func`;
2. accept its function-tag address only when above the current threshold;
3. reject a non-none unresolved closure function;
4. otherwise accept a sufficiently large raw integer;
5. fall through to `extract_func_addr` for non-int values.

`extract_func_addr` accepts:

- function-tag payload;
- heap pointer address;
- integer payload.

This is representation extraction, not semantic resolution. A numeric
threshold is not code ownership, ABI proof, or executable-memory lifetime.

`is_exec_function_value` describes a separate interpreted callable path. Some
consumers accept an exec-function marker without set membership and dispatch
through interpreter behavior. The target must preserve that distinction
rather than putting exec-function records into a native/JIT address catalog.

## Current reader policies

| Row | Owner | Current policy |
|---:|---|---|
| `1474` | `call_set_name_if_present` | gate descriptor `__set_name__` |
| `1608` | `mb_class_register_named_impl` | cache initial `__init__` address plus historical bool |
| `1712` | `dispatch_type_new_creation_hooks` | gate base `__init_subclass__`, with separate exec-function acceptance |
| `2004` | `mb_class_update_bases` | recompute cached `__init__` after base update |
| `2518` | `refresh_cached_init` | rebuild cached constructor address/bool |
| `3696` | `call_init_for_instance` | gate positional `__init__` |
| `3744` | `call_init_for_instance_kwargs` | gate kwargs `__init__` |
| `4104` | `instance_new_with_init_impl` | gate custom construction/init branch |
| `5737` | `user_abc_subclasshook` | gate user `__subclasshook__` |
| `9779` | `call_method_value2` | gate fixed two-value helper |
| `9809` | `call_method_value_with_args` | gate spread-argument helper |
| `10016` | `invoke_descriptor_set` | gate descriptor `__set__` |
| `10078` | `invoke_descriptor_delete` | gate property `fdel` |
| `10093` | `invoke_descriptor_delete` | gate general descriptor `__delete__` |
| `12978` | `invoke_binop_method` | gate binary/comparison dunder |
| `14960` | `mb_obj_getitem` | gate type `__class_getitem__` |
| `15145` | `mb_obj_getitem` | gate instance `__getitem__` |
| `16271` | `mb_context_exit` | gate context-manager `__exit__` |
| `16508` | `mb_call_method1` | gate single-argument method helper |
| `21164` | `mb_call_method` | gate first class/instance lookup branch and descriptor-kind shaping |
| `21395` | `mb_call_method` | gate super/MRO lookup branch and descriptor-kind shaping |
| `21563` | `mb_call_method` | gate general method branch plus variadic/kwargs shaping |

The cached `(address, is_registered)` pair does not revalidate current class
version, callable owner, code generation, or executable-memory lifetime when
later read.

## Current unsafe invocation

Many reader paths cast an admitted address directly to one fixed
`extern "C"` signature. `call_registered_method_addr` centralizes a subset:

```text
args.len() 0..8
  -> select an arity-specific extern "C" function type
  -> unsafe transmute(address)
  -> invoke raw address
  -> only after return, query ABI/native TLS flags
  -> decide whether to rebox the raw result
```

The post-call query combines:

- `is_boxed_return_func`;
- `is_variadic_func`;
- `is_kwargs_func`;
- `is_native_func`.

Those flags affect result normalization after the unsafe call. They do not
pre-call validate that the selected signature matches the callee.

Other `mb_call_method` branches inspect variadic/kwargs flags to shape inputs,
but the facts still live in separate TLS sets and remain raw-address keyed.
No single typed lookup returns callable identity, call shape, ABI, and code
lifetime together.

## Current lifetime and transport

The source proves:

- set entries are raw integers;
- registration is additive;
- member removal does not revoke entries;
- snapshot clones the set without a code/module lease;
- replace raw-overwrites the current TLS set;
- cleanup conditionally clears only the current TLS set;
- TLS exit drops only the Rust set.

The admitted surface does not prove a current executable-module unload/reuse
event. The bounded hazard is architectural: if code is dropped or a virtual
address is reused, this set has no identity, generation, or lease with which
to reject its stale member.

Copying a set into a worker changes address visibility without establishing
code lifetime. Clearing one OS thread neither retires another thread's copy
nor distinguishes process-native entries from context-owned JIT entries.

## Existing test boundary

The 16 admitted rows cover local registration and dispatch for:

- `__init_subclass__`;
- kwargs forwarding;
- `__class_getitem__`;
- descriptor `__set_name__`, set, and delete;
- closure-backed methods;
- metaclass result/init behavior;
- binary and missing-method dispatch.

They do not prove:

- two-context isolation;
- a live JIT module lease;
- class-definition-version lifetime;
- worker lookup without a copied set;
- address-reuse rejection;
- native/JIT collision handling;
- module retirement ordering;
- context retirement.

## Target class member binding

Each immutable `ClassDefinitionVersion` maps a member name to an owned member
value and, when callable, one typed `CallableHandle`.

The handle names the semantic callable and lifetime domain. It does not copy a
raw address or historical boolean into the definition.

Replacing/removing a method:

1. builds a new immutable definition version;
2. detaches the old handle from future current-version lookup at commit;
3. leaves an already leased old `Arc<ClassDefinition>` valid;
4. retires the old callable/module authority only after the last definition
   and invocation lease drops.

Visibility and lifetime are separate. Removal prevents new current-version
lookup but cannot invalidate in-flight work.

## Target resolver

The non-owning `CallableResolver` accepts:

- current `ContextHandle`;
- `CallableHandle`;
- owning `ClassDefinitionVersion`;
- requested `CallShape`.

Resolution is:

```text
if handle is process-native:
    NativeCallableCatalog.lookup(handle)
else:
    current_context.JitSession.CallableAbiRegistry.lookup(handle)

validate semantic id
validate native/JIT domain
validate context and ModuleId
validate CodeGeneration
validate requested arity/kwargs/variadic shape
acquire ProcessImage or JitModuleLease
return CallableInvocationLease
```

The returned lease contains:

- semantic handle;
- code address;
- ABI flags and validated call shape;
- native owner or JIT module/generation;
- lifetime authority.

Raw address, pointer, integer, display name, set membership, or copied
`MbValue` cannot obtain a lease.

## Target invocation

Catalog/session guards protect only lookup and lease acquisition. They end
before:

- Python argument preparation;
- callback or descriptor work;
- unsafe native/JIT invocation;
- exception construction;
- result boxing;
- release/deallocation.

`CallableInvocationLease` pins callable metadata and code through the raw call
and result normalization. Dropping it after the operation may unblock module
retirement.

A call-shape mismatch fails before `transmute`. No unsafe call is attempted
and a typed Python error is produced outside owner guards.

## Target worker and retirement behavior

Worker snapshots omit callable authorization sets. Same-context children query
the immutable process catalog plus their shared current-context JIT registry.
Independent contexts cannot resolve one another's JIT handles.

JIT module retirement:

1. marks the module retiring;
2. rejects new handle resolution;
3. detaches address and symbol records;
4. waits for definition and invocation leases;
5. releases executable memory only after quiescence;
6. retires the module generation.

A recycled address belongs to a new `(ModuleId, CodeGeneration)` and inherits
no old authorization.

Context retirement drains class-definition versions and active invocation
leases before releasing JIT sessions/modules. It never clears or rebuilds the
process-native catalog.

The native catalog retires only with `ProcessImage`.

## Target invariants

1. `CALLABLE_REGISTRY` membership is never lifetime authority.
2. Raw integers and pointers never grant execution.
3. Native records belong only to `NativeCallableCatalog`.
4. JIT records belong only to the current context's `CallableAbiRegistry`.
5. `CallableResolver` owns no records.
6. No class-owned duplicate catalog exists.
7. Every callable class member stores a typed `CallableHandle`.
8. Class definitions store no raw address authorization boolean.
9. Each handle distinguishes process-native from JIT lifetime.
10. Each JIT handle names `ModuleId` and `CodeGeneration`.
11. Closure handles resolve to semantic callables before authorization.
12. Exec-function dispatch remains a separate interpreted domain.
13. Current `ContextHandle` participates in every JIT resolution.
14. Wrong-context lookup fails closed.
15. Requested call shape is validated before unsafe invocation.
16. ABI flags are returned by the same typed resolution.
17. An invocation lease contains the resolved code address.
18. An invocation lease contains code-lifetime authority.
19. Catalog/session guards end before unsafe invocation.
20. No owner guard spans Python callbacks.
21. No owner guard spans result boxing or deallocation.
22. Definition replacement publishes a new immutable version.
23. New lookup cannot see a removed old-version handle.
24. Existing old-version leases remain valid.
25. Member removal never revokes in-flight calls.
26. Module retirement rejects new resolutions first.
27. Address records detach before executable memory release.
28. Module retirement waits for definition leases.
29. Module retirement waits for invocation leases.
30. Address reuse creates a new generation.
31. Reused addresses inherit no ABI or permission.
32. Native/JIT address collisions fail closed.
33. Worker snapshots contain no callable authorization set.
34. Same-context children share typed JIT lookup.
35. Independent contexts isolate JIT callables.
36. Cleanup cannot erase the process-native catalog.
37. Context retirement drains JIT records before memory.
38. Process-native records live with `ProcessImage`.
39. Unsafe arity dispatch never precedes call-shape validation.
40. Planned tests are not executed evidence.

## Forbidden changes

1. Do not replace the TLS set with another class-owned set/map.
2. Do not make the resolver a duplicate owner.
3. Do not store raw `u64` authorization in `ClassDefinition`.
4. Do not preserve cached `is_registered` as authority.
5. Do not treat extraction as semantic resolution.
6. Do not merge exec-function markers into native/JIT catalogs.
7. Do not put JIT addresses in the process catalog.
8. Do not copy callable authorization through worker snapshots.
9. Do not permit cross-context JIT lookup.
10. Do not release executable memory with active leases.
11. Do not reuse an address without a new typed generation.
12. Do not hold owner locks across unsafe calls.
13. Do not invalidate old leased definition versions on rebinding.
14. Do not clear the process-native catalog during context cleanup.

## Planned implementation paths

| Path | Planned responsibility |
|---|---|
| `projects/mamba/src/runtime/execution_context.rs` | context authority, resolver access, retirement |
| `projects/mamba/src/runtime/class/mod.rs` | typed member handles and class dispatch migration |
| `projects/mamba/src/runtime/module.rs` | ABI lookup and native/JIT owner split |
| `projects/mamba/src/runtime/closure.rs` | closure-handle semantic resolution and module relation |
| `projects/mamba/src/codegen/cranelift/jit.rs` | module id, generation, executable-memory lease |
| `projects/mamba/src/runtime/mod.rs` | compatibility binding and resolver surface |

## Planned focused tests

1. descriptor-unwrapped and raw representations resolve to one semantic handle;
2. initial class publication binds typed callable members;
3. dynamic class attribute publication binds a typed handle;
4. replacement detaches old current lookup;
5. leased old definition remains callable until its final lease;
6. member removal cannot authorize new lookup;
7. closure handle resolves the underlying callable;
8. exec-function dispatch remains distinct;
9. every admitted dispatch family uses typed resolution;
10. arity mismatch fails before unsafe call;
11. kwargs/variadic shape mismatch fails before unsafe call;
12. boxed-return normalization uses the resolved ABI;
13. native process-catalog lookup;
14. current-context JIT lookup returns module authority;
15. wrong-context JIT lookup denial;
16. same-context worker lookup without snapshot copy;
17. independent-context isolation;
18. module retirement waits for active invocation lease;
19. module retirement waits for leased old class definition;
20. recycled address does not inherit authorization;
21. native/JIT collision fails closed;
22. cleanup leaves native catalog unchanged;
23. no owner guard spans unsafe invocation;
24. context retirement drains JIT records before executable memory.

These tests are planned and were not executed by this measure-only slice.

## Acceptance boundary

This slice is complete when:

- all 45 code references reconcile as 29 production plus 16 test;
- all three mutation paths and 22 reader policies are explicit;
- extraction is separated from identity, ABI, and lifetime;
- unsafe call/reboxing order is explicit;
- accepted native/JIT owners remain sole owners;
- class versions store typed handles and preserve old leased versions;
- invocation is guard-free under a live typed lease;
- worker, module, address-reuse, and context retirement rules fail closed;
- implementation remains a later AGY-owned `projects/mamba/src/**` slice.

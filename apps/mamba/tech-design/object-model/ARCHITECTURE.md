# object-model — architecture (as-is, 2026-07-15)

Scope: class registry, MRO, attribute/method dispatch, descriptors, slots, super, metaclass/PEP 487,
isinstance, dict key/value model. Source: `src/runtime/class/mod.rs` (~26k lines), `src/runtime/class/descriptors.rs`,
`src/runtime/dict_ops.rs`, lowering side in `src/lower/hir_to_mir.rs`. Fix TDs in this dir are NOT restated — see cross-refs.

## Responsibilities

- Runtime class identity: registry-keyed `MbClass` records (name/display_name/bases/mro/methods/class_attrs/metaclass/cached_init).
- Attribute & method dispatch for every value shape: `mb_getattr` cascade, descriptor protocol, bound/unbound method synthesis.
- Class-statement lifecycle: lowering-time `PendingClassRegistration` → runtime define → base update → slots → attrs → metaclass/PEP 487 hooks → decorators.
- Instance construction: metaclass `__call__` routing, custom `__new__`, cached `__init__` dispatch, ABC/Protocol instantiation rejection.
- `super()` proxy semantics and MRO-after-skip lookup, incl. builtin/type/object terminal arms.
- Python dict key semantics (`DictKey` hash/eq domains) — consumed by every stdlib module that probes `ObjData::Dict`.

## Key structures & invariants

| Structure | Where | Invariant |
|---|---|---|
| `MbClass` | mod.rs:69 | `.name` = REGISTRY key; `.display_name` = Python `__name__`. Never feed display names into registry lookups (see `identity-and-keys.md` §Domain 1). |
| `CLASS_REGISTRY` | mod.rs:124 (thread_local `HashMap<String, MbClass>`) | Classes flow through the runtime **as bare class-name strings**; a `type` object is an `Instance{class_name:"type"}` with `__name__` field. |
| Runtime keys | hir_to_mir.rs:945 `__mamba_user_class__:<file>:<symId>:<Name>`; per-execution alias `<declkey>@<serial>` via `mb_class_runtime_key` (mod.rs:1366) + `CLASS_RUNTIME_KEY_ALIASES` (mod.rs:126); dynamic `type()` classes get `__mamba_dynamic_class__:<name>@<n>` (`fresh_dynamic_class_runtime_key`, mod.rs:1359, never aliased) | key ← `type_object_registry_key` (builtins/type_objects.rs:265) for type objects, raw `Instance.class_name` for instances; display ← `class_display_name` (mod.rs:1377), errors ONLY. |
| `PendingClassRegistration` | hir_to_mir.rs:48 | One record per lowered `class` stmt; emitted at the textual `ClassDefPlaceholder` (hir_to_mir.rs:5670). |
| `USER_CLASSES` | mod.rs:135 | Marks Python-created classes; AttributeError-on-miss only fires when the ENTIRE MRO is user-defined or `object` (mod.rs:~8717) — native ancestors keep lenient `None`. |
| `CALLABLE_REGISTRY` | mod.rs:139 | Only registered addresses are dispatched. Populate/probe via `extract_registered_func_addr` (mod.rs:2938), NEVER `extract_func_addr` (mod.rs:2922) for method values — see `class-construction.md` §Instance construction. |
| `SLOTS_REGISTRY` vs `OWN_SLOTS_REGISTRY` | mod.rs:146/152 | merged instance LAYOUT vs declared `__slots__` VALUE — distinct concepts (#1523); `DICT_SUPPRESSED` (mod.rs:155) when slots declared without `'__dict__'`. |
| `METHOD_CACHE` + `METHOD_CACHE_GEN` | mod.rs:183/187 | Invalidated (cleared) on any class registration / class-attr mutation; `SIMPLE_CLASS_CACHE` (mod.rs:192) skips descriptor checks in `mb_setattr`. |
| `creation_hooks_pending` (PEP 487) | MbClass field, mod.rs:91 | Namespace hasn't crossed `type.__new__`; hooks fire once via `dispatch_type_new_creation_hooks` (mod.rs:1631): `__set_name__` (class attrs, alpha order) THEN base `__init_subclass__` with class kwargs. |
| `METACLASS_DEFINITION_STACK` | mod.rs:175 | While a custom metaclass `__new__` runs, the first matching `type.__new__` claims the staged class identity (`claim_staged_type_new_target` mod.rs:1393, consumed type_objects.rs:520) instead of allocating a duplicate. |
| `MbClass.cached_init: (addr, is_registered)` | mod.rs:94 | Resolved at register/update-bases time; `is_registered=false` ⇒ construction *silently skips* `__init__` — the #1594 failure mode. |
| `DictKey` | dict_ops.rs:506 | `Str`/`StrCodepoints`/`Instance` hash in the Python-semantic domain (`Hash` impl dict_ops.rs:714, `dict_string_hash_value` :623), NOT Rust `str` hash. Probe only via `dict_get_exact_str` (dict_ops.rs:860) — see `identity-and-keys.md` §Domain 2. `Instance`/`Tuple`/`FrozenSet` keys retain their ptr (Clone/Drop rc-managed). |
| `ThreadClassState` | mod.rs:104, snapshot/replace mod.rs:21345/21379 | All ~15 thread_locals swap atomically for thread spawn; caches reset on swap. |
| Marker instance classes | `"__super__"`, `"__unbound_method__"` (mod.rs:5236), `"method"` (:5252), `"__bound_native_method__"` (:5444), `"__classmethod__"/"__staticmethod__"/"__property__"`, `"member_descriptor"` (descriptors.rs:81) | Synthetic `Instance`s carrying dispatch metadata in fields; every consumer must special-case these names before treating `class_name` as a real class. |

## Control flow

**Class statement** (lowering: hir_to_mir.rs:5670 `ClassDefPlaceholder`):
1. `emit_pending_class_registrations` → `emit_class_registration` (hir_to_mir.rs:5813): `mb_class_runtime_key` (mint `@serial` alias) → `mb_user_type_obj` → `mb_class_define_multi[_named]` (mod.rs:1796/1814 → `mb_class_define_multi_impl` :1832 → `mb_class_register_user_named` :1504 → `mb_class_register_named_impl` :1540: register callables, `compute_mro`, insert MbClass with `creation_hooks_pending=true`, cache `__init__`, install abc mixins).
2. `emit_runtime_class_bases_for` → `mb_class_update_bases` (mod.rs:1919): re-resolve bases, recompute MRO, re-cache `__init__`, set `__orig_bases__`/`__parameters__`.
3. `emit_class_slots_for` (drains `pending_class_slots`, hir_to_mir.rs:1626) → `mb_register_slots` (mod.rs:11385) — AFTER base update, see `class-construction.md` §Registration pipeline.
4. class body stmts → `emit_class_attrs_for` → finalizers → `mb_class_finalize_definition*` (mod.rs:2651+ → `finalize_class_definition` :2589): resolve explicit/inherited metaclass; if custom, push `METACLASS_DEFINITION_STACK`, call meta `__new__` (namespace dict, mod.rs:~2530), then meta `__init__`; else dispatch PEP 487 hooks directly.
5. dataclass field recording → class decorators (bottom-up) → bind name.

**Instance construction** `mb_instance_new_with_init[_kwargs]` (mod.rs:3256/3262) → `instance_new_with_init_impl` (mod.rs:3872):
1. Reject abstract/Protocol instantiation (5 checks); enum class call intercept.
2. Metaclass `__call__` if present (unless `skip_metaclass` — `instance_new_default` mod.rs:3273 is the `super().__call__()`/`type.__call__` bypass); closure-unwrap via `extract_registered_func_addr` (#1525).
3. `custom_new_method` (mod.rs:3472) or allocate `Instance`; seed builtin payloads (int/str/list/... subclass hidden `__mamba_*_value__` fields, mod.rs:60-65) only when no custom `__new__` (#968).
4. `call_init_for_instance[_kwargs]` (mod.rs:3517/3549): `cached_init` fast path, else MRO `lookup_method` + registry check.

**Attribute read** `mb_getattr` (mod.rs:6643) → `mb_getattr_impl` (:6651), in order:
1. `__getattribute__` override on Instances (skipped for `mb_object_getattribute_lookup` re-entry :6647).
2. `__class__`/`__dict__`/enum specials; `__super__`/`__unbound_method__` marker dispatch; native `Box<T>` wrapper getters; #2097 dict/module fast path.
3. Function/generator/coroutine/handle metadata registries (`__name__`, `__code__`, `__defaults__`, ...).
4. Instance path (mod.rs:~8566): data descriptor (`is_data_descriptor` :9668 → `invoke_descriptor_get` :9735) → instance fields → non-data descriptor / bound-method synthesis (`make_bound_method` / `make_bound_native_method`) → explicit-None class attr (`class_attr_lookup`) → builtin-payload & special-cased native method arms (Thread/deque/...) → `__getattr__` dunder → AttributeError iff pure-user MRO or exception class, else `None`.

**Method lookup**: `lookup_method` (mod.rs:11988) = METHOD_CACHE → MRO walk over methods then class_attrs; `lookup_method_including_none` (:12026) for `__hash__ = None` sentinels. MRO: `compute_mro` (mod.rs:12208) — duplicate-base TypeError first, linear chain for ≤1 base, else `c3_merge` (:12295); inconsistent hierarchy sets catchable TypeError + trivial-MRO fallback; always appends `"object"`.

**super()**: `mb_super`/`mb_super_checked`/`mb_super_no_args_error`/`mb_super_argcount_error` (mod.rs:13745-13809) build a `"__super__"` proxy → `mb_super_getattr` (:13843): resolve instance class (type-object receiver ⇒ class_context), `super_dispatch_class` metaclass hop (:13828), `lookup_method_after` (:14006, MRO after skip_class), descriptor unwrap + bind by kind, then `super_builtin_native_method` (:13940) terminal arms, then `SUPER_MISSING_INIT_METHOD` no-op for `__init__`, else AttributeError. Semantics + red lines: `class-construction.md` §Super machinery + error semantics.

**isinstance** `mb_isinstance` (mod.rs:12722): Union/tuple/union-type recursion → arg-2 type check → numbers-tower rank → resolve target name (`resolve_class_name` :21244 / func-pointer class map) → ModuleType/non-runtime_checkable-Protocol TypeError → metaclass `__instancecheck__` (falls back to nominal on exception/None) → PathLike/ABC virtual+structural (`user_abc_issubclass`) → nominal MRO containment + payload/enum/protocol structural arms.

## Known hazards

| WHAT | WHY dangerous |
|---|---|
| Display name fed to `CLASS_REGISTRY` | Silent miss for user classes (namespaced keys) — dispatch falls back / wrong cls; family + accessors: `identity-and-keys.md` §Domain 1. |
| Raw `&str` `.get()` on `IndexMap<DictKey,_>` | Wrong hash domain since #1028 ⇒ present keys read as absent, no error; may pass unit tests via accidental collisions: `identity-and-keys.md` §Domain 2. |
| `extract_func_addr` on a method value | Closure handles (any method using `__class__`/zero-arg `super()`, unconditional since #1379) yield garbage pseudo-addresses ⇒ `is_registered=false` ⇒ `__init__`/metaclass `__call__`/`__init_subclass__` silently skipped: `class-construction.md` §Instance construction. |
| `mb_register_slots` before `mb_class_update_bases` | Pre-update MRO has no bases ⇒ inherited slots dropped: `class-construction.md` §Registration pipeline. |
| Adding `__init__` to the super `__new__`-at-type/object arm | Regresses no-op `super().__init__()` idiom — explicit RED LINE in `class-construction.md` §Super machinery + error semantics and comment mod.rs:13960. |
| `ObjData::Str` doubles as class reference AND plain str value (#1009, mod.rs:8750) | A str whose CONTENT equals a registered class name can shadow into class-attr lookup; str-method dispatch must win for genuine str methods. |
| `SLOTS_REGISTRY` read as `cls.__slots__` value | Layout ≠ declared tuple (#1523); use `OWN_SLOTS_REGISTRY`/`class_slots_value` (mod.rs:12098). |
| Miss `invalidate_method_cache()` after registry mutation | Stale METHOD_CACHE serves shadowed methods; every write path (register/update_bases/set_class_attr) must bump. |
| `cleanup_all_classes` (mod.rs:21406) clears without releasing | Deliberate leak (refcount imbalance makes release unsafe); don't "fix" by adding releases. |
| Cached `is_registered=false` short-circuits `call_init_for_instance` returning `true` | Construction reports success with `__init__` never run — silent-skip class of bugs; probe with attribute round-trips, not exit codes. |
| Marker class names (`"__super__"` etc.) reaching generic paths | e.g. `super().__class__` must yield `super`, not the marker (#1581, mod.rs:6687/13849); new consumers must branch on markers first. |

## Extension points

- New builtin-type method surface → `builtin_type_method_names_by_name` table + `make_bound_native_method` call in `mb_call_method`; getattr synthesis arms follow automatically (mod.rs:~8620 payload arm).
- New descriptor kind → `DescriptorKind` + `unwrap_descriptor_method` (descriptors.rs:39) + bind arms in `mb_super_getattr`/instance step 3.
- New class-definition metadata (kwargs, slots, abstractmethods...) → paired `mb_class_set_*` extern (registered in symbols.rs) + thread_local registry + `ThreadClassState` field (snapshot/replace/cleanup — all three, mod.rs:21345+).
- New per-class registry → MUST be added to `ThreadClassState` and `cleanup_all_classes` or thread spawn/test isolation breaks.
- New isinstance semantics (ABCs, protocols) → dedicated arm in `mb_isinstance` BEFORE the nominal check; structural registries follow `RUNTIME_CHECKABLE_PROTOCOLS`/`ABC_VIRTUAL_SUBCLASSES` pattern.
- New lowering-ordered class artifact → queue-and-drain on the Lowerer (`pending_class_slots` model, hir_to_mir.rs:1626) drained at `ClassDefPlaceholder` in the documented order; never emit inline at registration.
- New stdlib dict probes → only through `dict_ops` helpers (`dict_get_exact_str`, `BorrowedDictStrKey`).

## EC surface

Per `external-contracts/README.md` (positive contracts — must run & byte-match python3.12):

- `tests/cpython/_regression/core/class_system/` — registry, construction, attribute semantics.
- `tests/cpython/_regression/core/mro_super/` — MRO + super dispatch and error paths.
- `tests/cpython/_regression/core/language/` (esp. `metaclasses/{behavior,surface}.py`) — metaclass/PEP 487.
- `tests/cpython/_regression/core/descriptors/` + `tests/cpython/behavior/core/descr/` — descriptor protocol.
- Adjacent proof (dict-key domain regressions land in owning stdlib dims): `behavior/std-libs/{logging,xml_etree,socket}/...` per `identity-and-keys.md` §Domain 2.
- Gate: `cargo test -p mamba --release --test conformance` (~3 min); per-fix evidence = before/after readings.

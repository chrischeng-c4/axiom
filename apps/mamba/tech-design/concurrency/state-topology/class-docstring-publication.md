# Class docstring publication state topology

Issue: #3049
Parent inventory: #2968
Behavior counterpart: #3056
Source revision: `18e8d58b4fcc57032dfaeae5301bce1a8222eec0`

This Stage 1 DDD slice classifies the class-body docstring registry in
`runtime/class/mod.rs`. It extends the class definition registry topology
without authorizing source migration before #2839.

The slice is unusual among the #2968 inventory: measuring it proved that the
registry is written correctly and then read by almost nobody. Three of the four
`__doc__` surfaces are defective today, so the destination design must fix
reachability, not merely relocate storage.

## Aggregate boundary

`ExecutionContext` remains the aggregate root. A class docstring is not a
registry of its own; it is a direct immutable field of the leased class
definition version, consistent with the ownership frozen in #3024 / #3026 /
#3033.

```text
ExecutionContext
└── RuntimeRegistrySet
    └── classes
        └── definitions[ClassRuntimeKey]
            └── Arc<ClassDefinition>
                └── doc: Option<String>
```

`ClassRuntimeKey` is the `@`-serial-suffixed key minted by
`fresh_class_runtime_key`, not a display name and not a declaration key.

Deliberately excluded destinations: a second TLS map, a per-field lease nested
inside the definition, and any aggregate shared with function docstrings.
`FUNC_DOCS` keeps its own destination, `FunctionIdentityMetadata.doc` (#2972).

## Frozen inventory

Selector:

```bash
rg -n 'static CLASS_DOCS|CLASS_DOCS\.with' apps/mamba/src/runtime/class/mod.rs
```

Denominator: 4 production rows, 0 test rows. The test module boundary is
`class/mod.rs:22077`. Selector digest
`2ad334b261611c0055a2dd4e32797de2783828eb3ba5602b4b9a199886570fe2`.

| Line | Role | Enclosing scope | Destination |
|---|---|---|---|
| `12396` | TLS declaration | module-level `thread_local!` | retired |
| `12403` | write | `pub fn mb_class_set_doc` (`12401`) | `ClassDefinition.doc` at publication |
| `12409` | read | `pub(crate) fn class_doc` (`12408`) | lease read of `ClassDefinition.doc` |
| `12413` | clear | `pub(crate) fn cleanup_class_docs` (`12412`) | version retirement |

Values are Rust `String`; keys are Rust `String`. No `MbValue` is stored, so no
`retain_if_ptr` / `release_if_ptr` participates.

## Publication path

The writer key is established by the instruction that defines it, not by the
name of the vreg holding it.

1. `lower_class` (`ast_to_hir.rs:5607`) captures the first bare string literal
   of the class body at `6205` and assigns it to `HirClass.doc` at `6238`
   (`hir/mod.rs:198`).
2. `prepare_hir_to_mir_with_symbols_src` (`hir_to_mir.rs:716`) pushes
   `(runtime_key, class_sym, doc)` into `pending_class_docs` at `1023`.
3. `emit_class_registration` (`hir_to_mir.rs:5879`) defines `name_vreg` at
   `5880`-`5888` as the destination of `mb_class_runtime_key` applied to
   `emit_str_const(&registration.runtime_key)`. `display_name_vreg` (`5889`)
   goes only to `mb_user_type_obj`.
4. The drain at `6248`-`6262` destructures `(_, _, doc)`, discarding the pushed
   `runtime_key` tuple element, and emits
   `mb_class_set_doc(name_vreg, doc_vreg)` at `6259`.
5. `mb_class_runtime_key` (`class/mod.rs:1367`) mints through
   `fresh_class_runtime_key` (`1351`) as `format!("{identity}@{serial}")` over
   the process-global atomic `NEXT_CLASS_RUNTIME_KEY` (`217`), and records
   `declaration_key -> runtime_key` in `CLASS_RUNTIME_KEY_ALIASES`.

Consequences that the destination design must preserve or repair:

- The stored key is always `@`-serial-suffixed. Every execution of a class
  statement mints a fresh serial, so the compiled path never reuses a key. The
  current failure mode is unbounded retention with no per-class removal API,
  not same-key staleness.
- Emission is conditional on `cls.doc.is_some()`. A class with no docstring
  emits no writer and therefore never clears a prior entry.
- `mb_class_define_multi_named` (`6238`) is the only class registration emit
  site, so the drain is not duplicated across registration paths.

## Reader reachability

This is the load-bearing finding of the slice. The fault is reachability, not
key-space disagreement.

A compiled user class value is the type object returned by `mb_user_type_obj`
(`builtins/type_objects.rs:342`) through `make_type_object_with_display_name`
(`145`), whose payload is `ObjData::Instance { class_name: "type", fields }`.
`extract_str` (`async_rt.rs:1218`) returns `Some` only for `ObjData::Str`.

| Surface | Path | Result |
|---|---|---|
| `inspect.getdoc(Cls)` | `d_getdoc` (`inspect_mod.rs:1061`) gates its class branch on `extract_str(obj)` at `1086`, which yields `None` for a type object; control falls to the instance branch where `inst_class_name` (`59`) returns the literal `"type"` and `class_doc("type")` misses | `None` |
| `inspect.getdoc(instance)` | `inst_class_name` yields the `@`-serial runtime key, which matches the written key | correct |
| `Cls.__doc__` | `make_type_object_with_display_name` eagerly inserts `__doc__` = `format!("{display_name} type object.")` at `186`; `type_surface_attr_value` (`class/mod.rs:402`) returns that stored field before any fallback | placeholder, never consults `CLASS_DOCS` |
| `instance.__doc__` | `__doc__` is filtered out of instance-side inheritance by the own-field / dunder filter lists (`class/mod.rs:731`, `7850`, `11202`) | `None` |

`CLASS_DOCS` therefore has exactly one live reader: `inspect_mod.rs:1099`. The
two class-keyed callers at `1087` and `1091`, including the MRO ancestor walk,
are dead code for compiled user classes.

The missing resolution hop already exists in the runtime:
`make_type_object_with_display_name` records
`registry_keys: val.to_bits() -> registry_key` in `TYPE_OBJECT_STATE`
(`type_objects.rs:130`, `204`, `273`), and `resolve_class_name`
(`class/mod.rs:21875`) wraps that lookup. Aligning writer and reader key spaces
would not, on its own, restore `inspect.getdoc(Cls)`.

Measured against the CPython oracle on `target/debug/mamba` for a class `Foo`
with docstring `"Foo docstring."` and instance `f`:

| Expression | CPython | mamba | Status |
|---|---|---|---|
| `inspect.getdoc(Foo)` | `'Foo docstring.'` | `None` | defect, #3056 |
| `inspect.getdoc(f)` | `'Foo docstring.'` | `'Foo docstring.'` | correct |
| `Foo.__doc__` | `'Foo docstring.'` | `'Foo type object.'` | defect, #3056 |
| `f.__doc__` | `'Foo docstring.'` | `None` | defect, #3056 |

A second writer of class `__doc__` therefore exists outside the registry, and
it shadows the real docstring with a synthesized placeholder. The same
placeholder text is also synthesized as a fallback at `class/mod.rs:423`, which
remains correct for builtin types that genuinely have no docstring. Dynamic
`type(name, bases, dict)` (`type_objects.rs:570`-`600`) is a third path: it
stores `__doc__` straight into the namespace fields and bypasses `CLASS_DOCS`
entirely.

## Context ownership

`ThreadClassState` (`class/mod.rs:104`-`120`) has exactly 15 fields:
`class_registry`, `class_runtime_key_aliases`, `user_classes`,
`callable_registry`, `slots_registry`, `own_slots_registry`, `dict_suppressed`,
`kwargs_registry`, `classcell_required`, `classcell_symbol_ids`,
`classcell_values`, `namedtuple_base_shapes`, `runtime_checkable_protocols`,
`abc_virtual_subclasses`, `user_abc_own_abstract`. It carries neither
`CLASS_DOCS` nor `ABSTRACT_METHODS`.

The asymmetry is therefore: `CLASS_DOCS` is absent from
`snapshot_thread_class_state` (`21981`) and `replace_thread_class_state`
(`22015`), but present in `cleanup_all_classes` (`22042`), which calls
`cleanup_class_docs()` at `22072` between the `ABSTRACT_METHODS` clear
(`22071`) and the `ABC_VIRTUAL_SUBCLASSES` clear (`22073`).

Two-context scenario on one thread: context 1 publishes `Foo@1 -> "Doc A"`;
`replace_thread_class_state` installs context 2; the entry survives, and
context 2 can read `Foo@1`'s docstring even though it never published it.
Because keys are process-unique, this is a leak of readability and retention,
not a misattribution: no context can observe a *different* class's docstring
under a key it minted itself.

`cleanup_class_docs` uses `try_borrow_mut`, so a failed borrow silently skips
the clear and reports nothing. No Python callback (`__new__`, `__init__`,
`__init_subclass__`, `__set_name__`) runs inside either the writer or the
reader borrow, so the current borrows are not reentrancy hazards; that property
must be preserved, not assumed, once the value moves behind a lease.

## Invariants

1. A class docstring is a direct immutable field of one `Arc<ClassDefinition>`
   version, never a separate registry or a nested per-field lease.
2. The docstring is keyed by `ClassRuntimeKey`, never by display name or
   declaration key.
3. A class published without a docstring stores `None`, matching CPython, and
   never inherits the docstring of a prior version under any key.
4. Attribute rebinding on a module or class never mutates, transfers, or
   inherits a published docstring.
5. Class and function docstrings remain in separate aggregates.
6. Every `__doc__` surface — `inspect.getdoc(Cls)`, `inspect.getdoc(instance)`,
   `Cls.__doc__`, `instance.__doc__` — resolves through the same published
   field. No surface may synthesize a placeholder for a class that published a
   docstring.
7. Builtin types with no docstring keep the existing `"... type object."`
   placeholder; the placeholder must not shadow a published docstring.
8. Dynamic `type(name, bases, dict)` publishes through the same path as the
   compiled lowering path.
9. Stored docstrings are Rust-owned `String`; no Python refcounting
   participates.
10. Docstring retirement happens exactly when the last `Arc<ClassDefinition>`
    reference for that version drops, not at a broad TLS reset.
11. Context retirement leaves no readable docstring behind for a successor
    context on the same thread.
12. Two contexts that independently define the same display name observe
    isolated docstrings.
13. Readers must resolve a class value to its `ClassRuntimeKey` explicitly; no
    reader may depend on a class value being an `ObjData::Str`.
14. Neither the writer nor the reader may execute a Python callback while
    holding the definition lease.

## Forbidden changes

1. Do not introduce a second TLS or context-level side table for class
   docstrings.
2. Do not key class docstring storage by display name to make the existing
   reader work.
3. Do not merge `FUNC_DOCS` and class docstring storage into one table.
4. Do not widen `extract_str` to accept type objects as a way to reach the
   class branch of `d_getdoc`.
5. Do not delete the dead class branch at `inspect_mod.rs:1087`-`1091` without
   replacing it with a key-resolving branch; the MRO ancestor walk is required
   behavior, merely unreachable.
6. Do not carry a prior version's docstring forward on re-publication.
7. Do not remove the builtin `"... type object."` placeholder outright.
8. Do not let dynamic `type(...)` keep its own private `__doc__` field path.
9. Do not add `CLASS_DOCS` to `ThreadClassState` as a compatibility shim.
10. Do not retain best-effort silent-failure cleanup at domain teardown.
11. Do not invoke Python refcounting on stored Rust docstrings.
12. Do not migrate storage and repair reachability in one ticket; #3049 freezes
    ownership, #3056 repairs behavior.

## Dependency and source order

1. Finish the remaining #2968 Stage 1 inventory slices.
2. Implement #2839 Stage 2 aggregate shell and scoped restoring binding.
3. Repair the `__doc__` reachability defects under #3056 against the key space
   frozen here, without inventing a third key space.
4. Migrate class definition ownership, carrying `doc` as a direct field, in a
   Stage 4 ticket.
5. Prove version-scoped retirement before claiming freedom from stale entries.

## Verification surface

- Inventory count: 4 production rows, 0 test rows.
- Selector digest:
  `2ad334b261611c0055a2dd4e32797de2783828eb3ba5602b4b9a199886570fe2`.
- Exact source: `apps/mamba/src/runtime/class/mod.rs`.
- Behavior evidence: the four-row oracle table above, reproducible with
  `inspect.getdoc` and `__doc__` on a docstringed class and one instance.
- Snapshot rule: #3049 permits no repository changes from AGY and no
  `apps/mamba/src/**` changes from the controller.

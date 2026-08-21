# import-system — architecture (as-is, 2026-07-15)

Runtime import machinery: the `import`-statement lowering targets, the module
cache (`sys.modules` analog), the file finder+loader, circular-import handling,
and the vendored-stdlib resolution path. Native module *shells* and the
vendored-vs-native-vs-sentinel decision are owned by `../stdlib/ARCHITECTURE.md`
and `../stdlib/module-hazards.md` — referenced here, not restated. Scope:
`src/runtime/module.rs` (3.8k L, the whole subsystem), `stdlib/vendor_lib.rs`,
`stdlib/importlib_mod.rs`, `builtins/import_hook.rs`.

## Responsibilities

- Lower every `import` statement form to a runtime call: `mb_import`, `mb_import_from`, `mb_import_star`, `mb_import_relative(_star)`, `mb_module_getattr(_relative)`, `mb_dunder_import` (`__import__`, `import_hook.rs:9`).
- Own `MODULES` — the thread-local import cache and `sys.modules` peer; sentinel pre-caching for circular-import safety.
- Finder: `find_module` — resolve a dotted name to a `.py` path across package `__path__` / `SCRIPT_DIR` / live `sys.path` / `SEARCH_PATHS`.
- Loader: `compile_and_exec_module` — run the full compile pipeline on a `.py` and reify its globals as module attrs.
- Vendored-stdlib **precedence** (native > script-dir > vendored > user), search-path init from `PYTHONPATH`, and hard-blocked roots (gevent/greenlet).
- Expose an `importlib` Python surface (`importlib_mod.rs`) — mostly stubs (see hazards).

## Key structures & invariants

| Structure | Where | Invariant |
|---|---|---|
| `MbModule` | `module.rs:18` | `{name,file,attrs,is_package,cached_value}`; `cached_value` is the ONE heap dict handed to user code — `import X; import X as Y ⇒ X is Y` (CPython Rule 2, `module_to_value:1846`) |
| `MODULES` (thread_local) | `module.rs:29` | the import cache; native pre-registration (`mb_module_register`) wins forever — `find_module` is never consulted for a cached name |
| `MODULE_VALUE_PTRS` | `module.rs:35` | every materialized module dict ptr is marked, else `isinstance(m, types.ModuleType)`/`type(m)` can't tell it from a plain dict (`is_module_value:1894`) |
| `SEARCH_PATHS` | `module.rs:37` | defaults to `["."]`; vendored tree inserted at index 0 (`mb_insert_search_path`), so it outranks `PYTHONPATH` appends |
| `SCRIPT_DIR` / `CURRENT_MODULE_PACKAGE` | `module.rs:92,98` | script dir probed first in `find_module`; package anchors relative imports (`resolve_relative_module_name:708`) |
| `MODULE_JIT_BACKENDS` | `module.rs:85` | compiled-module JIT backends are pushed and NEVER dropped (except `cleanup_all_modules`) — dropping dangles module fn ptrs |
| sentinel pre-cache | `module.rs:305-317` | an EMPTY `MbModule` is inserted under `name` BEFORE `find_module`/exec, so re-entrant import returns the partial module instead of recursing |
| `VENDORED_MODULES` | `vendor_lib.rs:44` | `(name, include_str!)` → materialized once to a content-hashed temp dir at `SEARCH_PATHS[0]`; NO native stub may exist for a vendored name (`vendor_lib.rs:303` test enforces) |

## Control flow — `mb_import(name)` (`module.rs:262`)

1. `extract_str` → `blocked_import_root` (gevent/greenlet) → raise `ImportError` with migration-guide URL (`module.rs:376,14`).
2. Cache probe: `name ∈ MODULES`? If file-backed AND dropped from `sys.modules` → evict (reload path, `module.rs:281`). Else `module_to_value_and_cache` + `update_sys_modules` → return (same ptr).
3. `ensure_parent_packages` — recursively `mb_import` each `a`, `a.b` prefix of `a.b.c` (`module.rs:394`).
4. Insert EMPTY sentinel under `name` (circular guard).
5. `find_module(name)` (`module.rs:1739`): parent-package `__path__` probe (dotted only) → `SCRIPT_DIR` → live `sys.path` (`live_sys_path_paths`) → `SEARCH_PATHS`; `probe_module_path` tries `base/parts.py` then `base/parts/__init__.py`.
6a. Found → set `file`/`is_package` on sentinel → `compile_and_exec_module` (§loader).
6b. Not found → remove sentinel → recover user `sys.modules["x"]=m` injection (`lookup_sys_modules:473`) → else raise `ModuleNotFoundError`.
7. `module_to_value_and_cache` + `update_sys_modules` → return.

### Loader — `compile_and_exec_module(path,name)` (`module.rs:1326`)

read → `parser::parse` (hardcoded `FileId(9999)`) → `pep695::desugar_module` → `TypeChecker::check_module` (proceed on type errors, dynamic like CPython) → `lower_module`(HIR) → `lower_hir_to_mir` → **save+clear** caller globals → `push_active_module_name` → set `__file__`/`__package__`/`__name__` globals → set `CURRENT_MODULE_PACKAGE` → `push_active_module_sym_ids` (§hazards) → JIT `codegen` → `main_fn()` runs the body (`:1493`) → snapshot globals → NaN-box + name-map to `attrs` (skip dunders except `__name__`/`__doc__`/`__all__`, `:1508`) → add user func ptrs + registered top-level classes + `__path__` (packages) → store `attrs` into `MODULES` (`:1574`) → keep JIT backend alive → `pop_active_module_*` → restore caller globals + package.

## CPython-parity semantics (the contract)

- **Object identity**: repeated import returns the identical `cached_value` ptr (Rule 2). `mb_import_from` returns a tuple of attrs; `mb_import_star` honors `__all__` else public (non-`_`) names AND binds them into caller globals (`:609`).
- **`__name__` of an imported module is its own dotted name, never `"__main__"`** (`:1460`) — a plain `import` must not trip the module's `if __name__=='__main__'` guard (regression #945).
- **`sys.modules` is authoritative but sync is one-way**: `MODULES` syncs INTO `sys.modules` on every load (`update_sys_modules`); the reverse (user injection) is recovered ONLY on a cache MISS (`:342`). Mutating `sys.modules` for an already-cached name has no effect.
- **Packages**: `__init__.py` ⇒ `is_package`, `__path__=[dir]`, `__package__==__name__`; a plain submodule's `__package__` is its parent (`:1435`).
- **Circular imports (DIVERGENCE)**: the sentinel is EMPTY and `attrs` are stored *atomically after* the body finishes (`:1574`). A module imported mid-body sees an empty namespace — CPython binds each name incrementally as the importee executes. mamba's contract is "partial = empty until complete," not CPython's incremental partial. (`test_circular_import_sentinel:3551`.)
- **`__import__`** honors only `name`; `globals`/`locals`/`fromlist`/`level` are dropped (`import_hook.rs:3`), so `__import__('os.path')`/relative `__import__` diverge.

## Known hazards

- **Circular reads see empty module** — attrs stored atomically post-exec (`:1574`); a co-recursive importer reads nothing, unlike CPython's incremental binding.
- **Active-module stack imbalance** — loader pushes module name + sym-ids; any dispatcher that runs user code without matching push/pop misresolves scoped names. See `../closures/capture-and-scope.md`, `../stdlib/module-hazards.md`.
- **Sym-id collision across modules (#983)** — every module's `SymbolTable` restarts numbering from the same baseline, so a nested submodule's raw global id can numerically collide with the parent's not-yet-written slot; `push_active_module_sym_ids` (`:1482`) is the guard — removing it flakily corrupts attrs by HashMap order.
- **`MODULE_JIT_BACKENDS` leak-forever** — required for fn-ptr validity; unbounded growth under repeated dynamic import; only `cleanup_all_modules` frees it.
- **Vendored shadowing** — any native stub for a vendored name pre-seeds `MODULES` and permanently shadows `py_src/*.py` (`vendor_lib.rs:229`). See `../stdlib/ARCHITECTURE.md`.
- **Vendoring flips are compiler-gated** — a from-source sweep regression is a core-compiler bug, not a `.py` fix; `vendor_lib.rs:52-224` chronicles 6 revert rounds (#943/#945/#953/#976/#1007/#1008/#1018…). The string-content/type-name collision (`'arg'` vs `ast.arg`) still bites vendored bodies.
- **`importlib` is largely stubs** — `reload` returns the module unchanged (`importlib_mod.rs:99`); `find_spec` fabricates a 4-key dict only for already-registered names, all fields `None` (`:116`); `util/abc/machinery/resources` are empty/None dicts; `invalidate_caches` no-op.
- **`FileId(9999)` hardcoded** for every imported module (`:1345`) — spans/diagnostics across distinct modules share one file id.
- **Reload is eviction-only** — `mb_import` re-loads a file-backed module iff it was dropped from `sys.modules`; there is no in-place reload.
- **Blocked roots** — `import gevent`/`greenlet` (and submodules) hard-raise `ImportError` (`:376`); no shim.

## Extension points

| Add… | Do this |
|---|---|
| Vendored pure-Python module | one `(name, include_str!("py_src/<name>.py"))` in `VENDORED_MODULES` (`vendor_lib.rs:44`); NEVER also register a native stub; may need a curated wall in `types/stdlib_sigs.rs` (see `../stdlib/ARCHITECTURE.md`) |
| New `import` statement form | new `mb_import_*` fn in `module.rs` + lower it in `src/lower/hir_to_mir.rs` |
| Search-path source | `mb_insert_search_path`(index) / `mb_add_search_path`(append) / `PYTHONPATH` via `mb_init_search_paths:1055` |
| Blocked module | append to `blocked_import_root` list (`module.rs:377`) |
| Real `importlib` behavior | fill the stubs in `importlib_mod.rs` (currently surface-only) |
| Native shell / sentinel / kit | owned by `../stdlib/ARCHITECTURE.md` (extension table) |

## EC surface

- **`std-libs/importlib`** (`config/manifests/std-libs/importlib.toml`, 19 cases): `surface` ×9 (callable/submodule probes), `errors` ×4 (`import_module_missing/empty_name/relative_no_package_raises`, `reload_non_module_raises`), `behavior` ×4 (`import_module_imports_real_module`, `find_spec_missing_returns_none`, `module_not_found_error_is_import_error`), `real_world` ×1 (`plugin_registry_dynamic_import`).
- **core lang** (`config/manifests/core/cpython321_core_lang.toml`): `lang_importlib_*_silent`, `lang_module_exec_del_silent`, `lang_asyncio_importlib_ast_dis_gettext_silent`.
- **File-based machinery** (SCRIPT_DIR / PYTHONPATH / packages / circular / relative / 4-layer precedence) is proven by `module.rs` `#[test]` units — `test_circular_import_sentinel:3551`, `test_vendored_lib_precedence_native_script_vendored_user:3617`, PYTHONPATH tests `:3036+` — NOT conformance fixtures: the one-file fixture harness (`../../tests/harness/cpython/conventions/FIXTURE-LAYOUT.md`) can't express multi-file packages. `vendor_lib.rs:288,303` unit tests prove materialization + no-shadow.
- Gate: `cargo test -p mamba --release --test conformance` (~3 min) for `importlib`; `cargo test -p mamba module::` for the file-based units.

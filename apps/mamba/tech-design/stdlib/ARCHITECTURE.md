# stdlib — architecture (as-is, 2026-07-15)

First TD in this domain (was "unassigned future context" in `tech-design/README.md`).
Scope: `src/runtime/stdlib/` — 211 files, ~213k LOC, per-module native shells.

## Responsibilities

- Native Rust shells for ~200 CPython stdlib + 3rd-party module surfaces (`*_mod.rs`), registered at startup into the thread-local module registry (`module.rs:MODULES`).
- Vendored pure-Python CPython 3.12 sources (`py_src/`, 9 modules) that mamba's own compiler executes instead of a native shell (`vendor_lib.rs`).
- `test.support` infra surface for the CPython test corpus: wide stub field (`test_mod.rs`) + measured behavior augmentation (`support_mod.rs`).
- Import-survival sentinels: one-attr probe markers (`thirdparty_shells_mod.rs`) and callable-sentinel shims (`pydantic_core_mod.rs`, `grpclib_mod.rs` #1514, ~30 files in the "identity-stable sentinel" family, #1496 pattern).
- Bridging native kits (`mambalibs.*`) via `#[distributed_slice(MAMBA_MODULES)]` FFI symbols into the same registry (`module.rs:1120`).
- Shared sub-layers: `compressed_file.rs` (one streaming file impl for bz2/lzma/gzip, #1480), `enum_class.rs` (class-body enum member machinery, #1448).

## Key structures & invariants

| Structure | Where | Invariant |
|---|---|---|
| `MODULES` | `module.rs:29` (thread_local) | module = dict of attrs; pre-registration wins the import cache forever (see precedence below) |
| `MODULE_VALUE_PTRS` | `module.rs:35` | every materialized module dict ptr is marked, else `isinstance(m, types.ModuleType)` can't tell it from a plain dict |
| `NATIVE_FUNC_ADDRS` | `module.rs:42` | every dispatcher addr handed to `MbValue::from_func` MUST be inserted, else `mb_call*` uses the wrong calling convention. ABI: `extern "C" fn(*const MbValue, usize) -> MbValue` |
| `NATIVE_TYPE_NAMES` | `module.rs:49` | addr → class name, consumed by `class/mod.rs:1417` isinstance; addr-keyed ⇒ each registered class-like dispatcher needs a **unique machine address** |
| `register_native_type_name` + `NATIVE_TYPE_NAME_COLLISIONS` | `module.rs:132,106` | the only collision-visible path (#962); never insert into `NATIVE_TYPE_NAMES` directly |
| `icf_guard!` | `module.rs:167` | first statement of every trivial dispatcher body registered by address; per-callsite fingerprint makes it LLVM-fold-immune |
| `SHELL_POOL` (460 slots) | `long_tail3_mod.rs:52` (also long_tail2/4, cgi, ctypes, dev_tools, http mods) | `dispatch_class_shell` is a MARKER address only; `register_with` (`long_tail3_mod.rs:344`) swaps every marker/class entry for a fresh pool slot. Pool fns fingerprint on `stringify!($name)`, NOT `line!()/column!()` (identical spans inside one macro repetition, `long_tail3_mod.rs:41-51`) |
| `VENDORED_MODULES` | `vendor_lib.rs:44` | `(name, include_str!)` pairs → materialized ONCE to a content-hashed temp dir, inserted at `SEARCH_PATHS[0]`. NO native stub may exist for a vendored name (enforced: `vendor_lib.rs:test_first_batch_shell_registers_do_not_shadow_vendored_source`) |
| `merge_register` | `support_mod.rs:399` | merge attrs + re-register through `mb_module_register` so the parent's materialized snapshot rebuilds; mutating `MODULES[leaf].attrs` alone leaves `from test.support import X` resolving the stale value |
| `mb_module_register` | `module.rs:188` | sets `__name__`/`__module__` on every func attr (addr-keyed FUNC_NAMES) and wires dotted names into parent packages (`propagate_submodule_to_parents`) |

Import precedence (`vendor_lib.rs:21-35` doc): **native `MODULES` cache → `SCRIPT_DIR` → vendored tree → `"."` + `PYTHONPATH`**.

## Control flow

1. Startup: driver → `stdlib::register_stdlib()` (`mod.rs:223`) — single-threaded, ordered.
2. `vendor_lib::register()` runs FIRST (search-path slot 0, before `mb_init_search_paths` reads `PYTHONPATH`).
3. ~200 `<mod>::register()` calls follow. Order-sensitive: `urllib_error_mod` BEFORE `http_mod` (umbrella snapshot, `mod.rs:270`); `support_mod` AFTER `test_mod` (in-place mutation, `mod.rs:333`); `http_cookies_mod`/`http_cookiejar_mod` AFTER `http_mod` (dotted-name override, `mod.rs:351`).
4. `import X` at runtime: `mb_import` (`module.rs:262`) — cache hit returns `module_to_value` namespace dict; miss falls to `find_module` (SCRIPT_DIR → SEARCH_PATHS) → compile+exec the `.py` (vendored or user) via `MODULE_JIT_BACKENDS`.
5. Call of a module attr: dynamic call checks `NATIVE_FUNC_ADDRS` → native ABI; the dispatcher decodes its own args/kwargs (raw slice + trailing kwargs-dict convention; DictKey probes).
6. `isinstance(x, NativeClass)`: class value is a fn pointer → `NATIVE_TYPE_NAMES` addr lookup → name compare (`class/mod.rs:1417, 7427`).
7. Class-ful shells: `mb_class_register` (`class/mod.rs:1496`) with method dispatcher maps (92 stdlib files); instances flow through the object-model domain.
8. `mambalibs.*` kits: binding crates force-linked in `main.rs:34-46` (`native-modules` feature) → registry loop (`module.rs:1120`) inserts FFI fn ptrs per name via `mb_module_register`.
9. Dispatchers that execute user code off-thread push/pop `ACTIVE_MODULE_NAMES` (`asyncio_mod.rs:900` ModuleGuard; `closure.rs:213 current_active_module_name`).

## Known hazards

- **ICF address folding (#954/#962/#1040)** — LLVM merges trivially-identical dispatcher bodies onto ONE address. Addr-keyed `NATIVE_TYPE_NAMES`/FUNC_NAMES then resolve the wrong class/`__name__` nondeterministically (HashMap order). Any new trivial dispatcher without `icf_guard!`/SHELL_POOL reintroduces it.
- **DictKey hash-domain mismatch** — probing `IndexMap<DictKey,_>` with a Rust-str-hashed key silently returns `None` for present keys; 4+ independent stdlib hits (socket kwargs, xml `Element.keys()`…). See `object-model/identity-and-keys.md` §Domain 2.
- **Runtime-key aliasing** — `type.__name__` (display) ≠ `CLASS_REGISTRY` key (`__mamba_user_class__:<file>:<line>:<Name>@<n>`); dispatchers reading `__name__` miss the registry (total_ordering `functools_mod.rs:2282`, singledispatch, logging). See `object-model/identity-and-keys.md` §Domain 1.
- **Active-module stack imbalance** — a dispatcher that runs user code (threads, exec, pickle-by-name `pickle_mod.rs:1637`) without push/pop of `ACTIVE_MODULE_NAMES` misresolves scoped symbols/globals for everything after it.
- **TESTFN CWD droppings** — `support_mod.rs:475` sets `os_helper.TESTFN = "@mamba_test_<pid>"` (relative); fixtures create files in the harness CWD = repo root; stale droppings were git-tracked (commit `90252aa09`). New fixtures must not use TESTFN (`FIXTURE-LAYOUT.md` hard constraints).
- **String-content/type-name collision** — a str whose CONTENT equals a registered native class name (`"arg"` vs `ast.arg`) mis-dispatches methods on that value; zero-import repro `'arg'.startswith('-')` (`vendor_lib.rs:152-161`). Every `register_native_type_name` widens the collision surface.
- **Vendored shadowing** — registering any native stub for a vendored name pre-seeds `MODULES` and permanently shadows `py_src/*.py` (`vendor_lib.rs:229-236`).
- **Sentinel shims lie** — the #1496-family shims return fresh empty dicts from every call; imports and surface probes pass while behavior is fake. Passing `surface/` fixtures for these modules proves nothing about `behavior/`.
- **Vendoring flips are compiler-gated** — `vendor_lib.rs:52-224` chronicles 6 rounds of flip→revert; every regression traced to a core-compiler bug (#943/#945/#953/#976/#977/#1007/#1008/#1018…), never to the vendored source. A from-source sweep regression = file a core bug, don't patch the `.py`.
- **`__enter__` retain** — CM dispatchers returning non-self must retain the returned value (TemporaryDirectory). See `memory/object-lifetime.md` §With-protocol refcount contract.
- **augment vs merge** — `augment_module` mutates the leaf only; use `support_mod.rs:merge_register` when the name is imported through a parent snapshot.

## Extension points

| Add… | Do this |
|---|---|
| Native module shell | new `<name>_mod.rs` + `pub mod` in `mod.rs:1-216` + `register()` call in `register_stdlib()` (respect dotted-override ordering) |
| Vendored pure-Python module | one `(name, include_str!("py_src/<name>.py"))` entry in `vendor_lib.rs:44`; no other call sites; NEVER also register a stub; may need a curated wall in `types/stdlib_sigs.rs` (colorsys `h: float`, getopt `args` precedents) |
| Long-tail import stub | `long_tail*_mod.rs:register_with(name, classes, dispatchers, consts…)` — SHELL_POOL slots drawn automatically |
| 3rd-party probe marker | append to `thirdparty_shells_mod.rs:register()` name list |
| Class-like constructor | `mb_class_register` + `register_native_type_name` (never raw `NATIVE_TYPE_NAMES` insert) + `icf_guard!` in the dispatcher body |
| `test.support` gap | `support_mod.rs` `augment_module`/`merge_register` — additive only, after `test_mod::register()` |
| Native kit (`mambalibs.*`) | new binding crate with `#[distributed_slice(MAMBA_MODULES)]` + force-link in `main.rs` and `driver/mod.rs:596` |

## EC surface

Per `external-contracts/README.md` (domain map row "stdlib (per module)"):

- **Positive (must run & byte-match python3.12 oracle):** `behavior/std-libs/<mod>`, `errors/std-libs/<mod>`, `real_world/std-libs/<mod>`; plus `surface/std-libs/<mod>` presence probes (the only dimension sentinel shims genuinely satisfy).
- **Negative:** none owned here — `type/…/*_wrong.py` walls (incl. curated `stdlib_sigs.rs` walls that vendored modules depend on) belong to type-system.
- Dotted modules use `_` in the `lib` key/dir (`xml_etree`); one case = one file with a `[tool.mamba]` record (`tests/harness/cpython/conventions/FIXTURE-LAYOUT.md`).
- Gate: `cargo test -p mamba --release --test conformance` (~3 min). Vendoring flips and shell rewrites are proven by per-fixture-dir before/after sweep readings, not unit tests.

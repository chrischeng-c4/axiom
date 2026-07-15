# mamba external-contracts

mamba's external contract IS CPython 3.12 behavior. The contract artifacts
already exist and are executable — this directory anchors them; it does not
duplicate them as prose.

## Global gates

| Contract | Artifact | Gate command |
|---|---|---|
| C1 functional parity | `tests/cpython/**` corpus (46k+; oracle = python3.12 byte-diff; xfail = acknowledged gaps) | `cargo test -p mamba --release --test conformance` (~3 min) |
| C2 performance | `tests/harness/cpython/config/perf/pins/*.toml` (external CPU/RSS, getrusage / `/usr/bin/time`, ratio asserted — `perf_pin.rs` D5.2) | perf pin sweep |

## Domain contract map (DDD — mirrors tech-design/README context map)

**Frontend — source → typed HIR**

| Domain | Positive contract (must run & match oracle) | Negative contract (must reject) |
|---|---|---|
| [frontend](frontend.md) | `_regression/core/{grammar,parse}`, `behavior/core/grammar` | — (SyntaxErrors proven positively via `errors/pep/*`) |
| [name-resolution](name-resolution.md) | `pep/572` (shared w/ closures), `_regression/core/{scope_resolution,scope_modifiers,comprehension_scope}` | `undefined_var.py` — the one `# RUN: typecheck` pipeline fixture this domain owns |
| [type-system](type-system.md) | — (dimension rule: every non-`type/` fixture corpus-wide must RUN) | `type/` dimension `*_wrong.py` walls; weakening a wall is a contract breach |

**Backend — HIR → machine**

| Domain | Positive contract (must run & match oracle) | Negative contract (must reject) |
|---|---|---|
| [codegen](codegen.md) | `behavior/core/sys_settrace` (trace-event emission) | — (must never reject a non-`type/` fixture) |
| [memory](memory.md) | `behavior`/`surface`/`std-libs/gc`, `_regression/core/stability` soaks; absence of hang/SIGTRAP corpus-wide | `type/std-libs/gc` |

**Runtime — object model & data types**

| Domain | Positive contract (must run & match oracle) | Negative contract (must reject) |
|---|---|---|
| [object-model](object-model.md) | `_regression/core/{class_system,mro_super,language,descriptors}`, `behavior/core/descr` | — |
| [calling-convention](calling-convention.md) | `core/args_kwargs_binding`, `std-libs/{call,userfunctions,keywordonlyarg,positional_only_arg,getargs,functools,extcall}` | `type/core/arg_annotation`, `type/std-libs/functools` |
| [numbers](numbers.md) | `std-libs/{int,float,complex,decimal,fractions,numbers,cmath,statistics,random}` | `type/std-libs/<mod>` — one per owned stdlib module |
| [strings](strings.md) | `std-libs/{string,unicode,codecs,struct,...}` methods/unicode/codecs/struct families | `type/std-libs/{string,unicode*,codecs,struct,encodings*}` |
| [collections](collections.md) | `behavior/core/{compare,dictcomps,listcomps,setcomps,container_float_roundtrip,value_equality_inference,iter}` | `type/core/{container_element,operator_dispatch}` |
| [iterators](iterators.md) | generators/`yield_from`, iterator protocol + itertools, async coroutines, `sys_settrace` (shared w/ codegen) | `type/std-libs/{itertools,_asyncio,unittest_async_case,asyncio_*}` |
| [exceptions](exceptions.md) | `_regression/core/exception*`, `behavior/core/exceptions` | — |
| [closures](closures.md) | `pep/572` (shared w/ name-resolution), capture-introspection fixtures | — (adjacent `inspect.getclosurevars` wall belongs to stdlib+type-system) |

**Runtime — services**

| Domain | Positive contract (must run & match oracle) | Negative contract (must reject) |
|---|---|---|
| [concurrency](concurrency.md) | dedicated `concurrency/` dimension (atomicity/safety/primitives), `std-libs/{asyncio,threading,multiprocessing,concurrent_futures}` | `type/std-libs/{asyncio_*,multiprocessing_*,concurrent_futures_*,threading}` |
| [import-system](import-system.md) | `behavior/core/import`, `_regression/core/{circular_import,import_cache,imports,relative_import,star_import}`, `std-libs/importlib` | `type/std-libs/{_frozen_importlib*,importlib*,zipimport}` |
| [stdlib](stdlib.md) (per module) | `behavior/std-libs/<mod>`, `errors/std-libs/<mod>`, `real_world/std-libs/<mod>` | — |

## Rules

- Dimension rule: fixtures under behavior/errors/real_world/surface/_regression/
  security/concurrency MUST run — a compile reject there is a type-system
  false positive by definition. Only `type/` fixtures are walls.
- A new WI's EC = name the fixture set in its TD's Verification contract.
  Write new fixtures only when the surface has no coverage.
- Gate readings are the only progress signal; per-fix evidence = before/after
  readings on the issue.

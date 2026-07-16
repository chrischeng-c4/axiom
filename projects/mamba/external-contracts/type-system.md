# type-system — external contract (as-is, 2026-07-15)

Domain map: `tech-design/type-system/ARCHITECTURE.md` (EC surface section). Verdict law: `HARNESS.md` step 3
(`runner.rs:run_type_strict` — STRICT_TYPE_OK compile/runtime vs MAMBA_TYPE_LEAKED). Global gates: `README.md` (same dir).

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

type-system owns almost no positive fixture dirs; its positive half is the corpus-wide **dimension rule**: every
non-`type/` fixture must RUN — a compile reject there is a type-system false positive by definition.

| Surface | Live count | Notes |
|---|---|---|
| All non-`type/` dimensions (behavior/errors/real_world/surface/_regression/security/concurrency/pep/perf) | 37,851 .py (46,620 corpus − 8,769 walls) | implied positive contract; each over-wall is a breach |
| `_regression/core/typecheck/` | 2 .py (0 xfail) | `# RUN: typecheck` pipeline fixtures pinning diagnostics (type_mismatch, undefined_var) |

## Negative contract — what must be REJECTED

`type/` is THE wall dimension: any path component `== "type"` ⇒ strict-type rules; weakening a wall is a contract breach.
All 8,769 fixtures carry `# mamba-strict-type:`; 8,698 are `*_wrong.py` (71 use `*_rejects_*`-style names, e.g. core/param_types).

| Guard surface | Live .py | xfail | Notes |
|---|---|---|---|
| `type/core/` | 63 | 0 | 7 axes: param_types 39, arg_annotation 12, operator_dispatch 3, return_annotation 3, container_element 2, method_resolution 2, var_annotation 2 |
| `type/builtin-libs/builtins/` | 449 | 1 | per-builtin argument walls |
| `type/std-libs/` | 8,257 (570 libs) | 1,199 | generated stdlib-spec walls; top xfail libs: tkinter 143, turtle 109, typing 74, tkinter_ttk 65 |
| **Total walls** | **8,769** | **1,200** | 7,569 live-enforcing today (xfail = full skip) |

Non-fixture negative proof: `src/driver/tests/strict_type_dynamic_ingress.rs` (dynamic routes reject pre-body,
catchable TypeError) + `runtime/builtins/mod.rs:7039-7147` unit contracts.

## Known contract gaps

- **1,200 xfail'd walls (13.7%) enforce nothing**; a seeded 15-fixture re-run shows ~93% now correctly reject — est. ~1,100 walls of recoverable coverage; 2 std-libs surfaces fully xfailed (asyncio_graph, unittest_async_case). tracked: #1768.
- **Guard mass inverted**: 99.3% of negative fixtures guard ONE wall family (generated stdlib-spec arg walls, `check_expr.rs:4643`); ~57% of the checker's 82 error-emission sites have zero `type/` guards — generics/PEP-695 (30/30 sites unguarded), compile-time operator walls (9 sites; existing operator_dispatch trio deliberately eval()-bypasses to the runtime path), call-arity, non-callable, match class-pattern, nonlocal-without-binding, container-literal set/tuple arms. tracked: #1769.
- **Silently-weakened wall, live divergence**: unknown annotation names (`x: NotARealType = 1`) silently become Any and run; CPython 3.12 raises NameError. Zero guards pin either behavior. tracked: #1769/#1770.
- **Widening rows missing halves** (`tech-design/type-system/` topic docs): list/set/frozenset kwargs-reject guards never landed (#1549 row); self-referential-mutation widening (#1536) and ==-search container-arg widening have no positive fixture; unbound-`__init__` widening has neither half. tracked: #1769.
- **Live over-walls in positive dimensions** (dimension-rule breaches): behavior/std-libs/typing xfail cluster is ~29/30 compile-time type rejects masked as xfail; `behavior/std-libs/selectors/select_detects_readable_socket.py` compile-rejects (`list[None]` literal later assigned a socket) while the oracle passes 150/150; slice-assign with tuple/str RHS walls though legal in CPython (`check_stmt.rs:1519-1533`). Rolling family: `tech-design/type-system/walls-and-widening.md` §Fire/defer semantics, remainder #1615; xfail-masked clusters tracked: #1768/#1770.
- **Contract-invisible wall**: PEP 634 OR-pattern binding mismatch emits `syntax error`, which `run_type_strict` cannot classify as STRICT_TYPE_OK — no fixture can host it in any dimension. tracked: #1771.
- **Suffix collision**: `behavior/std-libs/subclassinit/test__test_init_subclass_wrong.py` is the only `*_wrong.py` outside `type/` (CPython test name, runtime-positive, stale xfail). tracked: #1768/#1772.

## Verification

```bash
# Full C1 gate (~3 min; type/ walls + corpus-wide dimension rule together)
cargo test -p mamba --release --test conformance
# Focused wall sweeps (runner-parity verdicts; set MAMBA_BIN after a release build)
python3 tests/harness/cpython/tools/sweep.py tests/cpython/type/core tests/cpython/type/builtin-libs
python3 tests/harness/cpython/tools/sweep.py tests/cpython/type/std-libs/<lib>
# Dynamic-ingress slice (in-crate driver tests)
cargo test -p mamba --release strict_type_dynamic_ingress
# Accounting / regen tooling
python3 tests/harness/cpython/tools/strict_type_accounting.py   # wall bookkeeping
python3 tests/harness/cpython/tools/type_wall_gen.py --emit-rust  # regen stdlib_sigs_generated.rs (typeshed-pinned)
```

Manifest: `tests/harness/cpython/config/manifests/type-strict/param_types.toml` (generator source for the
core/param_types surface); std-libs walls regenerate from per-lib manifests under `config/manifests/std-libs/`.
Every fix in this domain shows the `*_wrong.py` guard set unweakened before/after (sibling TDs' verification contracts).

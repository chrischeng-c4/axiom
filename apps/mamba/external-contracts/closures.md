# closures — external contract (as-is, 2026-07-15)

Domain map: `tech-design/closures/ARCHITECTURE.md` (EC surface section); topic doc
`tech-design/closures/capture-and-scope.md` (cells, the two-pass hazard, introspection rule — not restated
here). Verdict law: `HARNESS.md`. Oracle = live python3.12 byte-diff; xfail = acknowledged gap, still contract
(skip, never executed).

Scope per ARCHITECTURE.md: capture cells (runtime), scoping decisions made by the CHECKER half of the two-pass
system, and capture introspection. `external-contracts/name-resolution.md` (not listed in this README's own
domain-map table) covers the same walrus/scope fixture dirs as its "resolver half" contract and explicitly
disclaims capture cells ("closures/' contract — not restated here"); see Known contract gaps for the overlap
this creates.

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

Live counts 2026-07-15 (`tests/cpython/` relative; xfail via `grep -rl "mamba-xfail"` per dir).

Closures-exclusive (cell mechanics / introspection):

| Dir | .py | xfail | Covers |
|---|---|---|---|
| `_regression/core/closure_capture/` | 4 | 0 | late-binding capture, box mutation, `nonlocal` write-back, shared-cell state — all via `closure_late_binding.py` alone (see gap: the other 3 files are generic scaffold); double-booked with `name-resolution.md` (marked "(shared with closures/)" there), not exclusive — see Known contract gaps |
| `behavior/std-libs/inspect/getclosurevars_reports_nonlocals_and_builtins.py` | 1 | 0 | capture-introspection byte-identical gate (241 TD) — the one truly exclusive fixture in this table |

Checker-half scoping decisions — named in ARCHITECTURE.md's own EC surface, and simultaneously counted in
`name-resolution.md`'s total (see gap):

| Dir | .py | xfail | Covers |
|---|---|---|---|
| `behavior/pep/572/` | 21 | 0 | walrus placement: comprehension leak, genexp deferral, conditional binds |
| `errors/pep/572/` | 8 | 0 | illegal walrus-target rejections (positive proof — byte-matches oracle SyntaxError, not a wall) |
| `surface/pep/572/` | 1 | 0 | walrus parse/bind smoke |
| `_regression/core/scope_resolution/` | 5 | 0 | local/global/nonlocal resolution, name mangling |
| `_regression/core/scope_modifiers/` | 9 | 1 | `global`/`nonlocal` edge cases (the 1 xfail is unrelated to scoping — see gap) |
| `_regression/core/comprehension_scope/` | 5 | 0 | comprehension isolation + class-cell (`__class__`) interaction |

Totals: **54 fixtures, 1 xfail** (53 live) across both tables; the closures-exclusive table alone is 5/0.

Supplementary, not oracle-governed (informational only): `src/runtime/closure.rs` carries 45 `#[test]` fns
(slab lifecycle, cell aliasing/sharing, decorator/property construction, cleanup, global-id namespace) — cited
by 241 TD; `src/lower/hir_to_mir.rs` carries a class-cell capture assert among its 50 `#[test]` fns (ARCHITECTURE.md
cites hir_to_mir.rs:12126 — the file has grown since, the assert now sits further down near the
`class_cell_required`/`mb_class_bind_classcell` block).

## Negative contract — what must be REJECTED

None. Closures owns no `type/` walls (README.md dimension rule; confirmed by ARCHITECTURE.md's own EC surface
line). All compile-reject surfaces adjacent to this domain belong to type-system:

- `type/std-libs/inspect/getclosurevars__func_as__IntrospectableCallable_wrong.py` (1 fixture, 0 xfail,
  `# mamba-strict-type: TypeError`) — an arg-shape wall on the SAME `inspect.getclosurevars` API this domain's
  positive fixture proves; owned by stdlib/inspect + type-system, not closures.
- No `type/` dir matches `scope`, `walrus`, or `closure` under `tests/cpython/type/core/` — confirmed empty by
  direct listing. `name-resolution.md`'s own negative-contract section notes the same: any scope/walrus-named
  `type/` hit belongs to stdlib arg-shape walls, not this domain.

## Known contract gaps

- **Domain-map double-booking with `name-resolution.md`**: 53 of this domain's 54 EC-surface fixtures (the full
  `pep/572` family + `scope_resolution` + `scope_modifiers` + `comprehension_scope` + `closure_capture/`) are
  independently counted in `external-contracts/name-resolution.md`'s own total (122/33), and `closure_capture/` is
  explicitly marked "(shared with closures/)" there — so it is double-booked, not exclusive, despite this doc's own
  Positive contract table framing it as closures-only. `README.md`'s domain contract map table doesn't list
  `name-resolution` as a row
  at all, so no doc is the arbiter for these dirs — a fixture edited in one doc's table can silently drift from
  the other's. Same root cause `name-resolution.md` already cites for its own prose-only mapping, and the same
  pattern as ARCHITECTURE.md citing a renamed doc (`getclosurevars-capture-cells.md` — no such file exists under
  `tech-design/closures/`; the live equivalent is `capture-and-scope.md`). tracked: #1771.
- **`closure_capture/` is 75% generic scaffold**: of its 4 fixtures, `behavior.py`/`errors.py`/`surface.py` are
  boilerplate language-scaffold content (generic `1+1==2`/`NameError`/builtins-presence asserts, byte-identical
  in structure to the same-named scaffold in `scope_resolution/`) with zero closure-specific assertions; only
  `closure_late_binding.py` (6 clauses: late-binding, default-arg freeze, rebind-after, box mutation, `nonlocal`
  write-back, shared-cell) actually proves capture semantics. The directory's "4 fixtures" headline overstates
  closure-specific coverage 4x.
- **f-string walrus has zero fixture coverage**: ARCHITECTURE.md's hazards document that an f-string
  replacement-field walrus leaks its binding to the enclosing scope while suppressing the field's own type errors
  (`check_expr.rs:480-515`, the `Expr::FString` arm's `walk`/`truncate_errors` pattern; `capture-and-scope.md`
  does not cover this case) — no fixture under `behavior/pep/572/` or any `pep/fstrings/` dimension exercises
  `f"{(x := ...)}"`; documented behavior, not proven behavior.
- **Two sibling `getclosurevars` fixtures in the same dir are xfail'd**: `behavior/std-libs/inspect/` also holds
  `test_get_closure_vars__test_getclosurevars_empty.py` and `..._error.py`, both "auto-ported CPython test; mamba
  promotion pending" — not in this domain's EC surface (stdlib/inspect owns the dir) but they exercise the
  identical capture-introspection API (241 TD) this contract is about. tracked: #1768.
- **The one xfail in this domain's surface is not a scope/closure bug**: `_regression/core/scope_modifiers/del_slice_raises.py`
  is `del lst[a:b]` being a no-op (collections/slice-delete divergence, per the fixture's own comment) —
  `name-resolution.md` characterizes this identically. The in-fixture "(#5)" is not a GitHub issue — it's an
  internal item index inside the `project_mamba_module_exec_del_silent_divergences` finding set (sibling
  fixtures reuse the same string with (#1)-(#8)); no real tracker currently covers this divergence.
- **Triple name-resolution pass, only partially guarded**: resolver (`pass.rs`), checker (`check_expr.rs` walrus
  arm), and the lowering prescan (`ast_to_hir.rs`) each re-implement scope-binding rules independently; only the
  target scanners are shared. ARCHITECTURE.md documents a historical divergence that silently corrupted an outer
  symbol's recorded type (`check_expr.rs:1536-1543`, old always-enclosing rule: an inner-function walrus on the
  same name as an outer module variable re-defined the symbol at module scope) — no fixture pins that specific
  outer-type-corruption regression by name, so a reintroduction would surface only as an unrelated type error
  elsewhere, not as a failure in this contract.

## Verification

```bash
# inner loop (seconds; runner-parity verdicts, shared oracle cache; set MAMBA_BIN after a release build)
python3 tests/harness/cpython/tools/sweep.py tests/cpython/_regression/core/closure_capture \
  tests/cpython/behavior/pep/572 tests/cpython/errors/pep/572 tests/cpython/surface/pep/572 \
  tests/cpython/_regression/core/scope_resolution tests/cpython/_regression/core/scope_modifiers \
  tests/cpython/_regression/core/comprehension_scope \
  tests/cpython/behavior/std-libs/inspect/getclosurevars_reports_nonlocals_and_builtins.py

# cargo slice of the full gate (datatest filter is a path substring)
cargo test -p mamba --release --test conformance -- closure_capture
cargo test -p mamba --release --test conformance -- pep/572

# Rust unit level (cell/slab mechanics + class-cell capture assert; not oracle-governed)
cargo test -p mamba --lib runtime::closure::
cargo test -p mamba --lib lower::hir_to_mir::

# full C1 gate (~3 min; this domain's slice rides inside it) — never concurrent with a cargo build
cargo test -p mamba --release --test conformance
```

Manifests: `config/manifests/pep/{572,walrus}.toml` cover the `pep/572` dirs (enumerating case names like
`bare_named_expr_binds`, `walrus_parses_and_binds`, `bare_walrus_statement_rejected`); the `_regression/core/*`
dirs (`closure_capture`, `scope_resolution`, `scope_modifiers`, `comprehension_scope`) and the inspect fixture
have no manifest (hand-authored/ported). No dedicated C2 perf pin exists for this domain
(`tests/harness/cpython/config/perf/pins/` has no `closure`/`scope`/`walrus`/`572` entry).

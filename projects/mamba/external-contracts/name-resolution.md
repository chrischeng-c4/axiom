# name-resolution — external contract (as-is, 2026-07-15)

Contract = CPython 3.12 parity for static scoping: assign-makes-local, `global`/`nonlocal`,
class-scope non-lexicality, comprehension isolation, PEP 572 walrus placement/leak. Mechanism and
invariants: `tech-design/name-resolution/ARCHITECTURE.md`; verdict law: `HARNESS.md` (same dir).
Runtime capture cells are `closures/`' contract — not restated here.

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

Live counts (2026-07-15; `find -name '*.py'` + `grep -l '# mamba-xfail'` per dir):

| Fixture dir (under `tests/cpython/`) | .py | xfail |
|---|---|---|
| `_regression/core/scope_resolution/` | 5 | 0 |
| `_regression/core/scope_modifiers/` | 9 | 1 |
| `_regression/core/comprehension_scope/` | 5 | 0 |
| `_regression/core/walrus/` | 4 | 1 |
| `_regression/core/closure_capture/` (shared with closures/) | 4 | 0 |
| `_regression/core/language/{scope,walrus,comprehensions}/` | 9 | 0 |
| `{behavior,surface,errors}/pep/572/` (21+1+8) | 30 | 0 |
| `{behavior,surface,errors}/pep/walrus/` + `_regression/pep/walrus/` (7+6+1+1) | 15 | 0 |
| `behavior/std-libs/scope/` (CPython `test_scope.py` port) | 41 | 31 |
| **Total** | **122** | **33** |

xfail = acknowledged gap, still contract; the only re-enable is deleting the directive — never
weaken the fixture. Boundary dirs owned elsewhere: `_regression/core/grammar/test_{walrus,comprehensions}`
(frontend, pipeline-run), `behavior/core/comprehension_float_inference` (type-system),
`_regression/core/language/closures` (closures).

## Negative contract — what must be REJECTED

| Surface | Count | Gate |
|---|---|---|
| `_regression/core/typecheck/undefined_var.py` — `# RUN: typecheck` + `EXPECT-ERROR: undefined name` | 1 | `--test conformance_pipeline` |
| `type/` dimension walls owned by this domain | 0 | — (all scope/walrus/nonlocal-named `type/` hits are stdlib arg-shape walls, type-system's) |

The checker's `no binding for nonlocal` wall (`src/types/check_stmt.rs:954`) does reject, but has
zero guard fixtures — see gaps.

## Known contract gaps

- `behavior/std-libs/scope`: 31/41 xfail'd under the blanket "auto-ported CPython test; mamba
  promotion pending" campaign — never executed, staleness unmeasured (tracked: #1768).
- nonlocal-without-binding wall unguarded: fires as `type error` where CPython raises compile-time
  SyntaxError; the entire `# RUN: typecheck` surface is 2 fixtures, so this domain's static
  rejection contract rests on 1 undefined-name guard (tracked: #1769).
- `_regression/core/walrus/assignment_expression.py` xfail: if-context walrus reads back as a
  denormal float — NaN-box reinterpretation, codegen-owned bug resident in this dir (tracked: #2813; #1770 family).
- `_regression/core/scope_modifiers/del_slice_raises.py` xfail: `del lst[a:b]` is a no-op —
  collections-owned bug resident in this dir (tracked: #5; #1770 family).
- `config/manifests/pep/{572,walrus}.toml` header comments point at the retired
  `tests/cpython/pep/<lib>/<dim>/` layout (live = `<dim>/pep/<lib>/`) — part of the 113-manifest
  stale-path finding (tracked: #1767).
- Domain→fixture-dir mapping is prose-only (this file); no machine-readable manifest backs a
  one-command domain verify (tracked: #1771).
- `Resolver` (`src/resolve/pass.rs`) has no production callers: every fixture above actually proves
  the CHECKER's scoping arm; resolver-only drift is invisible to this EC (ARCHITECTURE invariant).

## Verification

- Inner loop (seconds): `python3 tests/harness/cpython/tools/sweep.py tests/cpython/_regression/core/scope_resolution tests/cpython/_regression/core/scope_modifiers tests/cpython/_regression/core/comprehension_scope tests/cpython/_regression/core/walrus tests/cpython/_regression/core/closure_capture tests/cpython/_regression/core/language/scope tests/cpython/_regression/core/language/walrus tests/cpython/_regression/core/language/comprehensions tests/cpython/behavior/pep/572 tests/cpython/surface/pep/572 tests/cpython/errors/pep/572 tests/cpython/behavior/pep/walrus tests/cpython/surface/pep/walrus tests/cpython/errors/pep/walrus tests/cpython/_regression/pep/walrus tests/cpython/behavior/std-libs/scope` — mind HARNESS.md's stale-`MAMBA_BIN` and build/sweep-contention hazards.
- Cargo slice of the full gate: `cargo test -p mamba --release --test conformance -- pep/572` (and `pep/walrus`, `scope`, `walrus` path substrings).
- Negative half: `cargo test -p mamba --release --test conformance_pipeline -- typecheck`.
- Rust unit level (the only exercise of the standalone `Resolver`): `cargo test -p mamba --lib resolve::`.
- Full gate this slices into: `cargo test -p mamba --release --test conformance` (~3 min, C1).
- Manifests: `tests/harness/cpython/config/manifests/pep/{572,walrus}.toml`; the `_regression/*` and `behavior/std-libs/scope` dirs have no manifest (hand-authored/ported).

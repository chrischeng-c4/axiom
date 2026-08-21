# frontend — external contract (as-is, 2026-07-15)

Domain map: `../tech-design/frontend/ARCHITECTURE.md` (its EC-surface table is the source of this doc's scope). Verdict semantics: `HARNESS.md`; global gates + dimension rule: `README.md` (this dir). Counts are live (2026-07-15); fixture paths relative to `tests/cpython/`.

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

| Fixture set | .py | xfail | Executed by |
|---|---|---|---|
| `_regression/core/grammar/` | 62 | 0 inline, 1 manifest | `grammar.rs` parser-only (all 62); `pipeline.rs` (the 54 `# RUN: parse`); `runner.rs` oracle (the 8 record fixtures: behavior/errors/surface/finally_control_flow/name_mangling/numeric_literals/operators_precedence/subscript_slicing) |
| `_regression/core/parse/` | 79 | 0 | `pipeline.rs` (76 `# RUN: parse`); `runner.rs` oracle (3 record) |
| `behavior/core/grammar/` | 80 | 45 | `runner.rs` oracle |
| `behavior/core/syntax/` | 37 | 34 | `runner.rs` oracle |
| `behavior/core/fstring/` | 85 | 65 | `runner.rs` oracle |
| `behavior/pep/{498,501,526,572,634,695,fstrings,positional_only,variable_annotations,walrus}` | 178 | 0 | `runner.rs` oracle |
| `errors/pep/` (same pep slice) | 42 | 0 | `runner.rs` oracle |
| `errors/core/{bigaddrspace,flufl,global}` | 8 | 8 | `runner.rs` oracle (currently all skipped) |
| `surface/pep/` (same pep slice) | 35 | 0 | `runner.rs` oracle (`inspect.signature` / param-kind introspection via `HirParamSig`) |

Totals: **606 fixtures, 152 inline xfail + 1 manifest xfail**. The 130 `# RUN: parse` fixtures prove parser **acceptance** only — never lowered/executed (#2546, `[cpython_compat:parser-only]` banner); everything else is full oracle byte-match. PEP 634/695 sets are shared surfaces: parse shape is frontend; match runtime / generics semantics belong to object-model / type-system.

## Negative contract — what must be REJECTED

No `type/` walls are owned here (walls = type-system; see `../tech-design/type-system/ARCHITECTURE.md`). Frontend's compile-time SyntaxErrors are proven **positively**: errors-dimension fixtures scope the reject inside `compile()` + `except SyntaxError` and byte-match the oracle (shape: `errors/pep/572/bare_walrus_statement_rejected.py`). Fixture-backed reject surfaces: bare/misplaced walrus (`errors/pep/572/` 8 + `walrus/` 1), solo-`/` and posonly/kwonly misuse (`errors/pep/positional_only/` 3), f-string field errors (`errors/pep/498/` 15, `fstrings/` 2, `501/` 1), annotation syntax (`526/` + `variable_annotations/` 3), match-pattern shape (`errors/pep/634/` 6), PEP 695 (3). One parser reject is contract-invisible — see gaps.

## Known contract gaps

- Blanket auto-port xfails: 144/202 `behavior/core/{fstring,grammar,syntax}` fixtures carry `auto-ported CPython test; mamba promotion pending`; the runtime runner has no xpass detection (`grammar.rs` does), so hidden PASSes rot silently. tracked: #1768, #1771.
- `errors/core/` is 8/8 xfail'd — the whole core SyntaxError-taxonomy dir is disabled: PEP 401 flufl `<>` (4), `global`-decl SyntaxErrors (3), bigaddrspace (1). tracked: #1768.
- Manifest xfail `_regression/core/grammar/behavior`: chained-`if` comprehension filter (`[v for v in xs if a if b]`) not parsed (`cpython_known_failures.toml`; entry carries no issue ref — the unowned-xfail problem). tracked: #1768.
- PEP 634 OR-pattern reject ("alternative patterns bind different names") emits `syntax error`, so no dimension can host it: `type/` verdicts require "type error", and positive dims cannot catch a whole-file fail-fast parse abort. Contract-invisible. tracked: #1769.
- 5 vacuous parse walks in `src/driver/tests/behavioral_lang.rs:707-746` traverse pre-migration dirs that no longer exist and go green on 0 files. tracked: #1771.
- Authoring ghosts: `tools/fixture_gen.py` emits to a dead root, and pep manifest headers (e.g. `tests/harness/cpython/config/manifests/pep/572.toml`) name the retired `tests/cpython/pep/<n>/<dim>/` layout — by-the-docs new frontend fixtures land where no gate collects them. tracked: #1767.
- Documented parity deviations with zero pinning fixtures: byte-width indentation (tab = 1 col, no `TabError`) and unknown-byte silent skip (`ARCHITECTURE.md` "CPython-parity semantics"); the 17 TabError-mentioning fixtures only assert the builtin name exists. Untracked (not among #1770's confirmed divergences).
- No core-bucket manifest for `grammar`/`syntax`/`fstring` (hand-ported; `config/manifests/core/` holds only args_kwargs_binding, cpython321_core_lang, generators) — additions there are manifest-less hand edits.

## Verification

- Parser acceptance: `cargo test -p mamba --release --test conformance_cpython_grammar` — 62 fixtures, xfail manifest `apps/mamba/cpython_known_failures.toml`, xpass warns on stderr.
- `# RUN:` pipeline: `cargo test -p mamba --release --test conformance_pipeline` — frontend slice = the 130 `# RUN: parse`; co-collected jit/typecheck fixtures belong to codegen/type-system.
- Focused oracle slice (seconds, verdict-parity with the cargo gate; sweep SKIPs `# RUN:` files, so `_regression` dirs contribute only their record fixtures):
  `MAMBA_BIN=target/release/mamba python3.12 tests/harness/cpython/tools/sweep.py _regression/core/grammar _regression/core/parse behavior/core/grammar behavior/core/syntax behavior/core/fstring behavior/pep errors/pep errors/core surface/pep`
  (the pep dirs are supersets including neighbor-domain peps — all must pass regardless, per the dimension rule).
- Full-gate slice: frontend fixtures ride C1 — `cargo test -p mamba --release --test conformance` (~3 min, 46k+); no narrower cargo filter exists, use sweep for the inner loop.
- Manifests: `tests/harness/cpython/config/manifests/pep/{498,501,526,572,634,695}.toml` → `fixture_gen`/`fixture_lint` per `tests/harness/cpython/conventions/FIXTURE-LAYOUT.md` (mind the #1767 dead-root caveat before regenerating).

# calling-convention — external contract (as-is, 2026-07-15)

Domain map: `tech-design/calling-convention/ARCHITECTURE.md` (EC surface section). Verdict law: `HARNESS.md`.
Oracle = python3.12 byte-diff; xfail = acknowledged gap, still contract (never executed — see HARNESS.md rot hazard).

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

All paths under `tests/cpython/`. Live counts (2026-07-15): **582 fixtures, 483 xfail** (99 live).

| Fixture dir | .py | xfail | Covers |
|---|---|---|---|
| `{behavior 1, _regression 4}/core/args_kwargs_binding` | 5 | 0 | dynamic-value kwargs bind-by-name incl. bound methods (manifest `core/args_kwargs_binding.toml`, mamba #1432) |
| `behavior/core/funcattrs` | 33 | 28 | `__defaults__`/`__kwdefaults__` rewrite |
| `behavior/std-libs/extcall` (`test_extcall.py`) | 1 | 1 | spread-call entry |
| `behavior/std-libs/call` (`test_call.py`) | 137 | 134 | general call binding + error-message parity |
| `behavior/std-libs/userfunctions` (`test_sqlite3/test_userfunctions.py`) | 62 | 62 | sqlite3 `create_function`/`create_aggregate` Python-callback binding |
| `behavior/std-libs/keywordonlyarg` (`test_keywordonlyarg.py`) | 11 | 10 | kw-only params |
| `behavior/std-libs/positional_only_arg` (`test_positional_only_arg.py`) | 28 | 24 | pos-only params |
| `behavior/std-libs/getargs` (`test_capi/test_getargs.py`) | 76 | 76 | C-API `getargs`-parity arg parsing |
| `{behavior 172, surface 27, errors 10, real_world 1, _regression 3}/std-libs/functools` (`test_functools.py`) | 213 | 148 | `functools.partial` prepend/merge/recurse (Control flow step 4) |

Adjacent — same topical binder, not named in ARCHITECTURE.md's EC surface bullets (documentation
under-enumeration, not a scope dispute): `_regression/core/args_kwargs` (11, call-site unpacking/kwarg
patterns) + `_regression/core/star_call` (5, `*args`/`**kwargs` call-site spread) = 16 fixtures, 0 xfail.

Adjacent, counted elsewhere: `*/pep/positional_only` (surface 3 + `_regression` 1 + errors 3 + behavior 7 = 14,
4 xfail, 10 live) — PEP 570 **syntax** parsing is the frontend/parser domain, not the runtime binder owned
here.

## Negative contract — what must be REJECTED

No wall dimension of its own (README.md dimension rule: this domain's own dirs run). Walls over this domain's
surface, owned by type-system:

- `type/core/arg_annotation` — 12 fixtures, 0 xfail. Per-kind scalar rejection: `func_int_arg_called_with_str`,
  `{keyword_only,positional_only,varargs,kwargs}_int_arg_called_with_str`, `func_str_arg_called_with_bytes`,
  `default_int_arg_uses_str_default`, plus `dynamic_callable_*` and `func_{dict,list}_arg_called_with_*`
  variants. Cross-ref `../type-system/walls-and-widening.md`.
- `type/std-libs/functools` — 24 fixtures, 15 xfail (`force-typed arg enforcement pending`), same stratum
  as `type/std-libs/traceback` in `exceptions.md`; epic #861 (closed), remaining backlog tracked #1768.

Weakening either wall family is a contract breach even though this domain doesn't gate them directly — the
runtime binder re-checks the same scalar contract at ingress (`validate_and_adapt_declared_frame`).

## Known contract gaps

- **#1754 (confirmed open)** — two runtime-binder defects, both spot-checked against current source
  (2026-07-15) and still present as described: (1) `kwargs_dict_pairs` (`runtime/builtins/mod.rs:5949`)
  filters to `DictKey::Str` only — `f(**{1:'x'})` silently loses the key instead of CPython's
  `TypeError: keywords must be strings`; (2) `dispatch_jit_frame`'s arity-keyed transmute (`mod.rs:6424`)
  covers 0..=8 slots only (`_ => MbValue::none()`) — a 9+ param declared frame binds correctly then
  dispatches to `None`, a silent wrong result with clean compile.
- **Un-xfail dominant stratum**: 483/566 owned-and-named fixtures are xfail, and every sampled reason is the
  generic `auto-ported`/`auto-extracted CPython test; mamba promotion pending` marker (100% of the xfails in
  call/userfunctions/keywordonlyarg/positional_only_arg/getargs/functools/funcattrs) — `getargs` and
  `userfunctions` are 100% skipped end-to-end. Un-xfail campaign: #1768.
- **Empty-frame sentinel ambiguity** (`bind_declared_call_frame`, `mod.rs:6043/6102/6130`): `Some(vec![])`
  means "raised", `None` means "no param metadata", `Some(frame)` means "success" — a caller that skips the
  `current_exception_type()` check after binding double-dispatches a callee whose bind already raised.
  Architectural hazard named in ARCHITECTURE.md; no isolating fixture found, not yet a confirmed live bug.
- **entry_abi/contract desync risk**: `contract` (rejection) and `entry_abi` (unbox representation) are
  independent fields threaded off the same 7-tuple; old 5/6-field metadata falls back to boxed silently, so a
  lowering path that sets one without the other either misses a scalar wall or mis-unboxes. Same status —
  named hazard, no isolating fixture found.
- **Static default-fill blind to `__defaults__`/`__kwdefaults__` mutation**: installing defaults the source
  signature lacks must route through `funcs_with_mutated_defaults` → `build_mutated_defaults_call`
  (`ast_to_hir.rs:9123`); any new call-lowering arm that bypasses this reintroduces a missing-arg misfire.
  Same status — named hazard, no isolating fixture found.
- **Variadic scalar contract validates elements only** (`mod.rs:6286` kind 2, `:6314` kind 4): the
  `*args`/`**kwargs` *container* itself is never scalar-adapted, only its members.

## Verification

```bash
# inner loop (seconds; paths relative to tests/cpython) — from apps/mamba/tests/harness/cpython/
python3 tools/sweep.py _regression/core/args_kwargs_binding _regression/core/args_kwargs _regression/core/star_call \
  behavior/core/args_kwargs_binding behavior/core/funcattrs behavior/std-libs/extcall behavior/std-libs/call \
  behavior/std-libs/userfunctions behavior/std-libs/keywordonlyarg behavior/std-libs/positional_only_arg \
  behavior/std-libs/getargs                                    # core binder + kw-only/pos-only slice
python3 tools/sweep.py behavior/std-libs/functools surface/std-libs/functools errors/std-libs/functools \
  real_world/std-libs/functools _regression/std-libs/functools type/std-libs/functools   # functools family + its walls
# cargo gate slice (datatest filter is a path substring; one dir per filter run)
cargo test -p mamba --release --test conformance -- args_kwargs_binding
cargo test -p mamba --release --test conformance -- std-libs/call
# full C1 gate (~3 min; this domain's slice rides inside it) — never concurrent with a cargo build
cargo test -p mamba --release --test conformance
# manifests: config/manifests/core/args_kwargs_binding.toml + config/manifests/std-libs/functools.toml
# (13 surface/10 errors/24 behavior/1 real_world cases) exist; the other 7 std-libs dirs above are
# auto-port/auto-extract authored with no manifest of their own
```

Verdict semantics, xfail-is-skip rot, sweep load-degradation: `HARNESS.md` (this dir).

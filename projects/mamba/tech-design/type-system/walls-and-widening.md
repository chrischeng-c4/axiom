# Walls and widening — where the checker fires, defers, and must yield

The strict-type checker's product value is rejecting genuinely wrong code
(walls) without rejecting legal Python (over-walling). This topic holds the
dividing rules and the catalogue of proven widenings.

## The dimension rule (the domain's outer contract)

- `type/` fixtures are the NEGATIVE contract: they must be compile-rejected;
  weakening one is a contract breach.
- Every other dimension (behavior/errors/real_world/surface/_regression/
  security/concurrency) must RUN and match the oracle — a compile reject there
  is a false positive by definition.

## Fire/defer semantics (as-is)

Walls fire only when the evidence is total: all-Rejected, no-Indeterminate,
and not in the `constructor_bypass` allowlist (check_expr.rs:~4517; allowlist
covers builtins.{ImportError,range,type}, functools.{partial,reduce,lru_cache}
— errors/-dimension "raises at runtime" shapes extend it case-by-case,
tracked: #1615). The legacy path skips when unsure (enforceable +
concrete-scalar-disjoint). Signature truth stacks in four layers: generated
TypeSpec manifest → curated STDLIB_SIGS overrides → intrinsic `Ty::Fn` →
user `FunctionParamSigs` (see ARCHITECTURE.md); layer 1 is authoritative
end-to-end — `check_expr.rs:669`'s `structured_stdlib_authoritative` flag
fully bypasses the compact-scalar and `check_user_fn_argument_type`
mismatch checks (incl. the `bytes_literal_str_mismatch` special case) once
`check_structured_stdlib_call` resolves a call, so a layer-3 intrinsic
`def_builtin` registration (`builtins.rs`) can look wrong in isolation and
still never fire. Worked example: `ord` looks monomorphic-`str`-only at
`builtins.rs:131` (`def_builtin("ord", &[str_ty], int)`), but the generated
manifest row (`stdlib_sigs_generated.rs:~21797`, `p("c", CoreTy::Typed,
false)`) owns every real call site and is permissive — `ord(b"A")` /
`ord(bytearray(b"Z"))` compile and run correctly today (verified directly:
`chr/behavior.py` byte-matches the oracle unmodified). Confirm empirically
against a fresh binary before trusting a layer-3 signature reading; #1550's
`ord`/bytes suspicion did not reproduce.

## Proven widenings (each keeps its guard set red)

| Legal shape | Rule established | Where |
|---|---|---|
| Structural numeric protocols | `SupportsIndex` ⊇ Int/Bool; `SupportsFloat` ⊇ Int/Float/Bool in the general engine, not just private projections | types_compatible_inner |
| Open inheritance | a user class with an unresolved external base is "open" — never hard-reject vs nominal stdlib params (mirror stdlib_type_relation_inner's leniency) | nominal catch-all |
| Duck-typed closed inheritance | closed-inheritance classes satisfying the used surface don't hard-wall vs nominal stdlib types | stdlib_type_relation_inner |
| Inherited NamedTuple `__init__` | MRO resolution reaching `typing.NamedTuple`'s functional factory is skipped when arrived via inheritance; the DIRECT factory-call wall stays | is_named_tuple_base_owner |
| Self-referential mutation | `x.append(x)`-shape widens the element type instead of erroring (aliased receiver `y=x; x.append(y)` still walls — acceptable residual) | self_referential_mutation_widen (check_expr.rs:1573) |
| ==-search container args | `.index/.count/.remove` accept container-shaped args (search never raises on shape); scalar walls stay | container_receiver_relaxed_call |
| Slice-assign RHS | any non-container RHS defers to runtime (CPython accepts any iterable) | check_subscript_assignment |
| Unbound `__init__` receiver | `Class.__init__(recv, …)` always skips arg 0 — `__init__` can never be class/staticmethod | both call-resolution paths |
| Per-builtin kwargs facts | list/set/frozenset REJECT kwargs; dict ACCEPTS them (tracked: #1549) — never share the arms | ast_to_hir rejection arm |

## Known wall gaps (negative-contract violations)

- classmethod/staticmethod explicit `.__get__` has no signature → 4 guards
  pass through (tracked: #1611). Fix = curated-table signatures for the
  explicit-call shape only.
- NamedTuple functional form walls a LEGAL factory call — the retained
  direct-call signature is misaligned with the real factory shape
  (tracked: #1628).
- Residual over-walling clusters (~470 corpus-wide post wave-1/2) — same
  fix pattern; goal-loop waves burn them down (tracked: #1615).
- `all(42)`/`any(42)`-shape non-iterable args to iterable-consuming builtins
  compile-reject instead of deferring to the runtime `TypeError` the fixture
  expects to catch — live-reproduced 2026-07-15 across ≥10 fixtures in
  `_regression/builtin-libs/builtins/` (`all`, `any`, `map`, `max`, `min`,
  `reversed`, `isinstance`, `bytearray_methods`, `errors.py`,
  `long_tail/slice/slice_basic.py`); `range_broad.py` separately DIVERGEs
  on an int-vs-float value (int arithmetic returning `55.0` not `55`).
  Newly discovered during #1550 verification, distinct root cause(s) from
  #1550's original 4 — tracked: #1775.

## Working rule for any new widening

Name the legal shape precisely; widen at the narrowest applicable point;
run the guard (`*_wrong.py`) set for that surface plus the full gate; a
widening that flips any `type/` fixture is wrong by construction.

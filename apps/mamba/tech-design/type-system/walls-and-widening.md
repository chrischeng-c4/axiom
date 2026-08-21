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

`isinstance(obj, class_or_tuple, /)` is positional-only in real CPython; a
keyword-shaped call is always illegal. `isinstance` is also one of the
#1775 iterable-arg bypass targets (`check_expr.rs:4493`), deferring the
whole call to legacy `check_stdlib_call` (`:4818`) — but `ParamSig` has no
positional-only/keyword-only concept at all (`stdlib_sigs.rs:90-96`), so
its keyword-name-matching loop (`check_expr.rs:5040`) would silently
accept a keyword-shaped call despite `isinstance`'s own row being
`enforceable: true`. A dedicated pre-bypass check (`:4450`) rejects
keyword-shaped `isinstance` calls before the bypass fires, leaving the
positional-call path unaffected (tracked: #1794).

AbstractSet params are the nominal exception to this permissive stack:
`collections.abc.Set` defines no `__subclasshook__` (unlike structural
`Iterable`/`Sized`/`Container`), so `set`/`frozenset`'s rich/in-place
operators (`__and__ __or__ __sub__ __xor__ __iand__ __ior__ __isub__
__ixor__ __ge__ __gt__ __le__ __lt__`) defer out of
`check_structured_stdlib_call` (`check_expr.rs:4566`) instead of resolving
there — the generated evaluator has no class-hierarchy model, so it would
otherwise silently accept any class merely defining `__contains__`. The
legacy path's `check_stdlib_scalar_arg`'s `TypedNamed("AbstractSet")` arm
(`check_expr.rs:5099`) enforces the real requirement via
`set_derived_classes`/`set_derived_class_symbols` (`check.rs:865-866`),
populated at `Stmt::ClassDef` (`check.rs:2511`) with the same
single-forward-lookup chain resolution as `numeric_derived_classes`; the
builtin roots `set`/`frozenset` are matched by name and never populate the
registry themselves (tracked: #1794). `Ty::Set` has no mutable/frozen split, so a
`Ty::Set` receiver's member signatures resolve via the `set` STDLIB_SIGS
qualifier, a strict superset of frozenset's rows (`check_expr.rs:4911`).

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
| Loop/try-body accumulator reassignment | `total = 0; for x in xs: total += x` (tracked: #1775) or `total = total + x` (tracked: #1794, one level+ inside while/for/async-for/try-body/else/finally) widens instead of forcing `Any`; if/elif/else and except branches keep the pessimistic treatment | same_scope_loop_reassign_counts (check.rs:598) |

## Known wall gaps (negative-contract violations)

- classmethod/staticmethod explicit `.__get__` has no signature → 4 guards
  pass through (tracked: #1611). Fix = curated-table signatures for the
  explicit-call shape only.
- NamedTuple functional form walls a LEGAL factory call — the retained
  direct-call signature is misaligned with the real factory shape
  (tracked: #1628).
- Residual over-walling clusters (~470 corpus-wide post wave-1/2) — same
  fix pattern; goal-loop waves burn them down (tracked: #1615).

## Working rule for any new widening

Name the legal shape precisely; widen at the narrowest applicable point;
run the guard (`*_wrong.py`) set for that surface plus the full gate; a
widening that flips any `type/` fixture is wrong by construction.

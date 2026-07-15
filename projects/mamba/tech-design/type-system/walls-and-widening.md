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
user `FunctionParamSigs` (see ARCHITECTURE.md).

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

## Working rule for any new widening

Name the legal shape precisely; widen at the narrowest applicable point;
run the guard (`*_wrong.py`) set for that surface plus the full gate; a
widening that flips any `type/` fixture is wrong by construction.

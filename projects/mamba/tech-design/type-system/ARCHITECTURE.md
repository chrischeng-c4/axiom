# type-system — architecture (as-is, 2026-07-15)

Domain per `tech-design/README.md`: strict-type checker, signatures, walls, ingress enforcement. Source: `src/types/`.
Fix TDs in this dir (cross-referenced, not restated): `walls-and-widening.md` §Fire/defer semantics, §Proven widenings.

## Responsibilities

- Compile-time strict checking pass: any surviving error aborts compilation before lowering (`driver/mod.rs:145,186` — errors are the *walls*; deliberate rejects of wrong-typed Python).
- Signature truth for stdlib calls, layered: generated typeshed TypeSpec manifest → generated compact scalar table → curated overrides → intrinsic `Ty::Fn::signature` → user `FunctionParamSig`s.
- Semantic annotation resolution consumed by lowering (`check.rs:564 resolved_type_exprs`; lowering does not re-resolve).
- Sourcing the *runtime* dynamic-call ingress contracts: per-param scalar contract strings derived from `declared_ty` (`lower/hir_to_mir.rs:2023`), enforced at dynamic dispatch by `runtime/builtins/mod.rs:6270 validate_and_adapt_declared_frame`.
- Structural protocols (`protocol.rs ProtocolRegistry`, #314) and PEP 695 generics/aliases (`generic.rs`, `check.rs:1165 register_type_params`).

## Key structures & invariants

| Structure | Where | Rule that must hold |
|---|---|---|
| `Ty` / `TypeId` interning | `ty.rs:170 Ty`, `context.rs TypeContext` | TypeId/TypeVarId are per-checker; never persisted into generated manifest (`check.rs:694-697` caches are context-local) |
| `TypeChecker` | `check.rs:524` | Provenance is symbol-keyed, flow-updated: `import_origins:666`, `instance_origins:671`, `builtin_class_aliases:685`, `class_ref_origins:681`, `user_bare_classes:627`, `numeric_derived_classes:635`, `class_inheritance_open:649` |
| Curated compact table | `stdlib_sigs.rs:133 STDLIB_SIGS` (`CoreTy:22` closed scalar set) | Anything non-scalar collapses to `CoreTy::Unknown` = skip-when-unsure; rows with `enforceable:false` are documented negative guards |
| Generated compact table | `stdlib_sigs_generated.rs STDLIB_SIGS_GENERATED` (13,451 rows, typeshed rev pinned in header; regen: `tools/type_wall_gen.py --emit-rust`) | `stdlib_sigs.rs:8651 get()` — curated row ALWAYS wins over generated |
| Generated TypeSpec manifest | `stdlib_typespec.rs:369 MANIFEST` ← `stdlib_specs_generated.json` (schema=2, interned strings/nodes) | Lossless overloads via `:602 overloads()`; compact tables are fallback only (`check_expr.rs:663-667`) |
| Three-valued relations | `check_expr.rs:26 StrictRelation`, `:19 StdlibSpecCandidate` | Only `Incompatible`/all-`Rejected` fires; `Indeterminate` always defers |
| Runtime param metadata | `hir_to_mir.rs:2057` 7-tuple `(name,kind,has_default,default,anno,entry_abi,contract)` | Contract ∈ {int,bool,float,str,None,bytes} only, and only when annotated (`:2023-2039`); ABI adaptation never rejects independently (`builtins/mod.rs:6247`) |
| `SymbolTable` | `resolve/scope.rs:66` | Symbol identity (not name text) keys all contracts — shadowing must not inherit an outer import's signature |

Wall fire/defer decision table:

| Path | Fires when | Defers when |
|---|---|---|
| Structured TypeSpec (`check_expr.rs:4320`) | ALL overload candidates `Rejected` AND none `Indeterminate` AND not `constructor_bypass` (`:4517`) | any `Accepted`/`Indeterminate`; `*`/`**` args (`:3891`); mixed-kind member overload sets (`:4400-4412`) |
| Legacy compact (`check_expr.rs:4549`) | `sig.enforceable` && concrete param `CoreTy` && actual is disjoint concrete scalar OR bare user class (`:4783 check_stdlib_scalar_arg`) | `CoreTy::Unknown`; past first `*` param (`:4721`); unknown kwarg names (#881 `:4760`); non-concrete actual |
| User/intrinsic `Ty::Fn` (`check_expr.rs:675`) | arity mismatch (no stars/kwargs/defaults, `:697`) or `types_compatible` false (`:839`) | `structured_stdlib_authoritative` (`:669`); Any/TypeVar/Error on either side (`check.rs:3594-3600`) |
| Runtime ingress (`builtins/mod.rs:6270`) | dynamic-route arg violates scalar contract → catchable TypeError pre-body | contract `None`; legacy 5/6-tuple metadata fail-open (`:7147` test); kind 2/4 packs validated element-wise, containers never replaced |

Constructor_bypass allowlist (`check_expr.rs:4510`, exact as-is): Constructor: `builtins.{ImportError,range,type}`, `functools.partial`; ModuleFn: `functools.{reduce,lru_cache}`. Entry criterion: mamba's runtime independently raises the matching TypeError (see comment block `:4471-4509`; extension protocol in `walls-and-widening.md` §Fire/defer semantics).

## Control flow

1. `driver/mod.rs:132 check` / `:157 build`: parse → PEP695 desugar → `TypeChecker::new` (registers builtins + exception hierarchy, `builtins.rs:register_builtins`) → `check_module`; `build` pre-checks imported deps into the same checker (`:185 check_dependencies`).
2. `check.rs:2685 check_module`: pass 1 = `preregister_defs` + `finalize_generic_metadata_in` + `refresh_function_signatures_in`; pass 2 = `check_stmt` per top-level stmt; returns drained errors.
3. `check_stmt.rs:230`: `VarDecl` wall = `resolve_type_expr` → `types_compatible` → error, plus container-literal element check (`:255`); every `Assign`/`VarDecl` records provenance via `set_binding_origins` (`:260`).
4. `check_expr.rs:473 check_expr`, `Expr::Call` arm (`:627`): (a) `self_referential_mutation_widen` pre-scan (`:1573`, #1536 — see hazards); (b) `check_structured_stdlib_call` (`:4320`); (c) on miss, legacy `check_stdlib_call` (`:4549`); (d) `Ty::Fn` arity + per-arg `types_compatible` walk with the #220 `container_receiver_relaxed_call` relaxation (`:782`), suppressed when (b) was authoritative.
5. `check_structured_stdlib_call`: `resolve_structured_stdlib_call` (`:2523` — `import_origins`/unshadowed-builtin → ModuleFn|Constructor|ClassMember|BoundMember) → `spec::overloads` → per-candidate `evaluate_stdlib_spec_candidate` (`:3880`: bind args to params, `stdlib_type_relation` (`:3192`) per pair) → fire/defer per table above; accepted returns union-dedup (`:4536`).
6. Errors → `filter_type_ignored` (`driver/mod.rs:47`, line-based `# type: ignore`) → first survivor is the compile abort; only then lowering consumes `resolved_type_exprs`/`tcx`/`symbols`.
7. Dynamic ingress (runtime half): `hir_to_mir.rs:2073 mb_func_set_params` primes contracts at module init; dynamic dispatch routes (Any-typed bindings, `globals()[...]`, `*args`/`**kwargs` spreads — callers at `builtins/mod.rs:5823,6494,6766`) run `validate_and_adapt_declared_frame` before the body. E2E proof: `driver/tests/strict_type_dynamic_ingress.rs`; unit: `builtins/mod.rs:7039-7147`.

## Known hazards

- **Over-walling is the standing failure mode** — a wall firing on a behavior/errors/real_world fixture is by definition a checker bug; `walls-and-widening.md` §Fire/defer semantics is the rolling family (remainder #1615). Never blanket-disable; per-shape alignment only.
- **`constructor_bypass` without runtime validation = silent leak** — adding a class whose mamba runtime does NOT validate args removes its whole error surface, regressing `type/` guards (`check_expr.rs:4480-4487`).
- **Dual signature sources starve each other** — structured `Some(..)` short-circuits the legacy curated walls: #1611 needed an explicit defer for `classmethod/staticmethod.__get__` (`:4343`); regenerating typeshed coverage can silently disable a curated wall with no test failing except `type/` guards.
- **Generated fixed arity ≠ CPython runtime arity** — typeshed folds overloads; CPython validates argc at runtime (#1550 `dir()` defer `:4352`). Weakening the evaluator instead of adding a targeted defer breaks other walls.
- **Element-type pinning vs self-reference** — `self_referential_mutation_widen` (`:1573`) only matches literal AST shapes (`x.append(x)`, `x.extend([x])`); an aliased receiver (`y = x; x.append(y)`) still walls. Note: the superseded fix TD (`walls-and-widening.md` §Proven widenings) listed status OPEN though the widen had shipped — TD status lines can lag code in this dir.
- **Unbound `__init__` arg-0 skip lives in TWO paths** — `check_expr.rs:4622` (import_origins class_sig) and `:4666` (Ty::Class fallback); a third resolution path without the skip re-introduces the off-by-one (`walls-and-widening.md` §Proven widenings).
- **kwargs acceptance is per-builtin, not per-family** — the `list/set/frozenset` kwargs rejection lives in ast_to_hir, NOT `src/types/`; `dict` accepts kwargs (#1549) (`walls-and-widening.md` §Proven widenings).
- **`structured_stdlib_authoritative` requires an Ident callee** (`:669-670 func_name.is_some()`) — attr-called stdlib fns can still hit the `Ty::Fn` double-check path.
- **Table lookups are linear scans** — `stdlib_sigs.rs:8651 get()` walks curated then 13k generated rows per call site; fine today, a trap for hot-path additions.
- **Protocol widenings are name-string arms** — `SupportsIndex→Int|Bool`, `SupportsFloat→Int|Float|Bool` hardcoded at `check.rs:3712-3713`; open-inheritance leniency keys off `class_inheritance_open` (`check.rs:3862-3877`, `check_expr.rs:3251`). New structural names default to rejection.
- **Deferring a structured call to legacy doesn't guarantee legacy re-checks the same shape** — `ParamSig` has no positional-only/keyword-only concept at all (`stdlib_sigs.rs:90-96`), so a bypassed callable's legacy row can't reject a keyword-shaped call to a real positional-only param by itself (`isinstance`'s #1775 bypass needed its own pre-check to keep that rejection, `check_expr.rs:4450`); conversely, deferring can be the only way to reach a check the structured evaluator can't express at all (`AbstractSet`'s nominal inheritance requirement — see `walls-and-widening.md`'s Fire/defer semantics, tracked: #1794).

## Extension points

| Adding | Plug in at |
|---|---|
| Wall for one stdlib callable | Curated `STDLIB_SIGS` row (`stdlib_sigs.rs:133`) — overrides generated; regen path is `tools/type_wall_gen.py` |
| Defer-a-wall-to-runtime | `constructor_bypass` (`check_expr.rs:4510`), only with proven runtime TypeError; or a targeted `return None` defer like `:4343/:4361` |
| Structural-protocol acceptance | Name arm in `check.rs:types_compatible_step` (~`:3712`) mirrored into `check_expr.rs:stdlib_type_relation_inner` |
| Method-family relaxation | `container_receiver_relaxed_call` idiom (`check_expr.rs:782`) / `self_referential_mutation_widen` (`:1573`) |
| Runtime ingress scalar | `hir_to_mir.rs:2026` contract match + `builtins/mod.rs:6221 strict_scalar_value` (both sides or fail-open) |
| New binding provenance | `TypeChecker` map + `check_stmt.rs:260 set_binding_origins` (symbol-keyed, cleared on rebind) |
| Generated-spec node kinds | `stdlib_typespec.rs TypeSpecNode` + materialization caches `check.rs:697-706` (transactional, `:702`) |

## EC surface

Per `external-contracts/README.md`:

- **Negative contract (this domain's own):** `tests/cpython/type/{core,builtin-libs,std-libs}` — `*_wrong.py` walls MUST keep rejecting; weakening one is a contract breach. `type/core/` axes: `arg_annotation, container_element, method_resolution, operator_dispatch, param_types, return_annotation, var_annotation`.
- **Positive contract (by implication, corpus-wide):** every fixture under `behavior/ errors/ real_world/ surface/ _regression/ security/ concurrency/` must RUN — a compile reject there is a type-system false positive by definition (the dimension rule).
- **Ingress-specific proof:** `src/driver/tests/strict_type_dynamic_ingress.rs` (dynamic routes reject pre-body, TypeError catchable) + `runtime/builtins/mod.rs:7039-7147` unit contracts.
- **Gate:** `cargo test -p mamba --release --test conformance` (~3 min); every fix here shows the `*_wrong.py` guard set unweakened before/after (verification contracts in the sibling TDs).

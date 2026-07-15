# Capture and scope — cells, the two-pass hazard, and introspection

How closures capture, how names resolve across the resolver and the checker,
and the rule for reading captures back.

## Capture cells

A closure carries `capture_ids`/`capture_cells` (1:1 with its freevars at
creation). `mb_func_prime_name` captures the defining module at DEFINE time —
`callable_module_name` (used by `with_callable_module`) otherwise falls back
to "whatever module is active," which for a dynamically-invoked callable is
wrong.

## Introspection reads cells directly

`inspect.getclosurevars` (and any capture introspection) MUST read the
closure's OWN `capture_ids`/`capture_cells`, never the module-scoped
active-cell map (`ACTIVE_CELLS` keyed by `{module, symbol}`). During the
introspection call, a native dispatcher pushes ITS `__module__` (e.g.
"inspect") as active via `with_callable_module`, so a module-scoped lookup
searches the wrong scope and reads None. Helper: `closure_capture_value_for_id`.
General rule (also in stdlib/module-hazards.md): never resolve user-scope
names through active-module state inside a dispatcher.

## The two-pass name-resolution hazard

Names are resolved in TWO passes that must agree: the resolver
(`resolve/pass.rs`) and the type checker (`types/check_expr.rs`, e.g. the
walrus arms). A scope fix often needs BOTH — a nested-comprehension walrus was
correctly bound by the resolver but mis-scoped by the checker
(`define_in_enclosing_scope` targeting a popped comp scope). When touching
scoping, check both passes.

## Walrus scope rules (as-is)

Function-body walrus targets are synthetic locals (SymbolId ≥ 1_000_000) and
must NOT emit StoreGlobal; comprehension walrus binds the ENCLOSING scope but
its vreg is needed DURING the comprehension and dropped AFTER (fix in the
ListComp arm post-loop, not the walrus arm). Genexp laziness and default-arg
walrus remain deeper open items.

## EC surface

`pep/572`, capture-introspection fixtures under `behavior/std-libs/inspect`.

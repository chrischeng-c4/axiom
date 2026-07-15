# name-resolution — architecture (as-is, 2026-07-15)

The resolver half of mamba's two-pass name system. Source: `src/resolve/scope.rs`
(`SymbolTable` data model), `src/resolve/pass.rs` (`Resolver` walk + shared scanners). The type
checker (`src/types/`) is the OTHER pass — see `../type-system/` and the two-pass hazard note in
`../closures/capture-and-scope.md`. Runtime cell mechanics live in `../closures/ARCHITECTURE.md`;
synthetic-ID allocation and StoreGlobal emission live in `../codegen/` (`src/lower/`).

## Responsibilities

- The reusable symbol-table data model: scopes, SymbolIds, `VariableClass`, nonlocal mapping
  (`scope.rs::SymbolTable`) — instantiated and OWNED by the checker (`types/check.rs:526,713`).
- Static binding of every name to a `SymbolId` across module/function/class/comprehension/lambda
  scopes, honoring Python's assign-makes-local rule (`pass.rs::Resolver`).
- `global`/`nonlocal` reclassification and inner→outer cell wiring (`pass.rs:361,389`).
- PEP 572 walrus target placement (comprehension → enclosing function scope).
- The SHARED prescan scanners `collect_assignment_targets` (pass.rs:726) + `collect_walrus_targets*`
  (pass.rs:964,1111) — the only part of this file the checker actually calls (`check_stmt.rs:1126`).
- NOT owned here: runtime capture cells, `>= 1_000_000` synthetic IDs, StoreGlobal (cross-refs below).

## Key structures & invariants

| Structure | file:symbol | Invariant |
|---|---|---|
| `SymbolTable` | scope.rs:66 | flat `scopes: Vec<Scope>` + flat `symbols: Vec<SymbolInfo>`; `current_scope` index + `scope_return_stack` for pop |
| `SymbolId(u32)` | scope.rs:5 | DENSE SEQUENTIAL — `SymbolId(self.symbols.len())` (scope.rs:113,211). NO gaps, NO `1_000_000` base here — that range is a LOWERING concept (`lower/ast_to_hir.rs:3724`), see codegen/ |
| `Scope{parent, symbols}` | scope.rs:42 | module scope = index 0, `parent: None`; `lookup` walks `parent` chain (scope.rs:123), `lookup_in_scope` does not walk (scope.rs:171) |
| `VariableClass` | scope.rs:29 | `Local`(default)/`Global`/`Free`/`Cell`; `get_var_class` returns `Local` if unset (scope.rs:153) |
| `nonlocal_mapping` | scope.rs:76 | inner(Free)→outer(Cell) SymbolId; set at pass.rs:417, read via `get_nonlocal_outer` |
| `scope_return_stack` | scope.rs:72 | a method's lexical parent skips its class body, but pop must RESUME the class scope — dynamic return points, not `parent` |
| `Resolver.function_scope_stack` | pass.rs:41 | scope indices that are true function boundaries; seeded `[0]`; walrus/nonlocal consult it; class scopes are NOT pushed here |
| `Resolver.class_scope_stack` | pass.rs:43 | class namespaces are executable but NOT lexical parents of nested defs (pass.rs:178 skips them) |
| `Resolver.comprehension_depth` | pass.rs:39 | >0 ⇒ walrus escapes to enclosing function scope (PEP 572) |
| `ResolveResult` | pass.rs:11 | `{symbols, errors, name_map: Vec<(Span,SymbolId)>}` — `name_map` ties each AST occurrence to its id |

INVARIANT (as-is reality): `resolve_module` (pass.rs:19) has **no production callers** — only its own
~1650-line unit suite (pass.rs:1250+) and a fixture-name string (`meta_gates/.../c3_...gate.rs:185`).
The checker builds its OWN `SymbolTable` in `check_module` and THAT feeds lowering. This file is LIVE
only through (a) the `scope.rs` types and (b) the shared scanners. Treat `Resolver` as a parallel
reference implementation of the scoping rules, not the production resolver.

## Control flow (`resolve_module`, pass.rs:19)

1. `register_top_level` (pass.rs:58) → `register_defs_in` (pass.rs:66): pre-register FnDef/Class/Enum/
   TypeAlias names, descending into try/if/while/for/with/match bodies but NOT function bodies →
   module-scope forward references resolve.
2. Walk each stmt via `resolve_stmt` (pass.rs:151).
3. **FnDef** (pass.rs:176): compute `lexical_parent` skipping any `class_scope_stack` entries →
   `push_scope_with_parent` → push `function_scope_stack` → define params → PRESCAN body
   (`collect_assignment_targets` + `collect_walrus_targets_in_stmts`, skipping names in `declared`)
   so every assigned name is Local before bodies walk → resolve body → pop both.
4. **ClassDef** (pass.rs:214): `push_scope` + push `class_scope_stack`; no prescan (class bodies bind
   left-to-right).
5. **Assign** (pass.rs:158): bare `x = v` where `lookup(x)` misses ⇒ define new Local; else resolve
   target+value (attribute/subscript targets create no symbol, pass.rs:169).
6. **Global** (pass.rs:361): in a function, reuse/create local id then `set_var_class(Global)`; at
   module level bind current scope directly to the scope-0 id via `bind_symbol_in_scope`.
7. **Nonlocal** (pass.rs:389): walk `parent` chain skipping non-function scopes and stopping before
   scope 0 (cannot bind globals); on hit mark outer `Cell`, inner `Free`, record mapping; miss ⇒
   `no binding for nonlocal` error (pass.rs:425).
8. **Comprehension** (ListComp/Set/Gen pass.rs:546, Dict pass.rs:574): `comprehension_depth++` +
   `push_scope`; iter targets bind in the comp scope (isolated); pop + `depth--`.
9. **Walrus** (pass.rs:596): `depth>0` ⇒ `define_in_scope(function_scope_stack.last())` (escape comp);
   else define in current scope.
10. **Ident** (pass.rs:477): `lookup` walk; miss ⇒ `undefined name` error. `resolve_pattern`
    (pass.rs:666) binds match-capture/`as`/mapping-rest/star names.

## CPython-parity semantics

- **Assign-makes-local**: any name assigned ANYWHERE in a function body is Local for the whole body,
  not just after the assignment — enforced by the whole-body prescan (pass.rs:194-207), mirrored by
  the checker (`check_stmt.rs:1126`). A read before the textual assignment is still Local (→ the
  checker/runtime raises `UnboundLocalError`, not a walk to the outer name).
- **Class scope is not lexical**: methods do NOT see class-body names as free vars — `lexical_parent`
  skips `class_scope_stack` (pass.rs:178). Comprehensions inside a class body likewise cannot see class
  locals except the leftmost iterable.
- **PEP 572 walrus**: comprehension walrus binds the ENCLOSING function scope and LEAKS after the
  comprehension; iteration variables stay isolated. Contract: `../closures/capture-and-scope.md`.
- **`nonlocal` never binds a module global** (pass.rs:398 stops before scope 0); **`global` in a nested
  function** reuses the scope-0 identity so all references share one SymbolId (pass.rs:371-383).
- **Star imports** (`from x import *`) bind nothing statically (pass.rs:450) — names are dynamic.
- **Forward references** work at module scope (register pass); not inside a function body.

## Known hazards

- **`resolve_module` is dead in production** — editing `Resolver` behavior changes NO shipped output;
  the checker (`types/check.rs`) is the real resolver. Fix scoping in BOTH or the change is inert.
- **Triple-implemented scoping rules** — resolver arm (pass.rs:596), checker arm
  (`check_expr.rs:1511 define_levels_up`), lowering prescan (`ast_to_hir.rs:4980`); only the scanners
  are shared. Divergence mis-scopes an outer symbol and corrupts its recorded type. Full write-up:
  `../closures/ARCHITECTURE.md` "Dual (really triple) name-resolution passes".
- **`define_in_enclosing_scope` (scope.rs:184) vs `define_levels_up` (scope.rs:196)** — single-comp
  walrus vs NESTED comps; using the 1-level form under nested comprehensions binds a popped comp scope.
- **SymbolId numbering is per-compilation and dense** — do NOT assume it matches runtime cell keys;
  runtime disambiguates by `(module, SymbolId)` (`closure.rs::ScopedSymbolKey`, cross-module collisions
  handled in the runtime, not here). See `../closures/ARCHITECTURE.md`.
- **`pop_scope` fallback** (scope.rs:104): empty `scope_return_stack` falls back to `parent`, then
  no-ops at root — a mismatched push/pop silently strands `current_scope`, not a panic.
- **Attribute/subscript assignment defines no symbol** (pass.rs:169) — `obj.x = v` resolves `obj` only.

## Extension points

| To add… | Plug in at |
|---|---|
| new binding-expression form | ALL passes: scanner (pass.rs:964), resolver arm (pass.rs:596), checker arm (`check_expr.rs:1511`), lowering prescans (`ast_to_hir.rs:4743/4980`) — see closures ext table |
| new scope kind | push via `push_scope_with_parent` (scope.rs:97); register in `function_scope_stack` (real fn boundary) or `class_scope_stack` (executable-but-not-lexical) |
| new statement that binds names | add an arm to `collect_assignment_targets` (pass.rs:726) AND `resolve_stmt` (pass.rs:151) |
| new VariableClass consumer | `get_var_class` (scope.rs:153) + `nonlocal_mapping` (scope.rs:222) |

## EC surface

- `tests/cpython/_regression/core/{scope_resolution,scope_modifiers,comprehension_scope,walrus,closure_capture}/`
  — behavior/errors/surface for local/free/cell/global/nonlocal and comprehension isolation.
- `tests/cpython/behavior/pep/572/` (+ `errors/`, `surface/`) — walrus placement/leak/isolation
  (shared with `../closures/`).
- Rust unit tests: `resolve::pass::tests` (pass.rs:1250+, ~60 cases) and `resolve::scope::tests`
  (scope.rs:238) — the ONLY exercise of the standalone `Resolver`.

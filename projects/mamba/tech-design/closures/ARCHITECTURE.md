# closures — architecture (as-is, 2026-07-15)

Scope (per `tech-design/README.md` context map): capture cells, scoping, walrus, introspection of
captures. Source: `src/runtime/closure.rs` (runtime), `src/resolve/pass.rs` + `src/types/check_expr.rs`
(name resolution), `src/lower/{ast_to_hir,hir_to_mir}.rs` (capture lowering).

## Responsibilities

- Runtime closure objects: creation, cell-backed captures, defaults/arity, callable metadata
  (`__name__`/`__qualname__`/`__doc__`/`__module__`/`__wrapped__`) — `runtime/closure.rs::MbClosure`.
- Cell-based variable sharing between enclosing and nested scopes (nonlocal, factory-shared cells),
  including deferred bodies (generators/coroutines) and cross-thread snapshots.
- Active-scope bookkeeping at call time: which module and which cells are "current" for free-variable
  and global reads/writes (`ACTIVE_CELLS`, `ACTIVE_MODULE_NAMES`).
- Compile-time scoping decisions: local vs free vs cell-backed; PEP 572 walrus target placement.
- Capture-introspection contract: read the closure's own arrays, never active-scope state
  (see `capture-and-scope.md` §Introspection reads cells directly).

## Key structures & invariants

| Structure | Where | Invariant |
|---|---|---|
| `MbClosure` | closure.rs:24 | `capture_ids` (SymbolIds) 1:1 positional with `capture_cells` (cell handles); `captures` is a creation-time value snapshot, cells are the live truth |
| `CLOSURES` slab | closure.rs:62 | thread-local `Vec<Option<MbClosure>>`; handle = NaN-boxed int ≥ `CLOSURE_HANDLE_BASE` (1<<39); cells ≥ `CELL_ID_BASE` (1<<38, slab closure.rs:1477) |
| `ACTIVE_CELLS` | closure.rs:64 | thread-local `HashMap<ScopedSymbolKey, cell>` — THE (module, SymbolId) → live-cell map for the current activation; mutated only via save/restore in scope wrappers |
| `ScopedSymbolKey{module,symbol}` | closure.rs:18 | raw SymbolIds are unique only WITHIN one module compilation (numbering restarts per module, closure.rs:1744 comment); the module field disambiguates |
| `ACTIVE_MODULE_NAMES` | closure.rs:66 | stack; `current_active_module_name` (closure.rs:213) defaults `"__main__"`; `scoped_symbol_key` (closure.rs:234) reads the top |
| checker-owned `SymbolTable` | types/check.rs:526,713 | `TypeChecker` builds its OWN table during `check_module`; THIS table feeds lowering (`ast_to_hir.rs:3867 self.checker.symbols.lookup`), NOT `resolve/pass.rs` output (`resolve_module` has no production callers — unit tests only) |
| `cell_override_syms` | ast_to_hir.rs:3552 | SymbolIds needing cell storage (shared outer/inner); drained into `HirFunction.captures` sorted by id (ast_to_hir.rs:5023) |
| `reserved_local_syms` | ast_to_hir.rs:3549 | #1053 whole-body prescan: every eventually-assigned local gets its SymbolId at function entry so an early nested def and a later assignment share one symbol/cell |
| `user_func_freevars` | hir_to_mir.rs:1805 | func_sym → freevars; non-empty ⇒ every function-value materialization routes through `emit_closure_for_func` (hir_to_mir.rs:12939) |
| `initialized_capture_cells` / `pending_let_cell_syms` | hir_to_mir.rs:1752 / `collect_let_target_cell_syms` hir_to_mir.rs:560 | only `HirStmt::Let` (first textual plain assignment) may reset a cell; aug-assign/for/with/walrus always `emit_capture_cell_set` (mutate in place) |

## Control flow

1. **Check (production name resolution)** — `TypeChecker::check_module`: function entry prescan via the
   shared scanners (`resolve::pass::collect_assignment_targets` + `collect_walrus_targets_in_stmts`)
   marks all assigned names local; walrus arm check_expr.rs:1511 binds in current scope, or escapes ALL
   comprehension scopes via `define_levels_up(comprehension_depth)` (PEP 572).
2. **Lower AST→HIR** — repeats the prescan (ast_to_hir.rs:4980-4985 → `local_assigned_names`); #1053
   reserves SymbolIds up front; free reads of enclosing locals become `cell_override_syms`
   (ast_to_hir.rs:7419-7436); on body end → `HirFunction.captures` (ast_to_hir.rs:5023,5069).
3. **Lower HIR→MIR** — def site: `emit_closure_for_func` (hir_to_mir.rs:12939) emits
   `mb_closure_new_with_cells(name, FuncRef, [boxed SymbolIds])` then `mb_func_prime_name` (define-time
   name/qualname/module); #1053 pre-vivifies EMPTY cells first (hir_to_mir.rs:5489-5495) for captured
   locals whose `Let` hasn't run. Body cell writes → `emit_capture_cell_{set,reset,reset_empty}`.
4. **Runtime creation** — `mb_closure_new_with_cells` (closure.rs:327): each id → `active_cell_for_id`
   (closure.rs:1640): ACTIVE_CELLS hit, else new cell seeded from `GLOBAL_ID_NAMESPACE`. Closures from
   the same factory activation share cell handles by design (closure.rs:41-43).
5. **Runtime call** — dispatchers (`mb_call*`, builtins/mod.rs:4735,16029,16308; class/mod.rs:3107) wrap:
   `with_callable_module` (closure.rs:272) pushes the callee's define-time `__module__` + qualname
   context (Drop-guarded) → `with_closure_cells` (closure.rs:546) installs the closure's cells into
   ACTIVE_CELLS under scoped keys (save/restore) → body resolves cell vars via
   `active_cell_get_id_raw`/`active_cell_set_id_raw` (closure.rs:1690,1718).
6. **Deferred bodies** — generators/coroutines snapshot (key, cell) pairs at construction
   (`capture_active_cell_context`, closure.rs:609; externs `mb_generator_capture_cells` /
   `mb_coroutine_capture_cells`, hir_to_mir.rs:2613/3066) and reinstall per resume
   (`with_captured_cell_context`, closure.rs:620). Threads: `snapshot/replace/merge_active_cells`
   (closure.rs:2086-2101).
7. **Introspection** — `closure_capture_value_for_id` (closure.rs:530): position in `capture_ids` →
   direct cell read; `mb_global_get_id_raw` only as defensive fallback (241 TD).

## Known hazards

- **Active-module lookup inside native dispatchers** — `with_callable_module` makes the DISPATCHER's own
  `__module__` (e.g. "inspect") active for the call's duration; any user-scope name resolved through
  ACTIVE_CELLS / `mb_global_get_id_raw` in that window reads the wrong scope and comes back unset. →
  `capture-and-scope.md` §Introspection reads cells directly.
- **Call-time module fallback (#239)** — `callable_module_name` (closure.rs:251) falls back to "whatever
  module is currently active" when a callable has no registered `__module__`; wrong for callbacks invoked
  from another module's dispatch code, corrupting every global read/write. Hence `mb_func_prime_name`
  (closure.rs:1175) MUST capture the module at define time — never derive it at call time.
- **Dual (really triple) name-resolution passes** — scoping rules are implemented independently in the
  resolver walrus arm (pass.rs:596: bind in `function_scope_stack` top) and the checker walrus arm
  (check_expr.rs:1511: `define_levels_up`), plus the lowering prescan (ast_to_hir.rs:4980). Only the
  scanners (`collect_walrus_targets*`, pass.rs:964) are shared. Divergence re-defines an outer symbol in
  the wrong scope and corrupts its recorded type (check_expr.rs:1516-1520 documents outer `i: int`
  flipped to float by an inner walrus under the old always-enclosing rule).
- **Late-assigned capture / disconnected cell (#1053)** — a nested def capturing an enclosing local
  assigned later textually must snapshot the SAME cell the later `Let` writes; without the empty-cell
  pre-vivify + `initialized_capture_cells` bookkeeping (hir_to_mir.rs:5478-5496), the `Let` installs a
  fresh cell that orphans the handle the closure already captured.
- **Empty cell ≠ Python None** — reading an empty cell raises NameError directly in
  `active_cell_get_id_raw` (closure.rs:1693-1711); falling through to the missing-global path would
  silently yield `None` because `missing_global_should_raise_name_error()` is normally off.
- **Cross-module SymbolId collision (#983)** — raw ids restart per compilation; `ACTIVE_MODULE_SYM_IDS`
  (closure.rs:1739) brackets nested imports so namespace merging can't donate a finished module's global
  into a numerically-colliding outer slot.
- **Conditional walrus** — `and`/`or` RHS walrus may never execute; the checker invalidates those
  bindings (`invalidate_conditional_binding_names`, check_expr.rs:548); f-string walrus leaks its binding
  but suppresses field type errors (check_expr.rs:497-515).
- **Closure handle is an int** — dispatch paths that compute addresses from a method value must unwrap
  handles (`extract_registered_func_addr`), or construction silently no-ops. →
  `../object-model/class-construction.md` §Instance construction.
- **Unwind-safety asymmetry** — `with_callable_module` restores via a Drop guard; `with_closure_cells`
  (closure.rs:572-589) restores only after `call()` returns — a Rust unwind through the body skips the
  ACTIVE_CELLS restore. All state is thread-local; cross-thread use requires explicit snapshot/merge.

## Extension points

| To add... | Plug in at |
|---|---|
| new binding-expression form (walrus-like) | ALL passes: scanner pass.rs:964, resolver arm pass.rs:596, checker arm check_expr.rs:1511, lowering prescans ast_to_hir.rs:4743/4980 |
| new capture-introspection API | direct-array pattern `closure_capture_value_for_id` (closure.rs:530); never active-module state |
| new deferred-execution construct | `capture_active_cell_context`/`with_captured_cell_context` (closure.rs:609/620) + a `mb_<thing>_capture_cells` extern mirroring hir_to_mir.rs:2613 |
| new per-callable metadata | `MbClosure` field + `FUNC_*` side-table pair (closure.rs:~1140-1240); prime in `mb_func_prime_name` if define-time |
| new call-dispatch path | wrap in `with_callable_module` + `with_closure_cells` (pattern: builtins/mod.rs:5831); unwrap handles per 1594 TD |
| cross-thread execution | `snapshot/replace/merge_active_cells` (closure.rs:2086-2101) |

## EC surface

Per `external-contracts/README.md` closures row: positive contract = `pep/572` + capture-introspection
fixtures; this domain owns no `type/` walls. Gate: `cargo test -p mamba --release --test conformance`.

- `tests/cpython/behavior/pep/572/` (20 fixtures: comprehension leak, genexp deferral, conditional
  binds), plus `errors/pep/572/` and `surface/pep/572/`.
- `tests/cpython/_regression/core/closure_capture/` (behavior, closure_late_binding, errors, surface)
  and `_regression/core/{scope_resolution,scope_modifiers,comprehension_scope}`.
- `tests/cpython/behavior/std-libs/inspect/getclosurevars_reports_nonlocals_and_builtins.py`
  (byte-identical gate for 241).
- Rust lib tests: `runtime::closure::` (cited by 241 TD) and lowering capture asserts (e.g.
  hir_to_mir.rs:12126 class-cell capture).

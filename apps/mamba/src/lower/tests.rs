//! Colocated unit tests for `hir_to_mir`'s module-scope `for _ in range(...)`
//! lowering (issue #4242).
//!
//! `lower_for_range` (the fast-path `for` lowering used whenever the
//! iterable is a literal `range(...)` call) suppresses `in_module_scope`
//! unconditionally while lowering the loop body, unlike the general
//! `lower_for` path, which only suppresses it `if !self.module_has_closures`.
//! When a module defines both a `def` (so `module_has_closures` is true) and
//! a module-scope `for _ in range(n): acc += 1` loop, every write to `acc`
//! inside the loop body goes through `HirStmt::Assign`'s `orig_vreg` branch,
//! which only emits the paired `MirInst::StoreGlobal` when
//! `self.in_module_scope` is true. Because `lower_for_range` always clears
//! `in_module_scope`, that per-iteration `StoreGlobal` is dropped, and the
//! post-loop `print(acc)` (a `LoadGlobal` read, since a `def` is present)
//! observes only the pre-loop value.
//!
//! This test does not run the compiled program (that observation is owned by
//! the e2e rollup, `apps/mamba/e2e/tier1_exit.rs`); it pins the MIR shape
//! that has to hold for the runtime behavior to be correct: at least one
//! `StoreGlobal` targeting `acc` must be emitted inside the range-loop body.

use crate::lower::hir_to_mir::lower_hir_to_mir_with_symbols;
use crate::lower::lower_module;
use crate::mir::MirInst;
use crate::parser::parse;
use crate::source::FileId;
use crate::types::TypeChecker;

/// The minimal reproduction from the #4242 core fixture
/// (`apps/mamba/e2e/tier1/core/module_loop_rebinds_global_with_def_present.py`):
/// a module that defines a function (so `module_has_closures` is set) and
/// then accumulates into a module-scope global inside a `for _ in range(n)`
/// loop. The lowered MIR must emit a `StoreGlobal` for `acc` inside the loop
/// body -- without it, the post-loop read observes the stale pre-loop value.
#[test]
fn for_range_loop_stores_global_when_module_has_closures() {
    let src = r#"
def f() -> None:
    pass

acc = 0
for _ in range(3):
    acc += 1
print(acc)
"#;
    let module = parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).expect("HIR lowering failed");

    let acc_sym = checker
        .symbols
        .lookup("acc")
        .expect("expected module-scope `acc` symbol");

    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    let store_global_count = mir
        .bodies
        .iter()
        .flat_map(|body| body.blocks.iter())
        .flat_map(|block| block.stmts.iter())
        .filter(|stmt| matches!(stmt, MirInst::StoreGlobal { name, .. } if *name == acc_sym))
        .count();

    // One store for the module body's `acc = 0` plus at least one more from
    // inside the loop body (three iterations would ideally emit three, but
    // the invariant this test protects is "at least one", i.e. the loop body
    // is not fully suppressing global sync once a closure is present).
    assert!(
        store_global_count >= 2,
        "expected at least 2 StoreGlobal writes for `acc` (one for `acc = 0`, \
         at least one from inside the `for _ in range(3)` body) when the module \
         defines a function elsewhere, got {store_global_count}"
    );
}

/// The `with … as name` alias binding inside a function must allocate a fresh
/// function-local symbol when the enclosing function assigns that name, rather
/// than resolving to a same-named module-level binding.
///
/// `ast::Stmt::With`'s alias arm binds through `resolve_name` with a
/// `define_local` fallback, so when a module-level `fh` already exists the
/// alias inside `worker` reuses the *module* symbol. `hir_to_mir` then classes
/// that symbol `VariableClass::Global` and emits a `StoreGlobal` for the
/// `__enter__` result, while the body's read of `fh` still goes through the
/// function's local slot — which was never written. The program dies with
/// `UnboundLocalError: cannot access local variable 'fh' …`.
///
/// This is the whole cause of the red
/// `apps/mamba/e2e/tier1/concurrency/identity/io/child_opened_file_writes_in_child.py`
/// fixture: its module scope runs `with open(path) as fh:` after the worker
/// function that also binds `fh`. The defect has nothing to do with threads —
/// the program below has none — so this test pins the scoping rule directly.
#[test]
fn with_alias_in_function_does_not_reuse_module_symbol() {
    let src = r#"
class CM:
    def __enter__(self) -> int:
        return 7
    def __exit__(self, a: object, b: object, c: object) -> bool:
        return False

fh = 1

def worker() -> int:
    with CM() as fh:
        return fh
"#;
    let module = parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).expect("HIR lowering failed");

    let module_fh = checker
        .symbols
        .lookup("fh")
        .expect("expected module-scope `fh` symbol");

    let alias = hir
        .functions
        .iter()
        .find_map(|f| first_with_alias(&f.body))
        .expect("expected a `with … as fh` alias inside a lowered function");

    assert_ne!(
        alias, module_fh,
        "`with CM() as fh` inside `worker` bound the module-level `fh` symbol \
         ({module_fh:?}) instead of a fresh function local; hir_to_mir then \
         stores the __enter__ result with StoreGlobal while the body reads the \
         unwritten local slot, which is the UnboundLocalError"
    );
}

/// First `with … as name` alias symbol found anywhere in a statement list.
fn first_with_alias(stmts: &[crate::hir::HirStmt]) -> Option<crate::resolve::SymbolId> {
    use crate::hir::HirStmt;
    for stmt in stmts {
        match stmt {
            HirStmt::With { items, body, .. } => {
                if let Some(sym) = items.iter().find_map(|(_, alias)| *alias) {
                    return Some(sym);
                }
                if let Some(found) = first_with_alias(body) {
                    return Some(found);
                }
            }
            HirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if let Some(found) = first_with_alias(then_body).or_else(|| first_with_alias(else_body)) {
                    return Some(found);
                }
            }
            HirStmt::While { body, .. } => {
                if let Some(found) = first_with_alias(body) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

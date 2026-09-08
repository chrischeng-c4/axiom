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

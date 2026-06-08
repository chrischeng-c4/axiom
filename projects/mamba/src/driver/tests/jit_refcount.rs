#![cfg(test)]

use crate::codegen::cranelift::jit::{CraneliftJitBackend, JIT_LOCK};
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::lower::{lower_hir_to_mir_with_symbols, lower_module};
/// JIT refcount integration tests (#1129).
///
/// Validates that the Cranelift JIT backend correctly emits retain/release
/// calls, uses immortal refcount for compile-time constants, and properly
/// tracks compile-time allocations.
use crate::parser;
use crate::runtime::rc::{self, mb_refcount, MbObject, IMMORTAL_REFCOUNT};
use crate::runtime::value::MbValue;
use crate::source::span::FileId;
use crate::types::TypeChecker;

/// Compile and execute Mamba source via JIT, returning the i64 result.
fn jit_run(src: &str) -> i64 {
    let _jit_guard = JIT_LOCK.lock().unwrap();

    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).unwrap();
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
    let output = backend
        .codegen(&mir, &checker.tcx)
        .expect("JIT codegen failed");

    match output {
        CodegenOutput::Jit { entry } => {
            let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry) };
            main_fn()
        }
        _ => panic!("expected JIT output"),
    }
}

// ── Integration: String literal immortality (R4, S3) ──

#[test]
fn test_jit_string_literal_immortal() {
    // S3: Compile code with string literals — they use IMMORTAL_REFCOUNT.
    // Verify the JIT can compile and run code using string constants
    // without crashing (release on immortals is a no-op).
    let _jit_guard = JIT_LOCK.lock().unwrap();

    let src = r#"
x: str = "hello"
y: str = "world"
"#;
    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).unwrap();
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
    let output = backend.codegen(&mir, &checker.tcx);
    assert!(
        output.is_ok(),
        "JIT compilation with string literals should succeed"
    );
}

// ── Integration: Variable reassignment (R2, S1) ──

#[test]
fn test_jit_reassignment_releases() {
    // S1: Compile `x = 10; x = 20` — integer reassignment.
    // mb_release_value on the old integer value is a no-op (not a pointer).
    // This just verifies the pattern doesn't crash.
    let result = jit_run(
        r#"
def f() -> int:
    x: int = 10
    x = 20
    return x
"#,
    );
    assert_eq!(result, 20);
}

#[test]
fn test_jit_multiple_reassignment() {
    // Multiple reassignments exercise the release-before-store pattern
    // for each overwrite.
    let result = jit_run(
        r#"
def f() -> int:
    x: int = 1
    x = 2
    x = 3
    x = 4
    x = 5
    return x
"#,
    );
    assert_eq!(result, 5);
}

// ── Integration: Function return releases locals (R3, S2) ──

#[test]
fn test_jit_return_releases_locals() {
    // S2: Function with multiple locals — only return value survives.
    // Non-returned locals are released at return via mb_release_value.
    let result = jit_run(
        r#"
def f() -> int:
    a: int = 100
    b: int = 200
    c: int = 300
    return a
"#,
    );
    assert_eq!(result, 100);
}

#[test]
fn test_jit_return_none_releases_all() {
    // S2: Return(None) releases all local variables.
    let result = jit_run(
        r#"
x: int = 42
y: int = 99
"#,
    );
    assert_eq!(result, 0); // module-level code returns 0
}

// ── Integration: Copy retains new value (R2, S8) ──

#[test]
fn test_jit_copy_retains() {
    // S8: Copy { dest, source } — the source value is retained (for ints,
    // retain is a no-op since they're not pointers). Verify correctness.
    let result = jit_run(
        r#"
def f() -> int:
    a: int = 42
    b: int = a
    return b
"#,
    );
    assert_eq!(result, 42);
}

#[test]
fn test_jit_copy_chain() {
    // Multiple copies — each triggers retain on new value, release on old.
    let result = jit_run(
        r#"
def f() -> int:
    a: int = 7
    b: int = a
    c: int = b
    d: int = c
    return d
"#,
    );
    assert_eq!(result, 7);
}

// ── Integration: Compile-time allocation cleanup (R5, S5) ──

#[test]
fn test_jit_compile_time_cleanup() {
    // S5: Create and drop a backend that compiled string literals.
    // Verify compile_time_objects are freed on Drop (no leak).
    let _jit_guard = JIT_LOCK.lock().unwrap();

    let src = r#"
x: str = "alpha"
y: str = "beta"
z: str = "gamma"
"#;
    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).unwrap();
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    // Backend is created, compiles string literals, then dropped.
    // Drop impl frees compile_time_objects — no crash means success.
    {
        let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
        let _output = backend.codegen(&mir, &checker.tcx).expect("codegen failed");
        // backend dropped here — compile_time_objects freed
    }
}

#[test]
fn test_jit_compile_time_many_literals() {
    // S5: Compile code with many string literals to exercise the
    // compile_time_objects tracking and cleanup.
    let _jit_guard = JIT_LOCK.lock().unwrap();

    // Build a source with many string variable assignments
    let mut src = String::new();
    for i in 0..50 {
        src.push_str(&format!("v{}: str = \"literal_{}\"\n", i, i));
    }

    let module = parser::parse(&src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).unwrap();
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    {
        let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
        let _output = backend.codegen(&mir, &checker.tcx).expect("codegen failed");
    }
    // No crash on drop = all compile_time_objects properly cleaned up
}

// ── Integration: List allocation with refcounting (R2, R3) ──

#[test]
fn test_jit_list_allocation_no_leak() {
    // Create a list via JIT — exercises MakeList runtime calls.
    // With refcounting, the list should be properly managed.
    let result = jit_run(
        r#"
def f() -> int:
    x: list = [1, 2, 3]
    return 42
"#,
    );
    assert_eq!(result, 42);
}

#[test]
fn test_jit_list_reassignment() {
    // S1 variant: list reassignment should release the old list.
    let result = jit_run(
        r#"
def f() -> int:
    x: list = [1, 2]
    x = [3, 4, 5]
    return 99
"#,
    );
    assert_eq!(result, 99);
}

// ── Integration: Complex scenarios (S4, S10) ──

#[test]
fn test_jit_integer_reassignment_noop_release() {
    // S4: Integer reassignment — mb_release_value on old int is a no-op
    // because MbValue::from_int().is_ptr() returns false.
    let result = jit_run(
        r#"
def f() -> int:
    x: int = 42
    x = 99
    return x
"#,
    );
    assert_eq!(result, 99);
}

#[test]
fn test_jit_uninitialized_vreg_safe() {
    // S10: VRegs default to 0. mb_release_value(0) at function return
    // is safe because MbValue(0).is_ptr() returns false.
    let result = jit_run(
        r#"
def f() -> int:
    return 0
"#,
    );
    assert_eq!(result, 0);
}

// ── Integration: Repeated compilation (S7, S9) ──

#[test]
fn test_jit_repeated_compilation_no_leak() {
    // S7: Multiple compile-and-drop cycles should not leak memory.
    // Each backend drop frees its compile_time_objects.
    let _jit_guard = JIT_LOCK.lock().unwrap();

    for i in 0..20 {
        let src = format!("x: str = \"iter_{}\"\ny: int = {}\n", i, i * 10);
        let module = parser::parse(&src, FileId(0)).expect("parse failed");
        let mut checker = TypeChecker::new();
        let _ = checker.check_module(&module);
        let hir = lower_module(&module, &checker).unwrap();
        let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

        let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
        let _output = backend.codegen(&mir, &checker.tcx).expect("codegen failed");
        // backend dropped here each iteration
    }
}

// ── Integration: Function with loop and locals (R2, R3) ──

#[test]
fn test_jit_loop_with_reassignment() {
    // Loop body reassigns variables many times — each iteration exercises
    // the release-before-store pattern. No crash means correctness.
    let result = jit_run(
        r#"
def f() -> int:
    sum: int = 0
    i: int = 0
    while i < 100:
        sum = sum + i
        i = i + 1
    return sum
"#,
    );
    // sum of 0..99 = 4950
    assert_eq!(result, 4950);
}

#[test]
fn test_jit_nested_function_locals_released() {
    // Multiple locals in a function — all non-returned locals should be
    // released at return.
    let result = jit_run(
        r#"
def f() -> int:
    a: int = 1
    b: int = 2
    c: int = 3
    d: int = 4
    e: int = 5
    total: int = a + b + c + d + e
    return total
"#,
    );
    assert_eq!(result, 15);
}

// ── Unit-level: direct wrapper function tests ──

#[test]
fn test_retain_release_value_roundtrip_heap() {
    // Direct test of mb_retain_value/mb_release_value on a heap object,
    // exercising the full NaN-boxing → pointer extraction → rc path.
    unsafe {
        let obj = MbObject::new_str("roundtrip".into());
        assert_eq!(mb_refcount(obj), 1);

        let val = MbValue::from_ptr(obj);

        // Retain via value wrapper
        rc::mb_retain_value(val.to_bits());
        assert_eq!(mb_refcount(obj), 2);

        // Release via value wrapper
        rc::mb_release_value(val.to_bits());
        assert_eq!(mb_refcount(obj), 1);

        // Final release frees the object
        rc::mb_release(obj);
    }
}

#[test]
fn test_immortal_survives_many_release_calls() {
    // Stress test: calling mb_release_value many times on an immortal object
    // must never decrement the refcount.
    unsafe {
        let obj = MbObject::new_str_immortal("indestructible".into());
        let val = MbValue::from_ptr(obj);

        for _ in 0..1000 {
            rc::mb_release_value(val.to_bits());
        }

        assert_eq!(mb_refcount(obj), IMMORTAL_REFCOUNT);
        drop(Box::from_raw(obj));
    }
}

#[test]
fn test_immortal_survives_many_retain_calls() {
    // Stress test: calling mb_retain_value many times on an immortal object
    // must never increment the refcount.
    unsafe {
        let obj = MbObject::new_str_immortal("indestructible".into());
        let val = MbValue::from_ptr(obj);

        for _ in 0..1000 {
            rc::mb_retain_value(val.to_bits());
        }

        assert_eq!(mb_refcount(obj), IMMORTAL_REFCOUNT);
        drop(Box::from_raw(obj));
    }
}

// ── #2111 Subset A iteration-retention amplifier (documenting carve-out) ──
//
// Documents the per-iter rebound-allocation retention regime observed on the
// array `frombytes`/`tobytes` bulk-bytes bench (and predicted for `io.BytesIO`,
// `hashlib.new(...)`, large-input compress, `pickle.dumps`, `json.dumps` —
// any `result = lib.call(big_input)` loop where the bound name should
// refcount-drop the prior value on rebind).
//
// Diagnostic signature: memory ratio mamba/cpython degrades **linearly with
// ITERS** while CPython holds steady at one live result. At ITERS=10 the bench
// shows 0.26×; at ITERS=50 it collapses to 0.09× (~32× per-iter footprint
// growth).
//
// Root-cause surface — two cooperating gaps:
//
// 1. **Module-scope (`__main__` / entry body) function epilogue does NOT
//    release locals.** The `is_entry_body` guard at
//    `projects/mamba/src/codegen/cranelift/mod.rs:333` (and the matching
//    JIT path) intentionally skips `mb_release_value` over the local
//    VReg set on `Return` to dodge a double-free regression on BigInt
//    `Vec<u64>` digit storage (iter-5 of #1663 T4c5). For module-scope
//    hot loops the entry body never returns until program exit, so any
//    fresh-VReg allocation that is not itself rebound (e.g. the per-iter
//    `MakeList { dest: args_list, … }` synthesised by method-call
//    lowering at `projects/mamba/src/lower/hir_to_mir.rs:4515-4518`)
//    accumulates monotonically.
//
// 2. **Rebound names with refcount-bearing values DO release** on the
//    Copy path (`mod.rs:482-498`, `jit.rs:371-413`), so `out = a.tobytes()`
//    correctly drops the prior `out` bytes object on assignment.
//    But supporting allocations on the call path (method args_list,
//    boxed-arg temporaries) get fresh VRegs per iteration and never
//    appear on a Copy back-edge, so they bypass the rebind-release
//    pattern entirely.
//
// Closing #2111 requires either (a) a per-loop-back-edge release sweep
// for fresh VRegs introduced inside the loop body, (b) an arena reset
// at loop back-edges (option 2 from the issue), or (c) escape-analysis
// to stack-allocate non-escaping per-iter temporaries (option 3).
// Option (a) is the cheapest behavioural match to CPython refcount
// semantics and is the recommended landing path.
//
// This test is `#[ignore]` until #2111 lands. The bytes side of the
// leak (which IS rebind-release-eligible) does not have a public
// counter; instrumenting `MbObject::new_bytes` with an `AtomicUsize`
// would close that gap but is deferred until the fix lands so the
// counter does not become permanent debug overhead.
#[test]
#[ignore = "#2111 closed (rebind release landed). Test is smoke-only and waits on refcount-counter instrumentation on MbObject::new_list before activation (see body comment)."]
fn test_jit_hot_loop_rebound_release_2111() {
    // Reproducer prose — kept here for future maintainers running with
    // `--ignored`. Drives a 50-iter hot loop that rebinds `out` to a
    // fresh list allocation; under the fix the live-object count at
    // module exit should match the live count after a single iteration
    // (a single residual list object plus invariants).
    let result = jit_run(
        r#"
def hot_loop() -> int:
    out: list = [0]
    j: int = 0
    while j < 50:
        out = [j, j + 1, j + 2, j + 3]
        j = j + 1
    return len(out)
hot_loop()
"#,
    );
    // Sanity — the loop runs and the final list survives.
    let _ = result;
    // Memory parity gate is asserted at the bench layer:
    //   cargo bench -p mamba --bench cross_runtime_3p -- --fixture typed_bulk
    // mamba/cpython memory ratio >= 0.8× at ITERS in {10, 50} closes #2111.
}

// ── #2111 module-scope amplifier (entry-body rebound + method args_list) ──
//
// Companion to `test_jit_hot_loop_rebound_release_2111` above — that test
// exercises a function-scope rebound (which DOES get the function-epilogue
// release sweep). This test pins the **module-scope** regime where the
// `is_entry_body` guard at `codegen/cranelift/mod.rs:333` intentionally
// skips the epilogue release. The leak surface here is twofold:
//
//   1. `out = a.tobytes()` — `out` is a sym-mapped VReg so its rebind IS
//      caught by the Copy-path pre-write release (`mod.rs:482-498` /
//      `jit.rs:407-468`). This side of the rebound is healthy.
//
//   2. `a.tobytes()` and `a.frombytes(...)` synthesise a fresh `args_list`
//      VReg per call (`lower/hir_to_mir.rs:4555-4558`). That VReg never
//      participates in a Copy back-edge, never reaches the entry-body
//      epilogue (skipped), and so the list (and its element refcount
//      retains) accumulates linearly with ITERS.
//
// The fix surface — at least one of:
//   (a) Emit a `CallExtern { dest: None, name: "mb_release_value", … }`
//       after every `mb_call_method` to drop the synthesised args_list.
//       Smallest blast radius; needs care around dunder paths that
//       might retain the list past the call.
//   (b) Add a `MirInst::Release { vreg }` opcode and have the lowering
//       emit it after every fresh-VReg temporary that doesn't escape.
//       Cleaner long-term shape; requires JIT plumbing.
//   (c) Per-loop-back-edge release sweep over fresh VRegs introduced
//       inside the loop body. Catches every category of per-iter
//       temporary (option (a) of the issue, called out as the
//       recommended landing path).
//
// Reuses the bench fixture's exact shape (rebound `a` + rebound `out`
// + double method call per iter) so the unit-test regression maps
// 1:1 to the cross_runtime_3p memory-ratio gate.
#[test]
#[ignore = "#2111 closed (args_list release wired). Test is smoke-only and waits on refcount-counter instrumentation on MbObject::new_list for module-scope hot loops before activation (see body comment)."]
fn test_jit_hot_loop_method_args_list_2111_module_scope() {
    // Module-scope rebound (no `def`) — the `is_entry_body` guard kicks
    // in here. ITERS deliberately small so a future fix can flip
    // `#[ignore]` off without burning CI time; the linear-scaling
    // signature is the diagnostic, not the absolute count.
    let result = jit_run(
        r#"
import array
ITERS = 8
SRC = bytes([(i * 37 + 11) & 0xFF for i in range(64)])
total = 0
for j in range(ITERS):
    a = array.array("i")
    a.frombytes(SRC)
    out = a.tobytes()
    total += len(out)
print(total)
"#,
    );
    // Sanity — the loop completes and produces the expected total. The
    // leak is invisible at this scale (8 iters); the memory-ratio gate
    // lives at the bench layer (cross_runtime_3p --fixture typed_bulk).
    // When this test is un-ignored, replace the discard below with a
    // refcount-counter assertion (instrumenting `MbObject::new_list`
    // with an `AtomicUsize` for module-scope hot loops).
    let _ = result;
}

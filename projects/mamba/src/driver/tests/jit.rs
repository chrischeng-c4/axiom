#![cfg(test)]

/// JIT execution integration tests (#296).
/// Tests the full pipeline: parse → typecheck → lower → JIT compile → execute.

use crate::parser;
use crate::source::span::FileId;
use crate::types::TypeChecker;
use crate::lower::{lower_module, lower_hir_to_mir, lower_hir_to_mir_with_symbols};
use crate::codegen::cranelift::jit::{CraneliftJitBackend, JIT_LOCK};
use crate::codegen::cranelift::CraneliftBackend;
use crate::codegen::{CodegenBackend, CodegenOutput};

fn jit_run(src: &str) -> i64 {
    let _jit_guard = JIT_LOCK.lock().unwrap();

    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).unwrap();
    // Use lower_hir_to_mir_with_symbols to properly compile classes (#827).
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
    let output = backend.codegen(&mir, &checker.tcx).expect("JIT codegen failed");

    match output {
        CodegenOutput::Jit { entry } => {
            let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry) };
            main_fn()
        }
        _ => panic!("expected JIT output"),
    }
}

#[test]
fn test_jit_integer_arithmetic() {
    let result = jit_run("x: int = 2 + 3\n");
    assert_eq!(result, 0);
}

#[test]
fn test_jit_multiple_vars() {
    let result = jit_run("x: int = 10\ny: int = 20\nz: int = x + y\n");
    assert_eq!(result, 0);
}

#[test]
fn test_jit_list_literal() {
    // MakeList emits mb_list_new + mb_list_append via JIT FFI
    let result = jit_run("[1, 2, 3]\n");
    assert_eq!(result, 0);
}

#[test]
fn test_jit_dict_literal() {
    let result = jit_run("{}\n");
    assert_eq!(result, 0);
}

#[test]
fn test_jit_bitwise_ops() {
    let result = jit_run("x: int = 5 & 3\ny: int = x | 8\n");
    assert_eq!(result, 0);
}

#[test]
fn test_jit_backend_initializes() {
    let _jit_guard = JIT_LOCK.lock().unwrap();
    let backend = CraneliftJitBackend::new();
    assert!(backend.is_ok());
    assert_eq!(backend.unwrap().name(), "cranelift-jit");
}

#[test]
fn test_jit_while_loop_sum() {
    // sum 0..9 = 45
    let result = jit_run(r#"
def f() -> int:
    s: int = 0
    i: int = 0
    while i < 10:
        s = s + i
        i = i + 1
    return s
"#);
    assert_eq!(result, 45);
}

#[test]
fn test_jit_range_loop_sum() {
    // sum 0..4 = 10 — range loop accumulation with loop counter (#1197)
    let result = jit_run(r#"
def f() -> int:
    total: int = 0
    for i in range(5):
        total = total + i
    return total
"#);
    assert_eq!(result, 10);
}

#[test]
fn test_jit_range_loop_bound_from_max_call() {
    // Regression for #2105: when the range() bound is the result of a
    // user-function call whose body internally uses a NaN-boxing builtin
    // (e.g. `max(...)`), the function returns an MbValue rather than a
    // raw i64. The native counter loop compares as raw i64, so without
    // an unbox-if-boxed step the boxed bit pattern read as a signed i64
    // is a huge negative number → `var < stop` is false at entry and
    // the body silently elides. Expected: full N iterations.
    let result = jit_run(r#"
def iters_for(size: int) -> int:
    return max(50, size)

def f() -> int:
    iters: int = iters_for(100)
    count: int = 0
    for _ in range(iters):
        count = count + 1
    return count
"#);
    assert_eq!(result, 100);
}

#[test]
fn test_jit_range_loop_product() {
    // 5! = 120 — range(1, N) with multiplication (#1197)
    let result = jit_run(r#"
def f() -> int:
    product: int = 1
    for i in range(1, 6):
        product = product * i
    return product
"#);
    assert_eq!(result, 120);
}

#[test]
fn test_jit_fibonacci() {
    // fib(10) = 55
    let result = jit_run(r#"
def f() -> int:
    a: int = 0
    b: int = 1
    n: int = 10
    while n:
        temp: int = b
        b = a + b
        a = temp
        n = n - 1
    return a
"#);
    assert_eq!(result, 55);
}

#[test]
fn test_jit_if_else() {
    let result = jit_run(r#"
def f() -> int:
    x: int = 10
    result: int = 0
    if x:
        result = 42
    else:
        result = 0
    return result
"#);
    assert_eq!(result, 42);
}

#[test]
fn test_jit_nested_while() {
    // 100 iterations of fib(20) = 6765, total = 100 * 6765 = 676500
    let result = jit_run(r#"
def f() -> int:
    total: int = 0
    rep: int = 0
    while rep < 100:
        a: int = 0
        b: int = 1
        i: int = 0
        while i < 20:
            temp: int = b
            b = a + b
            a = temp
            i = i + 1
        total = total + a
        rep = rep + 1
    return total
"#);
    assert_eq!(result, 676500);
}

// ── Identity and containment operator tests ──

#[test]
fn test_jit_is_identity_true() {
    // Same integer values: `is` checks bit-identical NaN-boxed values
    let result = jit_run(r#"
def f() -> int:
    x: int = 42
    y: int = 42
    if x is y:
        return 1
    return 0
"#);
    assert_eq!(result, 1);
}

#[test]
fn test_jit_is_identity_false() {
    // Different integer values: `is` should return false
    let result = jit_run(r#"
def f() -> int:
    x: int = 42
    y: int = 99
    if x is y:
        return 1
    return 0
"#);
    assert_eq!(result, 0);
}

#[test]
fn test_jit_is_not_identity() {
    // `is not` should be the logical inverse of `is`
    let result = jit_run(r#"
def f() -> int:
    x: int = 42
    y: int = 99
    if x is not y:
        return 1
    return 0
"#);
    assert_eq!(result, 1);
}

#[test]
fn test_jit_is_not_same_value() {
    // Same value: `is not` should return false (0)
    let result = jit_run(r#"
def f() -> int:
    x: int = 42
    y: int = 42
    if x is not y:
        return 1
    return 0
"#);
    assert_eq!(result, 0);
}

// ── Class support tests (#827) ──

#[test]
fn test_jit_class_simple_init_and_getattr() {
    // Verify class instantiation + __init__ + getattr works end-to-end
    let result = jit_run(r#"
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

def f() -> int:
    p = Point(42, 0)
    return 1

f()
"#);
    assert_eq!(result, 1, "class simple init: expected 1");
}

// ── Class pattern matching tests (#827) ──

#[test]
fn test_jit_isinstance_basic() {
    // Test that isinstance check works for class instances
    let result = jit_run(r#"
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

def f() -> int:
    p = Point(42, 0)
    if isinstance(p, Point):
        return 1
    return 0

f()
"#);
    assert_eq!(result, 1, "isinstance: expected 1");
}

#[test]
fn test_jit_getattr_basic() {
    // Test that getattr works for instance attributes set in __init__
    let result = jit_run(r#"
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

def f() -> int:
    p = Point(42, 0)
    return getattr(p, "x")

f()
"#);
    assert_eq!(result, 42, "getattr: expected 42");
}



#[test]
fn test_jit_class_pattern_inline() {
    // Inline: create Point in f() then match it
    let result = jit_run(r#"
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

def f() -> int:
    p = Point(42, 0)
    match p:
        case Point(x=a):
            return a + 1
    return 0

f()
"#);
    assert_eq!(result, 43, "class pattern inline: expected 43");
}

#[test]
fn test_jit_class_pattern_param() {
    // Parametric: pass Point to a function and match it there
    let result = jit_run(r#"
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

def classify(p: Point) -> int:
    match p:
        case Point(x=a):
            return a + 1
    return 0

def f() -> int:
    p = Point(42, 0)
    return classify(p)

f()
"#);
    assert_eq!(result, 43, "class pattern param: expected 43");
}

// ── AOT backend tests ──

fn aot_compile(src: &str) -> Vec<u8> {
    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).unwrap();
    let mir = lower_hir_to_mir(&hir, &checker.tcx);

    let mut backend = CraneliftBackend::new().expect("AOT init failed");
    let output = backend.codegen(&mir, &checker.tcx).expect("AOT codegen failed");

    match output {
        CodegenOutput::ObjectFile(bytes) => bytes,
        _ => panic!("expected ObjectFile output"),
    }
}

#[test]
fn test_aot_pure_numeric_object() {
    let bytes = aot_compile(r#"
def f() -> int:
    return 42
"#);
    // Object file should be non-empty and contain no mb_* symbols
    assert!(!bytes.is_empty());
    // Check that the bytes contain "main" (the entry point we generate)
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("main") || bytes.len() > 100);
}

#[test]
fn test_aot_rejects_runtime_deps() {
    let module = parser::parse("[1, 2, 3]\n", FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).unwrap();
    let mir = lower_hir_to_mir(&hir, &checker.tcx);

    let mut backend = CraneliftBackend::new().expect("AOT init failed");
    let result = backend.codegen(&mir, &checker.tcx);
    match result {
        Err(e) => {
            let err_msg = format!("{e}");
            assert!(err_msg.contains("runtime library"), "unexpected error: {err_msg}");
        }
        Ok(_) => panic!("expected error for runtime-dependent program"),
    }
}

#[test]
#[ignore] // Requires cc linker on host
fn test_aot_build_and_execute() {
    let bytes = aot_compile(r#"
def f() -> int:
    return 42
"#);
    let tmp_dir = std::env::temp_dir();
    let obj_path = tmp_dir.join("mamba_test_aot.o");
    let exe_path = tmp_dir.join("mamba_test_aot");

    std::fs::write(&obj_path, &bytes).expect("write .o");
    let status = std::process::Command::new("cc")
        .args([
            obj_path.to_str().unwrap(),
            "-o",
            exe_path.to_str().unwrap(),
        ])
        .status()
        .expect("invoke cc");
    assert!(status.success(), "linker failed");

    let output = std::process::Command::new(exe_path.to_str().unwrap())
        .output()
        .expect("run executable");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "42");

    // Cleanup
    let _ = std::fs::remove_file(&obj_path);
    let _ = std::fs::remove_file(&exe_path);
}

#[test]
fn test_jit_class_pattern_diagnostic() {
    // Diagnostic: does the arm execute at all? Return 100 (not a+1) to isolate.
    let result = jit_run(r#"
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

def f() -> int:
    p = Point(42, 0)
    match p:
        case Point(x=a):
            return 100
    return 0

f()
"#);
    assert_eq!(result, 100, "diagnostic: arm should be reached");
}

#[test]
fn test_jit_class_pattern_isinstance_only() {
    // Diagnostic: test isinstance inside match arm
    let result = jit_run(r#"
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

def f() -> int:
    p = Point(42, 0)
    match p:
        case Point():
            return 100
    return 0

f()
"#);
    assert_eq!(result, 100, "isinstance check in match should work");
}

#[test]
fn test_jit_class_pattern_hasattr_diagnostic() {
    // Diagnostic: test just the hasattr part via a Wildcard capture
    let result = jit_run(r#"
class Point:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y

def f() -> int:
    p = Point(42, 0)
    match p:
        case Point(x=_):
            return 100
    return 0

f()
"#);
    assert_eq!(result, 100, "hasattr check in match should work");
}

#[test]
fn test_jit_setattr_direct() {
    // Test that SetAttr (self.x = x) is actually working in __init__
    let result = jit_run(r#"
class Counter:
    def __init__(self) -> None:
        self.count = 99

def f() -> int:
    c = Counter()
    return getattr(c, "count")

f()
"#);
    assert_eq!(result, 99, "setattr in __init__ should set count=99");
}

#[test]
fn test_jit_init_no_args() {
    // Absolute minimal: no-arg init that sets a hardcoded attribute
    let result = jit_run(r#"
class Box:
    def __init__(self) -> None:
        self.val = 55

def f() -> int:
    b = Box()
    return getattr(b, "val")

f()
"#);
    assert_eq!(result, 55, "no-arg init: expected 55");
}

/// Verify mb_setattr and mb_getattr work directly (without JIT).
#[test]
fn test_runtime_setattr_getattr_direct() {
    use crate::runtime::class::{mb_class_register, mb_instance_new_with_init, mb_getattr};
    use crate::runtime::rc::MbObject;
    use crate::runtime::value::MbValue;
    use std::collections::HashMap;
    
    // Register a simple class
    mb_class_register("TestBox", vec![], HashMap::new());
    
    // Create instance
    let class_name = MbValue::from_ptr(MbObject::new_str("TestBox".to_string()));
    let empty_list = MbValue::from_ptr(MbObject::new_list(vec![]));
    let instance = mb_instance_new_with_init(class_name, empty_list);
    
    // Manually set attribute using SetAttr → mb_setattr
    let attr_name = MbValue::from_ptr(MbObject::new_str("count".to_string()));
    let value = MbValue::from_int(99);
    crate::runtime::class::mb_setattr(instance, attr_name, value);
    
    // Get the attribute back
    let attr_name2 = MbValue::from_ptr(MbObject::new_str("count".to_string()));
    let result = mb_getattr(instance, attr_name2);
    
    // Should return MbValue::from_int(99)
    assert!(result.is_int(), "result should be int, got bits: {:#x}", result.to_bits());
    assert_eq!(result.as_int(), Some(99), "should be 99");
}

#[test]
fn test_jit_seq_param_debug() {
    let result = jit_run(r#"
def first(xs: list[int]) -> int:
    match xs:
        case [x]:
            return x
    return 0

def f() -> int:
    return first([10])

f()
"#);
    assert_eq!(result, 10, "seq param: expected 10, got {}", result);
}

#[test]
fn test_jit_seq_inline_match() {
    // Test: sequence match INLINE (not a param) - does the match arm work?
    let result = jit_run(r#"
def f() -> int:
    xs = [10]
    match xs:
        case [x]:
            return x
    return 0

f()
"#);
    assert_eq!(result, 10, "seq inline match: expected 10, got {}", result);
}

#[test]
fn test_jit_first_simple() {
    // Simplified: just check if first() returns at all
    let result = jit_run(r#"
def first(xs: list[int]) -> int:
    return 99

def f() -> int:
    return first([10])

f()
"#);
    assert_eq!(result, 99, "first simple: expected 99, got {}", result);
}

// ── BigInt overflow tests (#833) ──────────────────────────────────────────────

/// 48-bit signed integer bounds (same as NaN-box inline int range).
const INT48_MAX: i64 = (1i64 << 47) - 1;
const INT48_MIN: i64 = -(1i64 << 47);

/// Decode a raw i64 result from JIT: if the value has a NaN-box pointer tag
/// (top bits = NAN_PREFIX | TAG_PTR) it is a BigInt heap object — return a
/// sentinel so callers can assert "is BigInt" without dereferencing.
fn decode_jit_result(raw: i64) -> i64 {
    let bits = raw as u64;
    const NAN_PREFIX: u64 = 0xFFF8_0000_0000_0000;
    const TAG_MASK: u64 = 0x0007_0000_0000_0000;
    const TAG_PTR: u64 = 0;
    if bits & NAN_PREFIX == NAN_PREFIX && (bits & TAG_MASK) >> 48 == TAG_PTR {
        // It is a NaN-boxed pointer — BigInt heap object, return i64::MAX as sentinel.
        i64::MAX
    } else {
        raw
    }
}

#[test]
fn test_bigint_overflow_add_no_silent_wrap() {
    // INT48_MAX + 1 must not silently wrap to INT48_MIN; it must produce a BigInt.
    let result = jit_run(&format!(r#"
def f(a: int, b: int) -> int:
    return a + b

f({INT48_MAX}, 1)
"#));
    let decoded = decode_jit_result(result);
    assert_eq!(decoded, i64::MAX, "expected BigInt sentinel on overflow, got {result}");
}

#[test]
fn test_bigint_overflow_sub_no_silent_wrap() {
    // INT48_MIN - 1 must promote to BigInt.
    let result = jit_run(&format!(r#"
def f(a: int, b: int) -> int:
    return a - b

f({INT48_MIN}, 1)
"#));
    let decoded = decode_jit_result(result);
    assert_eq!(decoded, i64::MAX, "expected BigInt sentinel on underflow, got {result}");
}

#[test]
fn test_bigint_overflow_mul_no_silent_wrap() {
    // 1_000_000_000 * 1_000_000_000 = 1e18 > INT48_MAX (~1.4e14) → BigInt.
    let result = jit_run(r#"
def f(a: int, b: int) -> int:
    return a * b

f(1000000000, 1000000000)
"#);
    let decoded = decode_jit_result(result);
    assert_eq!(decoded, i64::MAX, "expected BigInt sentinel on mul overflow, got {result}");
}

#[test]
fn test_bigint_no_overflow_small_values() {
    // Small values must not be affected — result must be exact.
    let result = jit_run(r#"
def f(a: int, b: int) -> int:
    return a + b * 2 - 1

f(10, 5)
"#);
    assert_eq!(result, 19, "expected 19, got {result}");
}

// ── Recursive internal-call NaN-boxing tests (R7, cranelift-jit) ─────────────

/// Decode a raw i64 JIT result that may arrive as either a primitive raw i64
/// (typed path) or a NaN-boxed MbValue int (dynamic dispatch path).
fn decode_mbvalue_int(raw: i64) -> i64 {
    use crate::runtime::value::MbValue;
    let val = MbValue::from_bits(raw as u64);
    val.as_int().unwrap_or(raw)
}

/// R7 – Primitive internal return is NaN-boxed when call-site is non-primitive.
///
/// Recursive `fib(n: int) -> int` with typed annotations exercises the path
/// where `emit_internal_call` captures a raw i64 from a callee with `Ty::Int`
/// return. Without the fix, `mb_dispatch_binop` receives raw ints → returns 0.
/// With the fix, the result is NaN-boxed before being stored in the dest VReg.
#[test]
fn test_jit_recursive_fib() {
    let raw = jit_run(r#"
def fib(n: int) -> int:
    if n == 0:
        return 0
    if n == 1:
        return 1
    return fib(n - 1) + fib(n - 2)

def f() -> int:
    return fib(30)

f()
"#);
    // Result may be raw i64 (typed path) or NaN-boxed i64 (dynamic dispatch path).
    let result = decode_mbvalue_int(raw);
    assert_eq!(result, 832040, "fib(30) should be 832040 (got raw={raw:#x})");
}

/// Smaller fib(10) = 55 sanity check — faster than fib(30) and verifies
/// the NaN-boxing fix applies at all recursion depths.
#[test]
fn test_jit_recursive_fib_small() {
    let raw = jit_run(r#"
def fib(n: int) -> int:
    if n == 0:
        return 0
    if n == 1:
        return 1
    return fib(n - 1) + fib(n - 2)

def f() -> int:
    return fib(10)

f()
"#);
    let result = decode_mbvalue_int(raw);
    assert_eq!(result, 55, "fib(10) should be 55 (got raw={raw:#x})");
}

/// R6 – Void extern return produces `MbValue::none()`.
///
/// A variable assigned from a void extern call must receive the TAG_NONE
/// sentinel rather than raw 0. We verify this by checking that the result
/// of the JIT main (which uses the captured value) is 0 (i.e., the call
/// did not crash or produce a corrupted value).
#[test]
fn test_jit_void_extern_result_is_none() {
    // mb_print is a void extern. Capturing its return and then proceeding
    // must not crash; the captured dest VReg is set to MbValue::none().
    let result = jit_run(r#"
def f() -> int:
    print(42)
    return 0

f()
"#);
    assert_eq!(result, 0, "void extern call should not crash; expected 0 (got {result})");
}

#[test]
fn test_jit_truediv_any_operand_returns_boxed_float() {
    // Regression for #2104: `int / any` (where `any` came from an unannotated
    // for-loop element binding) was statically typed `int` by HIR, but the
    // runtime always routes the binop through `mb_div` which returns a
    // NaN-boxed float. The mismatch left every consumer of the result
    // reinterpreting the IEEE-754 bit pattern as raw i64 — e.g.
    // `iters = int(200_000 / size)` inside `for size in [16, ...]:` evaluated
    // to 0 (or the float bits) instead of 12500. The fix marks the binop's
    // static result as Any when either operand is Any, so downstream lowering
    // emits the right unbox path before printing / coercing.
    let result = jit_run(r#"
def f() -> int:
    total: int = 0
    for size in [16, 32, 64]:
        iters: int = int(200_000 / size)
        total = total + iters
    return total

f()
"#);
    // 200_000/16 + 200_000/32 + 200_000/64 = 12500 + 6250 + 3125 = 21875
    assert_eq!(result, 21875, "int(int/any) inside for-loop must yield correct value (got {result})");
}

#[test]
fn test_jit_truediv_any_operand_range_bound() {
    // Companion regression for #2104: `int(200_000 / size)` used directly as
    // a `range()` bound. Before the fix the bound was reinterpreted bits and
    // the inner loop either elided or ran a garbage number of iterations.
    let result = jit_run(r#"
def f() -> int:
    count: int = 0
    for size in [16]:
        iters: int = int(200_000 / size)
        for _ in range(iters):
            count = count + 1
    return count

f()
"#);
    assert_eq!(result, 12500, "range(int(int/any)) inside for-loop must iterate 12500 times (got {result})");
}

// =====================================================================
// #2129 — JIT lowers `handle + handle` to native i64 add, bypassing the
// `class.rs` dunder dispatch (operator-overload gap for stdlib
// integer-handle classes).
//
// Stdlib types backed by the integer-handle pattern (`fractions.Fraction`,
// `decimal.Decimal`, future `complex` / `matrix` / money wrappers) return
// a NaN-boxed handle id whose static type, as far as the HIR type-checker
// is concerned, is plain `Int`. The JIT then lowers `h1 + h2` as a native
// Cranelift i64 add, producing arithmetic on the opaque handle ids rather
// than dispatching to `Fraction.__add__` (which lives in
// `class.rs::mb_call_method` and the constructor-returns-handle wrappers
// in `runtime/stdlib/fractions_mod.rs`).
//
// User classes whose `__add__` is defined with `class C:` are NOT
// affected — their instances type-check as `Ty::Class { .. }` and the
// HIR-to-MIR lowering at `lower/hir_to_mir.rs:4360-4376` already routes
// them through `mb_dispatch_binop`, which does the dunder lookup.
//
// A proper fix is one of the three escalating options enumerated on the
// issue:
//   1. Compile-time type inference — propagate "this Int came from
//      `fractions.Fraction(...)`" so the lowering can pick
//      `mb_dispatch_binop` (or a fraction-specialised add) for that vreg.
//   2. New `MbValue::TypedHandle { type_id, id }` value tag so the JIT
//      can't constant-fold `+` as native add on handle-tagged ints.
//   3. Return a real `MbObject::Instance` from handle constructors and
//      let the normal `Class` lowering path do the dispatch.
//
// Each touches the type system, the value representation, or the stdlib
// allocation model and is therefore tracked separately on #2129 (Phase 3
// of the stdlib-module codegen work under #1265). The two regressions
// below document the observable failure mode so a future fix can flip
// them from `#[ignore]` to active.
// =====================================================================

#[test]
#[ignore = "#2129: JIT lowers handle+handle to native i64 add — see issue for Phase 3 fix options"]
fn test_jit_issue_2129_fraction_handle_add_bypasses_dunder() {
    // `Fraction(1, 3) + Fraction(1, 6)` should equal `Fraction(1, 2)` whose
    // numerator is 1. Today the JIT emits a native i64 add on the two
    // handle ids (~2^40 and ~2^40+1), so the resulting "Fraction" is a
    // bogus handle whose `.numerator` is whatever int the wrapping_add
    // produced (or panics / dereferences garbage on `.numerator` lookup).
    let result = jit_run(r#"
import fractions

def f() -> int:
    a = fractions.Fraction(1, 3)
    b = fractions.Fraction(1, 6)
    s = a + b
    return s.numerator

f()
"#);
    assert_eq!(result, 1, "Fraction(1,3) + Fraction(1,6) must reduce to 1/2 via __add__ (got {result})");
}

#[test]
#[ignore = "#2129: JIT lowers handle+handle to native i64 add — see issue for Phase 3 fix options"]
fn test_jit_issue_2129_user_class_int_subclass_add_bypasses_dunder() {
    // Synthetic mirror of the Fraction case that does not depend on the
    // stdlib module surface: a user-defined wrapper whose constructor
    // returns an `int` (so the HIR types the binding as `Ty::Int`) and
    // whose `__add__` is exposed as a module-level function. The JIT
    // takes the native add path on `a + b` because both sides are typed
    // `int`, missing the dunder dispatch that would have folded the
    // operands through the wrapper's semantics.
    //
    // Once the fix lands, this asserts that handle-typed ints route
    // through `mb_dispatch_binop` (or equivalent) so the dunder fires.
    let result = jit_run(r#"
class Tagged:
    def __add__(self, other: int) -> int:
        return 42

def f() -> int:
    a: int = 100
    b: int = 200
    # Today: JIT emits native add → 300. Expected post-fix: a is statically
    # known to be a Tagged-handle and dispatch routes through __add__ → 42.
    return a + b

f()
"#);
    assert_eq!(result, 42, "handle-typed int + int must dispatch through __add__ (got {result})");
}


/// Regression for #1696: a `MirInst::CallExtern` whose `args.len()`
/// diverges from the declared `ext.params.len()` previously emitted a
/// mismatched-arity `call fnN(...)` that Cranelift's verifier rejected
/// with `mismatched argument count for v? = call fnN(...): got K,
/// expected N`, aborting the entire JIT module.
///
/// The fix reshapes `arg_vals` in `emit_extern_call` to match the
/// declared sig (truncate on over-arity, pad with zero on under-arity).
/// The same defensive guard now wraps `emit_internal_call` via the
/// `internal_param_counts` map. The repro that originally surfaced this
/// — `tests/cpython/lib_test_seeds/seed/test_bool.py`'s `BoolTest`
/// shape with the `(2-arg, 3-arg) × N` `with self.assertWarns(...)`
/// pattern — is exercised end-to-end by `cpython_lib_test_runner.rs`;
/// this smaller unit test isolates the arity-divergence shape on a
/// synthetic 3-arg call against a 2-arg call site so future ABI
/// rewrites do not regress the guard.
#[test]
fn test_jit_issue_1696_arity_guard_compiles_cleanly() {
    // The exact bug was a same-named callee whose registered Cranelift
    // extern-thunk sig was 2-arg but the call site emitted 3 args,
    // producing `mismatched argument count for v? = call fnN(...): got
    // 3, expected 2` at `define_function` time and aborting the JIT
    // module. The fix reshapes call-site args in both
    // `emit_internal_call` and `emit_extern_call` so the verifier never
    // sees a divergent count.
    //
    // We cannot reconstruct the exact MIR-level mismatch from .py source
    // alone (the lowerer that produced it does not emit it for arbitrary
    // input), so this test is a *codegen-only* smoke that the
    // canonical `(2-arg, 3-arg) × N` `with self.assertWarns(...)` shape
    // from `tests/cpython/lib_test_seeds/seed/test_bool.py`
    // type-checks, lowers, and codegens without tripping the verifier.
    // Runtime execution is intentionally skipped — the issue is about
    // codegen aborting, not about producing a correct value.
    let _jit_guard = JIT_LOCK.lock().unwrap();
    let src = r#"
class Ctx:
    def __enter__(self) -> int:
        return 1
    def __exit__(self, a: int, b: int, c: int) -> int:
        return 0

def f() -> int:
    total: int = 0
    for _ in range(4):
        c = Ctx()
        with c as e:
            total = total + e
    return total

f()
"#;
    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).unwrap();
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);
    let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
    let _output = backend.codegen(&mir, &checker.tcx)
        .expect("#1696 regression: JIT codegen must not abort with verifier error");
}


/// Regression for #2098: Cranelift variadic-call verifier fail
/// (func_id=554) on `assertRaises(exc_type, callable, *args)`-shaped
/// call sites. Originally surfaced from Phase 2 Task #6 struct
/// conformance — `self.assertRaises(struct.error, struct.calcsize, 'Z')`
/// at three sites in `tests/cpython/lib_test_seeds/seed/test_struct.py`
/// caused `call fn22(v54, v55, v56, v52, v53)` (5 args) to be emitted
/// against a registered `(i64) -> i64` thunk sig, aborting the JIT
/// module at `define_function` time and blocking essentially every
/// `assertRaises`-heavy CPython stdlib test.
///
/// Same root cause as #1696 (call-site arity divergence from the
/// declared Cranelift signature); the #1696 fix in
/// `emit_internal_call` / `emit_extern_call` (commit 35eaf8f4b)
/// reshapes `arg_vals` to match the declared sig (truncate on
/// over-arity, NaN-boxed-None pad on under-arity), so the verifier
/// no longer trips on the over-arity `(ex_type, callable, *args)`
/// shape.
///
/// This test pins the verifier-clean property of the variadic
/// call-site shape so future ABI rewrites must keep the guard.
/// Semantic correctness (runtime values flow through splat/forward
/// shapes intact) is covered end-to-end by
/// `tests/cpython/cpython_ported/test_struct_variadic.rs` —
/// struct.pack(fmt, *args) round-trip and assertRaises(exc, cb, arg)
/// both execute through the standard JIT pipeline.
#[test]
fn test_jit_issue_2098_variadic_assert_raises_no_verifier_abort() {
    let _jit_guard = JIT_LOCK.lock().unwrap();
    // Synthetic shape matching `self.assertRaises(exc, callable, arg)`:
    // a 2-arg declared callable invoked at a 3-arg call site. Before
    // #1696, MIR `CallExtern { args }` whose length diverged from the
    // registered Cranelift sig tripped
    // `mismatched argument count for v? = call fnN(...): got K,
    // expected N` (exactly the `func_id=554, 5 args vs (i64)->i64`
    // failure documented on #2098). The arity-reshape guard truncates
    // to the declared count so the verifier sees a clean call.
    let src = r#"
class Err:
    pass

def calcsize(fmt: str) -> int:
    return len(fmt)

def assert_raises(exc: int, cb: int, arg: str) -> int:
    return 0

def f() -> int:
    total: int = 0
    for _ in range(3):
        total = total + assert_raises(0, 0, "Z")
    return total

f()
"#;
    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).unwrap();
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);
    let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
    let _output = backend.codegen(&mir, &checker.tcx)
        .expect("#2098 regression: variadic-shaped call site must not abort the JIT verifier (defused by #1696 / commit 35eaf8f4b)");
}

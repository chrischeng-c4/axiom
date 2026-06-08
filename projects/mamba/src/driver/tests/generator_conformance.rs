#![cfg(test)]

/// Generator conformance integration tests (mamba-conformance-p0 change, #756).
///
/// Tests generator protocol edge cases end-to-end through the full JIT pipeline:
///   parse → type-check → HIR → MIR → Cranelift JIT → capture stdout → verify
///
/// T1:  Generator expressions (R1)
/// T2:  Generator send edge cases (R2)
/// T3:  Generator throw edge cases (R3)
/// T4:  Generator close edge cases (R4)
/// T5:  yield from send/throw passthrough (R5)
/// T6:  Generator state attributes (R6)
/// T10: Generator-based context manager (R10)
/// T11: Generator lifecycle (R11)
/// Regression: Existing generator fixtures

use crate::codegen::cranelift::jit::{CraneliftJitBackend, JIT_LOCK};
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::lower::{lower_hir_to_mir_with_symbols, lower_module};
use crate::parser;
use crate::runtime::cleanup_all_runtime_state;
use crate::runtime::output::{begin_capture, end_capture};
use crate::source::span::FileId;
use crate::types::TypeChecker;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const TEST_TIMEOUT_SECS: u64 = 10;

/// Run Python source through the full JIT pipeline, capturing stdout.
/// Acquires JIT_LOCK to serialize across concurrent test threads.
fn jit_capture(src: &str) -> String {
    let _jit_guard = JIT_LOCK.lock().unwrap();

    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    if !errors.is_empty() {
        panic!(
            "type errors: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
    }

    let hir = lower_module(&module, &checker).expect("HIR lowering failed");
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
    let output = backend
        .codegen(&mir, &checker.tcx)
        .expect("JIT codegen failed");

    match output {
        CodegenOutput::Jit { entry } => {
            let entry_addr = entry as usize;
            let (tx, rx) = mpsc::sync_channel(1);

            let handle = thread::spawn(move || {
                let prev = begin_capture();
                let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry_addr) };
                let _result = main_fn();
                cleanup_all_runtime_state();
                let captured = end_capture(prev);
                let _ = tx.send(captured);
            });

            let result = match rx.recv_timeout(Duration::from_secs(TEST_TIMEOUT_SECS)) {
                Ok(captured) => captured,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    panic!("JIT execution timed out after {TEST_TIMEOUT_SECS}s");
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("JIT execution thread panicked");
                }
            };

            let _ = handle.join();
            result
        }
        _ => panic!("expected JIT output"),
    }
}

/// Assert that captured output matches expected lines.
fn assert_output(actual: &str, expected: &str) {
    let actual_trimmed = actual.trim_end();
    let expected_trimmed = expected.trim_end();
    if actual_trimmed != expected_trimmed {
        let a_lines: Vec<&str> = actual_trimmed.lines().collect();
        let e_lines: Vec<&str> = expected_trimmed.lines().collect();
        let max = a_lines.len().max(e_lines.len());
        let mut diff = String::new();
        for i in 0..max {
            let a = a_lines.get(i).copied().unwrap_or("<missing>");
            let e = e_lines.get(i).copied().unwrap_or("<missing>");
            if a != e {
                diff.push_str(&format!("  line {}: expected {:?}, got {:?}\n", i + 1, e, a));
            }
        }
        panic!(
            "output mismatch:\n--- expected ---\n{expected_trimmed}\n--- actual ---\n{actual_trimmed}\n--- diff ---\n{diff}"
        );
    }
}

// =============================================================================
// T1: Generator Expressions (R1) — genexpr.py
// =============================================================================

/// T1.1: Basic generator expression with list().
#[test]
fn test_t1_1_genexpr_basic_square() {
    let output = jit_capture("print(list(x ** 2 for x in range(5)))\n");
    assert_output(&output, "[0, 1, 4, 9, 16]\n");
}

/// T1.2: Filtered generator expression.
#[test]
fn test_t1_2_genexpr_filtered() {
    let output = jit_capture("print(list(x for x in range(10) if x % 2 == 0))\n");
    assert_output(&output, "[0, 2, 4, 6, 8]\n");
}

/// T1.3: sum() with generator expression.
#[test]
fn test_t1_3_genexpr_sum() {
    let output = jit_capture("print(sum(x for x in range(4)))\n");
    assert_output(&output, "6\n");
}

/// T1.4: Nested generator expression.
#[test]
fn test_t1_4_genexpr_nested() {
    let output =
        jit_capture("print(list((x, y) for x in range(3) for y in range(2)))\n");
    assert_output(
        &output,
        "[(0, 0), (0, 1), (1, 0), (1, 1), (2, 0), (2, 1)]\n",
    );
}

/// T1.5: Generator expression as function argument — max().
#[test]
fn test_t1_5_genexpr_as_max_arg() {
    let output = jit_capture("print(max(x ** 2 for x in range(-3, 4)))\n");
    assert_output(&output, "9\n");
}

/// T1.6: Generator expression as function argument — min(abs()).
#[test]
fn test_t1_6_genexpr_as_min_arg() {
    let output = jit_capture("print(min(abs(x) for x in [-5, 3, -1, 4]))\n");
    assert_output(&output, "1\n");
}

// =============================================================================
// T2: Generator Send Edge Cases (R2) — send_edge_cases.py
// =============================================================================

/// T2.1: send(None) primes the generator — same as next().
#[test]
fn test_t2_1_send_none_primes() {
    let output = jit_capture(
        r#"def g():
    val = yield 1
    yield val * 2

gen = g()
print(gen.send(None))
"#,
    );
    assert_output(&output, "1\n");
}

/// T2.2: send(42) to just-started generator raises TypeError.
#[test]
fn test_t2_2_send_nonone_unstarted_typeerror() {
    let output = jit_capture(
        r#"def g():
    yield 1

gen = g()
try:
    gen.send(42)
except TypeError:
    print('TypeError: cannot send non-None')
"#,
    );
    assert_output(&output, "TypeError: cannot send non-None\n");
}

/// T2.3: send to exhausted generator raises StopIteration.
#[test]
fn test_t2_3_send_exhausted_stopiteration() {
    let output = jit_capture(
        r#"def g():
    yield 1

gen = g()
next(gen)
try:
    next(gen)
except StopIteration:
    print('exhausted')
try:
    gen.send(1)
except StopIteration:
    print('send to exhausted')
"#,
    );
    assert_output(&output, "exhausted\nsend to exhausted\n");
}

/// T2.4: send(value) returns next yielded value.
#[test]
fn test_t2_4_send_returns_next_yield() {
    let output = jit_capture(
        r#"def g():
    val = yield 1
    yield val * 2

gen = g()
print(gen.send(None))
print(gen.send(5))
"#,
    );
    assert_output(&output, "1\n10\n");
}

// =============================================================================
// T3: Generator Throw Edge Cases (R3) — throw_edge_cases.py
// =============================================================================

/// T3.1: throw with no matching except — propagates to caller.
#[test]
fn test_t3_1_throw_no_except() {
    let output = jit_capture(
        r#"def g():
    yield 1
    yield 2

gen = g()
next(gen)
try:
    gen.throw(TypeError('bad'))
except TypeError as e:
    print('propagated:', e)
"#,
    );
    assert_output(&output, "propagated: bad\n");
}

/// T3.2: throw into finally block — finally executes, exception propagates.
#[test]
fn test_t3_2_throw_into_finally() {
    let output = jit_capture(
        r#"def g():
    try:
        yield 1
    finally:
        print('cleanup')

gen = g()
next(gen)
try:
    gen.throw(ValueError('error'))
except ValueError:
    print('ValueError propagated after cleanup')
"#,
    );
    assert_output(&output, "cleanup\nValueError propagated after cleanup\n");
}

/// T3.3: throw on exhausted generator — exception raised immediately.
#[test]
fn test_t3_3_throw_exhausted() {
    let output = jit_capture(
        r#"def g():
    yield 1

gen = g()
next(gen)
try:
    next(gen)
except StopIteration:
    pass
try:
    gen.throw(RuntimeError('late throw'))
except RuntimeError as e:
    print('exhausted throw:', e)
"#,
    );
    assert_output(&output, "exhausted throw: late throw\n");
}

/// T3.4: throw exception caught by generator — generator continues.
#[test]
fn test_t3_4_throw_caught_by_generator() {
    let output = jit_capture(
        r#"def g():
    try:
        yield 1
        yield 2
    except ValueError as e:
        print('caught:', e)
        yield 99

gen = g()
print(next(gen))
print(gen.throw(ValueError('injected')))
"#,
    );
    assert_output(&output, "1\ncaught: injected\n99\n");
}

// =============================================================================
// T4: Generator Close Edge Cases (R4) — close_edge_cases.py
// =============================================================================

/// T4.1: close() on unstarted generator — silent no-op.
#[test]
fn test_t4_1_close_unstarted() {
    let output = jit_capture(
        r#"def g():
    yield 1
    yield 2

g1 = g()
g1.close()
print('unstarted close: ok')
"#,
    );
    assert_output(&output, "unstarted close: ok\n");
}

/// T4.2: close() on exhausted generator — silent no-op.
#[test]
fn test_t4_2_close_exhausted() {
    let output = jit_capture(
        r#"def g():
    yield 1
    yield 2

g1 = g()
next(g1)
try:
    while True:
        next(g1)
except StopIteration:
    pass
g1.close()
print('exhausted close: ok')
"#,
    );
    assert_output(&output, "exhausted close: ok\n");
}

/// T4.3: close() triggers GeneratorExit — generator's except handler runs.
#[test]
fn test_t4_3_close_triggers_generatorexit() {
    let output = jit_capture(
        r#"def g():
    try:
        yield 1
    except GeneratorExit:
        print('GeneratorExit caught')

gen = g()
next(gen)
gen.close()
"#,
    );
    assert_output(&output, "GeneratorExit caught\n");
}

/// T4.4: Generator ignores GeneratorExit (yields again) — RuntimeError.
#[test]
fn test_t4_4_close_ignored_generatorexit_runtime_error() {
    let output = jit_capture(
        r#"def g():
    try:
        yield 1
    except GeneratorExit:
        yield 2

gen = g()
next(gen)
try:
    gen.close()
except RuntimeError:
    print('RuntimeError: generator ignored GeneratorExit')
"#,
    );
    assert_output(
        &output,
        "RuntimeError: generator ignored GeneratorExit\n",
    );
}

/// T4.5: close() triggers finally block.
#[test]
fn test_t4_5_close_triggers_finally() {
    let output = jit_capture(
        r#"def g():
    try:
        yield 1
        yield 2
    finally:
        print('finally ran')

gen = g()
next(gen)
gen.close()
"#,
    );
    assert_output(&output, "finally ran\n");
}

// =============================================================================
// T5: yield from Send/Throw Passthrough (R5) — yield_from_passthrough.py
// =============================================================================

/// T5.1: send(value) through yield-from to inner generator.
#[test]
fn test_t5_1_yield_from_send_passthrough() {
    let output = jit_capture(
        r#"def inner():
    val = yield 'ready'
    yield val * 10

def outer():
    result = yield from inner()

g = outer()
print(next(g))
print(g.send(5))
"#,
    );
    assert_output(&output, "ready\n50\n");
}

/// T5.2: throw(exc) through yield-from to inner generator.
#[test]
fn test_t5_2_yield_from_throw_passthrough() {
    let output = jit_capture(
        r#"def inner():
    try:
        yield 1
    except ValueError as e:
        yield str(e)

def outer():
    yield from inner()

g = outer()
print(next(g))
print(g.throw(ValueError('injected')))
"#,
    );
    assert_output(&output, "1\ninjected\n");
}

/// T5.3: Inner generator return value captured by outer via yield-from.
#[test]
fn test_t5_3_yield_from_return_value_capture() {
    let output = jit_capture(
        r#"def inner():
    yield 1
    return 42

def outer():
    result = yield from inner()
    print('got:', result)
    yield result

g = outer()
print(next(g))
print(next(g))
"#,
    );
    assert_output(&output, "1\ngot: 42\n42\n");
}

/// T5.4: close() through yield-from passes to inner generator.
#[test]
fn test_t5_4_yield_from_close_passthrough() {
    let output = jit_capture(
        r#"def inner():
    try:
        yield 1
    except GeneratorExit:
        print('inner closed')

def outer():
    yield from inner()

g = outer()
next(g)
g.close()
"#,
    );
    assert_output(&output, "inner closed\n");
}

// =============================================================================
// T6: Generator State Attributes (R6) — state_attributes.py
// =============================================================================

/// T6.1: gi_frame is not None before first next() (created state).
#[test]
fn test_t6_1_gi_frame_created() {
    let output = jit_capture(
        r#"def g():
    yield 1
    yield 2

gen = g()
print(gen.gi_frame is not None)
"#,
    );
    assert_output(&output, "True\n");
}

/// T6.2: gi_frame is not None after suspend (after first next).
#[test]
fn test_t6_2_gi_frame_suspended() {
    let output = jit_capture(
        r#"def g():
    yield 1
    yield 2

gen = g()
next(gen)
print(gen.gi_frame is not None)
"#,
    );
    assert_output(&output, "True\n");
}

/// T6.3: gi_frame is None after exhaustion.
#[test]
fn test_t6_3_gi_frame_exhausted() {
    let output = jit_capture(
        r#"def g():
    yield 1
    yield 2

gen = g()
try:
    while True:
        next(gen)
except StopIteration:
    pass
print(gen.gi_frame is None)
"#,
    );
    assert_output(&output, "True\n");
}

/// T6.4: gi_frame is None after close().
#[test]
fn test_t6_4_gi_frame_after_close() {
    let output = jit_capture(
        r#"def g():
    yield 1
    yield 2

gen = g()
next(gen)
gen.close()
print(gen.gi_frame is None)
"#,
    );
    assert_output(&output, "True\n");
}

// =============================================================================
// T10: Generator-Based Context Manager (R10) — context_manager_pattern.py
// =============================================================================

/// T10.1: try/yield/finally — normal path.
#[test]
fn test_t10_1_context_manager_normal_path() {
    let output = jit_capture(
        r#"def managed_resource():
    print('acquire')
    try:
        yield 'resource'
    finally:
        print('release')

g = managed_resource()
resource = next(g)
print('using:', resource)
try:
    next(g)
except StopIteration:
    pass
"#,
    );
    assert_output(&output, "acquire\nusing: resource\nrelease\n");
}

/// T10.2: try/yield/finally — exception in body.
#[test]
fn test_t10_2_context_manager_exception_path() {
    let output = jit_capture(
        r#"def managed_resource():
    print('acquire')
    try:
        yield 'resource'
    finally:
        print('release')

g = managed_resource()
resource = next(g)
print('using:', resource)
try:
    g.throw(ValueError('error in body'))
except ValueError:
    print('ValueError caught by caller')
"#,
    );
    assert_output(
        &output,
        "acquire\nusing: resource\nrelease\nValueError caught by caller\n",
    );
}

// =============================================================================
// T11: Generator Lifecycle (R11) — lifecycle.py
// =============================================================================

/// T11.1: del on active generator triggers close() and finally runs.
#[test]
fn test_t11_1_del_triggers_close() {
    let output = jit_capture(
        r#"def g():
    try:
        yield 1
    finally:
        print('finalized')

g2 = g()
next(g2)
del g2
"#,
    );
    assert_output(&output, "finalized\n");
}

/// T11.2: close() on generator with pending finally block.
#[test]
fn test_t11_2_close_pending_finally() {
    let output = jit_capture(
        r#"def g():
    try:
        yield 1
        yield 2
    finally:
        print('finally ran')

gen = g()
next(gen)
gen.close()
"#,
    );
    assert_output(&output, "finally ran\n");
}

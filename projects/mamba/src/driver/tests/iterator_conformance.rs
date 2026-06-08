#![cfg(test)]

/// Iterator conformance integration tests (mamba-conformance-p0 change, #756).
///
/// Tests iterator protocol edge cases end-to-end through the full JIT pipeline:
///   parse → type-check → HIR → MIR → Cranelift JIT → capture stdout → verify
///
/// T7:  Custom iterator class (R7)
/// T8:  Iterator composition with generators (R8)
/// T9:  iter(callable, sentinel) (R9)
/// T12: Iterable unpacking with generators (R12)
/// Regression: Existing iterator fixtures
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
                diff.push_str(&format!(
                    "  line {}: expected {:?}, got {:?}\n",
                    i + 1,
                    e,
                    a
                ));
            }
        }
        panic!(
            "output mismatch:\n--- expected ---\n{expected_trimmed}\n--- actual ---\n{actual_trimmed}\n--- diff ---\n{diff}"
        );
    }
}

/// Load a fixture file and its golden expected output, run through JIT, and compare.
fn run_fixture(fixture_path: &str) {
    let src = std::fs::read_to_string(fixture_path)
        .unwrap_or_else(|e| panic!("read fixture {fixture_path}: {e}"));
    let expected_path = fixture_path.replace(".py", ".expected");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|e| panic!("read expected {expected_path}: {e}"));

    // Strip xfail directive from source before running
    let src_clean: String = src
        .lines()
        .filter(|line| !line.trim().starts_with("# mamba-xfail:"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    let output = jit_capture(&src_clean);
    assert_output(&output, &expected);
}

// =============================================================================
// T7: Custom Iterator Class (R7) — custom_iterator.py
// =============================================================================

/// T7.1: for loop over custom iterator (Fibonacci).
#[test]
fn test_t7_1_custom_iterator_for_loop() {
    let output = jit_capture(
        r#"class Fibonacci:
    def __init__(self, n):
        self.n = n
        self.a = 0
        self.b = 1
        self.count = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.count >= self.n:
            raise StopIteration
        val = self.a
        self.a, self.b = self.b, self.a + self.b
        self.count += 1
        return val

for x in Fibonacci(6):
    print(x)
"#,
    );
    assert_output(&output, "0\n1\n1\n2\n3\n5\n");
}

/// T7.2: list() on custom iterator.
#[test]
fn test_t7_2_custom_iterator_list() {
    let output = jit_capture(
        r#"class Fibonacci:
    def __init__(self, n):
        self.n = n
        self.a = 0
        self.b = 1
        self.count = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.count >= self.n:
            raise StopIteration
        val = self.a
        self.a, self.b = self.b, self.a + self.b
        self.count += 1
        return val

print(list(Fibonacci(6)))
"#,
    );
    assert_output(&output, "[0, 1, 1, 2, 3, 5]\n");
}

/// T7.3: next() with StopIteration on custom iterator.
#[test]
fn test_t7_3_custom_iterator_stopiteration() {
    let output = jit_capture(
        r#"class Fibonacci:
    def __init__(self, n):
        self.n = n
        self.a = 0
        self.b = 1
        self.count = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.count >= self.n:
            raise StopIteration
        val = self.a
        self.a, self.b = self.b, self.a + self.b
        self.count += 1
        return val

it = Fibonacci(2)
print(next(it))
print(next(it))
try:
    next(it)
except StopIteration:
    print('StopIteration raised')
"#,
    );
    assert_output(&output, "0\n1\nStopIteration raised\n");
}

/// T7.4: `in` operator on custom iterator.
#[test]
fn test_t7_4_custom_iterator_in_operator() {
    let output = jit_capture(
        r#"class SimpleRange:
    def __init__(self, limit):
        self.limit = limit
        self.current = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.current >= self.limit:
            raise StopIteration
        val = self.current
        self.current += 1
        return val

print(3 in SimpleRange(5))
print(7 in SimpleRange(5))
"#,
    );
    assert_output(&output, "True\nFalse\n");
}

/// T7.5: Unpacking from custom iterator.
#[test]
fn test_t7_5_custom_iterator_unpacking() {
    let output = jit_capture(
        r#"class ThreeItems:
    def __init__(self):
        self.items = [10, 20, 30]
        self.index = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.index >= len(self.items):
            raise StopIteration
        val = self.items[self.index]
        self.index += 1
        return val

a, b, c = ThreeItems()
print(a, b, c)
"#,
    );
    assert_output(&output, "10 20 30\n");
}

/// T7 fixture: custom_iterator.py (passing subset — for-loop only) fixture matches golden output.
#[test]
fn test_t7_fixture_custom_iterator() {
    run_fixture("tests/cpython/fixtures/core/iterators/custom_iterator.py");
}

// =============================================================================
// T8: Iterator Composition with Generators (R8) — composition.py
// =============================================================================

/// T8.1: enumerate with generator.
#[test]
fn test_t8_1_enumerate_with_generator() {
    let output = jit_capture(
        r#"def gen():
    yield 'a'
    yield 'b'
    yield 'c'

print(list(enumerate(gen())))
"#,
    );
    assert_output(&output, "[(0, 'a'), (1, 'b'), (2, 'c')]\n");
}

/// T8.2: zip with generator.
#[test]
fn test_t8_2_zip_with_generator() {
    let output = jit_capture(
        r#"def gen():
    yield 'a'
    yield 'b'
    yield 'c'

def nums():
    yield 1
    yield 2
    yield 3

print(list(zip(gen(), nums())))
"#,
    );
    assert_output(&output, "[('a', 1), ('b', 2), ('c', 3)]\n");
}

/// T8.3: map with generator.
#[test]
fn test_t8_3_map_with_generator() {
    let output = jit_capture(
        r#"def gen():
    yield 1
    yield 2
    yield 3
    yield 4

print(list(map(lambda x: x * 2, gen())))
"#,
    );
    assert_output(&output, "[2, 4, 6, 8]\n");
}

/// T8.4: filter with generator.
#[test]
fn test_t8_4_filter_with_generator() {
    let output = jit_capture(
        r#"def gen():
    yield 1
    yield 2
    yield 3
    yield 4

print(list(filter(lambda x: x % 2 == 0, gen())))
"#,
    );
    assert_output(&output, "[2, 4]\n");
}

/// T8.5: Chained composition — enumerate(filter(pred, map(fn, iterable))).
#[test]
fn test_t8_5_chained_composition() {
    let output = jit_capture(
        "print(list(enumerate(filter(lambda x: x > 0, map(lambda x: x - 2, [1, 2, 3, 4, 5])))))\n",
    );
    assert_output(&output, "[(0, 1), (1, 2), (2, 3)]\n");
}

/// T8 fixture: composition.py (passing subset — enumerate only) fixture matches golden output.
#[test]
fn test_t8_fixture_composition() {
    run_fixture("tests/cpython/fixtures/core/iterators/composition.py");
}

// =============================================================================
// T9: iter(callable, sentinel) (R9) — callable_sentinel.py
// =============================================================================

/// T9.1: iter(fn, sentinel) stops at sentinel.
#[test]
fn test_t9_1_iter_callable_sentinel() {
    let output = jit_capture(
        r#"vals = iter([3, 2, 1, 0])
print(list(iter(lambda: next(vals), 0)))
"#,
    );
    assert_output(&output, "[3, 2, 1]\n");
}

/// T9.2: iter(callable, sentinel) with closure counter.
#[test]
fn test_t9_2_iter_callable_sentinel_counter() {
    let output = jit_capture(
        r#"count = 0
def counter():
    global count
    count += 1
    return count

print(list(iter(counter, 4)))
"#,
    );
    assert_output(&output, "[1, 2, 3]\n");
}

/// T9 fixture: Full callable_sentinel.py fixture matches golden output.
#[test]
fn test_t9_fixture_callable_sentinel() {
    run_fixture("tests/cpython/fixtures/core/iterators/callable_sentinel.py");
}

// =============================================================================
// T12: Iterable Unpacking with Generators (R12) — unpacking.py
// =============================================================================

/// T12.1: Basic unpacking — a, b, c = gen().
#[test]
fn test_t12_1_basic_unpacking() {
    let output = jit_capture(
        r#"def gen():
    yield 1
    yield 2
    yield 3

a, b, c = gen()
print(a, b, c)
"#,
    );
    assert_output(&output, "1 2 3\n");
}

/// T12.2: Starred unpacking — first, *rest = gen().
#[test]
fn test_t12_2_starred_rest_unpacking() {
    let output = jit_capture(
        r#"def gen():
    yield 1
    yield 2
    yield 3

first, *rest = gen()
print(first, rest)
"#,
    );
    assert_output(&output, "1 [2, 3]\n");
}

/// T12.3: Starred unpacking — a, *mid, last = gen().
#[test]
fn test_t12_3_starred_mid_unpacking() {
    let output = jit_capture(
        r#"def gen():
    yield 1
    yield 2
    yield 3

a, *mid, last = gen()
print(a, mid, last)
"#,
    );
    assert_output(&output, "1 [2] 3\n");
}

/// T12.4: Unpacking size mismatch — too few values raises ValueError.
#[test]
fn test_t12_4_unpacking_too_few_values() {
    let output = jit_capture(
        r#"def gen():
    yield 1
    yield 2

try:
    a, b, c = gen()
except ValueError:
    print('too few values')
"#,
    );
    assert_output(&output, "too few values\n");
}

/// T12.5: Unpacking size mismatch — too many values raises ValueError.
#[test]
fn test_t12_5_unpacking_too_many_values() {
    let output = jit_capture(
        r#"def gen():
    yield 1
    yield 2
    yield 3
    yield 4

try:
    a, b = gen()
except ValueError:
    print('too many values')
"#,
    );
    assert_output(&output, "too many values\n");
}

/// T12 fixture: Full unpacking.py fixture matches golden output.
#[test]
fn test_t12_fixture_unpacking() {
    run_fixture("tests/cpython/fixtures/core/iterators/unpacking.py");
}

// =============================================================================
// Regression: Existing iterator fixtures must continue to pass
// =============================================================================

/// Regression: existing iterators/protocol.py fixture still passes.
#[test]
fn test_regression_iterators_protocol() {
    run_fixture("tests/cpython/fixtures/core/iterators/protocol.py");
}

/// Regression: existing builtins/iteration.py fixture still passes.
#[test]
fn test_regression_builtins_iteration() {
    run_fixture("tests/cpython/fixtures/builtin-libs/builtins/iteration.py");
}

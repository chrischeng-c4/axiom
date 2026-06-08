#![cfg(test)]

/// Py3.12 behavioral conformance: language features + infrastructure (T13-T20).
///
/// Part of py312-behavioral-conformance spec (mamba-conformance-p0 change):
///   T13: Decorator conformance (R14)
///   T14: Class system conformance (R15)
///   T15: Exception conformance (R16)
///   T16: Pattern matching conformance (R17)
///   T17: Comprehension scope (R18)
///   T18: Context manager conformance (R20)
///   T19: Lambda and closure conformance (R21)
///   T20: CLI runner / infrastructure (R22)
///
/// Each test runs Python source through the full JIT pipeline:
///   parse -> type-check -> HIR -> MIR -> Cranelift JIT -> capture stdout -> verify
///
/// Tests marked `#[ignore]` require features not yet implemented (tracked as xfail
/// in the fixture-based harness). Remove `#[ignore]` as features land.

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
                diff.push_str(&format!("  line {}: expected {:?}, got {:?}\n", i + 1, e, a));
            }
        }
        panic!(
            "output mismatch:\n--- expected ---\n{expected_trimmed}\n--- actual ---\n{actual_trimmed}\n--- diff ---\n{diff}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// T13: Decorator Conformance (R14)
// ═══════════════════════════════════════════════════════════════════════════════

/// T13.1: Stacked decorators apply bottom-up.
#[test]
fn test_t13_1_stacked_decorator_order() {
    let output = jit_capture(
        r#"def d1(f):
    print('d1 applied')
    return f

def d2(f):
    print('d2 applied')
    return f

@d1
@d2
def foo():
    pass
"#,
    );
    assert_output(&output, "d2 applied\nd1 applied\n");
}

/// T13.2: Parameterized decorator @repeat(3).
#[test]
fn test_t13_2_parameterized_decorator() {
    let output = jit_capture(
        r#"def repeat(n):
    def decorator(f):
        def wrapper():
            for _ in range(n):
                f()
        return wrapper
    return decorator

@repeat(3)
def greet():
    print('hi')

greet()
"#,
    );
    assert_output(&output, "hi\nhi\nhi\n");
}

/// T13.3: functools.wraps preserves __name__.
#[test]
fn test_t13_3_functools_wraps() {
    let output = jit_capture(
        r#"from functools import wraps

def my_deco(f):
    @wraps(f)
    def wrapper(*a, **kw):
        return f(*a, **kw)
    return wrapper

@my_deco
def my_func():
    """docstring"""
    pass

print(my_func.__name__)
"#,
    );
    assert_output(&output, "my_func\n");
}

/// T13.4: Class decorator.
#[test]
fn test_t13_4_class_decorator() {
    let output = jit_capture(
        r#"def add_greeting(cls):
    cls.greet = lambda self: "hello"
    return cls

@add_greeting
class MyClass:
    pass

obj = MyClass()
print(obj.greet())
"#,
    );
    assert_output(&output, "hello\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T14: Class System Conformance (R15)
// ═══════════════════════════════════════════════════════════════════════════════

/// T14.1: Diamond MRO — C3 linearization.
#[test]
fn test_t14_1_diamond_mro() {
    let output = jit_capture(
        r#"class A:
    pass

class B(A):
    pass

class C(A):
    pass

class D(B, C):
    pass

print([cls.__name__ for cls in D.__mro__])
"#,
    );
    assert_output(&output, "['D', 'B', 'C', 'A', 'object']\n");
}

/// T14.2: staticmethod and classmethod dispatch.
#[test]
fn test_t14_2_static_class_method() {
    let output = jit_capture(
        r#"class Foo:
    @staticmethod
    def s():
        return 'static'

    @classmethod
    def c(cls):
        return cls.__name__

print(Foo.s())
print(Foo.c())
"#,
    );
    assert_output(&output, "static\nFoo\n");
}

/// T14.3: property getter/setter.
#[test]
fn test_t14_3_property_descriptor() {
    let output = jit_capture(
        r#"class Circle:
    def __init__(self, radius):
        self._radius = radius

    @property
    def radius(self):
        return self._radius

    @radius.setter
    def radius(self, value):
        self._radius = value

c = Circle(5)
print(c.radius)
c.radius = 10
print(c.radius)
"#,
    );
    assert_output(&output, "5\n10\n");
}

/// T14.4: __init_subclass__ hook.
#[test]
fn test_t14_4_init_subclass() {
    let output = jit_capture(
        r#"class Base:
    def __init_subclass__(cls, **kwargs):
        print(f'subclass created: {cls.__name__}')
        super().__init_subclass__(**kwargs)

class Child(Base):
    pass
"#,
    );
    assert_output(&output, "subclass created: Child\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T15: Exception Conformance (R16)
// ═══════════════════════════════════════════════════════════════════════════════

/// T15.1: raise ValueError from ZeroDivisionError — __cause__ set.
#[test]
fn test_t15_1_raise_from_cause() {
    let output = jit_capture(
        r#"try:
    try:
        1 / 0
    except ZeroDivisionError as e:
        raise ValueError('bad') from e
except ValueError as e:
    print(type(e.__cause__).__name__)
"#,
    );
    assert_output(&output, "ZeroDivisionError\n");
}

/// T15.2: Implicit chaining — __context__ set.
#[test]
fn test_t15_2_implicit_context() {
    let output = jit_capture(
        r#"try:
    try:
        1 / 0
    except ZeroDivisionError:
        raise ValueError('during handling')
except ValueError as e:
    print(type(e.__context__).__name__)
"#,
    );
    assert_output(&output, "ZeroDivisionError\n");
}

/// T15.3: raise from None suppresses context.
#[test]
fn test_t15_3_suppress_context() {
    let output = jit_capture(
        r#"try:
    try:
        1 / 0
    except ZeroDivisionError:
        raise ValueError('clean') from None
except ValueError as e:
    print(e.__cause__)
    print(e.__suppress_context__)
"#,
    );
    assert_output(&output, "None\nTrue\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T16: Pattern Matching Conformance (R17)
// ═══════════════════════════════════════════════════════════════════════════════

/// T16.1: Mapping pattern with capture.
#[test]
fn test_t16_1_mapping_pattern() {
    let output = jit_capture(
        r#"match {'action': 'move', 'x': 10, 'y': 20}:
    case {'action': 'move', 'x': x, 'y': y}:
        print(f'move to {x},{y}')
    case _:
        print('unknown')
"#,
    );
    assert_output(&output, "move to 10,20\n");
}

/// T16.2: OR pattern.
#[test]
fn test_t16_2_or_pattern() {
    let output = jit_capture(
        r#"match 2:
    case 1 | 2 | 3:
        print('small')
    case _:
        print('other')
"#,
    );
    assert_output(&output, "small\n");
}

/// T16.3: Guard pattern.
#[test]
fn test_t16_3_guard_pattern() {
    let output = jit_capture(
        r#"match 42:
    case 1 | 2 | 3:
        print('small')
    case x if x > 40:
        print(f'big: {x}')
    case _:
        print('other')
"#,
    );
    assert_output(&output, "big: 42\n");
}

/// T16.4: Sequence pattern with star unpacking.
#[test]
fn test_t16_4_sequence_star_pattern() {
    let output = jit_capture(
        r#"match [1, 2, 3, 4, 5]:
    case [first, *rest]:
        print(f'first={first}, rest={rest}')
"#,
    );
    assert_output(&output, "first=1, rest=[2, 3, 4, 5]\n");
}

/// T16.5: Nested patterns.
#[test]
fn test_t16_5_nested_pattern() {
    let output = jit_capture(
        r#"match {'users': [{'name': 'Alice'}, {'name': 'Bob'}]}:
    case {'users': [{'name': first_name}, *_]}:
        print(f'first user: {first_name}')
"#,
    );
    assert_output(&output, "first user: Alice\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T17: Comprehension Scope (R18)
// ═══════════════════════════════════════════════════════════════════════════════

/// T17.1: List comprehension variable does not leak to outer scope.
#[test]
fn test_t17_1_list_comp_no_leak() {
    let output = jit_capture(
        r#"x = 'outer'
result = [x for x in range(3)]
print(x)
print(result)
"#,
    );
    assert_output(&output, "outer\n[0, 1, 2]\n");
}

/// T17.2: Nested comprehension scoping — inner/outer independent.
#[test]
fn test_t17_2_nested_comp_scope() {
    let output = jit_capture(
        r#"outer = 10
matrix = [[i + j for j in range(3)] for i in range(3)]
print(matrix)
print(outer)
"#,
    );
    assert_output(&output, "[[0, 1, 2], [1, 2, 3], [2, 3, 4]]\n10\n");
}

/// T17.3: Dict comprehension scope isolation.
#[test]
fn test_t17_3_dict_comp_scope() {
    let output = jit_capture(
        r#"y = 'preserved'
d = {k: v for k, v in enumerate(range(3))}
print(y)
print(d)
"#,
    );
    assert_output(&output, "preserved\n{0: 0, 1: 1, 2: 2}\n");
}

/// T17 supplemental: Set comprehension scope isolation.
#[test]
fn test_t17_set_comp_scope() {
    let output = jit_capture(
        r#"z = 'kept'
s = {x * 2 for x in range(4)}
print(z)
print(sorted(s))
"#,
    );
    assert_output(&output, "kept\n[0, 2, 4, 6]\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T18: Context Manager Conformance (R20)
// ═══════════════════════════════════════════════════════════════════════════════

/// T18.1: __exit__ returns True suppresses exception.
#[test]
fn test_t18_1_exit_suppresses_exception() {
    let output = jit_capture(
        r#"class Suppress:
    def __enter__(self):
        return self
    def __exit__(self, exc_type, exc_val, exc_tb):
        return True

with Suppress():
    raise ValueError('suppressed')
print('after suppression')
"#,
    );
    assert_output(&output, "after suppression\n");
}

/// T18.2: Multiple context managers — LIFO exit order.
#[test]
fn test_t18_2_multiple_managers_lifo() {
    let output = jit_capture(
        r#"class CM:
    def __init__(self, n):
        self.n = n
    def __enter__(self):
        print(f'enter {self.n}')
        return self
    def __exit__(self, *a):
        print(f'exit {self.n}')
        return False

with CM(1), CM(2):
    print('body')
"#,
    );
    assert_output(&output, "enter 1\nenter 2\nbody\nexit 2\nexit 1\n");
}

/// T18.3: Exception in __enter__ means __exit__ not called.
#[test]
fn test_t18_3_exception_in_enter() {
    let output = jit_capture(
        r#"class BadEnter:
    def __enter__(self):
        raise ValueError('bad enter')
    def __exit__(self, *a):
        print('exit called')
        return False

try:
    with BadEnter():
        print('body')
except ValueError:
    print('caught')
"#,
    );
    assert_output(&output, "caught\n");
}

/// T18.4: Nested with blocks — correct nesting.
#[test]
fn test_t18_4_nested_with() {
    let output = jit_capture(
        r#"class CM:
    def __init__(self, n):
        self.n = n
    def __enter__(self):
        print(f'enter {self.n}')
        return self
    def __exit__(self, *a):
        print(f'exit {self.n}')
        return False

with CM(3):
    with CM(4):
        print('nested body')
"#,
    );
    assert_output(&output, "enter 3\nenter 4\nnested body\nexit 4\nexit 3\n");
}

/// T18.5: `with...as` binding — __enter__ return value accessible in body (module scope).
#[test]
fn test_t18_5_with_as_binding_module_scope() {
    let output = jit_capture(
        r#"class CM:
    def __enter__(self):
        return 42
    def __exit__(self, *args):
        pass

with CM() as val:
    print(val)
"#,
    );
    assert_output(&output, "42\n");
}

/// T18.6: `with...as` binding inside a function (Local scope).
#[test]
fn test_t18_6_with_as_binding_local_scope() {
    let output = jit_capture(
        r#"class CM:
    def __enter__(self):
        return 99
    def __exit__(self, *args):
        pass

def run():
    with CM() as val:
        print(val)

run()
"#,
    );
    assert_output(&output, "99\n");
}

/// T18.7: Nested `with...as` — each binding is independently accessible.
#[test]
fn test_t18_7_nested_with_as_bindings() {
    let output = jit_capture(
        r#"class CM:
    def __init__(self, n):
        self.n = n
    def __enter__(self):
        return self.n
    def __exit__(self, *args):
        pass

with CM(1) as a:
    with CM(2) as b:
        print(a)
        print(b)
"#,
    );
    assert_output(&output, "1\n2\n");
}

/// T18.8: Multiple `as` bindings in a single `with` statement.
#[test]
fn test_t18_8_multiple_with_as_bindings() {
    let output = jit_capture(
        r#"class CM:
    def __init__(self, n):
        self.n = n
    def __enter__(self):
        return self.n
    def __exit__(self, *args):
        pass

with CM(10) as x, CM(20) as y:
    print(x)
    print(y)
"#,
    );
    assert_output(&output, "10\n20\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T19: Lambda and Closure Conformance (R21)
// ═══════════════════════════════════════════════════════════════════════════════

/// T19.1: Closure over loop variable with default arg captures per-iteration.
#[test]
fn test_t19_1_closure_loop_default() {
    let output = jit_capture(
        r#"fns = [lambda x=i: x for i in range(3)]
print([f() for f in fns])
"#,
    );
    assert_output(&output, "[0, 1, 2]\n");
}

/// T19.2: Nested lambda — correct scoping.
#[test]
fn test_t19_2_nested_lambda() {
    let output = jit_capture(
        r#"compose = lambda f, g: lambda x: f(g(x))
double = lambda x: x * 2
add1 = lambda x: x + 1
print(compose(double, add1)(3))
"#,
    );
    assert_output(&output, "8\n");
}

/// T19.3: Lambda as sort key.
#[test]
fn test_t19_3_lambda_sort_key() {
    let output = jit_capture(
        r#"words = ['banana', 'apple', 'cherry', 'date']
print(sorted(words, key=lambda w: len(w)))
"#,
    );
    assert_output(&output, "['date', 'apple', 'banana', 'cherry']\n");
}

/// T19 supplemental: Lambda with multiple args.
#[test]
fn test_t19_lambda_multi_args() {
    let output = jit_capture(
        r#"add = lambda x, y: x + y
print(add(3, 7))
"#,
    );
    assert_output(&output, "10\n");
}

/// T19 supplemental: Lambda capturing outer variable.
#[test]
fn test_t19_lambda_closure() {
    let output = jit_capture(
        r#"x = 42
f = lambda: x
print(f())
"#,
    );
    assert_output(&output, "42\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// T20: CLI Runner (R22) — infrastructure-level checks
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// Regression: Existing fixtures must compile/parse
// ═══════════════════════════════════════════════════════════════════════════════

/// Regression: Verify existing builtins fixtures parse successfully.
#[test]
fn test_regression_builtins_parse() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(crate::conformance::FIXTURES_ROOT).join("builtin-libs/builtins");
    verify_all_parse(&base);
}

/// Regression: Verify existing stdlib fixtures parse successfully.
#[test]
fn test_regression_stdlib_parse() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cpython/stdlib");
    verify_all_parse(&base);
}

/// Regression: Verify existing language fixtures parse successfully.
#[test]
fn test_regression_language_parse() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(crate::conformance::FIXTURES_ROOT).join("core/language");
    verify_all_parse(&base);
}

/// Regression: Verify existing class_system fixtures parse successfully.
#[test]
fn test_regression_class_system_parse() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(crate::conformance::FIXTURES_ROOT).join("core/class_system");
    verify_all_parse(&base);
}

/// Regression: Verify existing exceptions fixtures parse successfully.
#[test]
fn test_regression_exceptions_parse() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(crate::conformance::FIXTURES_ROOT).join("core/exceptions");
    verify_all_parse(&base);
}

fn verify_all_parse(dir: &std::path::Path) {
    if !dir.exists() {
        return;
    }
    let mut count = 0;
    verify_parse_dir(dir, &mut count);
    assert!(count > 0, "No .py files found in {}", dir.display());
}

fn verify_parse_dir(dir: &std::path::Path, count: &mut usize) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                verify_parse_dir(&path, count);
            } else if path.extension().map_or(false, |ext| ext == "py") {
                let src = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
                let clean_src: String = src
                    .lines()
                    .filter(|l| !l.trim().starts_with("# mamba-xfail:"))
                    .collect::<Vec<_>>()
                    .join("\n")
                    + "\n";
                let result = parser::parse(&clean_src, FileId(0));
                assert!(
                    result.is_ok(),
                    "{}: parse failed: {:?}",
                    path.display(),
                    result.err()
                );
                *count += 1;
            }
        }
    }
}

// ──── Multi-level closure capture regression tests ────
// Regression: 3-level nesting with captured variable from grandparent scope.
// The HIR lowering must propagate ancestor scope names transitively.

#[test]
fn test_t20_parameterized_decorator() {
    let output = jit_capture(
        r#"def repeat(n):
    def decorator(f):
        def wrapper():
            for _ in range(n):
                f()
        return wrapper
    return decorator

@repeat(3)
def greet():
    print('hi')

greet()
"#,
    );
    assert_output(&output, "hi\nhi\nhi\n");
}

#[test]
fn test_t20_triple_nesting_capture() {
    let output = jit_capture(
        r#"def outer(n):
    def middle(f):
        def inner():
            for _ in range(n):
                f()
        return inner
    return middle

wrapped = outer(3)(lambda: print('hi'))
wrapped()
"#,
    );
    assert_output(&output, "hi\nhi\nhi\n");
}

#[test]
fn test_chained_comparison() {
    let output = jit_capture(
        r#"x = 5
print(1 < x < 10)
print(1 < 0 < 10)
print(1 < 2 < 3 < 4)
"#,
    );
    assert_output(&output, "True\nFalse\nTrue\n");
}

#[test]
fn test_walrus_basic() {
    let output = jit_capture(
        r#"if (n := 10) > 5:
    print(n)
"#,
    );
    assert_output(&output, "10\n");
}

#[test]
fn test_kwargs_basic() {
    let output = jit_capture(
        r#"def greet(name, greeting="Hello"):
    print(greeting, name)
greet("World", greeting="Hi")
"#,
    );
    assert_output(&output, "Hi World\n");
}

#[test]
fn test_kwargs_reorder() {
    let output = jit_capture(
        r#"def f(a, b, c):
    print(a, b, c)
f(1, c=3, b=2)
"#,
    );
    assert_output(&output, "1 2 3\n");
}

#[test]
fn test_kwargs_defaults() {
    let output = jit_capture(
        r#"def f(a, b=10, c=20):
    return a + b + c
print(f(1))
print(f(1, c=100))
"#,
    );
    assert_output(&output, "31\n111\n");
}

#[test]
fn test_t17_walrus_in_comprehension() {
    let output = jit_capture(
        r#"results = [y := x**2 for x in range(4)]
print(results, y)
"#,
    );
    assert_output(&output, "[0, 1, 4, 9] 9\n");
}

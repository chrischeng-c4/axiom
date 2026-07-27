//! Ported from Lib/test/test_functools_ported.py
//! Integration tests: stdlib/functools.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_functools_ported.py`.
#[test]
fn test_functools_reduce_sum() {
    let out = jit_capture(
        r#"import functools
print(functools.reduce(lambda a, b: a + b, [1, 2, 3, 4]))
"#,
    );
    assert_output(&out, "10\n");
}

/// Ported from `Lib/test/test_functools_ported.py`.
#[test]
fn test_functools_reduce_product_with_initial() {
    let out = jit_capture(
        r#"import functools
print(functools.reduce(lambda a, b: a * b, [1, 2, 3, 4], 1))
"#,
    );
    assert_output(&out, "24\n");
}

/// Ported from `Lib/test/test_functools_ported.py`.
#[test]
fn test_functools_reduce_single_element() {
    let out = jit_capture(
        r#"import functools
print(functools.reduce(lambda a, b: a + b, [42]))
"#,
    );
    assert_output(&out, "42\n");
}

/// Ported from `Lib/test/test_functools_ported.py`.
#[test]
fn test_functools_partial_one_bound() {
    let out = jit_capture(
        r#"import functools
add = functools.partial(lambda x, y: x + y, 10)
print(add(5))
"#,
    );
    assert_output(&out, "15\n");
}

/// Ported from `Lib/test/test_functools_ported.py`.
#[test]
fn test_functools_partial_multiple_calls() {
    let out = jit_capture(
        r#"import functools
mul = functools.partial(lambda x, y: x * y, 3)
print(mul(4))
print(mul(7))
"#,
    );
    assert_output(&out, "12\n21\n");
}

/// Ported from `Lib/test/test_functools_ported.py`.
#[test]
fn test_functools_lru_cache_recursive_fib() {
    let out = jit_capture(
        r#"import functools

@functools.lru_cache(maxsize=None)
def fib(n):
    return n if n < 2 else fib(n-1) + fib(n-2)

print(fib(10))
print(fib(20))
"#,
    );
    assert_output(&out, "55\n6765\n");
}

/// Ported from `Lib/test/test_functools_ported.py`.
#[test]
fn test_functools_lru_cache_maxsize_bounded() {
    let out = jit_capture(
        r#"import functools

@functools.lru_cache(maxsize=2)
def square(n):
    return n * n

print(square(3))
print(square(4))
print(square(3))
"#,
    );
    assert_output(&out, "9\n16\n9\n");
}

/// Ported from `Lib/test/test_functools_ported.py`.
#[test]
fn test_functools_cache_basic() {
    let out = jit_capture(
        r#"import functools

@functools.cache
def square(n):
    return n * n

print(square(7))
print(square(8))
"#,
    );
    assert_output(&out, "49\n64\n");
}

/// Ported from `Lib/test/test_functools_ported.py`.
#[test]
fn test_functools_wraps_preserves_name() {
    let out = jit_capture(
        r#"import functools

def deco(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        return f(*args, **kwargs)
    return wrapper

@deco
def hello():
    return "hi"

print(hello())
print(hello.__name__)
"#,
    );
    assert_output(&out, "hi\nhello\n");
}


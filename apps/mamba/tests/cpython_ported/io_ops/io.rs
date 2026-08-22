//! Ported from Lib/test/test_io_ported.py
//! Integration tests: io_ops/io.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_io_ported.py`.
#[test]
fn test_closure_counter_via_list_cell() {
    let out = jit_capture(
        r#"def make_counter():
    n = [0]
    def inc():
        n[0] = n[0] + 1
        return n[0]
    return inc

c = make_counter()
print(c())
print(c())
print(c())
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

/// Ported from `Lib/test/test_io_ported.py`.
#[test]
fn test_function_wrapping_decorator_style() {
    let out = jit_capture(
        r#"def double(f):
    def wrapped(x):
        return f(x) * 2
    return wrapped

def add_one(x):
    return x + 1

d = double(add_one)
print(d(5))
print(d(10))
print(d(0))
"#,
    );
    assert_output(&out, "12\n22\n2\n");
}

/// Ported from `Lib/test/test_io_ported.py`.
#[test]
fn test_decorator_chain_three_deep() {
    let out = jit_capture(
        r#"def add_one(x):
    return x + 1

def double(f):
    def wrapped(x):
        return f(x) * 2
    return wrapped

def negate(f):
    def wrapped(x):
        return -f(x)
    return wrapped

d = double(add_one)
nd = negate(d)
print(d(3))
print(nd(3))
print(nd(7))
"#,
    );
    assert_output(&out, "8\n-8\n-16\n");
}

/// Ported from `Lib/test/test_io_ported.py`.
#[test]
fn test_sum_min_max_avg() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4, 5]
print(sum(xs))
print(min(xs))
print(max(xs))
print(sum(xs) / len(xs))
"#,
    );
    assert_output(&out, "15\n1\n5\n3.0\n");
}

/// Ported from `Lib/test/test_io_ported.py`.
#[test]
fn test_sorted_reverse_and_dedupe() {
    let out = jit_capture(
        r#"xs = [3, 1, 4, 1, 5, 9, 2, 6]
print(sorted(xs))
print(sorted(xs, reverse=True))
print(sorted(set(xs)))
"#,
    );
    assert_output(
        &out,
        "[1, 1, 2, 3, 4, 5, 6, 9]\n[9, 6, 5, 4, 3, 2, 1, 1]\n[1, 2, 3, 4, 5, 6, 9]\n",
    );
}

/// Ported from `Lib/test/test_io_ported.py`.
#[test]
fn test_min_max_with_negatives_and_zero() {
    let out = jit_capture(
        r#"xs = [-5, -1, 0, 3, 7]
print(min(xs))
print(max(xs))
print(sum(xs))
"#,
    );
    assert_output(&out, "-5\n7\n4\n");
}


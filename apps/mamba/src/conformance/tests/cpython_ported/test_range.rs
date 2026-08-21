//! Py3.12 conformance tests for range (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_range.py — RangeTest
//!
//! Coverage: range(stop), range(start, stop), range(start, stop, step),
//! len(), iteration, indexing, list conversion, membership, sum, negative
//! step, bool.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_range_stop_only_len() {
    let out = jit_capture(
        r#"r = range(5)
print(len(r))
"#,
    );
    assert_output(&out, "5\n");
}

#[test]
fn test_range_stop_zero_len() {
    let out = jit_capture(
        r#"r = range(0)
print(len(r))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_range_start_stop_len() {
    let out = jit_capture(
        r#"r = range(2, 7)
print(len(r))
"#,
    );
    assert_output(&out, "5\n");
}

#[test]
fn test_range_start_stop_step_len() {
    let out = jit_capture(
        r#"r = range(0, 10, 2)
print(len(r))
"#,
    );
    assert_output(&out, "5\n");
}

#[test]
fn test_range_iteration_sum() {
    let out = jit_capture(
        r#"total = 0
for x in range(10):
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "45\n");
}

#[test]
fn test_range_iteration_start_stop() {
    let out = jit_capture(
        r#"total = 0
for x in range(3, 7):
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "18\n");
}

#[test]
fn test_range_step_iteration() {
    let out = jit_capture(
        r#"out = []
for x in range(0, 10, 2):
    out.append(x)
print(out)
"#,
    );
    assert_output(&out, "[0, 2, 4, 6, 8]\n");
}

#[test]
fn test_range_negative_step() {
    let out = jit_capture(
        r#"out = []
for x in range(5, 0, -1):
    out.append(x)
print(out)
"#,
    );
    assert_output(&out, "[5, 4, 3, 2, 1]\n");
}

#[test]
fn test_range_empty_when_start_ge_stop() {
    let out = jit_capture(
        r#"count = 0
for x in range(5, 5):
    count = count + 1
print(count)
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_range_to_list() {
    let out = jit_capture(
        r#"xs = list(range(4))
print(len(xs))
print(xs[0])
print(xs[3])
"#,
    );
    assert_output(&out, "4\n0\n3\n");
}

#[test]
fn test_range_to_list_start_stop() {
    let out = jit_capture(
        r#"xs = list(range(2, 6))
print(xs)
"#,
    );
    assert_output(&out, "[2, 3, 4, 5]\n");
}

#[test]
fn test_range_indexing_positive() {
    let out = jit_capture(
        r#"r = range(10, 20)
print(r[0])
print(r[5])
print(r[9])
"#,
    );
    assert_output(&out, "10\n15\n19\n");
}

#[test]
fn test_range_indexing_negative() {
    let out = jit_capture(
        r#"r = range(10, 20)
print(r[-1])
print(r[-5])
"#,
    );
    assert_output(&out, "19\n15\n");
}

#[test]
fn test_range_contains_present() {
    let out = jit_capture(
        r#"r = range(10)
print(5 in r)
print(0 in r)
print(9 in r)
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

#[test]
fn test_range_contains_absent() {
    let out = jit_capture(
        r#"r = range(10)
print(10 in r)
print(-1 in r)
print(100 in r)
"#,
    );
    assert_output(&out, "False\nFalse\nFalse\n");
}

#[test]
fn test_range_contains_with_step() {
    let out = jit_capture(
        r#"r = range(0, 10, 2)
print(0 in r)
print(2 in r)
print(3 in r)
print(8 in r)
"#,
    );
    assert_output(&out, "True\nTrue\nFalse\nTrue\n");
}

#[test]
fn test_range_equality_same_args() {
    let out = jit_capture(
        r#"print(range(5) == range(5))
print(range(0, 5) == range(5))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

#[test]
fn test_range_inequality_different_args() {
    let out = jit_capture(
        r#"print(range(5) == range(6))
print(range(5) == range(1, 5))
"#,
    );
    assert_output(&out, "False\nFalse\n");
}

#[test]
fn test_range_bool_empty_is_false() {
    let out = jit_capture(
        r#"print(bool(range(0)))
print(bool(range(5, 5)))
"#,
    );
    assert_output(&out, "False\nFalse\n");
}

#[test]
fn test_range_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"print(bool(range(1)))
print(bool(range(0, 1)))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

#[test]
fn test_range_sum_builtin() {
    let out = jit_capture(
        r#"print(sum(range(10)))
print(sum(range(1, 11)))
"#,
    );
    assert_output(&out, "45\n55\n");
}

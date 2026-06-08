//! Py3.12 conformance tests for list (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_list.py — ListTest
//!
//! Coverage: construction, len, __getitem__, __setitem__, append, extend,
//! insert, pop, remove, slicing, iteration, equality, bool, count, index.
//!
//! Tests that require unimplemented features are marked `#[ignore]` with
//! a comment naming the missing behavior.
//!
//! @issue #759

use super::{assert_output, jit_capture};

// ── construction ──────────────────────────────────────────────────────────────

#[test]
fn test_list_empty_literal_len() {
    let out = jit_capture(
        r#"xs = []
print(len(xs))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_list_literal_len() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
print(len(xs))
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_list_constructor_empty() {
    let out = jit_capture(
        r#"xs = list()
print(len(xs))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_list_constructor_from_iterable() {
    let out = jit_capture(
        r#"xs = list((1, 2, 3))
print(len(xs))
print(xs[0])
print(xs[2])
"#,
    );
    assert_output(&out, "3\n1\n3\n");
}

// ── indexing ─────────────────────────────────────────────────────────────────

#[test]
fn test_list_getitem_positive() {
    let out = jit_capture(
        r#"xs = [10, 20, 30]
print(xs[0])
print(xs[1])
print(xs[2])
"#,
    );
    assert_output(&out, "10\n20\n30\n");
}

#[test]
fn test_list_getitem_negative() {
    let out = jit_capture(
        r#"xs = [10, 20, 30]
print(xs[-1])
print(xs[-2])
"#,
    );
    assert_output(&out, "30\n20\n");
}

#[test]
fn test_list_setitem() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
xs[1] = 99
print(xs[0])
print(xs[1])
print(xs[2])
"#,
    );
    assert_output(&out, "1\n99\n3\n");
}

// ── append / extend / insert ──────────────────────────────────────────────────

#[test]
fn test_list_append_increases_len() {
    let out = jit_capture(
        r#"xs = [1, 2]
xs.append(3)
print(len(xs))
print(xs[2])
"#,
    );
    assert_output(&out, "3\n3\n");
}

#[test]
fn test_list_extend_with_list() {
    let out = jit_capture(
        r#"xs = [1, 2]
xs.extend([3, 4])
print(len(xs))
print(xs[3])
"#,
    );
    assert_output(&out, "4\n4\n");
}

#[test]
fn test_list_insert_at_front() {
    let out = jit_capture(
        r#"xs = [2, 3]
xs.insert(0, 1)
print(len(xs))
print(xs[0])
print(xs[2])
"#,
    );
    assert_output(&out, "3\n1\n3\n");
}

// ── pop / remove ──────────────────────────────────────────────────────────────

#[test]
fn test_list_pop_default_last() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
v = xs.pop()
print(v)
print(len(xs))
"#,
    );
    assert_output(&out, "3\n2\n");
}

#[test]
fn test_list_pop_indexed() {
    let out = jit_capture(
        r#"xs = [10, 20, 30]
v = xs.pop(0)
print(v)
print(len(xs))
print(xs[0])
"#,
    );
    assert_output(&out, "10\n2\n20\n");
}

#[test]
fn test_list_remove_present() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 2]
xs.remove(2)
print(len(xs))
print(xs[1])
"#,
    );
    assert_output(&out, "3\n3\n");
}

// ── slicing ──────────────────────────────────────────────────────────────────

#[test]
fn test_list_slice_basic() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4, 5]
ys = xs[1:4]
print(len(ys))
print(ys[0])
print(ys[2])
"#,
    );
    assert_output(&out, "3\n2\n4\n");
}

#[test]
fn test_list_slice_open_start() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4, 5]
ys = xs[:3]
print(len(ys))
print(ys[0])
print(ys[2])
"#,
    );
    assert_output(&out, "3\n1\n3\n");
}

#[test]
fn test_list_slice_open_end() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4, 5]
ys = xs[2:]
print(len(ys))
print(ys[0])
"#,
    );
    assert_output(&out, "3\n3\n");
}

// ── iteration ────────────────────────────────────────────────────────────────

#[test]
fn test_list_iterate_sum() {
    let out = jit_capture(
        r#"xs = [1, 2, 3, 4]
total = 0
for x in xs:
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "10\n");
}

#[test]
fn test_list_iterate_empty_yields_nothing() {
    let out = jit_capture(
        r#"xs = []
count = 0
for x in xs:
    count = count + 1
print(count)
"#,
    );
    assert_output(&out, "0\n");
}

// ── equality ─────────────────────────────────────────────────────────────────

#[test]
fn test_list_equal_same_elements() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = [1, 2, 3]
print(a == b)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_list_not_equal_different_order() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = [3, 2, 1]
print(a == b)
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_list_not_equal_different_len() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = [1, 2]
print(a == b)
"#,
    );
    assert_output(&out, "False\n");
}

// ── bool / truthiness ────────────────────────────────────────────────────────

#[test]
fn test_list_bool_empty_is_false() {
    let out = jit_capture(
        r#"xs = []
print(bool(xs))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_list_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"xs = [0]
print(bool(xs))
"#,
    );
    assert_output(&out, "True\n");
}

// ── count / index ────────────────────────────────────────────────────────────

#[test]
fn test_list_count_present() {
    let out = jit_capture(
        r#"xs = [1, 2, 2, 3, 2]
print(xs.count(2))
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_list_count_absent_is_zero() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
print(xs.count(99))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_list_index_present() {
    let out = jit_capture(
        r#"xs = [10, 20, 30]
print(xs.index(20))
"#,
    );
    assert_output(&out, "1\n");
}

// ── concatenation / repetition ──────────────────────────────────────────────

#[test]
fn test_list_concatenation_operator() {
    let out = jit_capture(
        r#"a = [1, 2]
b = [3, 4]
c = a + b
print(len(c))
print(c[0])
print(c[3])
"#,
    );
    assert_output(&out, "4\n1\n4\n");
}

#[test]
fn test_list_repetition_operator() {
    let out = jit_capture(
        r#"xs = [1, 2] * 3
print(len(xs))
print(xs[0])
print(xs[5])
"#,
    );
    assert_output(&out, "6\n1\n2\n");
}

// ── in / not in ──────────────────────────────────────────────────────────────

#[test]
fn test_list_contains_present() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
print(2 in xs)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_list_contains_absent() {
    let out = jit_capture(
        r#"xs = [1, 2, 3]
print(99 in xs)
"#,
    );
    assert_output(&out, "False\n");
}

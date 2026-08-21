//! Py3.12 conformance tests for tuple (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_tuple.py — TupleTest
//!
//! Coverage: construction, indexing (positive + negative), len, iteration,
//! equality, bool, concatenation, repetition, membership, count, index.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_tuple_empty_literal_len() {
    let out = jit_capture(
        r#"t = ()
print(len(t))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_tuple_literal_len() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(len(t))
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_tuple_singleton_requires_comma() {
    let out = jit_capture(
        r#"t = (5,)
print(len(t))
print(t[0])
"#,
    );
    assert_output(&out, "1\n5\n");
}

#[test]
fn test_tuple_constructor_empty() {
    let out = jit_capture(
        r#"t = tuple()
print(len(t))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_tuple_constructor_from_list() {
    let out = jit_capture(
        r#"t = tuple([1, 2, 3])
print(len(t))
print(t[0])
print(t[2])
"#,
    );
    assert_output(&out, "3\n1\n3\n");
}

#[test]
fn test_tuple_getitem_positive() {
    let out = jit_capture(
        r#"t = (10, 20, 30)
print(t[0])
print(t[1])
print(t[2])
"#,
    );
    assert_output(&out, "10\n20\n30\n");
}

#[test]
fn test_tuple_getitem_negative() {
    let out = jit_capture(
        r#"t = (10, 20, 30)
print(t[-1])
print(t[-2])
"#,
    );
    assert_output(&out, "30\n20\n");
}

#[test]
fn test_tuple_iterate_sum() {
    let out = jit_capture(
        r#"t = (1, 2, 3, 4)
total = 0
for x in t:
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "10\n");
}

#[test]
fn test_tuple_iterate_empty_yields_nothing() {
    let out = jit_capture(
        r#"t = ()
count = 0
for x in t:
    count = count + 1
print(count)
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_tuple_equal_same_elements() {
    let out = jit_capture(
        r#"a = (1, 2, 3)
b = (1, 2, 3)
print(a == b)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_tuple_not_equal_different_order() {
    let out = jit_capture(
        r#"a = (1, 2, 3)
b = (3, 2, 1)
print(a == b)
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_tuple_not_equal_different_len() {
    let out = jit_capture(
        r#"a = (1, 2, 3)
b = (1, 2)
print(a == b)
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_tuple_bool_empty_is_false() {
    let out = jit_capture(
        r#"t = ()
print(bool(t))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_tuple_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"t = (0,)
print(bool(t))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_tuple_concatenation_operator() {
    let out = jit_capture(
        r#"a = (1, 2)
b = (3, 4)
c = a + b
print(len(c))
print(c[0])
print(c[3])
"#,
    );
    assert_output(&out, "4\n1\n4\n");
}

#[test]
fn test_tuple_repetition_operator() {
    let out = jit_capture(
        r#"t = (1, 2) * 3
print(len(t))
print(t[0])
print(t[5])
"#,
    );
    assert_output(&out, "6\n1\n2\n");
}

#[test]
fn test_tuple_contains_present() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(2 in t)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_tuple_contains_absent() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(99 in t)
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_tuple_count_present() {
    let out = jit_capture(
        r#"t = (1, 2, 2, 3, 2)
print(t.count(2))
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_tuple_count_absent_is_zero() {
    let out = jit_capture(
        r#"t = (1, 2, 3)
print(t.count(99))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_tuple_index_present() {
    let out = jit_capture(
        r#"t = (10, 20, 30)
print(t.index(20))
"#,
    );
    assert_output(&out, "1\n");
}

#[test]
fn test_tuple_unpack_into_variables() {
    let out = jit_capture(
        r#"a, b, c = (1, 2, 3)
print(a)
print(b)
print(c)
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

#[test]
fn test_tuple_nested_access() {
    let out = jit_capture(
        r#"t = ((1, 2), (3, 4))
print(t[0][1])
print(t[1][0])
"#,
    );
    assert_output(&out, "2\n3\n");
}

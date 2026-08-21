//! Py3.12 conformance tests for sequence unpacking (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_unpack.py — UnpackTest
//!
//! Coverage: tuple and list unpacking in assignment + for-loop targets,
//! swap idiom (a, b = b, a), nested unpacking, mixed-container unpacking
//! (tuple unpacking a list, list unpacking a tuple), unpacking a string
//! into its characters, and unpacking dict.items().
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_unpack_tuple_basic() {
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
fn test_unpack_list_basic() {
    let out = jit_capture(
        r#"a, b, c = [10, 20, 30]
print(a)
print(b)
print(c)
"#,
    );
    assert_output(&out, "10\n20\n30\n");
}

#[test]
fn test_unpack_swap() {
    let out = jit_capture(
        r#"a = 1
b = 2
a, b = b, a
print(a)
print(b)
"#,
    );
    assert_output(&out, "2\n1\n");
}

#[test]
fn test_unpack_three_way_rotation() {
    let out = jit_capture(
        r#"a = 1
b = 2
c = 3
a, b, c = c, a, b
print(a)
print(b)
print(c)
"#,
    );
    assert_output(&out, "3\n1\n2\n");
}

#[test]
fn test_unpack_pair_from_list() {
    let out = jit_capture(
        r#"x, y = [100, 200]
print(x)
print(y)
"#,
    );
    assert_output(&out, "100\n200\n");
}

#[test]
fn test_unpack_for_loop_tuple_pairs() {
    let out = jit_capture(
        r#"pairs = [(1, 2), (3, 4), (5, 6)]
for a, b in pairs:
    print(a + b)
"#,
    );
    assert_output(&out, "3\n7\n11\n");
}

#[test]
fn test_unpack_for_loop_list_of_lists() {
    let out = jit_capture(
        r#"rows = [[1, 2, 3], [4, 5, 6]]
for a, b, c in rows:
    print(a * 100 + b * 10 + c)
"#,
    );
    assert_output(&out, "123\n456\n");
}

#[test]
fn test_unpack_from_enumerate() {
    let out = jit_capture(
        r#"for i, x in enumerate(["a", "b", "c"]):
    print(i, x)
"#,
    );
    assert_output(&out, "0 a\n1 b\n2 c\n");
}

#[test]
fn test_unpack_from_zip() {
    let out = jit_capture(
        r#"for a, b in zip([1, 2, 3], [10, 20, 30]):
    print(a + b)
"#,
    );
    assert_output(&out, "11\n22\n33\n");
}

#[test]
fn test_unpack_from_dict_items() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2, "c": 3}
total = 0
for k, v in d.items():
    total = total + v
print(total)
"#,
    );
    assert_output(&out, "6\n");
}

#[test]
fn test_unpack_string_to_chars() {
    let out = jit_capture(
        r#"a, b, c = "xyz"
print(a)
print(b)
print(c)
"#,
    );
    assert_output(&out, "x\ny\nz\n");
}

#[test]
fn test_unpack_nested_tuple() {
    let out = jit_capture(
        r#"a, (b, c) = 1, (2, 3)
print(a)
print(b)
print(c)
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

#[test]
fn test_unpack_parens_grouping() {
    let out = jit_capture(
        r#"(a, b, c) = (10, 20, 30)
print(a + b + c)
"#,
    );
    assert_output(&out, "60\n");
}

#[test]
fn test_unpack_in_function_return() {
    let out = jit_capture(
        r#"def f():
    return 1, 2, 3

a, b, c = f()
print(a + b + c)
"#,
    );
    assert_output(&out, "6\n");
}

#[test]
fn test_unpack_single_element() {
    let out = jit_capture(
        r#"a, = (42,)
print(a)
"#,
    );
    assert_output(&out, "42\n");
}

//! Py3.12 conformance tests for `str.join` variants (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_str.py — join
//! sections): join over a list, edge cases (singleton, empty,
//! empty separator), join over a comprehension, a generator
//! expression, and a tuple.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_join_basic_and_edge_cases() {
    let out = jit_capture(
        r#"print(",".join(["a", "b", "c"]))
print("-".join(["one"]))
print("".join(["h", "e", "l", "l", "o"]))
print(" ".join([]))
"#,
    );
    assert_output(&out, "a,b,c\none\nhello\n\n");
}

#[test]
fn test_join_over_listcomp() {
    let out = jit_capture(
        r#"print(":".join([str(x) for x in range(5)]))
"#,
    );
    assert_output(&out, "0:1:2:3:4\n");
}

#[test]
fn test_join_over_genexp_and_tuple() {
    let out = jit_capture(
        r#"print(",".join(str(i) for i in range(6)))
print("|".join(("x", "y", "z")))
"#,
    );
    assert_output(&out, "0,1,2,3,4,5\nx|y|z\n");
}

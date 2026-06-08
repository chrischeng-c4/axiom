//! Py3.12 conformance tests for boolean short-circuit and `not`
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_bool.py and
//! test_grammar.py — boolean operator sections):
//!   `or` returns first truthy without evaluating remainder; `and`
//!   returns first falsy; `not` flips truthiness; default-via-`or`
//!   pattern.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_or_returns_first_truthy() {
    let out = jit_capture(
        r#"print(1 or 2)
print(0 or 5)
print(0 or 0 or 7)
"#,
    );
    assert_output(&out, "1\n5\n7\n");
}

#[test]
fn test_and_returns_first_falsy_or_last() {
    let out = jit_capture(
        r#"print(1 and 2)
print(0 and 5)
print(3 and 4 and 5)
print(3 and 0 and 5)
"#,
    );
    assert_output(&out, "2\n0\n5\n0\n");
}

#[test]
fn test_not_operator_flips_truthiness() {
    let out = jit_capture(
        r#"print(not True)
print(not False)
print(not 0)
print(not 1)
print(not "")
print(not "x")
"#,
    );
    assert_output(&out, "False\nTrue\nTrue\nFalse\nTrue\nFalse\n");
}

#[test]
fn test_or_default_pattern() {
    let out = jit_capture(
        r#"name = "" or "anonymous"
print(name)
count = 0 or 10
print(count)
items = [] or [1, 2]
print(items)
"#,
    );
    assert_output(&out, "anonymous\n10\n[1, 2]\n");
}

//! Py3.12 conformance tests for bool (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_bool.py — BoolTest
//!
//! Coverage: literal True/False, bool() constructor, arithmetic with bools
//! (treated as 0/1), and/or/not operators, comparison, str/repr.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_bool_literal_true() {
    let out = jit_capture(
        r#"print(True)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_bool_literal_false() {
    let out = jit_capture(
        r#"print(False)
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_bool_constructor_from_int_zero() {
    let out = jit_capture(
        r#"print(bool(0))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_bool_constructor_from_int_nonzero() {
    let out = jit_capture(
        r#"print(bool(1))
print(bool(-1))
print(bool(100))
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

#[test]
fn test_bool_constructor_from_str_empty() {
    let out = jit_capture(
        r#"print(bool(""))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_bool_constructor_from_str_nonempty() {
    let out = jit_capture(
        r#"print(bool("x"))
print(bool("0"))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

#[test]
fn test_bool_constructor_default_is_false() {
    let out = jit_capture(
        r#"print(bool())
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_bool_constructor_from_list_empty() {
    let out = jit_capture(
        r#"print(bool([]))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_bool_constructor_from_list_nonempty() {
    let out = jit_capture(
        r#"print(bool([0]))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_bool_constructor_from_none() {
    let out = jit_capture(
        r#"print(bool(None))
"#,
    );
    assert_output(&out, "False\n");
}

#[test]
fn test_bool_arithmetic_add() {
    let out = jit_capture(
        r#"print(True + True)
print(True + False)
print(False + False)
"#,
    );
    assert_output(&out, "2\n1\n0\n");
}

#[test]
fn test_bool_arithmetic_mul() {
    let out = jit_capture(
        r#"print(True * 5)
print(False * 5)
"#,
    );
    assert_output(&out, "5\n0\n");
}

#[test]
fn test_bool_and_short_circuit() {
    let out = jit_capture(
        r#"print(True and True)
print(True and False)
print(False and True)
print(False and False)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\nFalse\n");
}

#[test]
fn test_bool_or_short_circuit() {
    let out = jit_capture(
        r#"print(True or True)
print(True or False)
print(False or True)
print(False or False)
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nFalse\n");
}

#[test]
fn test_bool_not_operator() {
    let out = jit_capture(
        r#"print(not True)
print(not False)
print(not not True)
"#,
    );
    assert_output(&out, "False\nTrue\nTrue\n");
}

#[test]
fn test_bool_equality() {
    let out = jit_capture(
        r#"print(True == True)
print(True == False)
print(True == 1)
print(False == 0)
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nTrue\n");
}

#[test]
fn test_bool_str_conversion() {
    let out = jit_capture(
        r#"print(str(True))
print(str(False))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_bool_int_conversion() {
    let out = jit_capture(
        r#"print(int(True))
print(int(False))
"#,
    );
    assert_output(&out, "1\n0\n");
}

#[test]
fn test_bool_in_if_statement() {
    let out = jit_capture(
        r#"if True:
    print("yes")
if False:
    print("no")
else:
    print("else")
"#,
    );
    assert_output(&out, "yes\nelse\n");
}

#[test]
fn test_bool_comparison_lt() {
    let out = jit_capture(
        r#"print(False < True)
print(True < False)
print(False < False)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\n");
}

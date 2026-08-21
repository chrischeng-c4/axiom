//! Py3.12 conformance tests for operators (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_operator.py — OperatorTestCase
//!
//! Coverage: identity (is / is not), logical short-circuit (and / or),
//! chained comparison, unary +/-, not, membership in / not in,
//! comparison chain across mixed types, augmented assignment evaluation.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_operator_is_same_int() {
    let out = jit_capture(
        r#"a = 5
b = a
print(a is b)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_operator_is_none() {
    let out = jit_capture(
        r#"x = None
print(x is None)
print(x is not None)
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_operator_is_not_distinct_lists() {
    let out = jit_capture(
        r#"a = [1, 2]
b = [1, 2]
print(a is b)
print(a is not b)
"#,
    );
    assert_output(&out, "False\nTrue\n");
}

#[test]
fn test_operator_logical_and_short_circuit() {
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
fn test_operator_logical_or_short_circuit() {
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
fn test_operator_logical_and_returns_value() {
    let out = jit_capture(
        r#"print(1 and 2)
print(0 and 2)
print("a" and "b")
print("" and "b")
"#,
    );
    assert_output(&out, "2\n0\nb\n\n");
}

#[test]
fn test_operator_logical_or_returns_value() {
    let out = jit_capture(
        r#"print(1 or 2)
print(0 or 2)
print("" or "b")
print("a" or "b")
"#,
    );
    assert_output(&out, "1\n2\nb\na\n");
}

#[test]
fn test_operator_not_truthy() {
    let out = jit_capture(
        r#"print(not True)
print(not False)
print(not 0)
print(not 1)
print(not "")
print(not "x")
print(not [])
print(not [1])
"#,
    );
    assert_output(&out, "False\nTrue\nTrue\nFalse\nTrue\nFalse\nTrue\nFalse\n");
}

#[test]
fn test_operator_chained_comparison_lt() {
    let out = jit_capture(
        r#"print(1 < 2 < 3)
print(1 < 3 < 2)
print(3 < 2 < 1)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\n");
}

#[test]
fn test_operator_chained_comparison_mixed() {
    let out = jit_capture(
        r#"print(1 < 2 == 2)
print(1 <= 1 < 2)
print(3 > 2 > 1)
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

#[test]
fn test_operator_unary_negation() {
    let out = jit_capture(
        r#"print(-5)
print(-(-5))
print(--5)
"#,
    );
    assert_output(&out, "-5\n5\n5\n");
}

#[test]
fn test_operator_unary_plus() {
    let out = jit_capture(
        r#"print(+5)
print(+(-5))
"#,
    );
    assert_output(&out, "5\n-5\n");
}

#[test]
fn test_operator_membership_in_list() {
    let out = jit_capture(
        r#"print(2 in [1, 2, 3])
print(4 in [1, 2, 3])
print(2 not in [1, 2, 3])
print(4 not in [1, 2, 3])
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\nTrue\n");
}

#[test]
fn test_operator_membership_in_str() {
    let out = jit_capture(
        r#"print("ll" in "hello")
print("xy" in "hello")
print("ll" not in "hello")
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\n");
}

#[test]
fn test_operator_membership_in_tuple() {
    let out = jit_capture(
        r#"print(2 in (1, 2, 3))
print(4 in (1, 2, 3))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

#[test]
fn test_operator_membership_in_dict() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
print("a" in d)
print("c" in d)
print("a" not in d)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\n");
}

#[test]
fn test_operator_comparison_strings_lex() {
    let out = jit_capture(
        r#"print("a" < "b")
print("apple" < "banana")
print("abc" == "abc")
print("abc" != "abd")
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nTrue\n");
}

#[test]
fn test_operator_arith_precedence() {
    let out = jit_capture(
        r#"print(1 + 2 * 3)
print((1 + 2) * 3)
print(2 ** 3 ** 2)
print(20 // 3 % 4)
"#,
    );
    assert_output(&out, "7\n9\n512\n2\n");
}

#[test]
fn test_operator_bitwise_combined() {
    let out = jit_capture(
        r#"print(0b1100 & 0b1010)
print(0b1100 | 0b1010)
print(0b1100 ^ 0b1010)
print(~0)
print(1 << 4)
print(16 >> 2)
"#,
    );
    assert_output(&out, "8\n14\n6\n-1\n16\n4\n");
}

#[test]
fn test_operator_augmented_assignment_chain() {
    let out = jit_capture(
        r#"x = 10
x += 5
x -= 2
x *= 3
x //= 2
x %= 5
print(x)
"#,
    );
    assert_output(&out, "4\n");
}

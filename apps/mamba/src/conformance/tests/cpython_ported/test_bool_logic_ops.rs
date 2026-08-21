//! Py3.12 conformance tests for boolean logical operators and ternary
//! conditional (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_bool.py and
//! test_grammar.py — logical-operator and conditional-expression
//! sections):
//!   `and`/`or`/`not` short-circuiting, XOR-like composition, and the
//!   `x if cond else y` ternary expression.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_and_or_not_basic() {
    let out = jit_capture(
        r#"a = True
b = False
print(a and b)
print(a or b)
print(not a)
print(not b)
"#,
    );
    assert_output(&out, "False\nTrue\nFalse\nTrue\n");
}

#[test]
fn test_compound_logical_expression() {
    let out = jit_capture(
        r#"a = True
b = False
print(a and not b)
print((a or b) and not (a and b))
print(not (a and b) and (a or b))
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

#[test]
fn test_ternary_conditional() {
    let out = jit_capture(
        r#"a = True
b = False
print(True if a else False)
print("yes" if a else "no")
print(a if not b else b)
print(10 if 2 > 1 else 20)
print("small" if 3 < 5 else "big")
"#,
    );
    assert_output(&out, "True\nyes\nTrue\n10\nsmall\n");
}

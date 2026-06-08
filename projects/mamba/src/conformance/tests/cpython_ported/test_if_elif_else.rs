//! Py3.12 conformance tests for `if` / `elif` / `else` and ternary
//! expressions (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_grammar.py — `if`
//! statement sections):
//!   elif chain selection, else fallback, conditional expression.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_elif_chain_picks_first_match() {
    let out = jit_capture(
        r#"x = 5
if x > 10:
    print("big")
elif x > 3:
    print("medium")
else:
    print("small")
"#,
    );
    assert_output(&out, "medium\n");
}

#[test]
fn test_else_runs_when_no_branch_matches() {
    let out = jit_capture(
        r#"if False:
    print("no")
else:
    print("else")
if True:
    print("yes")
"#,
    );
    assert_output(&out, "else\nyes\n");
}

#[test]
fn test_conditional_expression_ternary() {
    let out = jit_capture(
        r#"score = 85
grade = "A" if score >= 90 else "B" if score >= 80 else "C"
print(grade)
print("yes" if 1 else "no")
print("yes" if 0 else "no")
"#,
    );
    assert_output(&out, "B\nyes\nno\n");
}

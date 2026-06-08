//! Py3.12 conformance tests for `else` clauses on try / for / while
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_exceptions.py +
//! Lib/test/test_grammar.py — try/else and loop/else sections):
//!   try/except/else runs `else` only on success, for/else skips `else`
//!   when `break` fires, while/else runs `else` after natural exit.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_try_else_runs_on_success_only() {
    let out = jit_capture(
        r#"def f(x):
    try:
        y = 100 // x
    except ZeroDivisionError:
        return "div by zero"
    else:
        return f"ok {y}"
print(f(0))
print(f(5))
print(f(10))
"#,
    );
    assert_output(&out, "div by zero\nok 20\nok 10\n");
}

#[test]
fn test_for_else_skipped_on_break() {
    let out = jit_capture(
        r#"for i in range(3):
    if i == 1:
        break
    print(i)
else:
    print("not reached")
print("after")
"#,
    );
    assert_output(&out, "0\nafter\n");
}

#[test]
fn test_while_else_runs_after_natural_exit() {
    let out = jit_capture(
        r#"i = 0
while i < 3:
    print(i)
    i += 1
else:
    print("while done")
"#,
    );
    assert_output(&out, "0\n1\n2\nwhile done\n");
}

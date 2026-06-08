//! Py3.12 conformance tests for the `assert` statement (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_grammar.py — assertion
//! statements):
//!   passing assert is a no-op, failing assert raises AssertionError
//!   carrying the supplied message.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_assert_passes_when_truthy() {
    let out = jit_capture(
        r#"def check(x):
    assert x > 0, "must be positive"
    return x
print(check(5))
"#,
    );
    assert_output(&out, "5\n");
}

#[test]
fn test_assert_raises_with_message() {
    let out = jit_capture(
        r#"def check(x):
    assert x > 0, "must be positive"
    return x
try:
    check(-1)
except AssertionError as e:
    print("assertion:", e)
"#,
    );
    assert_output(&out, "assertion: must be positive\n");
}

//! Py3.12 conformance tests for `for`/`while` else clauses with
//! interleaved `break` and `continue` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_grammar.py — loop
//! else sections): for-else fires when no break, for-else suppressed
//! by break, and the nested-for-with-continue-and-break composition.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_for_else_fires_when_no_break() {
    let out = jit_capture(
        r#"for n in [1, 2, 3]:
    if n == 99:
        break
else:
    print("not found")
"#,
    );
    assert_output(&out, "not found\n");
}

#[test]
fn test_for_else_suppressed_by_break() {
    let out = jit_capture(
        r#"for n in [1, 2, 99, 3]:
    if n == 99:
        print("found")
        break
else:
    print("missing")
"#,
    );
    assert_output(&out, "found\n");
}

#[test]
fn test_nested_loop_with_continue_and_else() {
    let out = jit_capture(
        r#"for i in range(3):
    for j in range(3):
        if i == j:
            continue
        if i + j >= 4:
            break
        print(i, j)
    else:
        print("inner else", i)
"#,
    );
    assert_output(
        &out,
        "0 1\n0 2\ninner else 0\n1 0\n1 2\ninner else 1\n2 0\n2 1\ninner else 2\n",
    );
}

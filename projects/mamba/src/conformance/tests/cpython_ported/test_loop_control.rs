//! Py3.12 conformance tests for `break` / `continue` in loops
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_grammar.py — loop
//! control sections):
//!   `continue` skips body, `break` terminates loop, nested loops with
//!   inner `break` exit only the inner level.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_for_continue_and_break_combined() {
    let out = jit_capture(
        r#"for i in range(10):
    if i == 3:
        continue
    if i == 7:
        break
    print(i)
"#,
    );
    assert_output(&out, "0\n1\n2\n4\n5\n6\n");
}

#[test]
fn test_while_continue_skips_even() {
    let out = jit_capture(
        r#"i = 0
while i < 10:
    i += 1
    if i % 2 == 0:
        continue
    print(i)
"#,
    );
    assert_output(&out, "1\n3\n5\n7\n9\n");
}

#[test]
fn test_nested_for_inner_break_only() {
    let out = jit_capture(
        r#"for i in range(3):
    for j in range(3):
        if j > i:
            break
        print(i, j)
"#,
    );
    assert_output(&out, "0 0\n1 0\n1 1\n2 0\n2 1\n2 2\n");
}

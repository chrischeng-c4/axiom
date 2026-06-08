//! Py3.12 conformance tests for assignment expressions (PEP 572,
//! walrus operator) (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_named_expressions.py):
//!   walrus in if-condition, walrus in list literal, walrus in while.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_walrus_in_if_condition() {
    let out = jit_capture(
        r#"data = [1, 2, 3, 4, 5]
if (n := len(data)) > 3:
    print(f"got {n}")
"#,
    );
    assert_output(&out, "got 5\n");
}

#[test]
fn test_walrus_in_list_literal() {
    let out = jit_capture(
        r#"print([y := 10, y + 1, y + 2])
"#,
    );
    assert_output(&out, "[10, 11, 12]\n");
}

#[test]
fn test_walrus_in_while_drain_index() {
    let out = jit_capture(
        r#"nums = [1, 2, 3, 4, 5]
total = 0
i = 0
while (n := nums[i] if i < len(nums) else None) is not None:
    total += n
    i += 1
print(total)
"#,
    );
    assert_output(&out, "15\n");
}

#[test]
fn test_walrus_captures_for_later_use() {
    let out = jit_capture(
        r#"if (x := 7 * 6) > 40:
    print(x)
print(x + 1)
"#,
    );
    assert_output(&out, "42\n43\n");
}

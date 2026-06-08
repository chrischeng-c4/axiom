//! Py3.12 conformance tests for `round` and `sum` builtins (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py — `round`
//! and `sum` sections):
//!   `round` to integer (banker's rounding on .5), `round` with
//!   `ndigits`, `sum` over int / float / `range` iterables.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_round_to_integer_uses_bankers_rounding() {
    let out = jit_capture(
        r#"print(round(3.7))
print(round(3.4))
print(round(2.5))
print(round(3.5))
"#,
    );
    assert_output(&out, "4\n3\n2\n4\n");
}

#[test]
fn test_round_with_ndigits() {
    let out = jit_capture(
        r#"print(round(3.14159, 2))
print(round(3.14159, 4))
print(round(2.71828, 3))
"#,
    );
    assert_output(&out, "3.14\n3.1416\n2.718\n");
}

#[test]
fn test_sum_over_int_float_range() {
    let out = jit_capture(
        r#"print(sum([1, 2, 3, 4]))
print(sum([1.5, 2.5]))
print(sum(range(10)))
print(sum([]))
"#,
    );
    assert_output(&out, "10\n4.0\n45\n0\n");
}

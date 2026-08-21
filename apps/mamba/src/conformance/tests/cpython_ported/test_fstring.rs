//! Py3.12 conformance tests for f-strings (PEP 498) (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_fstring.py — basic
//! interpolation and format-spec coverage):
//!   variable interpolation, expression interpolation, alignment-width,
//!   float precision, hex format with `#`, percentage format.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_fstring_variable_interpolation() {
    let out = jit_capture(
        r#"x = 42
print(f"x={x}")
"#,
    );
    assert_output(&out, "x=42\n");
}

#[test]
fn test_fstring_expression_interpolation() {
    let out = jit_capture(
        r#"print(f"{2+3}")
"#,
    );
    assert_output(&out, "5\n");
}

#[test]
fn test_fstring_right_align_width() {
    let out = jit_capture(
        r#"print(f"{'hi':>5}")
"#,
    );
    assert_output(&out, "   hi\n");
}

#[test]
fn test_fstring_float_precision() {
    let out = jit_capture(
        r#"print(f"{3.14:.2f}")
"#,
    );
    assert_output(&out, "3.14\n");
}

#[test]
fn test_fstring_alt_hex_format() {
    let out = jit_capture(
        r#"print(f"{255:#x}")
"#,
    );
    assert_output(&out, "0xff\n");
}

#[test]
fn test_fstring_percentage_format() {
    let out = jit_capture(
        r#"print(f"{0.5:.0%}")
"#,
    );
    assert_output(&out, "50%\n");
}

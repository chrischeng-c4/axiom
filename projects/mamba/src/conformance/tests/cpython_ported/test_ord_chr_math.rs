//! Py3.12 conformance tests for `ord`/`chr` roundtrips and
//! basic `math` module functions (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_unicode.py and
//! Lib/test/test_math.py — character/number primitive sections):
//! ASCII codepoint roundtrips, `math.floor`/`math.ceil` toward
//! zero/away semantics, and `math.sqrt` on integer and float inputs.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_ord_chr_ascii_roundtrip() {
    let out = jit_capture(
        r#"print(ord('A'))
print(ord('z'))
print(chr(65))
print(chr(122))
print(chr(ord('A') + 1))
"#,
    );
    assert_output(&out, "65\n122\nA\nz\nB\n");
}

#[test]
fn test_math_floor_ceil_positive_and_negative() {
    let out = jit_capture(
        r#"import math
print(math.floor(3.7))
print(math.ceil(3.2))
print(math.floor(-2.3))
print(math.ceil(-2.7))
"#,
    );
    assert_output(&out, "3\n4\n-3\n-2\n");
}

#[test]
fn test_math_sqrt_int_and_float() {
    let out = jit_capture(
        r#"import math
print(math.sqrt(16))
print(math.sqrt(2.0))
"#,
    );
    assert_output(&out, "4.0\n1.4142135623730951\n");
}

//! Py3.12 conformance tests for f-string expression embedding
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_fstring.py —
//! expression-embedding sections): arithmetic in expression position,
//! method calls, `!r` conversion specifier, and a ternary expression
//! inside an f-string.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_fstring_arithmetic_and_method_call() {
    let out = jit_capture(
        r#"name = "Alice"
age = 30
print(f"{age * 2}")
print(f"{1 + 2 + 3}")
print(f"len={len(name)}")
print(f"{name.upper()}")
"#,
    );
    assert_output(&out, "60\n6\nlen=5\nALICE\n");
}

#[test]
fn test_fstring_repr_conversion() {
    let out = jit_capture(
        r#"name = "Alice"
print(f"{name!r}")
print(f"{'hi'!r}")
print(f"{42!r}")
"#,
    );
    assert_output(&out, "'Alice'\n'hi'\n42\n");
}

#[test]
fn test_fstring_ternary_expression() {
    let out = jit_capture(
        r#"age = 30
print(f"{'yes' if age > 18 else 'no'}")
print(f"{'adult' if age >= 18 else 'child'}")
print(f"{'+' if 1 > 0 else '-'}")
"#,
    );
    assert_output(&out, "yes\nadult\n+\n");
}

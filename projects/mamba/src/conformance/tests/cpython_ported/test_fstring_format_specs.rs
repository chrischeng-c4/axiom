//! Py3.12 conformance tests for f-string format spec variants
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_fstring.py —
//! format-spec sections): float precision (`:.Nf`), integer
//! width padding, integer base conversion (`:x`, `:b`), and
//! embedded expressions / method calls inside replacement fields.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_fstring_float_precision() {
    let out = jit_capture(
        r#"pi = 3.14159
print(f"pi ~ {pi:.2f}")
print(f"pi to 4 = {pi:.4f}")
"#,
    );
    assert_output(&out, "pi ~ 3.14\npi to 4 = 3.1416\n");
}

#[test]
fn test_fstring_int_width_and_base() {
    let out = jit_capture(
        r#"n = 42
print(f"padded: {n:5}")
print(f"hex: {n:x}")
print(f"bin: {n:b}")
"#,
    );
    assert_output(&out, "padded:    42\nhex: 2a\nbin: 101010\n");
}

#[test]
fn test_fstring_embedded_expressions_and_methods() {
    let out = jit_capture(
        r#"name = "world"
n = 42
print(f"hello {name}")
print(f"{n} squared is {n * n}")
print(f"upper: {name.upper()}")
print(f"len = {len(name)}")
print(f"{'x' * 3}")
"#,
    );
    assert_output(
        &out,
        "hello world\n42 squared is 1764\nupper: WORLD\nlen = 5\nxxx\n",
    );
}

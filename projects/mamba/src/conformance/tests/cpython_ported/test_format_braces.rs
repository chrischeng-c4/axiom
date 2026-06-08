//! Py3.12 conformance tests for `str.format` placeholder forms
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_format.py — brace
//! placeholder sections): positional `{}`/`{0}`, named `{name}`,
//! integer width and zero-pad, fixed precision, and right/left/center
//! alignment.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_positional_and_named_placeholders() {
    let out = jit_capture(
        r#"print("Name: {}, Age: {}".format("Alice", 30))
print("{0} + {1} = {2}".format(2, 3, 5))
print("{name} is {age}".format(name="Bob", age=25))
"#,
    );
    assert_output(&out, "Name: Alice, Age: 30\n2 + 3 = 5\nBob is 25\n");
}

#[test]
fn test_int_width_zero_pad_and_precision() {
    let out = jit_capture(
        r#"print("{:5d}".format(42))
print("{:05d}".format(42))
print("{:.2f}".format(3.14159))
print("{:.0f}".format(2.5))
print("{:6.3f}".format(1.5))
"#,
    );
    assert_output(&out, "   42\n00042\n3.14\n2\n 1.500\n");
}

#[test]
fn test_align_right_left_center() {
    let out = jit_capture(
        r#"print("{:>10}".format("right"))
print("{:<10}|".format("left"))
print("{:^10}|".format("mid"))
print("{:*^9}".format("hi"))
"#,
    );
    assert_output(
        &out,
        "     right\nleft      |\n   mid    |\n***hi****\n",
    );
}

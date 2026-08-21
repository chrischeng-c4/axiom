//! Py3.12 conformance tests for function call signatures (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_call.py — basic call
//! signature sections):
//!   default arguments, keyword arguments, positional+keyword mix.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_default_argument_value() {
    let out = jit_capture(
        r#"def greet(name, greeting="Hello"):
    print(greeting, name)
greet("World")
greet("Alice", "Hi")
"#,
    );
    assert_output(&out, "Hello World\nHi Alice\n");
}

#[test]
fn test_keyword_argument_at_call_site() {
    let out = jit_capture(
        r#"def make(a, b, c):
    print(a, b, c)
make(1, 2, 3)
make(a=1, b=2, c=3)
make(1, c=3, b=2)
"#,
    );
    assert_output(&out, "1 2 3\n1 2 3\n1 2 3\n");
}

#[test]
fn test_default_overridden_by_keyword() {
    let out = jit_capture(
        r#"def power(base, exp=2):
    return base ** exp
print(power(3))
print(power(3, 3))
print(power(3, exp=4))
"#,
    );
    assert_output(&out, "9\n27\n81\n");
}

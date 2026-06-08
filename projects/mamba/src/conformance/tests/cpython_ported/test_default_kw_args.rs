//! Py3.12 conformance tests for default and keyword arguments
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_funcattrs.py /
//! test_grammar.py — call-signature sections):
//!   omitted defaults, positional override of defaults, keyword call
//!   syntax, and out-of-order keyword arguments.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_default_value_used_when_omitted() {
    let out = jit_capture(
        r#"def greet(name="world"):
    return "Hello, " + name + "!"

print(greet())
print(greet("Alice"))
print(greet(name="Bob"))
"#,
    );
    assert_output(&out, "Hello, world!\nHello, Alice!\nHello, Bob!\n");
}

#[test]
fn test_multiple_defaults_and_keyword_call() {
    let out = jit_capture(
        r#"def power(base, exp=2):
    return base ** exp

print(power(3))
print(power(3, 3))
print(power(base=4))
"#,
    );
    assert_output(&out, "9\n27\n16\n");
}

#[test]
fn test_keyword_args_out_of_order() {
    let out = jit_capture(
        r#"def power(base, exp=2):
    return base ** exp

print(power(exp=4, base=2))
print(power(exp=0, base=99))
print(power(base=5, exp=3))
"#,
    );
    assert_output(&out, "16\n1\n125\n");
}

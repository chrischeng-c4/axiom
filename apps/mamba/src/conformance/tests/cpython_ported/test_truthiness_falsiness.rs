//! Py3.12 conformance tests for object truthiness (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_bool.py —
//! truthiness sections): `bool` coercion across types, truthiness
//! used in `if`, and chained logical operators on mixed-type values.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_bool_coercion_across_types() {
    let out = jit_capture(
        r#"print(bool(0))
print(bool(1))
print(bool(""))
print(bool("hi"))
print(bool([]))
print(bool([0]))
print(bool({}))
print(bool({1: 2}))
print(bool(None))
"#,
    );
    assert_output(
        &out,
        "False\nTrue\nFalse\nTrue\nFalse\nTrue\nFalse\nTrue\nFalse\n",
    );
}

#[test]
fn test_truthiness_in_if() {
    let out = jit_capture(
        r#"if [] or [1]:
    print("nonempty")
if not 0 and "x":
    print("not0 and x")
if None or 0 or "":
    print("never")
else:
    print("all falsy")
if "" or {} or 0:
    print("never2")
else:
    print("else2")
"#,
    );
    assert_output(&out, "nonempty\nnot0 and x\nall falsy\nelse2\n");
}

#[test]
fn test_short_circuit_returns_operand() {
    let out = jit_capture(
        r#"print(0 or "fallback")
print("first" or "second")
print(0 and "never")
print("yes" and "winner")
print([] or [1, 2])
print([3] and [4, 5])
"#,
    );
    assert_output(&out, "fallback\nfirst\n0\nwinner\n[1, 2]\n[4, 5]\n");
}

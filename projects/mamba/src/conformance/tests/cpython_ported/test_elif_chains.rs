//! Py3.12 conformance tests for module-level elif chains
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_compile.py and
//! Lib/test/test_syntax.py — elif chain sections): multi-branch
//! grade table, signed-range classifier, and an even/odd/zero
//! discriminator. Module-level elif keeps the test independent
//! of function-call typing.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_grade_table_via_elif_chain() {
    let out = jit_capture(
        r#"for s in [95, 85, 72, 65, 50]:
    if s >= 90:
        print(s, "A")
    elif s >= 80:
        print(s, "B")
    elif s >= 70:
        print(s, "C")
    elif s >= 60:
        print(s, "D")
    else:
        print(s, "F")
"#,
    );
    assert_output(&out, "95 A\n85 B\n72 C\n65 D\n50 F\n");
}

#[test]
fn test_signed_range_classifier_via_elif() {
    let out = jit_capture(
        r#"for n in [-3, 0, 5, 42, 999]:
    if n < 0:
        print(n, "neg")
    elif n == 0:
        print(n, "zero")
    elif n < 10:
        print(n, "small")
    elif n < 100:
        print(n, "med")
    else:
        print(n, "big")
"#,
    );
    assert_output(
        &out,
        "-3 neg\n0 zero\n5 small\n42 med\n999 big\n",
    );
}

#[test]
fn test_three_way_elif_discriminator() {
    let out = jit_capture(
        r#"for n in [-2, -1, 0, 1, 2]:
    if n == 0:
        print(n, "zero")
    elif n % 2 == 0:
        print(n, "even")
    else:
        print(n, "odd")
"#,
    );
    assert_output(
        &out,
        "-2 even\n-1 odd\n0 zero\n1 odd\n2 even\n",
    );
}

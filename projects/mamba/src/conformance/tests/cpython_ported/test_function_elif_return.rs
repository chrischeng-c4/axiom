//! Py3.12 conformance attack tests for function-internal
//! `if/elif`-chain early returns (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_compile.py and
//! Lib/test/test_funcattrs.py — function control-flow sections):
//! a function body that selects one of several return values via
//! an `if`/`elif`/`else` ladder must return the value of the
//! first satisfied branch, not always fall through to `else`.
//!
//! These tests directly attack a class of regressions where the
//! function lowering collapses or reorders early-return arms.
//!
//! @issue #759

use super::{assert_output, jit_capture};

// FIXME: mamba bug — function-internal `if/elif/elif/else: return` chain
// always falls through to the `else` branch. Module-level elif and 2-arm
// (`if cond: return X; return Y`) forms work correctly; only the 3+-arm
// function-internal form is broken. Captured here as a regression target.
#[test]
#[ignore]
fn test_function_if_elif_else_early_returns_grade() {
    let out = jit_capture(
        r#"def grade(score):
    if score >= 90:
        return "A"
    elif score >= 80:
        return "B"
    elif score >= 70:
        return "C"
    elif score >= 60:
        return "D"
    else:
        return "F"

for s in [95, 85, 72, 65, 50]:
    print(s, grade(s))
"#,
    );
    assert_output(&out, "95 A\n85 B\n72 C\n65 D\n50 F\n");
}

// FIXME: mamba bug — function-internal `if/elif/elif/else` chain that
// assigns to a local then returns it always returns the FIRST arm's
// value (opposite failure mode from the early-return variant above).
#[test]
#[ignore]
fn test_function_if_elif_assign_then_return() {
    let out = jit_capture(
        r#"def classify(n):
    if n < 0:
        r = "neg"
    elif n == 0:
        r = "zero"
    elif n < 10:
        r = "small"
    else:
        r = "big"
    return r

for n in [-3, 0, 5, 999]:
    print(n, classify(n))
"#,
    );
    assert_output(&out, "-3 neg\n0 zero\n5 small\n999 big\n");
}

#[test]
fn test_function_two_arm_early_return() {
    let out = jit_capture(
        r#"def sign(n):
    if n > 0:
        return "pos"
    return "nonpos"

print(sign(5))
print(sign(0))
print(sign(-3))
"#,
    );
    assert_output(&out, "pos\nnonpos\nnonpos\n");
}

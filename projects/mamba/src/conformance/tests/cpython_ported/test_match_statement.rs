//! Py3.12 conformance tests for `match`/`case` (PEP 634) (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_patma.py — pattern
//! matching):
//!   literal patterns, guard clauses, tuple destructuring, sequence
//!   patterns with capture and star-rest.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_match_literal_with_guard_and_default() {
    let out = jit_capture(
        r#"def classify(x):
    match x:
        case 0:
            return "zero"
        case n if n > 0:
            return "positive"
        case _:
            return "negative"
print(classify(0))
print(classify(5))
print(classify(-2))
"#,
    );
    assert_output(&out, "zero\npositive\nnegative\n");
}

#[test]
fn test_match_tuple_destructuring_with_zero() {
    let out = jit_capture(
        r#"def describe(p):
    match p:
        case (0, 0):
            return "origin"
        case (x, 0):
            return f"x-axis at {x}"
        case (0, y):
            return f"y-axis at {y}"
        case (x, y):
            return f"point ({x}, {y})"
print(describe((0, 0)))
print(describe((3, 0)))
print(describe((0, 5)))
print(describe((1, 2)))
"#,
    );
    assert_output(
        &out,
        "origin\nx-axis at 3\ny-axis at 5\npoint (1, 2)\n",
    );
}

#[test]
fn test_match_sequence_lengths_with_star_rest() {
    let out = jit_capture(
        r#"def classify(xs):
    match xs:
        case []:
            return "empty"
        case [a]:
            return f"single {a}"
        case [a, b]:
            return f"pair {a},{b}"
        case [a, *rest]:
            return f"head {a} rest_len {len(rest)}"
print(classify([]))
print(classify([5]))
print(classify([1, 2]))
print(classify([1, 2, 3, 4]))
"#,
    );
    assert_output(
        &out,
        "empty\nsingle 5\npair 1,2\nhead 1 rest_len 3\n",
    );
}

#[test]
fn test_match_string_literal() {
    let out = jit_capture(
        r#"def label(s):
    match s:
        case "yes":
            return 1
        case "no":
            return 0
        case _:
            return -1
print(label("yes"))
print(label("no"))
print(label("maybe"))
"#,
    );
    assert_output(&out, "1\n0\n-1\n");
}

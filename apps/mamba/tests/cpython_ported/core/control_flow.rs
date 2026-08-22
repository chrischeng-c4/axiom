//! Ported from Lib/test/test_grammar_ported.py
//! Integration tests: core/control_flow.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_grammar_ported.py`.
#[test]
fn test_for_continue_and_break_combined() {
    let out = jit_capture(
        r#"for i in range(10):
    if i == 3:
        continue
    if i == 7:
        break
    print(i)
"#,
    );
    assert_output(&out, "0\n1\n2\n4\n5\n6\n");
}

/// Ported from `Lib/test/test_grammar_ported.py`.
#[test]
fn test_while_continue_skips_even() {
    let out = jit_capture(
        r#"i = 0
while i < 10:
    i += 1
    if i % 2 == 0:
        continue
    print(i)
"#,
    );
    assert_output(&out, "1\n3\n5\n7\n9\n");
}

/// Ported from `Lib/test/test_grammar_ported.py`.
#[test]
fn test_nested_for_inner_break_only() {
    let out = jit_capture(
        r#"for i in range(3):
    for j in range(3):
        if j > i:
            break
        print(i, j)
"#,
    );
    assert_output(&out, "0 0\n1 0\n1 1\n2 0\n2 1\n2 2\n");
}

/// Ported from `Lib/test/test_grammar_ported.py`.
#[test]
fn test_for_else_fires_when_no_break() {
    let out = jit_capture(
        r#"for n in [1, 2, 3]:
    if n == 99:
        break
else:
    print("not found")
"#,
    );
    assert_output(&out, "not found\n");
}

/// Ported from `Lib/test/test_grammar_ported.py`.
#[test]
fn test_for_else_suppressed_by_break() {
    let out = jit_capture(
        r#"for n in [1, 2, 99, 3]:
    if n == 99:
        print("found")
        break
else:
    print("missing")
"#,
    );
    assert_output(&out, "found\n");
}

/// Ported from `Lib/test/test_grammar_ported.py`.
#[test]
fn test_nested_loop_with_continue_and_else() {
    let out = jit_capture(
        r#"for i in range(3):
    for j in range(3):
        if i == j:
            continue
        if i + j >= 4:
            break
        print(i, j)
    else:
        print("inner else", i)
"#,
    );
    assert_output(
        &out,
        "0 1\n0 2\ninner else 0\n1 0\n1 2\ninner else 1\n2 0\n2 1\ninner else 2\n",
    );
}

/// Ported from `Lib/test/test_grammar_ported.py`.
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

/// Ported from `Lib/test/test_grammar_ported.py`.
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
    assert_output(&out, "origin\nx-axis at 3\ny-axis at 5\npoint (1, 2)\n");
}

/// Ported from `Lib/test/test_grammar_ported.py`.
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
    assert_output(&out, "empty\nsingle 5\npair 1,2\nhead 1 rest_len 3\n");
}

/// Ported from `Lib/test/test_grammar_ported.py`.
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

// FIXME: mamba bug — function-internal `if/elif/elif/else: return` chain
// always falls through to the `else` branch. Module-level elif and 2-arm
// (`if cond: return X; return Y`) forms work correctly; only the 3+-arm
// function-internal form is broken. Captured here as a regression target.
#[test]
#[ignore]
/// Ported from `Lib/test/test_grammar_ported.py`.
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
/// Ported from `Lib/test/test_grammar_ported.py`.
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

/// Ported from `Lib/test/test_grammar_ported.py`.
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

/// Ported from `Lib/test/test_grammar_ported.py`.
#[test]
fn test_nested_accumulator() {
    let out = jit_capture(
        r#"total = 0
for i in range(3):
    for j in range(3):
        total = total + 1
print(total)
"#,
    );
    assert_output(&out, "9\n");
}

/// Ported from `Lib/test/test_grammar_ported.py`.
#[test]
fn test_matrix_print_with_end_space() {
    let out = jit_capture(
        r#"for i in range(3):
    for j in range(3):
        print(i * 3 + j, end=" ")
    print()
"#,
    );
    assert_output(&out, "0 1 2 \n3 4 5 \n6 7 8 \n");
}

/// Ported from `Lib/test/test_grammar_ported.py`.
#[test]
fn test_pair_combinations() {
    let out = jit_capture(
        r#"items = [1, 2, 3, 4]
pairs = []
for i in range(len(items)):
    for j in range(i + 1, len(items)):
        pairs.append((items[i], items[j]))
print(pairs)
"#,
    );
    assert_output(&out, "[(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)]\n");
}


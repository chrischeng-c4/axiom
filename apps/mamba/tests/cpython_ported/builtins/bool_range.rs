//! Ported from Lib/test/test_bool_ported.py
//! Integration tests: builtins/bool_range.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_forward_steps() {
    let out = jit_capture(
        r#"print(list(range(10)))
print(list(range(2, 8)))
print(list(range(0, 10, 2)))
"#,
    );
    assert_output(
        &out,
        "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\n[2, 3, 4, 5, 6, 7]\n[0, 2, 4, 6, 8]\n",
    );
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_reverse_and_empty() {
    let out = jit_capture(
        r#"print(list(range(10, 0, -1)))
print(list(range(10, 0, -2)))
print(list(range(5, 5)))
print(list(range(0, -5, -1)))
"#,
    );
    assert_output(
        &out,
        "[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]\n[10, 8, 6, 4, 2]\n[]\n[0, -1, -2, -3, -4]\n",
    );
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_len_and_sum() {
    let out = jit_capture(
        r#"print(len(range(100)))
print(sum(range(1, 11)))
"#,
    );
    assert_output(&out, "100\n55\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_literal_true() {
    let out = jit_capture(
        r#"print(True)
"#,
    );
    assert_output(&out, "True\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_literal_false() {
    let out = jit_capture(
        r#"print(False)
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_constructor_from_int_zero() {
    let out = jit_capture(
        r#"print(bool(0))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_constructor_from_int_nonzero() {
    let out = jit_capture(
        r#"print(bool(1))
print(bool(-1))
print(bool(100))
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_constructor_default_is_false() {
    let out = jit_capture(
        r#"print(bool())
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_constructor_from_none() {
    let out = jit_capture(
        r#"print(bool(None))
"#,
    );
    assert_output(&out, "False\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_arithmetic_add() {
    let out = jit_capture(
        r#"print(True + True)
print(True + False)
print(False + False)
"#,
    );
    assert_output(&out, "2\n1\n0\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_arithmetic_mul() {
    let out = jit_capture(
        r#"print(True * 5)
print(False * 5)
"#,
    );
    assert_output(&out, "5\n0\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_not_operator() {
    let out = jit_capture(
        r#"print(not True)
print(not False)
print(not not True)
"#,
    );
    assert_output(&out, "False\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_equality() {
    let out = jit_capture(
        r#"print(True == True)
print(True == False)
print(True == 1)
print(False == 0)
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_int_conversion() {
    let out = jit_capture(
        r#"print(int(True))
print(int(False))
"#,
    );
    assert_output(&out, "1\n0\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_in_if_statement() {
    let out = jit_capture(
        r#"if True:
    print("yes")
if False:
    print("no")
else:
    print("else")
"#,
    );
    assert_output(&out, "yes\nelse\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_bool_comparison_lt() {
    let out = jit_capture(
        r#"print(False < True)
print(True < False)
print(False < False)
"#,
    );
    assert_output(&out, "True\nFalse\nFalse\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_and_or_not_basic() {
    let out = jit_capture(
        r#"a = True
b = False
print(a and b)
print(a or b)
print(not a)
print(not b)
"#,
    );
    assert_output(&out, "False\nTrue\nFalse\nTrue\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_compound_logical_expression() {
    let out = jit_capture(
        r#"a = True
b = False
print(a and not b)
print((a or b) and not (a and b))
print(not (a and b) and (a or b))
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_ternary_conditional() {
    let out = jit_capture(
        r#"a = True
b = False
print(True if a else False)
print("yes" if a else "no")
print(a if not b else b)
print(10 if 2 > 1 else 20)
print("small" if 3 < 5 else "big")
"#,
    );
    assert_output(&out, "True\nyes\nTrue\n10\nsmall\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_one_and_two_arg_forms() {
    let out = jit_capture(
        r#"print(list(range(5)))
print(list(range(2, 7)))
"#,
    );
    assert_output(&out, "[0, 1, 2, 3, 4]\n[2, 3, 4, 5, 6]\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_with_positive_step() {
    let out = jit_capture(
        r#"print(list(range(1, 10, 2)))
print(list(range(0, 20, 5)))
"#,
    );
    assert_output(&out, "[1, 3, 5, 7, 9]\n[0, 5, 10, 15]\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_with_negative_step() {
    let out = jit_capture(
        r#"print(list(range(10, 0, -1)))
print(list(range(10, 0, -2)))
"#,
    );
    assert_output(&out, "[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]\n[10, 8, 6, 4, 2]\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_len_and_empty() {
    let out = jit_capture(
        r#"print(len(range(100)))
print(list(range(0)))
print(list(range(5, 5)))
"#,
    );
    assert_output(&out, "100\n[]\n[]\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_stop_only_len() {
    let out = jit_capture(
        r#"r = range(5)
print(len(r))
"#,
    );
    assert_output(&out, "5\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_stop_zero_len() {
    let out = jit_capture(
        r#"r = range(0)
print(len(r))
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_start_stop_len() {
    let out = jit_capture(
        r#"r = range(2, 7)
print(len(r))
"#,
    );
    assert_output(&out, "5\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_start_stop_step_len() {
    let out = jit_capture(
        r#"r = range(0, 10, 2)
print(len(r))
"#,
    );
    assert_output(&out, "5\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_iteration_sum() {
    let out = jit_capture(
        r#"total = 0
for x in range(10):
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "45\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_iteration_start_stop() {
    let out = jit_capture(
        r#"total = 0
for x in range(3, 7):
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "18\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_step_iteration() {
    let out = jit_capture(
        r#"out = []
for x in range(0, 10, 2):
    out.append(x)
print(out)
"#,
    );
    assert_output(&out, "[0, 2, 4, 6, 8]\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_negative_step() {
    let out = jit_capture(
        r#"out = []
for x in range(5, 0, -1):
    out.append(x)
print(out)
"#,
    );
    assert_output(&out, "[5, 4, 3, 2, 1]\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_empty_when_start_ge_stop() {
    let out = jit_capture(
        r#"count = 0
for x in range(5, 5):
    count = count + 1
print(count)
"#,
    );
    assert_output(&out, "0\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_to_list() {
    let out = jit_capture(
        r#"xs = list(range(4))
print(len(xs))
print(xs[0])
print(xs[3])
"#,
    );
    assert_output(&out, "4\n0\n3\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_indexing_positive() {
    let out = jit_capture(
        r#"r = range(10, 20)
print(r[0])
print(r[5])
print(r[9])
"#,
    );
    assert_output(&out, "10\n15\n19\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_indexing_negative() {
    let out = jit_capture(
        r#"r = range(10, 20)
print(r[-1])
print(r[-5])
"#,
    );
    assert_output(&out, "19\n15\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_contains_present() {
    let out = jit_capture(
        r#"r = range(10)
print(5 in r)
print(0 in r)
print(9 in r)
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_contains_absent() {
    let out = jit_capture(
        r#"r = range(10)
print(10 in r)
print(-1 in r)
print(100 in r)
"#,
    );
    assert_output(&out, "False\nFalse\nFalse\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_contains_with_step() {
    let out = jit_capture(
        r#"r = range(0, 10, 2)
print(0 in r)
print(2 in r)
print(3 in r)
print(8 in r)
"#,
    );
    assert_output(&out, "True\nTrue\nFalse\nTrue\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_equality_same_args() {
    let out = jit_capture(
        r#"print(range(5) == range(5))
print(range(0, 5) == range(5))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_inequality_different_args() {
    let out = jit_capture(
        r#"print(range(5) == range(6))
print(range(5) == range(1, 5))
"#,
    );
    assert_output(&out, "False\nFalse\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_bool_empty_is_false() {
    let out = jit_capture(
        r#"print(bool(range(0)))
print(bool(range(5, 5)))
"#,
    );
    assert_output(&out, "False\nFalse\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_bool_nonempty_is_true() {
    let out = jit_capture(
        r#"print(bool(range(1)))
print(bool(range(0, 1)))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_sum_builtin() {
    let out = jit_capture(
        r#"print(sum(range(10)))
print(sum(range(1, 11)))
"#,
    );
    assert_output(&out, "45\n55\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_arity_and_step() {
    let out = jit_capture(
        r#"print(list(range(5)))
print(list(range(2, 8)))
print(list(range(0, 10, 2)))
print(list(range(10, 0, -1)))
print(list(range(10, 0, -2)))
"#,
    );
    assert_output(
        &out,
        "[0, 1, 2, 3, 4]\n[2, 3, 4, 5, 6, 7]\n[0, 2, 4, 6, 8]\n[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]\n[10, 8, 6, 4, 2]\n",
    );
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_range_empty_and_len_and_sum() {
    let out = jit_capture(
        r#"print(list(range(0)))
print(list(range(5, 5)))
print(len(range(100)))
print(sum(range(11)))
print(sum(range(1, 101)))
"#,
    );
    assert_output(&out, "[]\n[]\n100\n55\n5050\n");
}

/// Ported from `Lib/test/test_bool_ported.py`.
#[test]
fn test_enumerate_default_and_start() {
    let out = jit_capture(
        r#"print(list(enumerate(["a", "b", "c"])))
print(list(enumerate(["x", "y"], start=10)))
print(list(enumerate([])))
print(list(enumerate("abc")))
"#,
    );
    assert_output(
        &out,
        "[(0, 'a'), (1, 'b'), (2, 'c')]\n[(10, 'x'), (11, 'y')]\n[]\n[(0, 'a'), (1, 'b'), (2, 'c')]\n",
    );
}


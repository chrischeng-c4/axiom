use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/augmented_int_then_float.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_augmented_int_then_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "augmented_int_then_float"
# subject = "augmented assignment int += float promotes value and type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""x bound to int then x += 0.5 rebinds to the correct float (0.5), not bit garbage."""
x = 0
x += 0.5
assert x == 0.5, x
assert isinstance(x, float), type(x)
x += 1
assert x == 1.5, x
assert isinstance(x, float), type(x)
print("augmented_int_then_float OK")
"###);
    assert_output(&out, r###"augmented_int_then_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/bool_plus_float.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_bool_plus_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "bool_plus_float"
# subject = "bool + float promotes to float value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bool participates as int 1/0 then promotes to float (True + 1.0 == 2.0)."""
r = True + 1.0
assert r == 2.0, r
assert isinstance(r, float), type(r)
assert (False + 0.5) == 0.5, False + 0.5
print("bool_plus_float OK")
"###);
    assert_output(&out, r###"bool_plus_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/bool_sum_is_int.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_bool_sum_is_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "bool_sum_is_int"
# subject = "sum() over bools yields int total"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bool counts as int under sum(): sum([True, True]) == 2 with int type."""
n = sum([True, True])
assert n == 2, n
assert isinstance(n, int), type(n)
assert sum([True, False, True]) == 2, sum([True, False, True])
print("bool_sum_is_int OK")
"###);
    assert_output(&out, r###"bool_sum_is_int OK
"###);
}

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/division_result_types.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_division_result_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "division_result_types"
# subject = "true division returns float vs floor division returns int"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Even when evenly divisible, / returns float and // returns int (4/2 is 2.0 float; 4//2 is 2 int)."""
true_div = 4 / 2
floor_div = 4 // 2
assert true_div == 2.0, true_div
assert isinstance(true_div, float), type(true_div)
assert floor_div == 2, floor_div
assert isinstance(floor_div, int), type(floor_div)
assert true_div == floor_div, (true_div, floor_div)
print("division_result_types OK")
"###);
    assert_output(&out, r###"division_result_types OK
"###);
}

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/float_floor_division_is_float.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_float_floor_division_is_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "float_floor_division_is_float"
# subject = "float // int floor division returns a whole-valued float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""float // int floors but stays float (7.0 // 2 == 3.0 with float type, not int 3)."""
fd = 7.0 // 2
assert fd == 3.0, fd
assert isinstance(fd, float), type(fd)
assert (7 // 2.0) == 3.0, 7 // 2.0
assert isinstance(7 // 2.0, float), type(7 // 2.0)
print("float_floor_division_is_float OK")
"###);
    assert_output(&out, r###"float_floor_division_is_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/float_sum_comparison.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_float_sum_comparison() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "float_sum_comparison"
# subject = "IEEE-754 float addition then comparison (0.1 + 0.2 > 0.3)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""0.1 + 0.2 is 0.30000000000000004, so it compares strictly greater than 0.3."""
total = 0.1 + 0.2
assert total > 0.3, total
assert total != 0.3, total
assert isinstance(total, float), type(total)
print("float_sum_comparison OK")
"###);
    assert_output(&out, r###"float_sum_comparison OK
"###);
}

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/floor_division_stays_int.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_floor_division_stays_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "floor_division_stays_int"
# subject = "int // int floor division return value and type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int // int floor division stays int (3 // 2 == 1, exact value and type)."""
f = 3 // 2
assert f == 1, f
assert isinstance(f, int), type(f)
assert (7 // 2) == 3, 7 // 2
print("floor_division_stays_int OK")
"###);
    assert_output(&out, r###"floor_division_stays_int OK
"###);
}

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/int_mod_float.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_int_mod_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "int_mod_float"
# subject = "int % float remainder value and type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int % float promotes to a float remainder (5 % 2.0 == 1.0), not bit garbage."""
m = 5 % 2.0
assert m == 1.0, m
assert isinstance(m, float), type(m)
assert (5.5 % 2) == 1.5, 5.5 % 2
print("int_mod_float OK")
"###);
    assert_output(&out, r###"int_mod_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/int_plus_float_in_var.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_int_plus_float_in_var() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "int_plus_float_in_var"
# subject = "int + float through named variables"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int + float bound through variables promotes to the correct float (3 + 0.5 == 3.5)."""
a = 3
b = 0.5
s = a + b
assert s == 3.5, s
assert isinstance(s, float), type(s)
print("int_plus_float_in_var OK")
"###);
    assert_output(&out, r###"int_plus_float_in_var OK
"###);
}

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/int_pow_half_is_sqrt.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_int_pow_half_is_sqrt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "int_pow_half_is_sqrt"
# subject = "int ** float fractional exponent yields float root"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int ** 0.5 returns the float square root (4 ** 0.5 == 2.0; 2 ** 0.5 ~ 1.4142)."""
root4 = 4 ** 0.5
assert root4 == 2.0, root4
assert isinstance(root4, float), type(root4)
root2 = 2 ** 0.5
assert isinstance(root2, float), type(root2)
assert abs(root2 - 1.4142135623730951) < 1e-12, root2
assert abs(root2 * root2 - 2.0) < 1e-12, root2 * root2
print("int_pow_half_is_sqrt OK")
"###);
    assert_output(&out, r###"int_pow_half_is_sqrt OK
"###);
}

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/int_times_float.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_int_times_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "int_times_float"
# subject = "int * float product value and type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int * float yields the correct float product (2 * 1.5 == 3.0), not NaN-box bits."""
p = 2 * 1.5
assert p == 3.0, p
assert isinstance(p, float), type(p)
assert (3 * 0.25) == 0.75, 3 * 0.25
print("int_times_float OK")
"###);
    assert_output(&out, r###"int_times_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/mixed_numeric_inference/int_true_division_is_float.py`.
#[test]
fn test_gen_behavior_core_mixed_numeric_inference_int_true_division_is_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "int_true_division_is_float"
# subject = "int / int true division return value and type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int / int true division yields a float, never integer floor (1/2 == 0.5)."""
q = 1 / 2
assert q == 0.5, q
assert isinstance(q, float), type(q)
assert (7 / 2) == 3.5, 7 / 2
print("int_true_division_is_float OK")
"###);
    assert_output(&out, r###"int_true_division_is_float OK
"###);
}

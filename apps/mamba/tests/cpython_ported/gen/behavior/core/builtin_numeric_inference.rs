use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/abs_float_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_abs_float_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "abs_float_assign"
# subject = "abs() of a negative float, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abs(-1.5) assigned to a variable must yield 1.5 as a float."""

a = abs(-1.5)
assert a == 1.5, a
assert isinstance(a, float), type(a)
plus = a + 0.5
assert plus == 2.0, plus
print("abs_float_assign OK")
"###);
    assert_output(&out, r###"abs_float_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/abs_int_stays_int.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_abs_int_stays_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "abs_int_stays_int"
# subject = "abs() of a negative int stays int, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abs(-7) assigned to a variable must stay int 7."""

a = abs(-7)
assert a == 7, a
assert isinstance(a, int), type(a)
assert not isinstance(a, bool), type(a)
prod = a * 3
assert prod == 21, prod
print("abs_int_stays_int OK")
"###);
    assert_output(&out, r###"abs_int_stays_int OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/divmod_float_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_divmod_float_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "divmod_float_assign"
# subject = "divmod(float, float) tuple unpack, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""divmod(7.5, 2.0) unpacked into variables must yield the correct float quotient and remainder."""

q, r = divmod(7.5, 2.0)
assert q == 3.0, q
assert r == 1.5, r
assert isinstance(q, float), type(q)
assert isinstance(r, float), type(r)
total = q * 2.0 + r
assert total == 7.5, total
print("divmod_float_assign OK")
"###);
    assert_output(&out, r###"divmod_float_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/max_float_list_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_max_float_list_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "max_float_list_assign"
# subject = "max() over a list of floats, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""max() of a float list assigned to a variable must yield the correct float value."""

m = max([3.5, 1.5, 2.5])
assert m == 3.5, m
assert isinstance(m, float), type(m)
half = m / 2.0
assert half == 1.75, half
print("max_float_list_assign OK")
"###);
    assert_output(&out, r###"max_float_list_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/max_float_varargs_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_max_float_varargs_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "max_float_varargs_assign"
# subject = "max() over float varargs, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""max(a, b, c) of float varargs assigned to a variable must yield the correct float value."""

m = max(2.25, 9.75, 4.5)
assert m == 9.75, m
assert isinstance(m, float), type(m)
diff = m - 0.75
assert diff == 9.0, diff
print("max_float_varargs_assign OK")
"###);
    assert_output(&out, r###"max_float_varargs_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/min_float_list_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_min_float_list_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "min_float_list_assign"
# subject = "min() over a list of floats, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""min() of a float list assigned to a variable must yield the correct float value."""

m = min([3.5, 1.5, 2.5])
assert m == 1.5, m
assert isinstance(m, float), type(m)
times = m * 4.0
assert times == 6.0, times
print("min_float_list_assign OK")
"###);
    assert_output(&out, r###"min_float_list_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/pow_float_base_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_pow_float_base_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "pow_float_base_assign"
# subject = "pow(float, int) return value, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pow(2.0, 3) assigned to a variable must yield 8.0 as a float."""

p = pow(2.0, 3)
assert p == 8.0, p
assert isinstance(p, float), type(p)
half = p / 2.0
assert half == 4.0, half
print("pow_float_base_assign OK")
"###);
    assert_output(&out, r###"pow_float_base_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/pow_negative_exp_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_pow_negative_exp_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "pow_negative_exp_assign"
# subject = "pow(int, negative-int) returns float, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pow(2, -1) returns the float 0.5; assigned then used must keep the float value."""

p = pow(2, -1)
assert p == 0.5, p
assert isinstance(p, float), type(p)
times = p * 8.0
assert times == 4.0, times
print("pow_negative_exp_assign OK")
"###);
    assert_output(&out, r###"pow_negative_exp_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/round_literal_ndigits_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_round_literal_ndigits_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "round_literal_ndigits_assign"
# subject = "round(float, ndigits) on literals, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""round(2.567, 2) assigned to a variable must yield 2.57 as a float."""

r = round(2.567, 2)
assert r == 2.57, r
assert isinstance(r, float), type(r)
scaled = r * 100.0
assert scaled == 257.0, scaled
print("round_literal_ndigits_assign OK")
"###);
    assert_output(&out, r###"round_literal_ndigits_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/round_no_ndigits_returns_int.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_round_no_ndigits_returns_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "round_no_ndigits_returns_int"
# subject = "round(float) with no ndigits returns int, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""round(2.567) with no ndigits must return the int 3 (banker's rounding) when assigned then used."""

r = round(2.567)
assert r == 3, r
assert isinstance(r, int), type(r)
# banker's rounding: round(2.5) is 2, round(3.5) is 4
assert round(2.5) == 2, round(2.5)
assert round(3.5) == 4, round(3.5)
nxt = r + 1
assert nxt == 4, nxt
print("round_no_ndigits_returns_int OK")
"###);
    assert_output(&out, r###"round_no_ndigits_returns_int OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/round_var_args_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_round_var_args_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "round_var_args_assign"
# subject = "round(x, n) with variable args, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""round(x, n) where x and n are variables must yield the correct float when assigned then used."""

x = 3.14159
n = 3
r = round(x, n)
assert r == 3.142, r
assert isinstance(r, float), type(r)
scaled = r * 1000.0
assert scaled == 3142.0, scaled
print("round_var_args_assign OK")
"###);
    assert_output(&out, r###"round_var_args_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/sum_float_genexpr_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_sum_float_genexpr_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "sum_float_genexpr_assign"
# subject = "sum() over a generator expression of floats, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum() of a float generator expression assigned to a variable must yield the float value."""

s = sum(x * 1.0 for x in [1, 2, 3, 4])
assert s == 10.0, s
assert isinstance(s, float), type(s)
plus = s + 0.5
assert plus == 10.5, plus
print("sum_float_genexpr_assign OK")
"###);
    assert_output(&out, r###"sum_float_genexpr_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/sum_float_list_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_sum_float_list_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "sum_float_list_assign"
# subject = "sum() over a list of floats, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum() of a float list assigned to a variable must yield the float value, not leaked NaN-box bits."""

s = sum([1.5, 2.5, 3.0])
assert s == 7.0, s
assert isinstance(s, float), type(s)
# use the value after assignment (the leak shows up on assign-then-use)
doubled = s * 2.0
assert doubled == 14.0, doubled
print("sum_float_list_assign OK")
"###);
    assert_output(&out, r###"sum_float_list_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/sum_float_start_assign.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_sum_float_start_assign() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "sum_float_start_assign"
# subject = "sum() with a float start argument, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum([...], start=0.0) assigned to a variable must yield the float value."""

s = sum([1.5, 2.5, 4.0], 0.0)
assert s == 8.0, s
assert isinstance(s, float), type(s)
minus = s - 1.0
assert minus == 7.0, minus
print("sum_float_start_assign OK")
"###);
    assert_output(&out, r###"sum_float_start_assign OK
"###);
}

/// Ported from `tests/cpython/behavior/core/builtin_numeric_inference/sum_int_list_stays_int.py`.
#[test]
fn test_gen_behavior_core_builtin_numeric_inference_sum_int_list_stays_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "sum_int_list_stays_int"
# subject = "sum() over a list of ints stays int, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum() of an int list assigned to a variable must stay an int with the correct value."""

s = sum([1, 2, 3])
assert s == 6, s
assert isinstance(s, int), type(s)
assert not isinstance(s, bool), type(s)
prod = s * 2
assert prod == 12, prod
print("sum_int_list_stays_int OK")
"###);
    assert_output(&out, r###"sum_int_list_stays_int OK
"###);
}

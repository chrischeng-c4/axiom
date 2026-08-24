use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/dictcomp_float_key.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_dictcomp_float_key() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "dictcomp_float_key"
# subject = "dict comprehension key from a user function returning float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A dict comprehension whose keys are user-func floats stores correct float keys."""


def ff(j):
    return j + 0.5


d = {ff(j): j for j in range(4)}
keys = sorted(d.keys())
assert keys == [0.5, 1.5, 2.5, 3.5], keys
for k in keys:
    assert isinstance(k, float), (k, type(k))
assert d[0.5] == 0, d
assert d[3.5] == 3, d
print("dictcomp_float_key OK")
"###);
    assert_output(&out, r###"dictcomp_float_key OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/dictcomp_value_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_dictcomp_value_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "dictcomp_value_float"
# subject = "dict comprehension value from a user function returning float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A dict comprehension whose values are user-func floats stores the correct float values."""


def ff(j):
    return j * 1.5


d = {j: ff(j) for j in range(4)}
vals = [d[k] for k in sorted(d)]
assert vals == [0.0, 1.5, 3.0, 4.5], vals
for v in vals:
    assert isinstance(v, float), (v, type(v))
print("dictcomp_value_float OK")
"###);
    assert_output(&out, r###"dictcomp_value_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/genexpr_assigned_then_list_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_genexpr_assigned_then_list_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "genexpr_assigned_then_list_float"
# subject = "generator expression of floats bound to a name then materialized"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator expression of floats bound to a name yields the correct floats when listed."""


def ff(j):
    return j * 0.5


g = (ff(j) for j in range(4))
xs = list(g)
assert xs == [0.0, 0.5, 1.0, 1.5], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("genexpr_assigned_then_list_float OK")
"###);
    assert_output(&out, r###"genexpr_assigned_then_list_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/list_of_genexpr_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_list_of_genexpr_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "list_of_genexpr_float"
# subject = "list() materializing a generator expression of floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""list() over a generator expression of user-func floats materializes the correct floats."""


def ff(j):
    return j + 0.25


xs = list(ff(j) for j in range(4))
assert xs == [0.25, 1.25, 2.25, 3.25], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("list_of_genexpr_float OK")
"###);
    assert_output(&out, r###"list_of_genexpr_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/listcomp_capture_outer_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_listcomp_capture_outer_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_capture_outer_float"
# subject = "comprehension element adding a captured outer float variable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A list comprehension that captures an outer float variable produces the correct sums."""


base = 0.5
xs = [base + j for j in range(4)]
assert xs == [0.5, 1.5, 2.5, 3.5], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_capture_outer_float OK")
"###);
    assert_output(&out, r###"listcomp_capture_outer_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/listcomp_filter_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_listcomp_filter_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_filter_float"
# subject = "filtered list comprehension whose surviving elements are floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A filtered list comprehension `j*0.5 for j if j%2==0` keeps the correct surviving floats."""


xs = [j * 0.5 for j in range(8) if j % 2 == 0]
assert xs == [0.0, 1.0, 2.0, 3.0], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_filter_float OK")
"###);
    assert_output(&out, r###"listcomp_filter_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/listcomp_func_captures_global_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_listcomp_func_captures_global_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_func_captures_global_float"
# subject = "comprehension element from a function reading a global float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A comprehension over a func that multiplies by a global float scale yields correct floats."""


scale = 0.25


def ff(j):
    return j * scale


xs = [ff(j) for j in range(4)]
assert xs == [0.0, 0.25, 0.5, 0.75], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_func_captures_global_float OK")
"###);
    assert_output(&out, r###"listcomp_func_captures_global_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/listcomp_index_into_float_list.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_listcomp_index_into_float_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_index_into_float_list"
# subject = "comprehension element from indexing a float list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A list comprehension element `data[j]` indexing a float list preserves the float values."""


data = [0.1, 0.2, 0.3, 0.4]
xs = [data[j] for j in range(4)]
assert xs == [0.1, 0.2, 0.3, 0.4], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_index_into_float_list OK")
"###);
    assert_output(&out, r###"listcomp_index_into_float_list OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/listcomp_reciprocal_division.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_listcomp_reciprocal_division() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_reciprocal_division"
# subject = "float division return value inside a list comprehension"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A list comprehension element `1.0/(j+1)` yields the correct true-division float."""


xs = [1.0 / (j + 1) for j in range(4)]
assert xs[0] == 1.0, xs
assert xs[1] == 0.5, xs
assert xs[2] == 1.0 / 3.0, xs
assert xs[3] == 0.25, xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_reciprocal_division OK")
"###);
    assert_output(&out, r###"listcomp_reciprocal_division OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/listcomp_scale_iterable_floats.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_listcomp_scale_iterable_floats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_scale_iterable_floats"
# subject = "scaling float iterable elements by a float literal in a comprehension"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A list comprehension `x*0.5` over a float iterable yields the correct scaled floats."""


xs = [x * 0.5 for x in [2.0, 4.0, 6.0, 7.0]]
assert xs == [1.0, 2.0, 3.0, 3.5], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_scale_iterable_floats OK")
"###);
    assert_output(&out, r###"listcomp_scale_iterable_floats OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/listcomp_userfunc_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_listcomp_userfunc_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "listcomp_userfunc_float"
# subject = "list comprehension element from user function returning float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A list comprehension whose element is a user-func call returning float stores the correct float."""


def ff(j):
    return j + 0.5


xs = [ff(j) for j in range(4)]
assert xs == [0.5, 1.5, 2.5, 3.5], xs
for v in xs:
    assert isinstance(v, float), (v, type(v))
print("listcomp_userfunc_float OK")
"###);
    assert_output(&out, r###"listcomp_userfunc_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/nested_listcomp_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_nested_listcomp_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "nested_listcomp_float"
# subject = "nested list comprehension element from a two-arg user func returning float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A nested `[[ff(i,j) for j] for i]` comprehension stores the correct float matrix."""


def ff(i, j):
    return i * 10.0 + j * 0.5


m = [[ff(i, j) for j in range(2)] for i in range(2)]
assert m == [[0.0, 0.5], [10.0, 10.5]], m
for row in m:
    for v in row:
        assert isinstance(v, float), (v, type(v))
print("nested_listcomp_float OK")
"###);
    assert_output(&out, r###"nested_listcomp_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/setcomp_division_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_setcomp_division_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "setcomp_division_float"
# subject = "set comprehension element from true division yielding float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A set comprehension `j/4` yields the correct distinct float members."""


s = {j / 4 for j in range(4)}
assert s == {0.0, 0.25, 0.5, 0.75}, s
for v in s:
    assert isinstance(v, float), (v, type(v))
print("setcomp_division_float OK")
"###);
    assert_output(&out, r###"setcomp_division_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/setcomp_userfunc_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_setcomp_userfunc_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "setcomp_userfunc_float"
# subject = "set comprehension element from a user function returning float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A set comprehension of user-func float results contains the correct float members."""


def ff(j):
    return j + 0.5


s = {ff(j) for j in range(4)}
assert s == {0.5, 1.5, 2.5, 3.5}, s
for v in s:
    assert isinstance(v, float), (v, type(v))
print("setcomp_userfunc_float OK")
"###);
    assert_output(&out, r###"setcomp_userfunc_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/sum_genexpr_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_sum_genexpr_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "sum_genexpr_float"
# subject = "sum of a generator expression of user-func floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum() over a generator expression of user-func floats produces the correct float total."""


def ff(j):
    return j + 0.5


total = sum(ff(j) for j in range(4))
assert total == 0.5 + 1.5 + 2.5 + 3.5, total
assert total == 8.0, total
assert isinstance(total, float), (total, type(total))
print("sum_genexpr_float OK")
"###);
    assert_output(&out, r###"sum_genexpr_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/comprehension_float_inference/tuple_of_genexpr_float.py`.
#[test]
fn test_gen_behavior_core_comprehension_float_inference_tuple_of_genexpr_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "tuple_of_genexpr_float"
# subject = "tuple() materializing a generator expression of floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tuple() over a generator expression of `j/2` floats materializes the correct float tuple."""


def ff(j):
    return j / 2


t = tuple(ff(j) for j in range(4))
assert t == (0.0, 0.5, 1.0, 1.5), t
for v in t:
    assert isinstance(v, float), (v, type(v))
print("tuple_of_genexpr_float OK")
"###);
    assert_output(&out, r###"tuple_of_genexpr_float OK
"###);
}

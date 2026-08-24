use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/value_equality_inference/bytes_eq_params.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_bytes_eq_params() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "bytes_eq_params"
# subject = "bytes == bytes as function params"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: distinct bytes objects with equal content compare equal by value."""


def check(result, expect):
    assert result == expect, (result, expect)


check(b"ab" + b"cd", b"abcd")
print("bytes_eq_params OK")
"###);
    assert_output(&out, r###"bytes_eq_params OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/check_via_alias.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_check_via_alias() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "check_via_alias"
# subject = "value equality through an aliased helper call (g = helper; g(a, b))"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A helper bound to a new name still compares its unannotated args by value."""


def helper(result, expect):
    assert result == expect, (result, expect)


g = helper
g([1, 2], [1, 2])
g("abc", "ab" + "c")
print("check_via_alias OK")
"###);
    assert_output(&out, r###"check_via_alias OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/dict_eq_order_independent.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_dict_eq_order_independent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "dict_eq_order_independent"
# subject = "dict == dict ignores insertion order"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Dict equality compares key/value pairs by value regardless of insertion order."""
a = {"x": 1, "y": 2, "z": 3}
b = {"z": 3, "y": 2, "x": 1}
assert (a == b) is True, a == b
print("dict_eq_order_independent OK")
"###);
    assert_output(&out, r###"dict_eq_order_independent OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/dict_eq_params.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_dict_eq_params() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "dict_eq_params"
# subject = "dict == dict as function params"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: dicts with equal key/value pairs compare equal by value."""


def check(result, expect):
    assert result == expect, (result, expect)


check({"a": 1, "b": 2}, {"a": 1, "b": 2})
print("dict_eq_params OK")
"###);
    assert_output(&out, r###"dict_eq_params OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/in_list_membership.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_in_list_membership() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "in_list_membership"
# subject = "in-operator membership tests by value over a list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""The in operator finds a member by value, not by identity."""
needle = "b" + "c"
haystack = ["ab", "bc", "cd"]
assert (needle in haystack) is True, needle
assert ("zz" in haystack) is False
print("in_list_membership OK")
"###);
    assert_output(&out, r###"in_list_membership OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/in_tuple_membership.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_in_tuple_membership() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "in_tuple_membership"
# subject = "in-operator membership tests by value over a tuple of lists"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""The in operator finds a list member of a tuple by value, not identity."""
needle = [1, 2]
container = ([0], [1, 2], [3])
assert (needle in container) is True, needle
assert ([9, 9] in container) is False
print("in_tuple_membership OK")
"###);
    assert_output(&out, r###"in_tuple_membership OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/list_eq_locals.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_list_eq_locals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "list_eq_locals"
# subject = "list == list compares by value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""list == list as locals compares element values, not object identity."""
a = [1, 2]
b = [1, 2]
assert (a == b) is True, a == b
assert a is not b
print("list_eq_locals OK")
"###);
    assert_output(&out, r###"list_eq_locals OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/list_eq_params.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_list_eq_params() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "list_eq_params"
# subject = "list == list inside def check(a, b)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated function params: list == list must compare by value (check pattern)."""


def check(result, expect):
    assert result == expect, (result, expect)


check([1, 2], [1, 2])
print("list_eq_params OK")
"###);
    assert_output(&out, r###"list_eq_params OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/list_ne_false.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_list_ne_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "list_ne_false"
# subject = "list != list is False for equal values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""!= must be False for two distinct lists holding equal values."""
a = [1, 2, 3]
b = [1, 2, 3]
assert (a != b) is False, a != b
print("list_ne_false OK")
"###);
    assert_output(&out, r###"list_ne_false OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/nested_list_eq.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_nested_list_eq() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "nested_list_eq"
# subject = "nested list == list recurses by value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Equality of nested lists recurses element-by-element by value."""
a = [[1, 2], [3, [4, 5]]]
b = [[1, 2], [3, [4, 5]]]
assert (a == b) is True, a == b
c = [[1, 2], [3, [4, 6]]]
assert (a == c) is False, a == c
print("nested_list_eq OK")
"###);
    assert_output(&out, r###"nested_list_eq OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/set_eq_params.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_set_eq_params() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "set_eq_params"
# subject = "set == set as function params"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: sets with equal members compare equal by value, order-independent."""


def check(result, expect):
    assert result == expect, (result, expect)


check({1, 2, 3}, {3, 2, 1})
print("set_eq_params OK")
"###);
    assert_output(&out, r###"set_eq_params OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/str_eq_params.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_str_eq_params() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "str_eq_params"
# subject = "str == str as function params"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: distinct str objects with equal content compare equal by value."""


def check(result, expect):
    assert result == expect, (result, expect)


check("hello" + "", "hello")
check("".join(["ab", "cd"]), "abcd")
print("str_eq_params OK")
"###);
    assert_output(&out, r###"str_eq_params OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/str_ne_true.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_str_ne_true() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "str_ne_true"
# subject = "str != str is True for differing values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""!= must be True for two strings whose content differs."""
a = "hello"
b = "world"
assert (a != b) is True, a != b
assert (a == b) is False, a == b
print("str_ne_true OK")
"###);
    assert_output(&out, r###"str_ne_true OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/tuple_eq_params.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_tuple_eq_params() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "tuple_eq_params"
# subject = "tuple == tuple as function params"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: tuples with equal elements compare equal by value."""


def check(result, expect):
    assert result == expect, (result, expect)


check((1, 2, 3), (1, 2, 3))
check(tuple([4, 5]), (4, 5))
print("tuple_eq_params OK")
"###);
    assert_output(&out, r###"tuple_eq_params OK
"###);
}

/// Ported from `tests/cpython/behavior/core/value_equality_inference/tuple_ne_true_params.py`.
#[test]
fn test_gen_behavior_core_value_equality_inference_tuple_ne_true_params() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "value_equality_inference"
# dimension = "behavior"
# case = "tuple_ne_true_params"
# subject = "tuple != tuple as params is True when contents differ"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Unannotated params: tuples that differ by one element are unequal (!= is True)."""


def differ(result, expect):
    assert result != expect, (result, expect)


differ((1, 2, 3), (1, 2, 4))
print("tuple_ne_true_params OK")
"###);
    assert_output(&out, r###"tuple_ne_true_params OK
"###);
}

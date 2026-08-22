use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/dict_str_to_float.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_dict_str_to_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "dict_str_to_float"
# subject = "dict[str, float] read back by string key"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A dict mapping str keys to float values must read each float back unchanged."""
prices = {"apple": 1.25, "pear": 2.5, "plum": 0.75}
assert prices["apple"] == 1.25, prices["apple"]
assert prices["pear"] == 2.5, prices["pear"]
assert prices["plum"] == 0.75, prices["plum"]
total = prices["apple"] + prices["pear"] + prices["plum"]
assert total == 4.5, total
print("dict_str_to_float OK")
"###);
    assert_output(&out, r###"dict_str_to_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/dict_value_by_key.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_dict_value_by_key() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "dict_value_by_key"
# subject = "float stored as a dict value and read back by key"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float stored as a dict value must read back as the same float by key."""
d = {}
d["pi"] = 3.125
d["e"] = 2.5
got = d["pi"]
assert got == 3.125, got
assert isinstance(got, float), type(got)
assert d["e"] == 2.5, d["e"]
print("dict_value_by_key OK")
"###);
    assert_output(&out, r###"dict_value_by_key OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/float_as_dict_key.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_float_as_dict_key() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "float_as_dict_key"
# subject = "float used as a dict key"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float used as a dict key must hash/compare by value so lookup returns the stored value."""
d = {1.5: "a", 2.25: "b"}
assert d[1.5] == "a", d[1.5]
assert d[2.25] == "b", d[2.25]
d[1.5] = "updated"
assert d[1.5] == "updated", d[1.5]
assert len(d) == 2, len(d)
print("float_as_dict_key OK")
"###);
    assert_output(&out, r###"float_as_dict_key OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/float_through_function_param.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_float_through_function_param() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "float_through_function_param"
# subject = "float pulled from a container and passed through a function param"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float read out of a list and passed through a function param must arrive unchanged."""
def echo(x):
    return x

xs = [7.5, 8.25]
got = echo(xs[0])
assert got == 7.5, got
assert isinstance(got, float), type(got)
assert echo(xs[1]) == 8.25, echo(xs[1])
print("float_through_function_param OK")
"###);
    assert_output(&out, r###"float_through_function_param OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/list_append_index.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_list_append_index() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "list_append_index"
# subject = "float stored in a list and read back by index"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float appended to a list must read back as the same float (no NaN-box-as-int leak)."""
xs = []
xs.append(3.5)
xs.append(0.25)
got = xs[0]
assert got == 3.5, got
assert isinstance(got, float), type(got)
assert xs[1] == 0.25, xs[1]
print("list_append_index OK")
"###);
    assert_output(&out, r###"list_append_index OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/list_index_augadd_float.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_list_index_augadd_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "list_index_augadd_float"
# subject = "augmented add of a float into a list element (list[i] += float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""list[i] += float must read, add, and store back the correct float value."""
xs = [1.0, 2.0, 3.0]
xs[1] += 0.5
assert xs[1] == 2.5, xs[1]
xs[1] += 0.5
assert xs[1] == 3.0, xs[1]
assert isinstance(xs[1], float), type(xs[1])
assert xs == [1.0, 3.0, 3.0], xs
print("list_index_augadd_float OK")
"###);
    assert_output(&out, r###"list_index_augadd_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/list_iter_float_values.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_list_iter_float_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "list_iter_float_values"
# subject = "iterating a list of floats yields the stored floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Iterating a list of floats must yield each stored float, not a leaked box-bits int."""
xs = [0.5, 1.5, 2.5]
collected = []
for v in xs:
    assert isinstance(v, float), type(v)
    collected.append(v)
assert collected == [0.5, 1.5, 2.5], collected
assert sum(xs) == 4.5, sum(xs)
print("list_iter_float_values OK")
"###);
    assert_output(&out, r###"list_iter_float_values OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/list_of_computed_floats_sum.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_list_of_computed_floats_sum() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "list_of_computed_floats_sum"
# subject = "sum over a list of computed floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Computed floats stored in a list must sum to the correct float total."""
xs = []
for i in range(4):
    xs.append(i * 0.5)
assert xs == [0.0, 0.5, 1.0, 1.5], xs
total = sum(xs)
assert total == 3.0, total
assert isinstance(total, float), type(total)
print("list_of_computed_floats_sum OK")
"###);
    assert_output(&out, r###"list_of_computed_floats_sum OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/list_update_element_computed.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_list_update_element_computed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "list_update_element_computed"
# subject = "list element overwritten with a computed float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Overwriting a list element with a computed float must store the correct float value."""
xs = [0.0, 0.0, 0.0]
xs[1] = 3.0 / 2.0
got = xs[1]
assert got == 1.5, got
assert isinstance(got, float), type(got)
assert xs == [0.0, 1.5, 0.0], xs
print("list_update_element_computed OK")
"###);
    assert_output(&out, r###"list_update_element_computed OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/nested_dict_of_floats.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_nested_dict_of_floats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "nested_dict_of_floats"
# subject = "float read back through a nested dict-of-dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float two levels deep in nested dicts must read back as the same float."""
d = {"outer": {"inner": 6.5}}
got = d["outer"]["inner"]
assert got == 6.5, got
assert isinstance(got, float), type(got)
print("nested_dict_of_floats OK")
"###);
    assert_output(&out, r###"nested_dict_of_floats OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/nested_list_of_floats.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_nested_list_of_floats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "nested_list_of_floats"
# subject = "float read back through a nested list-of-list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float in a list-of-lists must read back as the same float at [i][j]."""
grid = [[1.5, 2.5], [3.5, 4.5]]
assert grid[0][0] == 1.5, grid[0][0]
assert grid[0][1] == 2.5, grid[0][1]
assert grid[1][0] == 3.5, grid[1][0]
assert grid[1][1] == 4.5, grid[1][1]
assert isinstance(grid[1][1], float), type(grid[1][1])
print("nested_list_of_floats OK")
"###);
    assert_output(&out, r###"nested_list_of_floats OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/set_membership.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_set_membership() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "set_membership"
# subject = "float stored in a set tests membership by value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float in a set must answer membership by value (hash/eq must use the float, not box bits)."""
s = set()
s.add(1.5)
s.add(2.25)
assert 1.5 in s, s
assert 2.25 in s, s
assert 9.5 not in s, s
print("set_membership OK")
"###);
    assert_output(&out, r###"set_membership OK
"###);
}

/// Ported from `tests/cpython/behavior/core/container_float_roundtrip/tuple_element.py`.
#[test]
fn test_gen_behavior_core_container_float_roundtrip_tuple_element() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "tuple_element"
# subject = "float stored in a tuple and read back by index"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float held in a tuple must read back as the same float by index."""
t = (1.5, 2.75, 4.0)
assert t[0] == 1.5, t[0]
assert t[1] == 2.75, t[1]
assert t[2] == 4.0, t[2]
assert isinstance(t[1], float), type(t[1])
print("tuple_element OK")
"###);
    assert_output(&out, r###"tuple_element OK
"###);
}

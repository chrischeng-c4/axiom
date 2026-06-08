# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_copy_shallow_deep_ops"
# subject = "cpython321.test_copy_shallow_deep_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_copy_shallow_deep_ops.py"
# status = "filled"
# ///
"""cpython321.test_copy_shallow_deep_ops: execute CPython 3.12 seed test_copy_shallow_deep_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the copy stdlib module.
# Surface: copy.copy on list/dict/tuple/empty produces an equal but
# independent top-level container; copy.copy is shallow — nested
# containers stay shared between source and result so a mutation to
# the nested object is visible through both; copy.deepcopy walks the
# graph so nested containers are fully independent — mutating a
# nested container through the deep copy doesn't affect the original;
# deepcopy round-trips dict-of-list and dict-of-dict shapes.
import copy
_ledger: list[int] = []

# copy.copy of a list — equal but independent at the top level
a = [1, 2, 3]
b = copy.copy(a)
assert b == [1, 2, 3]; _ledger.append(1)
b.append(4)
assert a == [1, 2, 3]; _ledger.append(1)
assert b == [1, 2, 3, 4]; _ledger.append(1)

# copy.copy of a dict — top-level keys independent
d = {"k": 1}
e = copy.copy(d)
e["k"] = 2
assert d == {"k": 1}; _ledger.append(1)
assert e == {"k": 2}; _ledger.append(1)

# Shallow copy SHARES nested objects — mutating via either side
# is visible through the other
nested = [[1, 2], [3, 4]]
shallow = copy.copy(nested)
shallow[0].append(99)
# Inner list mutation visible through original too
assert nested[0] == [1, 2, 99]; _ledger.append(1)
assert shallow[0] == [1, 2, 99]; _ledger.append(1)

# deepcopy walks the whole graph — inner mutations stay local
nested2 = [[1, 2], [3, 4]]
deep = copy.deepcopy(nested2)
deep[0].append(999)
assert nested2[0] == [1, 2]; _ledger.append(1)
assert deep[0] == [1, 2, 999]; _ledger.append(1)
# The untouched inner sublist is also independent (but currently equal)
assert nested2[1] == [3, 4]; _ledger.append(1)
assert deep[1] == [3, 4]; _ledger.append(1)

# deepcopy on dict-of-list
dd = {"items": [1, 2, 3]}
dd2 = copy.deepcopy(dd)
dd2["items"].append(4)
assert dd == {"items": [1, 2, 3]}; _ledger.append(1)
assert dd2 == {"items": [1, 2, 3, 4]}; _ledger.append(1)

# deepcopy on dict-of-dict
nested_d = {"a": {"x": 1}, "b": {"y": 2}}
nd_deep = copy.deepcopy(nested_d)
nd_deep["a"]["x"] = 99
assert nested_d == {"a": {"x": 1}, "b": {"y": 2}}; _ledger.append(1)
assert nd_deep == {"a": {"x": 99}, "b": {"y": 2}}; _ledger.append(1)

# copy.copy of empty containers — equal to fresh empties
assert copy.copy([]) == []; _ledger.append(1)
assert copy.copy({}) == {}; _ledger.append(1)

# copy.copy of a tuple — equal in value
t = (1, 2, 3)
t2 = copy.copy(t)
assert t2 == (1, 2, 3); _ledger.append(1)
assert t == t2; _ledger.append(1)

# deepcopy of a mixed dict — both nested list and nested dict isolated
mixed = {"a": [1, 2], "b": {"c": 3}}
mc = copy.deepcopy(mixed)
mc["a"].append(99)
mc["b"]["c"] = 999
assert mixed == {"a": [1, 2], "b": {"c": 3}}; _ledger.append(1)
assert mc["a"] == [1, 2, 99]; _ledger.append(1)
assert mc["b"]["c"] == 999; _ledger.append(1)

# deepcopy of a list-of-list — outer and each inner independent
matrix = [[1, 2], [3, 4], [5, 6]]
mc2 = copy.deepcopy(matrix)
mc2[0][0] = 99
mc2.append([7, 8])
assert matrix == [[1, 2], [3, 4], [5, 6]]; _ledger.append(1)
assert mc2 == [[99, 2], [3, 4], [5, 6], [7, 8]]; _ledger.append(1)

# Round-trip: deepcopy of a deeply nested structure
deep_struct = {"items": [1, 2, [3, [4, 5]]], "meta": {"k": "v"}}
ds_copy = copy.deepcopy(deep_struct)
assert ds_copy == deep_struct; _ledger.append(1)
# Mutating the deep copy at a leaf doesn't affect the original
ds_copy["items"][2][1].append(99)
assert deep_struct["items"][2][1] == [4, 5]; _ledger.append(1)
assert ds_copy["items"][2][1] == [4, 5, 99]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_copy_shallow_deep_ops {sum(_ledger)} asserts")

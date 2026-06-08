# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "itemgetter_containers"
# subject = "operator.itemgetter"
# kind = "semantic"
# xfail = "operator.itemgetter(i)(row) returns 0 instead of row[i] under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: a single-key itemgetter indexes lists, tuples and dicts by that key, returning the element at that position/key"""
import operator

_row = [10, 20, 30, 40]
assert operator.itemgetter(2)(_row) == 30, f"list -> {operator.itemgetter(2)(_row)!r}"
assert operator.itemgetter(1)((100, 200, 300)) == 200, "tuple"
assert operator.itemgetter("key")({"key": "val"}) == "val", "dict"

print("itemgetter_containers OK")

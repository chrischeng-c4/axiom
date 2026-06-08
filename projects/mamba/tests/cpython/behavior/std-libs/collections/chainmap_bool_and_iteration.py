# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "chainmap_bool_and_iteration"
# subject = "collections.ChainMap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: a ChainMap is truthy iff any underlying map is non-empty, and iteration visits each distinct key exactly once across all maps"""
from collections import ChainMap, OrderedDict

assert not ChainMap(), "empty is falsy"
assert not ChainMap({}, {}), "all-empty is falsy"
assert ChainMap({1: 2}, {}), "front non-empty is truthy"
assert ChainMap({}, {1: 2}), "back non-empty is truthy"
ordered = ChainMap(OrderedDict(a=1, b=2), OrderedDict(b=99, c=3))
assert sorted(ordered) == ["a", "b", "c"], "iteration visits each distinct key once"
assert ordered["b"] == 2, "front map wins for a duplicate key"

print("chainmap_bool_and_iteration OK")

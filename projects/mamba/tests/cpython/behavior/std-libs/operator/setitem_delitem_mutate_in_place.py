# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "setitem_delitem_mutate_in_place"
# subject = "operator.setitem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.setitem: setitem and delitem mutate lists and dicts in place and both return None"""
import operator

a = list(range(3))
result = operator.setitem(a, 0, 99)
assert result is None, "setitem returns None"
assert a == [99, 1, 2], f"setitem mutated list -> {a!r}"

d = {}
operator.setitem(d, "k", "v")
assert d == {"k": "v"}, "setitem on dict"

b = [4, 3, 2, 1]
result = operator.delitem(b, 1)
assert result is None, "delitem returns None"
assert b == [4, 2, 1], f"delitem mutated list -> {b!r}"

m = {"x": 1, "y": 2}
operator.delitem(m, "x")
assert m == {"y": 2}, "delitem on dict"

print("setitem_delitem_mutate_in_place OK")

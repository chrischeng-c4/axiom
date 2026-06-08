# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_add_and_subtract_operators"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter + sums counts elementwise; Counter - subtracts and drops non-positive results"""
from collections import Counter

c1 = Counter(a=3, b=2, c=1)
c2 = Counter(a=1, b=2, c=3)
added = c1 + c2
assert added["a"] == 4 and added["b"] == 4 and added["c"] == 4, f"add = {dict(added)!r}"
subbed = c1 - c2
assert subbed["a"] == 2, f"sub a = {subbed['a']!r}"
assert "c" not in subbed, "non-positive results are dropped by -"

print("counter_add_and_subtract_operators OK")

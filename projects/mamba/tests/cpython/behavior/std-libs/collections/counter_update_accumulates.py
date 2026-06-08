# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_update_accumulates"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: Counter.update(mapping) adds the mapping's counts onto the existing tallies rather than replacing them"""
from collections import Counter

c = Counter(a=1)
c.update({"a": 2, "b": 1})
assert c["a"] == 3 and c["b"] == 1, f"update = {dict(c)!r}"

print("counter_update_accumulates OK")

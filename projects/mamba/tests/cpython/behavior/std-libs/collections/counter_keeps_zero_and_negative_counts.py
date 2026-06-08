# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "counter_keeps_zero_and_negative_counts"
# subject = "collections.Counter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: counts may reach zero or go negative and the keys persist until explicitly deleted (c['b'] -= 2 -> 0 still in c; c['e'] = -5 kept)"""
from collections import Counter

c = Counter("abcaba")  # a=3, b=2, c=1
c["a"] += 1   # 4
c["b"] -= 2   # 0
c["e"] = -5
assert c["b"] == 0 and "b" in c, "a zero count is kept until deleted"
assert c["e"] == -5, "a negative count is kept"

print("counter_keeps_zero_and_negative_counts OK")

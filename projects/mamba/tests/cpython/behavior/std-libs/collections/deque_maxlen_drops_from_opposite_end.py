# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "deque_maxlen_drops_from_opposite_end"
# subject = "collections.deque"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.deque: a maxlen-bounded deque silently drops the oldest element on overflow rather than raising, keeping only the most recent maxlen items"""
from collections import deque

d = deque(maxlen=3)
for i in range(6):
    d.append(i)
assert list(d) == [3, 4, 5], f"bounded keeps last maxlen = {list(d)!r}"
seeded = deque([1, 2, 3, 4, 5], maxlen=3)
assert list(seeded) == [3, 4, 5], "construction also truncates to the most recent"

print("deque_maxlen_drops_from_opposite_end OK")

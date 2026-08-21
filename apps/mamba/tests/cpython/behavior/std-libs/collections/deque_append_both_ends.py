# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "deque_append_both_ends"
# subject = "collections.deque"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.deque: deque supports appendleft/append/popleft on both ends and iterates in order"""
from collections import deque

d = deque([1, 2, 3])
d.appendleft(0)
d.append(4)
assert list(d) == [0, 1, 2, 3, 4], f"after appends = {list(d)!r}"
d.popleft()
assert list(d) == [1, 2, 3, 4], f"after popleft = {list(d)!r}"

print("deque_append_both_ends OK")

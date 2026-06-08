# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "deque_rotate_shifts_elements"
# subject = "collections.deque"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.deque: deque.rotate(n) shifts elements right by n (negative rotates left), wrapping around"""
from collections import deque

d = deque([1, 2, 3, 4, 5])
d.rotate(2)
assert list(d) == [4, 5, 1, 2, 3], f"rotate(2) = {list(d)!r}"
d.rotate(-1)
assert list(d) == [5, 1, 2, 3, 4], f"rotate(-1) = {list(d)!r}"

print("deque_rotate_shifts_elements OK")

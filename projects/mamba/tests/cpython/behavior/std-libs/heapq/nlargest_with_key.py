# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "nlargest_with_key"
# subject = "heapq.nlargest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.nlargest: nlargest(n, data, key=len) selects the n elements with the largest key, ordered largest-key first"""
import heapq

_data = ["banana", "apple", "cherry", "date", "elderberry"]
_top2 = heapq.nlargest(2, _data, key=len)
assert len(_top2[0]) >= len(_top2[1]), f"nlargest by len = {_top2!r}"
assert _top2[0] == "elderberry", f"longest = {_top2[0]!r}"
print("nlargest_with_key OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "count_and_index_like_list"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: count returns the number of matching elements and index returns the position of the first match, like list"""
import array

a = array.array("i", [1, 2, 2, 3, 2])
assert a.count(2) == 3, f"count(2) = {a.count(2)!r}"
assert a.index(3) == 3, f"index(3) = {a.index(3)!r}"

print("count_and_index_like_list OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "slice_deletion_removes_range"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: del array[a:b] removes the contiguous range of elements"""
import array

d = array.array("i", [0, 1, 2, 3, 4])
del d[1:3]
assert d.tolist() == [0, 3, 4], f"del-slice = {d.tolist()!r}"

print("slice_deletion_removes_range OK")

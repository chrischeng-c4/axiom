# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "reverse_in_place"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: reverse reverses the array elements in place"""
import array

a = array.array("i", [3, 1, 4, 1, 5])
a.reverse()
assert a.tolist() == [5, 1, 4, 1, 3], f"reversed = {a.tolist()!r}"

print("reverse_in_place OK")

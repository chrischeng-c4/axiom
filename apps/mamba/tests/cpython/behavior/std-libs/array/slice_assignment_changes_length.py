# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "slice_assignment_changes_length"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: assigning an array to a slice can change the length of the target array"""
import array

s = array.array("i", [0, 1, 2, 3, 4])
s[1:3] = array.array("i", [10, 20, 30])
assert s.tolist() == [0, 10, 20, 30, 3, 4], f"slice-assign = {s.tolist()!r}"

print("slice_assignment_changes_length OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "slice_returns_new_array_same_typecode"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: slicing returns a new array of the same typecode holding the sliced values"""
import array

a = array.array("i", [0, 1, 2, 3, 4])
s = a[1:4]
assert isinstance(s, array.array), f"slice type = {type(s)!r}"
assert s.typecode == "i", f"slice typecode = {s.typecode!r}"
assert s.tolist() == [1, 2, 3], f"slice values = {s.tolist()!r}"

print("slice_returns_new_array_same_typecode OK")

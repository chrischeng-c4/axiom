# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "fromlist_appends_values"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: fromlist appends each value to the end of the array and membership tests scan by value"""
import array

a = array.array("i", [1])
a.fromlist([2, 3])
assert a.tolist() == [1, 2, 3], f"fromlist = {a.tolist()!r}"
assert 2 in a, "value present"
assert 9 not in a, "value absent"

print("fromlist_appends_values OK")

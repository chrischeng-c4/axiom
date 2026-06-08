# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "extend_accepts_list_and_array"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: extend appends from any iterable, accepting both a plain list and another array of the same typecode"""
import array

a = array.array("i", [1, 2])
a.extend([3, 4, 5])
assert a.tolist() == [1, 2, 3, 4, 5], f"extend list = {a.tolist()!r}"
b = array.array("i", [6, 7])
a.extend(b)
assert a.tolist() == [1, 2, 3, 4, 5, 6, 7], f"extend array = {a.tolist()!r}"

print("extend_accepts_list_and_array OK")

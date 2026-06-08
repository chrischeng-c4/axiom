# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "concat_same_typecode"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: the + operator concatenates two same-typecode arrays into a new array"""
import array

x = array.array("i", [1, 2, 3])
y = array.array("i", [4, 5])
joined = x + y
assert isinstance(joined, array.array), f"concat type = {type(joined)!r}"
assert joined.tolist() == [1, 2, 3, 4, 5], f"concat = {joined.tolist()!r}"

print("concat_same_typecode OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "byte_array_typecode_itemsize"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: a 'b' (signed char) array reports typecode 'b', itemsize 1, and stores values across the -128..127 range"""
import array

a = array.array("b", [-1, 0, 127])
assert a.typecode == "b", f"typecode = {a.typecode!r}"
assert a.itemsize == 1, f"itemsize = {a.itemsize!r}"
assert a.tolist() == [-1, 0, 127], f"values = {a.tolist()!r}"

print("byte_array_typecode_itemsize OK")

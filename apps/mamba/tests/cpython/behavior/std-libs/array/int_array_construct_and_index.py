# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "int_array_construct_and_index"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: array('i', [1,2,3]) reports typecode 'i', itemsize 4, len 3, and supports positive and negative indexing"""
import array

a = array.array("i", [1, 2, 3])
assert isinstance(a, array.array), f"array type = {type(a)!r}"
assert a.typecode == "i", f"typecode = {a.typecode!r}"
assert a.itemsize == 4, f"itemsize = {a.itemsize!r}"
assert len(a) == 3, f"len = {len(a)!r}"
assert a[0] == 1, f"a[0] = {a[0]!r}"
assert a[-1] == 3, f"a[-1] = {a[-1]!r}"

print("int_array_construct_and_index OK")

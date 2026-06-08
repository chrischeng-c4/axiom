# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "double_array_typecode_itemsize"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: a 'd' (double) array reports typecode 'd' and itemsize 8 and stores floats with full precision"""
import array

a = array.array("d", [1.1, 2.2, 3.3])
assert a.typecode == "d", f"typecode = {a.typecode!r}"
assert a.itemsize == 8, f"itemsize = {a.itemsize!r}"
assert abs(a[0] - 1.1) < 1e-10, f"float stored = {a[0]!r}"

print("double_array_typecode_itemsize OK")

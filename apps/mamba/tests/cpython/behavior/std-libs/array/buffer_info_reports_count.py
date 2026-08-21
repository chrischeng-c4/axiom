# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "buffer_info_reports_count"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: buffer_info() returns a (address, element_count) tuple whose second item equals len(array)"""
import array

a = array.array("i", [1, 2, 3])
bi = a.buffer_info()
assert isinstance(bi, tuple), f"buffer_info type = {type(bi)!r}"
assert bi[1] == len(a), f"buffer_info count = {bi[1]!r}"

print("buffer_info_reports_count OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "bytes_initializer_consumed_as_raw_words"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: a bytes initializer is consumed as raw machine words, not values; b'1234' (4 bytes) into typecode 'H' (itemsize 2) yields 2 elements"""
import array

# b'1234' is 4 bytes; for typecode 'H' (itemsize 2) that is 2 elements.
a = array.array("H", b"1234")
assert len(a) * a.itemsize == 4, f"raw bytes consumed = {len(a) * a.itemsize!r}"
assert len(a) == 2, f"H elements from 4 bytes = {len(a)!r}"

print("bytes_initializer_consumed_as_raw_words OK")

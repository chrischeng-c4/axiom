# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "buffer_info_is_callable"
# subject = "array.array.buffer_info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.buffer_info: buffer_info_is_callable (surface)."""
import array

assert callable(array.array.buffer_info)
print("buffer_info_is_callable OK")

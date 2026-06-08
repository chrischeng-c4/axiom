# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "index_is_callable"
# subject = "array.array.index"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.index: index_is_callable (surface)."""
import array

assert callable(array.array.index)
print("index_is_callable OK")

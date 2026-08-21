# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "count_is_callable"
# subject = "array.array.count"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.count: count_is_callable (surface)."""
import array

assert callable(array.array.count)
print("count_is_callable OK")

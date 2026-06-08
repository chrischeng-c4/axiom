# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "append_is_callable"
# subject = "array.array.append"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.append: append_is_callable (surface)."""
import array

assert callable(array.array.append)
print("append_is_callable OK")

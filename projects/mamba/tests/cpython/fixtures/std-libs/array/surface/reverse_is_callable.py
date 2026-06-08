# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "reverse_is_callable"
# subject = "array.array.reverse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.reverse: reverse_is_callable (surface)."""
import array

assert callable(array.array.reverse)
print("reverse_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "extend_is_callable"
# subject = "array.array.extend"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.extend: extend_is_callable (surface)."""
import array

assert callable(array.array.extend)
print("extend_is_callable OK")

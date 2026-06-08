# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "tolist_is_callable"
# subject = "array.array.tolist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.tolist: tolist_is_callable (surface)."""
import array

assert callable(array.array.tolist)
print("tolist_is_callable OK")

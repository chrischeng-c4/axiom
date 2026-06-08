# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "remove_is_callable"
# subject = "array.array.remove"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.remove: remove_is_callable (surface)."""
import array

assert callable(array.array.remove)
print("remove_is_callable OK")

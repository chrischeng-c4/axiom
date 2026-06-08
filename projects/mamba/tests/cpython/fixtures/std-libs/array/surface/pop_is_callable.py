# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "pop_is_callable"
# subject = "array.array.pop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.pop: pop_is_callable (surface)."""
import array

assert callable(array.array.pop)
print("pop_is_callable OK")

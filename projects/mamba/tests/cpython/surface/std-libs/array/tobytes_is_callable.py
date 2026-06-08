# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "tobytes_is_callable"
# subject = "array.array.tobytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.tobytes: tobytes_is_callable (surface)."""
import array

assert callable(array.array.tobytes)
print("tobytes_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "frombytes_is_callable"
# subject = "array.array.frombytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.frombytes: frombytes_is_callable (surface)."""
import array

assert callable(array.array.frombytes)
print("frombytes_is_callable OK")

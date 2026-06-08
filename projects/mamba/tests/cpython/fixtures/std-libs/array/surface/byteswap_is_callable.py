# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "byteswap_is_callable"
# subject = "array.array.byteswap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.byteswap: byteswap_is_callable (surface)."""
import array

assert callable(array.array.byteswap)
print("byteswap_is_callable OK")

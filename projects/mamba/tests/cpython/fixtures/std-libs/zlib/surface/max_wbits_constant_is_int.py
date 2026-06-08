# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "max_wbits_constant_is_int"
# subject = "zlib.MAX_WBITS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.MAX_WBITS: max_wbits_constant_is_int (surface)."""
import zlib

assert type(zlib.MAX_WBITS).__name__ == "int"
print("max_wbits_constant_is_int OK")

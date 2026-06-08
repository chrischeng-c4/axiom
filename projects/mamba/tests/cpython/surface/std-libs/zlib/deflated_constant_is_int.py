# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "surface"
# case = "deflated_constant_is_int"
# subject = "zlib.DEFLATED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.DEFLATED: deflated_constant_is_int (surface)."""
import zlib

assert type(zlib.DEFLATED).__name__ == "int"
print("deflated_constant_is_int OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "write_constant_is_int"
# subject = "gzip.WRITE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.WRITE: write_constant_is_int (surface)."""
import gzip

assert type(gzip.WRITE).__name__ == "int"
print("write_constant_is_int OK")

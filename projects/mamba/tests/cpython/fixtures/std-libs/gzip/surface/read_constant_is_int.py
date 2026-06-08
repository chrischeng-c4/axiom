# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "read_constant_is_int"
# subject = "gzip.READ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.READ: read_constant_is_int (surface)."""
import gzip

assert type(gzip.READ).__name__ == "int"
print("read_constant_is_int OK")

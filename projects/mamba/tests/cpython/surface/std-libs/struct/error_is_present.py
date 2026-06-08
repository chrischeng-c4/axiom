# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "error_is_present"
# subject = "struct"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct: error_is_present (surface)."""
import struct

assert hasattr(struct, "error")
print("error_is_present OK")

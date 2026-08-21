# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "surface"
# case = "calcsize_is_callable"
# subject = "struct.calcsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.calcsize: calcsize_is_callable (surface)."""
import struct

assert callable(struct.calcsize)
print("calcsize_is_callable OK")

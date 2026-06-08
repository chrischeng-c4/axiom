# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "hexversion_is_int"
# subject = "sys.hexversion"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.hexversion: hexversion_is_int (surface)."""
import sys

assert type(sys.hexversion).__name__ == "int"
print("hexversion_is_int OK")

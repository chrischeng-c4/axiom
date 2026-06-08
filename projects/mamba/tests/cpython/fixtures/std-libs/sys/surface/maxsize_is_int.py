# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "maxsize_is_int"
# subject = "sys.maxsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.maxsize: maxsize_is_int (surface)."""
import sys

assert type(sys.maxsize).__name__ == "int"
print("maxsize_is_int OK")

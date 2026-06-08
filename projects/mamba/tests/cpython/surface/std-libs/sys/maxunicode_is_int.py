# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "maxunicode_is_int"
# subject = "sys.maxunicode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.maxunicode: maxunicode_is_int (surface)."""
import sys

assert type(sys.maxunicode).__name__ == "int"
print("maxunicode_is_int OK")

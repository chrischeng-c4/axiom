# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "argv_is_list"
# subject = "sys.argv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.argv: argv_is_list (surface)."""
import sys

assert type(sys.argv).__name__ == "list"
print("argv_is_list OK")

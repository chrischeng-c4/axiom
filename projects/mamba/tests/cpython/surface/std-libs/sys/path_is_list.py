# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "path_is_list"
# subject = "sys.path"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.path: path_is_list (surface)."""
import sys

assert type(sys.path).__name__ == "list"
print("path_is_list OK")

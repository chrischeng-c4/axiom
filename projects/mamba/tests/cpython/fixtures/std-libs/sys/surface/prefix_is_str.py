# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "prefix_is_str"
# subject = "sys.prefix"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.prefix: prefix_is_str (surface)."""
import sys

assert type(sys.prefix).__name__ == "str"
print("prefix_is_str OK")

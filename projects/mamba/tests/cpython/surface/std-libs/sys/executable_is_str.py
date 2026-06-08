# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "executable_is_str"
# subject = "sys.executable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.executable: executable_is_str (surface)."""
import sys

assert type(sys.executable).__name__ == "str"
print("executable_is_str OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "linesep_is_str"
# subject = "os.linesep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.linesep: linesep_is_str (surface)."""
import os

assert type(os.linesep).__name__ == "str"
print("linesep_is_str OK")

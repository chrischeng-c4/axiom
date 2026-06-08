# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "sep_is_str"
# subject = "os.sep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.sep: sep_is_str (surface)."""
import os

assert type(os.sep).__name__ == "str"
print("sep_is_str OK")

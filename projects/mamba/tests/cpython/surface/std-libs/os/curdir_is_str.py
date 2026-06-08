# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "curdir_is_str"
# subject = "os.curdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.curdir: curdir_is_str (surface)."""
import os

assert type(os.curdir).__name__ == "str"
print("curdir_is_str OK")

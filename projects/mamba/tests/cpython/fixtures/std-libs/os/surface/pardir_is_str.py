# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "pardir_is_str"
# subject = "os.pardir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.pardir: pardir_is_str (surface)."""
import os

assert type(os.pardir).__name__ == "str"
print("pardir_is_str OK")

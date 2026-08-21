# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "devnull_is_str"
# subject = "os.devnull"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.devnull: devnull_is_str (surface)."""
import os

assert type(os.devnull).__name__ == "str"
print("devnull_is_str OK")

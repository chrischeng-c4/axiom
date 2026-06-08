# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "listdir_is_callable"
# subject = "os.listdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.listdir: listdir_is_callable (surface)."""
import os

assert callable(os.listdir)
print("listdir_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "scandir_is_callable"
# subject = "os.scandir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.scandir: scandir_is_callable (surface)."""
import os

assert callable(os.scandir)
print("scandir_is_callable OK")

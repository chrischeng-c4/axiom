# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "path_splitext_is_callable"
# subject = "os.path.splitext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.splitext: path_splitext_is_callable (surface)."""
import os.path

assert callable(os.path.splitext)
print("path_splitext_is_callable OK")

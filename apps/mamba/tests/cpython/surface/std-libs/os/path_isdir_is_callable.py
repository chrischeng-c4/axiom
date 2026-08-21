# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "path_isdir_is_callable"
# subject = "os.path.isdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.isdir: path_isdir_is_callable (surface)."""
import os.path

assert callable(os.path.isdir)
print("path_isdir_is_callable OK")

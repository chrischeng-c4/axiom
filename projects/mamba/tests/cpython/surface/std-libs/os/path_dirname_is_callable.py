# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "path_dirname_is_callable"
# subject = "os.path.dirname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.dirname: path_dirname_is_callable (surface)."""
import os.path

assert callable(os.path.dirname)
print("path_dirname_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "path_isfile_is_callable"
# subject = "os.path.isfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.isfile: path_isfile_is_callable (surface)."""
import os.path

assert callable(os.path.isfile)
print("path_isfile_is_callable OK")

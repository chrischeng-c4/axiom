# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "path_exists_is_callable"
# subject = "os.path.exists"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.exists: path_exists_is_callable (surface)."""
import os.path

assert callable(os.path.exists)
print("path_exists_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "path_join_is_callable"
# subject = "os.path.join"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.join: path_join_is_callable (surface)."""
import os.path

assert callable(os.path.join)
print("path_join_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "isdir_is_callable"
# subject = "os.path.isdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.isdir: isdir_is_callable (surface)."""
import os.path

assert callable(os.path.isdir)
print("isdir_is_callable OK")

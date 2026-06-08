# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "normpath_is_callable"
# subject = "os.path.normpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.normpath: normpath_is_callable (surface)."""
import os.path

assert callable(os.path.normpath)
print("normpath_is_callable OK")

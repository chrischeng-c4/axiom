# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "relpath_is_callable"
# subject = "os.path.relpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.relpath: relpath_is_callable (surface)."""
import os.path

assert callable(os.path.relpath)
print("relpath_is_callable OK")

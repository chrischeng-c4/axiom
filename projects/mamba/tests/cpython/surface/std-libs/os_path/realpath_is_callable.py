# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "realpath_is_callable"
# subject = "os.path.realpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.realpath: realpath_is_callable (surface)."""
import os.path

assert callable(os.path.realpath)
print("realpath_is_callable OK")

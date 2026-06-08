# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "commonpath_is_callable"
# subject = "os.path.commonpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.commonpath: commonpath_is_callable (surface)."""
import os.path

assert callable(os.path.commonpath)
print("commonpath_is_callable OK")

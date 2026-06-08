# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "getsize_is_callable"
# subject = "os.path.getsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.getsize: getsize_is_callable (surface)."""
import os.path

assert callable(os.path.getsize)
print("getsize_is_callable OK")

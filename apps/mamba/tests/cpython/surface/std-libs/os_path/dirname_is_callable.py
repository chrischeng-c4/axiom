# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "dirname_is_callable"
# subject = "os.path.dirname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.dirname: dirname_is_callable (surface)."""
import os.path

assert callable(os.path.dirname)
print("dirname_is_callable OK")

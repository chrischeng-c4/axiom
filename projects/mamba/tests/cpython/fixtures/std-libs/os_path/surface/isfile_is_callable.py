# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "isfile_is_callable"
# subject = "os.path.isfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.isfile: isfile_is_callable (surface)."""
import os.path

assert callable(os.path.isfile)
print("isfile_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "basename_is_callable"
# subject = "os.path.basename"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.basename: basename_is_callable (surface)."""
import os.path

assert callable(os.path.basename)
print("basename_is_callable OK")

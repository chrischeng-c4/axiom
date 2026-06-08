# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "splitext_is_callable"
# subject = "os.path.splitext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.splitext: splitext_is_callable (surface)."""
import os.path

assert callable(os.path.splitext)
print("splitext_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "which_is_callable"
# subject = "shutil.which"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.which: which_is_callable (surface)."""
import shutil

assert callable(shutil.which)
print("which_is_callable OK")

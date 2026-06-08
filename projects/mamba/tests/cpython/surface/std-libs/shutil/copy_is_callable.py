# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "copy_is_callable"
# subject = "shutil.copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.copy: copy_is_callable (surface)."""
import shutil

assert callable(shutil.copy)
print("copy_is_callable OK")

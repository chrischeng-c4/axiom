# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "copy2_is_callable"
# subject = "shutil.copy2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.copy2: copy2_is_callable (surface)."""
import shutil

assert callable(shutil.copy2)
print("copy2_is_callable OK")

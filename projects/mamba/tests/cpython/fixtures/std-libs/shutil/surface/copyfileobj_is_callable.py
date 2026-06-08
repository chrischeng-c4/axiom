# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "copyfileobj_is_callable"
# subject = "shutil.copyfileobj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.copyfileobj: copyfileobj_is_callable (surface)."""
import shutil

assert callable(shutil.copyfileobj)
print("copyfileobj_is_callable OK")

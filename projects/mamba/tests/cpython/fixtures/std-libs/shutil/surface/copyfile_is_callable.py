# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "copyfile_is_callable"
# subject = "shutil.copyfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.copyfile: copyfile_is_callable (surface)."""
import shutil

assert callable(shutil.copyfile)
print("copyfile_is_callable OK")

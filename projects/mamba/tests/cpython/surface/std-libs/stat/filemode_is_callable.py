# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "filemode_is_callable"
# subject = "stat.filemode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.filemode: filemode_is_callable (surface)."""
import stat

assert callable(stat.filemode)
print("filemode_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "surface"
# case = "fnmatch_is_callable"
# subject = "fnmatch.fnmatch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.fnmatch: fnmatch_is_callable (surface)."""
import fnmatch

assert callable(fnmatch.fnmatch)
print("fnmatch_is_callable OK")

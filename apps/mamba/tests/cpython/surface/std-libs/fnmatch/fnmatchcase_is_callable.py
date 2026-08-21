# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "surface"
# case = "fnmatchcase_is_callable"
# subject = "fnmatch.fnmatchcase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.fnmatchcase: fnmatchcase_is_callable (surface)."""
import fnmatch

assert callable(fnmatch.fnmatchcase)
print("fnmatchcase_is_callable OK")

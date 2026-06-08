# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "surface"
# case = "filter_is_callable"
# subject = "fnmatch.filter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.filter: filter_is_callable (surface)."""
import fnmatch

assert callable(fnmatch.filter)
print("filter_is_callable OK")

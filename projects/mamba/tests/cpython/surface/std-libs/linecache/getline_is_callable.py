# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "getline_is_callable"
# subject = "linecache.getline"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.getline: getline_is_callable (surface)."""
import linecache

assert callable(linecache.getline)
print("getline_is_callable OK")

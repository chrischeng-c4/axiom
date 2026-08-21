# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "getlines_is_callable"
# subject = "linecache.getlines"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.getlines: getlines_is_callable (surface)."""
import linecache

assert callable(linecache.getlines)
print("getlines_is_callable OK")

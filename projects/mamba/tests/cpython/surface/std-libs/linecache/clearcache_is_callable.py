# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "clearcache_is_callable"
# subject = "linecache.clearcache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.clearcache: clearcache_is_callable (surface)."""
import linecache

assert callable(linecache.clearcache)
print("clearcache_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "checkcache_is_callable"
# subject = "linecache.checkcache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.checkcache: checkcache_is_callable (surface)."""
import linecache

assert callable(linecache.checkcache)
print("checkcache_is_callable OK")

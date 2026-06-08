# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "updatecache_is_callable"
# subject = "linecache.updatecache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.updatecache: updatecache_is_callable (surface)."""
import linecache

assert callable(linecache.updatecache)
print("updatecache_is_callable OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "lazycache_is_callable"
# subject = "linecache.lazycache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.lazycache: lazycache_is_callable (surface)."""
import linecache

assert callable(linecache.lazycache)
print("lazycache_is_callable OK")

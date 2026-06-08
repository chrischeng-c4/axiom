# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "api_clearcache_is_present"
# subject = "linecache.clearcache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""linecache.clearcache: api_clearcache_is_present (surface)."""
import linecache

assert hasattr(linecache, "clearcache")
print("api_clearcache_is_present OK")

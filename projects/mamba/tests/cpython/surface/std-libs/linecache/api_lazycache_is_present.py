# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "api_lazycache_is_present"
# subject = "linecache.lazycache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""linecache.lazycache: api_lazycache_is_present (surface)."""
import linecache

assert hasattr(linecache, "lazycache")
print("api_lazycache_is_present OK")

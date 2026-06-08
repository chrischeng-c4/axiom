# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "api_checkcache_is_present"
# subject = "linecache.checkcache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""linecache.checkcache: api_checkcache_is_present (surface)."""
import linecache

assert hasattr(linecache, "checkcache")
print("api_checkcache_is_present OK")

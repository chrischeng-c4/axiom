# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "cache_is_dict"
# subject = "linecache.cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.cache: cache_is_dict (surface)."""
import linecache

assert type(linecache.cache).__name__ == "dict"
print("cache_is_dict OK")

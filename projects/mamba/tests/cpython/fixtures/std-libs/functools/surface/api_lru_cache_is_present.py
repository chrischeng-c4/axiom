# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "api_lru_cache_is_present"
# subject = "functools.lru_cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""functools.lru_cache: api_lru_cache_is_present (surface)."""
import functools

assert hasattr(functools, "lru_cache")
print("api_lru_cache_is_present OK")

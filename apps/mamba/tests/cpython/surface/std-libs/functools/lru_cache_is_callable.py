# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "lru_cache_is_callable"
# subject = "functools.lru_cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""functools.lru_cache: lru_cache_is_callable (surface)."""
import functools

assert callable(functools.lru_cache)
print("lru_cache_is_callable OK")

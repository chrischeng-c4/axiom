# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "cache_is_callable"
# subject = "functools.cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""functools.cache: cache_is_callable (surface)."""
import functools

assert callable(functools.cache)
print("cache_is_callable OK")

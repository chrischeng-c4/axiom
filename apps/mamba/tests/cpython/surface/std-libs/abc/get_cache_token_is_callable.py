# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "get_cache_token_is_callable"
# subject = "abc.get_cache_token"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abc.get_cache_token: get_cache_token_is_callable (surface)."""
import abc

assert callable(abc.get_cache_token)
print("get_cache_token_is_callable OK")

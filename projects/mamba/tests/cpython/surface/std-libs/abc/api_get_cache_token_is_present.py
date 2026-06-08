# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "api_get_cache_token_is_present"
# subject = "abc.get_cache_token"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""abc.get_cache_token: api_get_cache_token_is_present (surface)."""
import abc

assert hasattr(abc, "get_cache_token")
print("api_get_cache_token_is_present OK")

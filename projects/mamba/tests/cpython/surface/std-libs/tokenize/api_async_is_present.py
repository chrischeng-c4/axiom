# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_async_is_present"
# subject = "tokenize.ASYNC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.ASYNC: api_async_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "ASYNC")
print("api_async_is_present OK")

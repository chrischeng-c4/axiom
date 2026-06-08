# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_errortoken_is_present"
# subject = "tokenize.ERRORTOKEN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.ERRORTOKEN: api_errortoken_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "ERRORTOKEN")
print("api_errortoken_is_present OK")

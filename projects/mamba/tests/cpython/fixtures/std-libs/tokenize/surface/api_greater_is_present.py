# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_greater_is_present"
# subject = "tokenize.GREATER"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.GREATER: api_greater_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "GREATER")
print("api_greater_is_present OK")

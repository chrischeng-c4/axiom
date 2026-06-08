# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_slash_is_present"
# subject = "tokenize.SLASH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.SLASH: api_slash_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "SLASH")
print("api_slash_is_present OK")

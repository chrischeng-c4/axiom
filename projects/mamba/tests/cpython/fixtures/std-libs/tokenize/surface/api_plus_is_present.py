# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_plus_is_present"
# subject = "tokenize.PLUS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.PLUS: api_plus_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "PLUS")
print("api_plus_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_rarrow_is_present"
# subject = "tokenize.RARROW"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.RARROW: api_rarrow_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "RARROW")
print("api_rarrow_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_less_is_present"
# subject = "tokenize.LESS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.LESS: api_less_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "LESS")
print("api_less_is_present OK")

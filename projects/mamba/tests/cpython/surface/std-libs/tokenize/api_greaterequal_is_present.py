# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_greaterequal_is_present"
# subject = "tokenize.GREATEREQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.GREATEREQUAL: api_greaterequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "GREATEREQUAL")
print("api_greaterequal_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_rpar_is_present"
# subject = "tokenize.RPAR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.RPAR: api_rpar_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "RPAR")
print("api_rpar_is_present OK")

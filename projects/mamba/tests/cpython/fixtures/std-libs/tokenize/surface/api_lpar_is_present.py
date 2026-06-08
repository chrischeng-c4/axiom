# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_lpar_is_present"
# subject = "tokenize.LPAR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.LPAR: api_lpar_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "LPAR")
print("api_lpar_is_present OK")

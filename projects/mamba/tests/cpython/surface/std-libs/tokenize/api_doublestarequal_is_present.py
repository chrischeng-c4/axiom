# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_doublestarequal_is_present"
# subject = "tokenize.DOUBLESTAREQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.DOUBLESTAREQUAL: api_doublestarequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "DOUBLESTAREQUAL")
print("api_doublestarequal_is_present OK")

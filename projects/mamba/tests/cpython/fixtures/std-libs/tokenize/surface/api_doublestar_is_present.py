# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_doublestar_is_present"
# subject = "tokenize.DOUBLESTAR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.DOUBLESTAR: api_doublestar_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "DOUBLESTAR")
print("api_doublestar_is_present OK")

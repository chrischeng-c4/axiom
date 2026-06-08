# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_lessequal_is_present"
# subject = "tokenize.LESSEQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.LESSEQUAL: api_lessequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "LESSEQUAL")
print("api_lessequal_is_present OK")

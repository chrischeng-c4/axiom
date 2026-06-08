# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_slashequal_is_present"
# subject = "tokenize.SLASHEQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.SLASHEQUAL: api_slashequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "SLASHEQUAL")
print("api_slashequal_is_present OK")

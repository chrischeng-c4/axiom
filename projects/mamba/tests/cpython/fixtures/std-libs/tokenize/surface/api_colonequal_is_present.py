# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_colonequal_is_present"
# subject = "tokenize.COLONEQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.COLONEQUAL: api_colonequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "COLONEQUAL")
print("api_colonequal_is_present OK")

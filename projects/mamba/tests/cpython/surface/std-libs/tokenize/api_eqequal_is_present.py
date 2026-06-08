# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_eqequal_is_present"
# subject = "tokenize.EQEQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.EQEQUAL: api_eqequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "EQEQUAL")
print("api_eqequal_is_present OK")

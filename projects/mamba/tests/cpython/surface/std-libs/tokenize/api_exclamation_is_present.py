# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_exclamation_is_present"
# subject = "tokenize.EXCLAMATION"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.EXCLAMATION: api_exclamation_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "EXCLAMATION")
print("api_exclamation_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_minus_is_present"
# subject = "tokenize.MINUS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.MINUS: api_minus_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "MINUS")
print("api_minus_is_present OK")

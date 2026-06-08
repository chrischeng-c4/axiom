# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_iseof_is_present"
# subject = "tokenize.ISEOF"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.ISEOF: api_iseof_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "ISEOF")
print("api_iseof_is_present OK")

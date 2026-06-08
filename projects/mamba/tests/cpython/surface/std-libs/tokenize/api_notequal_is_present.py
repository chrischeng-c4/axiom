# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_notequal_is_present"
# subject = "tokenize.NOTEQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.NOTEQUAL: api_notequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "NOTEQUAL")
print("api_notequal_is_present OK")

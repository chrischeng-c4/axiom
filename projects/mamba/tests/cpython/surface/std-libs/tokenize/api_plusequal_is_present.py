# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_plusequal_is_present"
# subject = "tokenize.PLUSEQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.PLUSEQUAL: api_plusequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "PLUSEQUAL")
print("api_plusequal_is_present OK")

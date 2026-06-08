# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_lbrace_is_present"
# subject = "tokenize.LBRACE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.LBRACE: api_lbrace_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "LBRACE")
print("api_lbrace_is_present OK")

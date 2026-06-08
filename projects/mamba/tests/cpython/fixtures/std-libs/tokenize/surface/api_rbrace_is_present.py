# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_rbrace_is_present"
# subject = "tokenize.RBRACE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.RBRACE: api_rbrace_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "RBRACE")
print("api_rbrace_is_present OK")

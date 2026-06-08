# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_tilde_is_present"
# subject = "tokenize.TILDE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.TILDE: api_tilde_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "TILDE")
print("api_tilde_is_present OK")

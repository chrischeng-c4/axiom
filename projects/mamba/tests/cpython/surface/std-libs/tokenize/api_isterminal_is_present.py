# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_isterminal_is_present"
# subject = "tokenize.ISTERMINAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.ISTERMINAL: api_isterminal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "ISTERMINAL")
print("api_isterminal_is_present OK")

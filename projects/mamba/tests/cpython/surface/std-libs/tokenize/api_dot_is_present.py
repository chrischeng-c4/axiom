# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_dot_is_present"
# subject = "tokenize.DOT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.DOT: api_dot_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "DOT")
print("api_dot_is_present OK")

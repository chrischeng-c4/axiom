# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_tokenize_is_present"
# subject = "tokenize.tokenize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.tokenize: api_tokenize_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "tokenize")
print("api_tokenize_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_exact_token_types_is_present"
# subject = "tokenize.EXACT_TOKEN_TYPES"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.EXACT_TOKEN_TYPES: api_exact_token_types_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "EXACT_TOKEN_TYPES")
print("api_exact_token_types_is_present OK")

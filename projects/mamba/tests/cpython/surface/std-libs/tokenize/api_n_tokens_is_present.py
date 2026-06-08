# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_n_tokens_is_present"
# subject = "tokenize.N_TOKENS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.N_TOKENS: api_n_tokens_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "N_TOKENS")
print("api_n_tokens_is_present OK")

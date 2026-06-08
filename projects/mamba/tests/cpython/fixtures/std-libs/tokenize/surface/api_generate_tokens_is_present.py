# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_generate_tokens_is_present"
# subject = "tokenize.generate_tokens"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.generate_tokens: api_generate_tokens_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "generate_tokens")
print("api_generate_tokens_is_present OK")

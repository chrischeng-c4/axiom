# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_tok_name_is_present"
# subject = "tokenize.tok_name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.tok_name: api_tok_name_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "tok_name")
print("api_tok_name_is_present OK")

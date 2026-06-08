# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_isnonterminal_is_present"
# subject = "tokenize.ISNONTERMINAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.ISNONTERMINAL: api_isnonterminal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "ISNONTERMINAL")
print("api_isnonterminal_is_present OK")

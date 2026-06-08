# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_dedent_is_present"
# subject = "tokenize.DEDENT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.DEDENT: api_dedent_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "DEDENT")
print("api_dedent_is_present OK")

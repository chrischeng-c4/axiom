# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_lsqb_is_present"
# subject = "tokenize.LSQB"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.LSQB: api_lsqb_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "LSQB")
print("api_lsqb_is_present OK")

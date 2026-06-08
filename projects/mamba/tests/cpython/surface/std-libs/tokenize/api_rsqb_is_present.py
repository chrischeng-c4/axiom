# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_rsqb_is_present"
# subject = "tokenize.RSQB"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.RSQB: api_rsqb_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "RSQB")
print("api_rsqb_is_present OK")

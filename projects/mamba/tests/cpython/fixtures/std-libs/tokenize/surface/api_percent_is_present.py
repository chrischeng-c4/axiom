# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_percent_is_present"
# subject = "tokenize.PERCENT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.PERCENT: api_percent_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "PERCENT")
print("api_percent_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_leftshift_is_present"
# subject = "tokenize.LEFTSHIFT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.LEFTSHIFT: api_leftshift_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "LEFTSHIFT")
print("api_leftshift_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_fstring_middle_is_present"
# subject = "tokenize.FSTRING_MIDDLE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.FSTRING_MIDDLE: api_fstring_middle_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "FSTRING_MIDDLE")
print("api_fstring_middle_is_present OK")

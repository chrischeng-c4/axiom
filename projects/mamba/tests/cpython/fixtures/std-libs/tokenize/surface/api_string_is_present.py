# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_string_is_present"
# subject = "tokenize.STRING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.STRING: api_string_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "STRING")
print("api_string_is_present OK")

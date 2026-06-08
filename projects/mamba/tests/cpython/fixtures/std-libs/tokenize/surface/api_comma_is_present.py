# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_comma_is_present"
# subject = "tokenize.COMMA"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.COMMA: api_comma_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "COMMA")
print("api_comma_is_present OK")

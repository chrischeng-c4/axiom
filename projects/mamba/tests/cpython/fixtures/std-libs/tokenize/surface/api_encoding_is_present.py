# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_encoding_is_present"
# subject = "tokenize.ENCODING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.ENCODING: api_encoding_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "ENCODING")
print("api_encoding_is_present OK")

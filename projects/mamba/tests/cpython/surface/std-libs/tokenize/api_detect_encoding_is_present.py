# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_detect_encoding_is_present"
# subject = "tokenize.detect_encoding"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.detect_encoding: api_detect_encoding_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "detect_encoding")
print("api_detect_encoding_is_present OK")

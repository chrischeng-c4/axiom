# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_endmarker_is_present"
# subject = "tokenize.ENDMARKER"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.ENDMARKER: api_endmarker_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "ENDMARKER")
print("api_endmarker_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_minequal_is_present"
# subject = "tokenize.MINEQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.MINEQUAL: api_minequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "MINEQUAL")
print("api_minequal_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_atequal_is_present"
# subject = "tokenize.ATEQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.ATEQUAL: api_atequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "ATEQUAL")
print("api_atequal_is_present OK")

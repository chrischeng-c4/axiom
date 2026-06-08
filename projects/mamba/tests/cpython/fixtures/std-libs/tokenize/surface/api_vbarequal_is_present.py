# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_vbarequal_is_present"
# subject = "tokenize.VBAREQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.VBAREQUAL: api_vbarequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "VBAREQUAL")
print("api_vbarequal_is_present OK")

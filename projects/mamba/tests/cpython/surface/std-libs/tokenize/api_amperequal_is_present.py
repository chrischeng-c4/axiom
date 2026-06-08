# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_amperequal_is_present"
# subject = "tokenize.AMPEREQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.AMPEREQUAL: api_amperequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "AMPEREQUAL")
print("api_amperequal_is_present OK")

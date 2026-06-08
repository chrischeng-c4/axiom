# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_rightshiftequal_is_present"
# subject = "tokenize.RIGHTSHIFTEQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.RIGHTSHIFTEQUAL: api_rightshiftequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "RIGHTSHIFTEQUAL")
print("api_rightshiftequal_is_present OK")

# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_starequal_is_present"
# subject = "tokenize.STAREQUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.STAREQUAL: api_starequal_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "STAREQUAL")
print("api_starequal_is_present OK")
